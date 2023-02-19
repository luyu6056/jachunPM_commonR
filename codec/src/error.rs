use thiserror::Error;
use crate::NetError;

#[derive(Error, Debug, Clone)]
pub enum HttpError {
    #[error("Internal error.")]
    Internal(String),
    #[error("Not found.")]
    NotFound,
    #[error("Permission Denied.")]
    PermissionDenied,
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("IoError: {0}")]
    IoError(String),
    #[error("http2 raw not support.")]
    Http2RawNotSopport,
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

    #[error("Too_Many_Redirects")]
    TooManyRedirects,

    #[error("package is too large")]
    LargePackage,
}
impl HttpError {
    pub fn new_with_string(s: String) -> HttpError {
        HttpError::Custom(s)
    }
}
impl std::convert::From<std::num::ParseIntError> for HttpError {
    fn from(err: std::num::ParseIntError) -> Self {
        HttpError::InvalidArgument(err.to_string())
    }
}

impl std::convert::From<std::io::Error> for HttpError {
    fn from(err: std::io::Error) -> Self {
        HttpError::IoError(err.to_string())
    }
}

impl std::convert::From<std::string::FromUtf8Error> for HttpError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        HttpError::InvalidArgument(err.to_string())
    }
}
impl std::convert::From<native_tls::Error> for HttpError {
    fn from(err: native_tls::Error) -> Self {
        HttpError::IoError(err.to_string())
    }
}

impl std::convert::From<NetError> for HttpError {
    fn from(err: NetError) -> Self {
        HttpError::IoError(err.to_string())
    }
}

impl std::convert::From<HttpError> for NetError {
    fn from(err: HttpError) -> Self {
        NetError::Custom(err.to_string())
    }
}