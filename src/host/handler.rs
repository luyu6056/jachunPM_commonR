use async_trait::async_trait;
use net::err::NetError;
use net::{ EventHandler};
use protocol::msg::read_one_msg_from_bytes;

use std::time::Duration;

use codec::http1::Request;

#[derive(Copy, Clone)]
pub struct Handler {}
#[async_trait]
/*impl EventHandler<RpcServer> for Handler {
    async fn on_opened(
        &mut self,
        conn: &mut  RpcServer ,
    ) -> Result<(Option<Vec<u8>>, Action), NetError> {



        /*let srv = RpcServer::new(conn.getConnWrite());
        conn.setContext(srv);
        conn.after_fn(
            Duration::from_secs(10),
            |conn1: &ConnRead<S, RpcServer>| -> ConnAsyncResult {
                Box::pin(async move {
                    conn1.close(None);
                    Ok(())
                })
            },
        );*/
        return Ok((None, Action::None));
    }
    async fn react(
        &mut self,
        data: Vec<u8>,
        conn:&mut  RpcServer
    ) -> Result<(Option<Vec<u8>>, Action), NetError> {

        return Ok((Some(data),Action::None));
        let mut index = 0;
        while index < data.len() {
            match read_one_msg_from_bytes(&data[index..]) {
                Ok((l, msg)) => {
                    index += l;
                    if msg.cmd == protocol::host::CMD_MSG_HOST_regServer {}
                }
                Err(e) => println!("{:?}", e),
            }
            return Ok((None, Action::None));
        }

        Ok((None, Action::None))
    }
    async fn on_closed(
        &mut self,
        conn: &mut  RpcServer,
        reasion: Option<String>,
    ) -> Result<(Option<Vec<u8>>, Action), NetError> {

            //ctx.close(reasion).await;

        Ok((None, Action::None))
    }
}*/
#[async_trait]
impl EventHandler<Request<'_>> for Handler {
    async fn on_opened(
        &mut self,
        req: &mut Request ,
    ) -> Result<(), NetError> {

        return Ok(());
    }
    async fn react(
        &mut self,
        data: Vec<u8>,
        req:&mut  Request
    ) -> Result<(), NetError> {

        let b="HTTP/1.1 200 OK\r\ncontent-length:0\r\nconnection:keep-alive\r\n\r\n";
        //req.conn.as_mut().write(b.as_bytes().to_vec()).await?;

        req.read_finish().await?;

        unsafe {
            let conn=req.conn.as_mut();
            conn.write_data(b.as_bytes()).await?;
        }

        return Ok(());

    }
    async fn on_closed(
        &mut self,
        _ctx: &mut  Request,
        reasion: Option<String>,
    ) -> Result<(), NetError> {

        //ctx.close(reasion).await;
        Ok(())
    }
}