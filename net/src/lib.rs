#![feature(fn_traits)]
#![feature(trait_alias)]
#![feature(async_closure)]
#![feature(let_chains)]
#![feature(core_panic)]
#![feature(libstd_sys_internals)]
#![feature(rt)]
#![feature(if_let_guard)]

#![feature(raw_vec_internals)]
#![feature(strict_provenance)]
#![feature(ptr_as_uninit)]

use crate::codec::CodeC;
use crate::conn::handler;
pub use crate::conn::Conn;
pub use crate::conn::{ConnTraitS, ConnTraitT};
use ::tokio::macros::support::Poll::{Pending, Ready};
use async_trait::async_trait;
use err::NetError;
use std::collections::HashMap;
use std::future::Future;
use std::net::SocketAddr;
use std::ops::DerefMut;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::mpsc::{Sender,Receiver,channel};
use crate::buffer::MsgBuffer;
use crate::pool::Pool;
use crate::pool::PoolNewResult;
use std::u32::MAX;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, MutexGuard};
use tokio::time;
use tokio::time::sleep;
use tokio::time::timeout;
use tokio::time::Duration;

pub mod buffer;
pub mod codec;
pub mod conn;
pub mod err;
pub mod pool;

pub struct Server<C, T: ConnTraitT, E> {
    codec: C,
    addr: String,
    max_package_size: usize,
    eventhandler: E,
    ctx: NonNull<T>, //用来泛型凑数的，让编译器知道你的具体ctx类型
    readtimeout: Duration,
    writetimeout: Duration,
    max_buf_size: usize,
    default_buf_size: usize,
    buf_num: usize,
    conn_map: HashMap<SocketAddr, Pin<Box<Conn<T>>>>,
}
unsafe impl<C, T: ConnTraitT, E> Send for Server<C, T, E> {}
impl<C, T: ConnTraitT, E> Server<C, T, E>
where
    E: EventHandler<T> + Send + Copy + Sync + 'static,
    T: ConnTraitT + 'static,
    C: CodeC<T> + Send + 'static,
{
    //ctx传入具体的ctx类型，用于编译器确认，codec,eventhandler的泛型必须是ctx类型
    pub fn new_with_codec(addr: &str, codec: C, eventhandler: E) -> Server<C, T, E> {
        Server {
            addr: addr.to_string(),
            codec: codec,
            max_package_size: MAX as usize,
            eventhandler,
            ctx: NonNull::dangling(),
            readtimeout: Duration::from_secs(60),
            writetimeout: Duration::from_secs(60),
            max_buf_size: 64 * 1024,
            default_buf_size: 8 * 1024,
            buf_num: 128,
            conn_map: Default::default(),
        }
    }

    pub fn with_max_package_size(mut self, size: usize) -> Server<C, T, E> {
        self.max_package_size = size;
        self
    }
    pub fn with_readtimeout(mut self, readtimeout: Duration) -> Server<C, T, E> {
        self.readtimeout = readtimeout;
        self
    }
    pub fn with_writetimeout(mut self, writetimeout: Duration) -> Server<C, T, E> {
        self.writetimeout = writetimeout;
        self
    }

    //将会保留 size * max_buf_size
    pub fn with_buf_pool_num(mut self, size: usize) -> Server<C, T, E> {
        //self.buf_num=Arc::new(Mutex::new(size));
        //self.buf_pool=bounded(size);
        self
    }

    pub fn with_max_buf_size(mut self, size: usize) -> Server<C, T, E> {
        self.max_buf_size = size;
        self
    }
    pub fn with_default_buf_size(mut self, size: usize) -> Server<C, T, E> {
        self.default_buf_size = size;
        self
    }
    pub async fn start_server(mut self) -> Result<(), NetError> {
        let codec = self.codec;
        let mut eventhandler = self.eventhandler;
        let readtimeout = self.readtimeout;
        let writetimeout = self.writetimeout;
        let addr = self.addr.clone();
        let size = self.max_package_size;
        if self.default_buf_size > self.max_buf_size {
            self.default_buf_size = self.max_buf_size
        }
        let mut s = Arc::new(Mutex::new(self));
        let (exit_tx, mut exit_rx) = channel::<NetError>(1);
        let listener = TcpListener::bind(&addr).await.unwrap();

        println!("listen in {}", addr);
        let mut interval = time::interval(Duration::from_secs(1));

        let mut bufpool = Arc::new(Mutex::new(pool::new::<MsgBuffer>()));
        unsafe {
            mod __tokio_select_util {
                #[derive(Debug)]
                pub(super) enum Out<_0, _1, _2> {
                    _0(_0),
                    _1(_1),
                    _2(_2),
                }
            }

            loop {
                let output = {
                    ::tokio::macros::support::poll_fn(|cx| {
                        if let Ready(out) = listener.poll_accept(cx) {
                            return Ready(__tokio_select_util::Out::_0(out));
                        }
                        let f1 = &mut exit_rx.recv();
                        if let Ready(out) = Future::poll(Pin::new_unchecked(f1), cx) {
                            return Ready(__tokio_select_util::Out::_1(out));
                        }
                        if let Ready(out) = interval.poll_tick(cx) {
                            return Ready(__tokio_select_util::Out::_2(out));
                        }
                        Pending
                    })
                    .await
                };

                match output {
                    __tokio_select_util::Out::_0(accept) => {
                        let (stream, addr) = accept?;
                        let exit_tx1 = exit_tx.clone();
                        let exit_tx2 = exit_tx.clone();

                        let mut bufpool = bufpool.clone();
                        let mut s = s.clone();
                        tokio::spawn(async move {
                            let mut ctx = T::default();
                            let mut buf1 = bufpool.lock().await.get();
                            let mut conn = Conn::<T>::new(
                                stream,
                                addr,
                                &mut buf1 as *mut MsgBuffer,
                                readtimeout,
                                writetimeout,
                                exit_tx1.clone(),
                                size,
                            );
                            unsafe {
                                let map = &mut s.lock().await.conn_map;
                                map.insert(addr, conn);
                                let conn = map.get_mut(&addr);
                                let conn = conn.unwrap();

                                ctx.set_conn(NonNull::from((*conn).as_mut().get_unchecked_mut()));
                            }

                            let reason = match handler(&mut ctx, codec, eventhandler).await {
                                Ok(reason) => (reason),
                                Err(e) => {
                                    if let NetError::ShutdownServer(_) = e {
                                        if let Err(_) = exit_tx2.send(e.clone()).await {
                                            println!("Server 关闭于错误 {:?}", e)
                                        }
                                    }
                                    Some(e.to_string())
                                }
                            };

                            eventhandler.on_closed(&mut ctx,reason).await;
                            bufpool.lock().await.put(buf1);
                            s.lock().await.conn_map.remove(&addr);
                        });
                    }
                    __tokio_select_util::Out::_1(e) => {
                        exit_rx.close();
                        //buf_tx.close();
                        //buf_rx.close();

                        match e {
                            Some(e) => return Err(e),
                            None => return Err(NetError::ShutdownServer("".to_string())),
                        }
                    }
                    __tokio_select_util::Out::_2(now) => {
                        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?;
                        for (addr, conn) in &s.lock().await.conn_map {
                            if conn.readtimedead != Duration::ZERO && timestamp > conn.readtimedead
                            {
                                println!(
                                    "readtimeout {:?},{:?},{:?}",
                                    now,
                                    addr,
                                    conn.close(Some("read timeout".to_string()))
                                );
                            }
                            if conn.writetimedead != Duration::ZERO && timestamp > conn.writetimedead
                            {
                                println!(
                                    "write timeout {:?},{:?},{:?}",
                                    now,
                                    addr,
                                    conn.close(Some("write timeout".to_string()))
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

//EventHandler后面接具体的ctx类型
#[async_trait]
pub trait EventHandler<T: ConnTraitT> {
    //连接打开之后,返回的第一个值如果是Some,则会给客户端发消息
    async fn on_opened(&mut self, ctx: &mut T) -> Result<(), NetError>;
    //从decode里面读出来的一个数据，建议在decode里面对conn使用next或者shift清掉数据，当然Vec<u8>为空也可以
    //返回Some([u8])则会写出消息
    async fn react(&mut self, frame: Vec<u8>, tx: &mut T) -> Result<(), NetError>;

    async fn on_closed(&mut self, tx: &mut T, reasion: Option<String>) -> Result<(), NetError>;
}
