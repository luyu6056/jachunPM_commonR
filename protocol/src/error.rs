use thiserror::Error;
use tokio::time::error::Elapsed;
#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("UknowError")]
    UknowError,
    #[error("消息长度不够解析一条msg")]
    errMsgLen,
    #[error("消息长度不够解析data")]
    errMsgDataLen,
    #[error("{0}")]
    Custom(String),
}
impl Error {
    pub fn new_with_string(s: String) -> Error {
        Error::Custom(s)
    }
}
