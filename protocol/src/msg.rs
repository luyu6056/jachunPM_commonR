use crate::error::Error;
use net::err::NetError;

const MsgHeadLen: usize = 4 + 2 + 2 + 2 + 4 + 4 + 4 + 3; //不包含cmd
const MaxMsgLen: usize = (2 << 24) - 1 - MsgHeadLen;
const MaxMsgTtl: usize = 1000; //目前只允许查询1000次

pub struct msg {
    datalen:usize,
    msgNo:u32,
    ttlTimeout:u16,
    local:u16,
    remoteID:u16,
    db:MsgDB,
    queryID:u32,
    uid:u32,
    pub cmd:u32,
}
pub struct MsgDB{
    transactionNo:u32,
}
pub fn read_one_msg_from_bytes(data: &[u8]) -> Result<(usize,msg), Error> {
    if data.len() < MaxMsgLen {
        return Err(Error::errMsgLen);
    }
    let datalen = data[MsgHeadLen-3] as usize | (data[MsgHeadLen-2] as usize) <<8 | (data[MsgHeadLen-1] as usize)<<16;
    if data.len() < MsgHeadLen+datalen {
        return Err(Error::errMsgDataLen);
    }
    Ok((datalen,msg{
        datalen,
        msgNo:data[0]as u32 | (data[1] as u32)<<8 | (data[2] as u32)<<16 | (data[3] as u32)<<24,
        ttlTimeout:(data[4] as u16) | (data[5] as u16)<<8,
        local:(data[6] as u16) | (data[7] as u16)<<8,
        remoteID:(data[8] as u16) | (data[9] as u16)<<8,
        db:MsgDB{
            transactionNo:(data[10]as u32) | (data[11] as u32)<<8 | (data[12] as u32)<<16 | (data[13] as u32)<<24,
        },
        queryID:(data[14]as u32) | (data[15] as u32)<<8 | (data[16] as u32)<<16 | (data[17] as u32)<<24,
        uid:(data[18]as u32) | (data[19] as u32)<<8 | (data[20] as u32)<<16 | (data[21] as u32)<<24,
        cmd:(data[MsgHeadLen]as u32) | (data[MsgHeadLen+1] as u32)<<8 | (data[MsgHeadLen+2] as u32)<<16 | (data[MsgHeadLen+3] as u32)<<24,
    }))




    //msg.DB.msg = msg

    //msg.buf = buf
}
