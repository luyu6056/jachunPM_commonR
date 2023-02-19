use std::time::SystemTimeError;
use thiserror::Error;
use tokio::time::error::Elapsed;
#[derive(Error, Debug, Clone)]
pub enum NetError {
    #[error("IoError: {0}")]
    IoError(String),

    #[error("{0}")]
    Custom(String),

    #[error("tcp read timeout")]
    TcpReadTimeout,

    #[error("tcp disconnected")]
    TcpDisconnected,

    #[error("UknowError")]
    UknowError,

    #[error("{0} Address error")]
    AddressError(String),

    #[error("package is too large")]
    LargePackage,

    #[error("ShutdownServer,reason: {0}")]
    ShutdownServer(String),

    #[error("no error")]
    None,

    #[error("close no error")]
    Close,
}
impl NetError {
    pub fn new_with_string(s: String) -> NetError {
        NetError::Custom(s)
    }
}
impl std::convert::From<std::io::Error> for NetError {
    fn from(err: std::io::Error) -> Self {
        NetError::IoError(err.to_string())
    }
}
impl std::convert::From<Elapsed> for NetError {
    fn from(err: Elapsed) -> Self {
        NetError::Custom(err.to_string())
    }
}
impl<T> std::convert::From<tokio::sync::mpsc::error::SendError<T>> for NetError {
    fn from(err: tokio::sync::mpsc::error::SendError<T>) -> Self {
        NetError::Custom(err.to_string())
    }
}
impl<T> std::convert::From<tokio::sync::mpsc::error::TrySendError<T>> for NetError {
    fn from(err:tokio::sync::mpsc::error::TrySendError<T>) -> Self {
        NetError::Custom(err.to_string())
    }
}
impl std::convert::From<async_channel::RecvError> for NetError {
    fn from(err: async_channel::RecvError) -> Self {
        NetError::Custom(err.to_string())
    }
}
impl std::convert::From<SystemTimeError> for NetError {
    fn from(err: SystemTimeError) -> Self {
        NetError::Custom(err.to_string())
    }
}
