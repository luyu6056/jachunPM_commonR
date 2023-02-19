#![feature(async_closure)]

mod host;


use codec::http1::{Http1Codec, Request};
use crate::host::handler::Handler;
use net::conn::Conn;
use net::{ Server};
use std::convert::Infallible;
use std::net::SocketAddr;
use std::ptr::NonNull;
use std::time::Duration;
use hyper::{Body, Server as S};
use hyper::Request as Req;
use hyper::Response as Resp;
use hyper::service::{make_service_fn, service_fn};
use net::err::NetError;
use net::buffer::MsgBuffer;
#[tokio::main]
async fn main() {

    //net::start_server::<Handler, Codec, RpcServer>("0.0.0.0:40000", Codec {}, Handler {}).await.unwrap();
    //net::start_server::<Handler, DefaultCodec, RpcServer>("0.0.0.0:40000", DefaultCodec {}, Handler {}).await.unwrap();

    tokio::spawn(async move{
        let mut server=Server::new_with_codec("0.0.0.0:81",Http1Codec{},Handler{}).with_buf_pool_num(20).with_default_buf_size(20).with_max_buf_size(10).with_readtimeout(Duration::from_secs(10)).with_writetimeout(Duration::from_secs(10));
        server.start_server().await;
    });

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    // A `Service` is needed for every connection, so this
    // creates one from our `hello_world` function.
    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(hello_world))
    });

    let server = S::bind(&addr).serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }

}
#[derive(Clone)]
struct RpcServer{
    server_no: u8, //服务序号
    id: i16,       //服务Id，有效值0-255
    /*conn: NonNull<Conn<RpcServer>>, ServerConn           gnet.Conn
                   encodebuf, decodebuf *[]byte //codec解压相关
                   setStatusOpenChan    chan string
                   closeChan            chan int
                   outChan              chan *libraries.MsgBuffer //指定本服务接收的消息
                   inChan               chan *protocol.Msg
                   startTime, busyTime  int64 //时间统计
                   Ip                   string
                   pongTime             int64
                   status               int
                   isCenter             bool   //是否中心服
                   window               int32  //发送窗口
                   local                uint16 //ServerNo与Id编码后的数值
                   CacheServer          *RpcServer
                   handlerFunc          *ants.PoolWithFunc
                   isDebug              bool //测试，会抢夺*/
}
impl RpcServer {
    /*pub fn new(conn: Conn) -> Self {
        RpcServer {
            server_no: 0,
            id: 0,
            //conn: NonNull::dangling(),
        }
    }*/
    pub async fn close(&self, reasion: Option<String>) {
        //self.conn.close(reasion).await;
    }

}

async fn hello_world(_req: Req<Body>) -> Result<Resp<Body>, Infallible> {
    Ok(Resp::new("Hello, World".into()))
}