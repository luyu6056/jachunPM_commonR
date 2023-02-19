use crate::async_trait;
use crate::buffer::MsgBuffer;
use crate::codec::CodeC;
use crate::timeout;
use crate::AsyncWriteExt;
use crate::Duration;
use crate::EventHandler;
use crate::NetError;
use crate::{Receiver, Sender};
use ::tokio::macros::support::Poll::{Pending, Ready};
use std::collections::BTreeMap;
use std::future::Future;
use std::marker::PhantomPinned;
use std::net::SocketAddr;
use std::ops::Add;
use std::pin::Pin;
use std::ptr::NonNull;
use std::thread::sleep;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tokio::io::BufReader;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time;
pub trait ConnTraitS = AsyncWriteExt + Unpin + 'static;

#[async_trait]
pub trait ConnTraitT: Send + Sync + Clone + Default {
    fn set_conn(&mut self, conn: NonNull<Conn<Self>>);
    fn get_conn(&self) -> NonNull<Conn<Self>>;
}
pub type ConnAsyncFn<T> =
    Box<dyn for<'a> FnOnce(&'a mut T) -> ConnAsyncResult<'a> + Send + Sync + 'static>;
pub type ConnAsyncResult<'a> = Pin<Box<dyn Future<Output = Result<(), NetError>> + Send + 'a>>;
pub struct Conn<T: ConnTraitT> {
    reader: TcpStream,
    addr: SocketAddr,
    readbuf: *mut MsgBuffer,
    write_tx: Sender<Vec<u8>>,
    write_rx: Receiver<Vec<u8>>,
    close_tx: Sender<Option<String>>,
    close_rx: Receiver<Option<String>>,
    exit_tx: Sender<NetError>,
    time_fn_queue: BTreeMap<Instant, ConnAsyncFn<T>>,
    max_package_size: usize,
    default_read_timeout: time::Duration,
    default_write_timeout: time::Duration,
    pub(crate) readtimedead: Duration,
    pub(crate) writetimedead: Duration,
}
unsafe impl<T: ConnTraitT> Send for Conn<T> {}
unsafe impl<T: ConnTraitT> Sync for Conn<T> {}

impl<T: ConnTraitT> Conn<T> {
    pub fn new(
        stream: TcpStream,
        addr: SocketAddr,
        readbuf: *mut MsgBuffer,
        read_timeout: Duration,
        write_timeout: Duration,
        exit_tx: Sender<NetError>,
        max_package_size: usize,
    ) -> Pin<Box<Self>> {
        let (write_tx, write_rx) = mpsc::channel::<Vec<u8>>(1024);
        let (close_tx, close_rx) = mpsc::channel::<Option<String>>(1);
        let c = Box::pin(Conn {
            reader: stream,
            addr: addr,
            readbuf: readbuf,
            write_tx: write_tx,
            write_rx: write_rx,
            close_tx: close_tx,
            close_rx: close_rx,
            exit_tx: exit_tx,
            time_fn_queue: BTreeMap::new(),
            max_package_size: max_package_size,
            default_read_timeout: read_timeout,
            default_write_timeout: write_timeout,
            readtimedead: Duration::ZERO,
            writetimedead: Duration::ZERO,
        });
        c
    }
    async fn read(&mut self) -> Result<(), NetError> {
        unsafe {
            #[doc(hidden)]
            mod __tokio_select_util {
                #[derive(Debug)]
                pub(super) enum Out<_0, _1> {
                    _0(_0),
                    _1(_1),
                }
            }

            let buffer = (*self.readbuf).spare(8192);
            let output = {
                ::tokio::macros::support::poll_fn(|cx| {
                    let f = &mut self.reader.read(buffer);
                    if let Ready(out) = Future::poll(Pin::new_unchecked(f), cx) {
                        return Ready(__tokio_select_util::Out::_0(out));
                    }
                    let f2 = &mut self.close_rx.recv();
                    if let Ready(out) = Future::poll(Pin::new_unchecked(f2), cx) {
                        return Ready(__tokio_select_util::Out::_1(out));
                    }
                    Pending
                })
                .await
            };
            match output {
                __tokio_select_util::Out::_0(res) => {
                    let size = res?;
                    self.buffer_add_size(size as isize)?;
                }
                __tokio_select_util::Out::_1(data) => {
                    if let Some(Some(data)) = data {
                        return Err(NetError::Custom(data));
                    }
                    return Err(NetError::Close);
                }
            }
            self.readtimedead=Duration::ZERO;
            Ok(())
        }
    }
    pub async fn read_data(&mut self) -> Result<(), NetError> {
        self.readtimedead =
            SystemTime::now().duration_since(UNIX_EPOCH)? + self.default_read_timeout;
        self.read().await
    }

    pub async fn read_data_with_timeout(&mut self, readtimeout: Duration) -> Result<(), NetError> {
        self.readtimedead = SystemTime::now().duration_since(UNIX_EPOCH)? + readtimeout;
        self.read().await
    }
    async fn write(&mut self, data: &[u8]) -> Result<(), NetError> {
        unsafe {
            #[doc(hidden)]
            mod __tokio_select_util {
                #[derive(Debug)]
                pub(super) enum Out<_0, _1> {
                    _0(_0),
                    _1(_1),
                }
            }
            let mut n = 0;
            while n < data.len() {
                let output = {
                    ::tokio::macros::support::poll_fn(|cx| {
                        let f = &mut self.reader.write(&data[n..]);
                        if let Ready(out) = Future::poll(Pin::new_unchecked(f), cx) {
                            return Ready(__tokio_select_util::Out::_0(out));
                        }
                        let f2 = &mut self.close_rx.recv();
                        if let Ready(out) = Future::poll(Pin::new_unchecked(f2), cx) {
                            return Ready(__tokio_select_util::Out::_1(out));
                        }
                        Pending
                    })
                    .await
                };
                match output {
                    __tokio_select_util::Out::_0(res) => {
                        let size = res?;
                        if size == 0 {
                            return Err(NetError::TcpDisconnected);
                        }
                        n += size
                    }
                    __tokio_select_util::Out::_1(data) => {
                        if let Some(Some(data)) = data {
                            return Err(NetError::Custom(data));
                        }
                        return Err(NetError::Close);
                    }
                }
            }
            self.writetimedead=Duration::ZERO;
            Ok(())
        }
    }
    pub async fn write_data(&mut self,data :&[u8]) -> Result<(), NetError> {
        self.writetimedead =
            SystemTime::now().duration_since(UNIX_EPOCH)? + self.default_write_timeout;
        self.write(data).await
    }

    pub async fn write_data_with_timeout(&mut self,data :&[u8], readtimeout: Duration) -> Result<(), NetError> {
        self.writetimedead = SystemTime::now().duration_since(UNIX_EPOCH)? + readtimeout;
        self.write(data).await
    }
    pub async fn readline(&mut self) -> Result<String, NetError> {
        loop {
            let buffer = self.bufferdata();
            let mut iter = buffer.iter().enumerate();
            while let Some((_, v1)) = iter.next() {
                if v1==&('\r' as u8) && let Some((n,v2))=iter.next() && v2==&('\n' as u8){
                    let res = buffer[..n].to_vec();
                    //self.buffer.copy_within(n.., 0);
                    //unsafe {
                    //    self.buffer.set_len(self.buffer.len() - n);
                    //    return Ok(String::from_utf8_unchecked(res));
                    //}
                }
            }
            self.read_data().await?;
        }
        Err(NetError::UknowError)
    }
    /*pub async fn next(&mut self, n: usize) -> Result<Vec<u8>, NetError> {
        while self.buffer.len() < n {
            self.read_msg().await?;
        }
        let res = self.buffer[..n].to_vec();
        self.buffer.copy_within(n.., 0);
        unsafe {
            self.buffer.set_len(self.buffer.len() - n);
        }
        Ok(res)
    }*/
    pub async fn shift(&mut self, n: usize) -> Result<(), NetError> {
        while self.bufferlen() < n {
            self.read_data().await?;
        }
        unsafe {
            (*self.readbuf).shift(n);
        }
        Ok(())
    }

    pub fn bufferdata(&self) -> &[u8] {
        unsafe { (*self.readbuf).as_slice() }
    }
    pub fn bufferlen(&self) -> usize {
        unsafe { (*self.readbuf).len() }
    }
    //关闭连接
    pub fn close(&self, reason: Option<String>) -> Result<(), NetError> {
        Ok(self.close_tx.try_send(reason)?)
    }
    //退出server
    pub fn exit_server(&self, reason: Option<String>) -> Result<(), NetError> {
        match reason {
            Some(r) => {
                self.exit_tx.try_send(NetError::ShutdownServer(r))?;
            }
            None => {
                self.exit_tx
                    .try_send(NetError::ShutdownServer("".to_string()))?;
            }
        }
        Ok(())
    }

    //延迟执行，最低单位 秒,编写例子
    //conn.after_fn(Duration::from_secs(10),|conn:&mut ConnRead<S,T>| ->ConnAsyncResult{Box::pin (async move{
    //    Ok(())
    //})});
    pub fn after_fn<F>(&mut self, delay: time::Duration, f: F)
    where
        F: for<'b> FnOnce(&'b mut T) -> ConnAsyncResult<'b> + Send + Sync + 'static,
    {
        self.time_fn_queue
            .insert(Instant::now().add(delay), Box::new(f));
    }

    pub fn channel_write(&self, data: Vec<u8>) -> Result<(), NetError> {
        //Ok(self.reader.write_all(data).await?)
        Ok(self.write_tx.try_send(data)?)
    }
    async fn do_tick(&mut self, now: Instant, ctx: &mut T) -> Result<(), NetError> {
        if let Some((k, _)) = &self.time_fn_queue.first_key_value() {
            if now > **k {
                if let Some((_, v)) = self.time_fn_queue.pop_first() {
                    v(ctx).await?;
                }
            }
        }
        Ok(())
    }
    fn buffer_add_size(&mut self, size: isize) -> Result<(), NetError> {
        unsafe {
            if size == 0 {
                return Err(NetError::TcpDisconnected);
            }
            let newlen = ((*self.readbuf).len() as isize + size) as usize;
            if newlen > self.max_package_size {
                return Err(NetError::LargePackage);
            }
            (*self.readbuf).truncate(newlen);
        }

        Ok(())
    }
}

pub(crate) async fn handler<C, E, T>(
    mut ctx: &mut T,
    mut codec: C,
    mut event_handler: E,
) -> Result<(Option<String>), NetError>
where
    T: ConnTraitT,
    E: EventHandler<T> + Send + Copy + Sync + 'static,
    C: CodeC<T>,
{
    unsafe {
        let conn = ctx.get_conn().as_mut();
        event_handler.on_opened(&mut ctx).await?;

        //主循环
        #[doc(hidden)]
        mod __tokio_select_util {
            #[derive(Debug)]
            pub(super) enum Out<_0, _1, _2> {
                _0(_0),
                _1(_1),
                _2(_2),
            }
        }

        loop {
            let buffer = (*conn.readbuf).spare(8192);
            let output = {
                ::tokio::macros::support::poll_fn(|cx| {
                    let f = &mut conn.reader.read(buffer);
                    if let Ready(out) = Future::poll(Pin::new_unchecked(f), cx) {
                        return Ready(__tokio_select_util::Out::_0(out));
                    }

                    let f1 = &mut conn.write_rx.recv();
                    if let Ready(out) = Future::poll(Pin::new_unchecked(f1), cx) {
                        return Ready(__tokio_select_util::Out::_1(out));
                    }

                    let f2 = &mut conn.close_rx.recv();
                    if let Ready(out) = Future::poll(Pin::new_unchecked(f2), cx) {
                        return Ready(__tokio_select_util::Out::_2(out));
                    }

                    Pending
                })
                .await
            };

            match output {
                __tokio_select_util::Out::_0(res) => {
                    let size = res?;
                    conn.buffer_add_size(size as isize)?;
                    if conn.readtimedead.is_zero() {
                        conn.readtimedead = SystemTime::now().duration_since(UNIX_EPOCH)?
                            + conn.default_read_timeout;
                    }

                    while conn.bufferlen()>0 && let Some(data) = codec.decode(&mut ctx).await? {
                         event_handler.react(data, &mut ctx).await?;
                        conn.readtimedead=Duration::ZERO;
                    }
                }
                __tokio_select_util::Out::_1(data) => {
                    if let Some(data) = data {
                        if let Some(data) = codec.encode(data, &mut ctx).await? {
                            conn.write(data.as_slice()).await?;
                        };
                    }
                }
                __tokio_select_util::Out::_2(data) => {
                    conn.write_rx.close();
                    conn.close_rx.close();
                    if let Some(data) = data {
                        return Ok(data);
                    }
                    return Ok(None);
                }
            }
        }
    }
}
