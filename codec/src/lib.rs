#![feature(let_chains)]

pub mod http1;
pub mod error;
pub mod response;
use net::conn::{ConnTraitT,ConnTraitS,Conn};
use net::codec::CodeC;
use net::err::NetError;
use async_trait::async_trait;