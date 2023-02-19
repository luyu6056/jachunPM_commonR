use crate::error::HttpError;
use crate::response::Response;
use crate::{async_trait, CodeC, Conn, ConnTraitT, NetError};

use std::borrow::Borrow;
use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Debug;
use std::io::{BufRead, BufWriter, Write};
use std::net::ToSocketAddrs;
use std::ops;
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use url::quirks::protocol;
use url::Url;
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ContentType {
    Length(usize),
    Chunked,
    None,
}
#[derive(Debug, Clone)]
pub struct Request<'a> {
    proto: &'a str, //协议
    uri: &'a str,
    method: &'a str,
    client_method: &'a str,
    path: &'a str,
    keep_alive: bool,
    query: String,
    pub content_length: ContentType,
    pub header: HashMap<String, &'a str>,
    err: String,
    pub host: String,
    port: u16,
    pub(crate) code: u16,
    max_redirects: u8, //重定向次数，默认5，最大100
    redirects: u8,
    request_host: String,
    max_package_size: usize, //最大数据包
    post_body: String,
    is_https: bool,
    body_start: usize, //body起始位
    pub conn: NonNull<Conn<Request<'a>>>,

    //buf: BufWriter<>Vec<u8>,
    read_timeout: Duration, //默认10秒超时
}
unsafe impl Send for Request<'_> {}
unsafe impl Sync for Request<'_> {}

impl <'a> Default for Request<'a> {
    fn default() -> Self {
        Request::new("")
    }
}
impl<'a> Request<'a> {
    pub fn new(url: &str) -> Request<'a> {
        let mut req = Request {
            body_start: 0,
            proto: "",
            method: "",
            client_method: "GET",
            path: "/",

            query: "".to_string(),
            uri: "",
            keep_alive: false,
            content_length: ContentType::None,
            header: Default::default(),
            err: "".to_string(),
            host: "".to_string(),
            port: 0,
            code: 0,
            max_redirects: 5,
            redirects: 0,
            request_host: "".to_string(),
            max_package_size: 10 * 1024 * 1024,
            post_body: "".to_string(),
            is_https: false,
            conn: NonNull::dangling(),
            //buf: Vec::new(),
            read_timeout: Duration::from_secs(10),
        };
        req.url_parse(url);
        //req.header.insert("User-Agent","Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.1 Safari/605.1.15");
        req
    }
    pub fn with_readtimeout(mut self, timeout: Duration) -> Self {
        self.read_timeout = timeout;
        self
    }
    pub fn set_max_redirects(&'a mut self, n: u8) -> &Request {
        if n > 100 {
            self.max_redirects = 100
        } else {
            self.max_redirects = n
        }
        self
    }
    fn format_message(&self) -> String {
        let mut new_herder = "".to_string();
        for (k, v) in &self.header {
            new_herder = new_herder + k + ": " + v + "\r\n";
        }
        if self.post_body != "" {
            new_herder =
                new_herder + "content-length: " + self.post_body.len().to_string().as_str();
        }

        format!("{method} {path} HTTP/1.1\r\nAccept-Language: zh-CN,zh-Hans;q=0.9\r\nAccept-Encoding: gzip, deflate\r\nHost: {host}\r\nConnection: keep-alive\r\n{header}\r\n\r\n{body}",method=self.client_method,path=self.path,host=self.request_host,header=new_herder,body=self.post_body)
    }
    /*pub async fn exec_by_method<S: ConnS, T: ConnT + 'a>(
        &mut self,
        method: &str,
        content_type: Option<&str>,
        body: &str,
    ) -> Result<Response<'a, S, T>, HttpError> {
        self.client_method = method.to_string();
        if let Some(content_type) = content_type {
            self.header
                .insert("Content-Type".to_string(), content_type.to_string());
        }
        self.post_body = body.to_string();

        self.exec().await
    }
    pub async fn doexec<S: ConnS, T: ConnT + 'a>(
        &'a mut self,
        stream: &'a mut ConnRead<'a, S, T>,
    ) -> Result<Response<'a, S, T>, HttpError> {
        self.do_http1raw_request(self.format_message().as_bytes(), stream)
            .await
    }
    pub async fn exec<S: ConnS, T: ConnT + 'a>(&mut self) -> Result<Response<'a, S, T>, HttpError> {
        let mut addrs_iter = format!("{}:{}", self.host, self.port).to_socket_addrs()?;
        match addrs_iter.next() {
            Some(addr) => {
                /*let  stream =Box::new(HttpTcpStream::new_with_addr(addr,self.is_https,&self.host).await?) ;
                let resp=self.doexec( stream).await?;
                return Ok(resp);*/
                return Err(HttpError::UknowError);
            }
            None => {
                return Err(HttpError::AddressError(format!(
                    "{}:{}",
                    self.host, self.port
                )))
            }
        }
    }
    pub fn get(url: &str) -> Request {
        let mut req = Request::new(url);
        req.client_method = "GET".to_string();

        req
    }
    pub fn post(url: &str, content_type: Option<&str>, body: &str) -> Request<'a,T> {
        let mut req = Request::new(url);
        req.client_method = "POST".to_string();

        if let Some(content_type) = content_type {
            req.header
                .insert("Content-Type".to_string(), content_type.to_string());
        }
        req.post_body = body.to_string();
        req
    }
    pub fn do_by_method(
        url: &str,
        method: String,
        content_type: Option<&str>,
        body: &str,
    ) -> Request<'a,T> {
        let mut req = Request::new(url);
        req.client_method = method;
        if let Some(content_type) = content_type {
            req.header
                .insert("Content-Type".to_string(), content_type.to_string());
        }
        req.post_body = body.to_string();
        req
    }*/

    pub fn url_parse(&mut self, url: &str) {
        /*unsafe {
            self.buf.truncate(0);
            match Url::parse(url) {
                Ok(u) => {
                    self.buf.extend_from_slice(u.path().as_bytes());
                    self.path = std::str::from_utf8_unchecked(&self.buf[0..self.buf.len()]);
                    if u.has_host() {
                        self.host = u.host_str().unwrap().to_string();
                    }
                    self.request_host = match u.port() {
                        Some(p) => format!("{}:{}", self.host, p),
                        None => self.host.clone(),
                    };
                    match u.port_or_known_default() {
                        Some(port) => self.port = port,
                        None => self.err = "port 不存在".to_string(),
                    }
                    if u.scheme() == "https" {
                        self.is_https = true
                    }
                }
                Err(e) => {
                    self.err = e.to_string();
                }
            };
        }*/
    }

    /*pub async fn do_http1raw_request<S: ConnS, T: ConnT + 'a>(
        &'a mut self,
        rawdata: &[u8],
        mut stream: &'a mut ConnRead<'a, S, T>,
    ) -> Result<Response<'a, S, T>, HttpError> {
        stream.write(rawdata.to_vec()).await?;
        Request::parsereq_from_stream(self, stream).await?;
        Response::from_request(self.clone(), stream)
    }*/
    //读一个消息，可以是client，可以是server
    async fn parsereq_from_stream(&mut self) -> Result<(), HttpError> {
        unsafe {
            loop {
                self.next_from_stream().await?;
                match self.code {
                    301 | 302 | 307 => {
                        self.read_finish().await?;
                        match self.max_redirects {
                            0 => return Ok(()),
                            _ => {
                                if self.redirects >= self.max_redirects {
                                    return Err(HttpError::TooManyRedirects);
                                }
                            }
                        }

                        match self.header.get("Location") {
                            Some(a) => {
                                self.path = a;
                                let rawdata = self.format_message();
                                self.conn.as_mut().write_data(rawdata.as_bytes()).await?;
                            }
                            None => {
                                return Err(HttpError::Custom(
                                    "Location url not found".to_string(),
                                ));
                            }
                        }
                    }
                    _ => {
                        self.conn.as_mut().shift(self.body_start).await?;
                        return Ok(());
                    }
                }
            }
        }
    }
    /*fn from_data(conn: &mut ConnRead) -> Result<Request<'static>, HttpError> {
        let mut req = Request::new("");
        req.conn = conn.ptr;

        Ok(req)
    }*/
    async fn next_from_stream(&mut self) -> Result<(), HttpError> {
        /*unsafe {
            let mut stream = self.conn.as_mut();
            self.content_length = ContentType::None;
            self.secondline = 0;
            self.header.clear();

            while self.content_length == ContentType::None {
                if stream.bufferlen() == 0 {
                    stream.read_msg().await?;
                }

                let l = stream.bufferlen();

                let sdata = String::from_utf8_unchecked(stream.bufferdata().to_vec());

                if self.secondline == 0 {
                    let method = match &sdata.find(' ') {
                        Some(i) => &sdata[..i.to_owned()],
                        None => continue,
                    };

                    let mut path = "";
                    let mut query = "";
                    let mut uri = "";
                    let mut proto = None;
                    let mut s = 0;
                    {
                        let mut q: i32 = -1;

                        let sdata = &sdata[method.len() + 1..];
                        for (i, char) in sdata.bytes().enumerate() {
                            if char == 63 && q == -1 {
                                q = i as i32;
                            } else if char == 32 {
                                if q != -1 {
                                    path = &sdata[s..(q as usize)];
                                    query = &sdata[(q as usize) + 1..i];
                                } else {
                                    path = &sdata[s..i];
                                }
                                uri = &sdata[s..i];

                                if let Some(f) = sdata.find(13 as char) {
                                    s = f;
                                    proto = Some(&sdata[i + 1..f])
                                }

                                //判断http返回
                                if method == "HTTP/1.1" || method == "HTTP/1.0" {
                                    match path.parse::<u16>() {
                                        Ok(code) => self.code = code,
                                        Err(_) => continue,
                                    }
                                    /*code, err := strconv.Atoi(req.path)
                                    if err == nil {
                                        //req.Code = code
                                        //req.CodeMsg = req.Proto
                                    }*/

                                    proto = Some(method);
                                    path = "";
                                }

                                break;
                            }
                        }
                    }

                    (self.proto, self.keep_alive) = match proto {
                        Some("HTTP/1.0") => ("HTTP/1.0".to_string(), false),
                        Some("HTTP/1.1") => ("HTTP/1.0".to_string(), true),
                        None => {
                            if sdata.len() > self.max_package_size {
                                return Err(HttpError::LargePackage);
                            };
                            ("".to_string(), self.keep_alive)
                        }
                        _ => continue,
                    };

                    self.uri = uri.to_string();
                    self.query = query.to_string();
                    self.path = path.to_string();
                    self.secondline = s + method.len() + 1;
                }

                let mut s = self.secondline;
                let mut i = 0;
                while s < l {
                    s += i + 2;

                    match sdata[s..].find(13 as char) {
                        Some(f) => {
                            i = f;
                            if i > 0 {
                                let line = &sdata[s..s + i];
                                match line.find(58 as char) {
                                    Some(f) => {
                                        self.header.insert(
                                            line[..f].to_lowercase(),
                                            line[f + 2..].to_string(),
                                        );
                                    }
                                    None => {
                                        self.content_length = ContentType::None;
                                        break;
                                    }
                                }
                            } else {
                                if let Some(keep_alive) = self.header.get("connection") {
                                    match keep_alive.as_str() {
                                        "close" => self.keep_alive = false,
                                        "keep-alive" => self.keep_alive = true,
                                        _ => (),
                                    }
                                }
                                if let Some(len) = self.header.get("content-length") {
                                    self.content_length =
                                        ContentType::Length(len.parse::<usize>()?);
                                }else if let Some(h) = self.header.get("transfer-encoding") && h == "chunked" {
                                        self.content_length = ContentType::Chunked;
                                }else{
                                    self.content_length=ContentType::Length(0);
                                }
                                self.body_start = s + 2;

                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }*/

        Ok(())
    }
    pub async fn read_finish(&mut self) -> Result<(), HttpError> {
        unsafe {
            //println!("{:?},{:?}",self.body_start,self.conn.as_ref().bufferdata().len());
            let mut stream = self.conn.as_mut();
            match self.content_length {
                ContentType::Length(size) => stream.shift(self.body_start + size).await?,
                ContentType::Chunked => {
                    loop {
                        stream.shift(self.body_start).await?;
                        let buf = stream.readline().await?;
                        let len = u32::from_str_radix(buf.as_str(), 16)?;
                        if len == 0 {
                            stream.readline().await?;
                            //stream.buffer=Vec::new();
                            break;
                        } else {
                            stream.shift((len + 2) as usize).await?
                        }
                    }
                }
                _ => {
                    return Err(HttpError::Custom("err content_length".to_string()));
                }
            }
        }

        Ok(())
    }
    fn readline(data: &[u8]) {}
    async fn check_data(&mut self) -> Result<(), NetError> {
        unsafe {
            let conn = self.conn.as_mut();
            let data: &[u8] = self.conn.as_mut().bufferdata();

            let mut iter = data.iter().enumerate();
            let mut pos = 0;
            self.header.clear();


            loop {
                match iter.next() {
                    Some((k, v1)) => {
                        if *v1==13 && let Some((_,v2))=iter.next() && *v2==10 {
                            //先处理到第一行
                            let mut strs=data[pos..k].split(|num|num==&32);
                            pos=k+2;
                            if let (Some(method),Some(path),Some(proto))=(strs.next(),strs.next(),strs.next()){
                                self.method=std::str::from_utf8_unchecked(method);
                                self.path=std::str::from_utf8_unchecked(path);
                                self.proto=std::str::from_utf8_unchecked(proto);
                                loop  {
                                    match iter.next(){
                                        Some((k,v1))=>{
                                            if *v1==13 && let Some((_,v2))=iter.next() && *v2==10 {
                                                if pos==k{
                                                    self.body_start=k+2;
                                                    self.content_length=ContentType::Length(0);
                                                    return Ok(());
                                                }else{
                                                    let mut strs=data[pos..k].split(|num|num==&58);
                                                    if let (Some(key),Some(val))=(strs.next(),strs.next()){
                                                        self.header.insert(std::str::from_utf8_unchecked(key).to_lowercase(),std::str::from_utf8_unchecked(val));
                                                    }

                                                    pos=k+2;
                                                }

                                            }
                                        },
                                        None=>conn.read_data().await?
                                    }

                                }
                            }

                        }
                    }
                    None => conn.read_data().await?,
                }
            }
        }
       // Err(NetError::UknowError)
    }
    fn get_conn(&'a mut self) -> &mut Conn<Request<'_>> {
        unsafe { return self.conn.as_mut() }
    }
}

impl<'a> ConnTraitT for Request<'a> {
    fn set_conn(& mut self, conn: NonNull<Conn<Request<'a>>>) {
        self.conn = conn
    }
    fn get_conn(&self)->NonNull<Conn<Self>>{
        self.conn
    }
}
#[derive(Copy, Clone)]
pub struct Http1Codec {}
#[async_trait]
impl CodeC<Request<'_>> for Http1Codec {
    async fn encode(
        &mut self,
        indata: Vec<u8>,
        ctx: &mut Request,
    ) -> Result<Option<Vec<u8>>, NetError> {
        Ok(Some(indata))
    }
    async fn decode(&mut self, req: &mut Request) -> Result<Option<Vec<u8>>, NetError> {
        req.check_data().await?;
        return Ok(Some(Vec::new()));
    }
}
