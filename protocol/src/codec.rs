use crate::async_trait;
use crate::AsyncReadExt;
use crate::AsyncWriteExt;
use crate::NetError;
use net::codec::CodeC;
use net::Conn;
use net::EventHandler;
use net::conn::{ConnTraitT};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct rpcNode{}
#[derive(Copy, Clone)]
pub struct Codec {}
/*#[async_trait]
impl CodeC<rpcNode> for Codec {
    async fn encode(&mut self, indata: Vec<u8>,conn: &mut ConnRead<rpcNode> ) -> Result<Option<Vec<u8>>, NetError> {
        Ok(Some(indata))
    }
    async fn decode(&mut self, conn: &mut rpcNode) -> Result<Option<Vec<u8>>, NetError> {
        /*match conn.bufferlen(){
            n if n>4=>{
                let len=conn.next(4).await?;
                let len=(len[0] as usize) +((len[1] as usize )<<8)+((len[2] as usize)<<16)+((len[3] as usize )<<24);
                Ok(Some(conn.next(len).await?))
            },
            _=>Ok(None),
        }*/
        Ok(None)
    }
}*/
