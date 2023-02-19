use crate::error::HttpError;
use crate::http1::Request;
use crate::http1::ContentType;
use std::collections::HashMap;
use std::io;
use std::io::prelude::*;
use std::ptr::NonNull;
use crate::{ConnTraitT,ConnTraitS,Conn,CodeC,async_trait,NetError};
use flate2::read::GzDecoder;


//#[derive(Debug)]
pub struct Response<'a> {
    pub code: u16,
    request : &'a mut Request<'a>,
    body_length:ContentType,
}
impl<'a>  Response<'a>
{
    pub fn from_request(req: &'a mut Request<'a>) -> Result<Response<'a>, HttpError> {

        let  resp = Response {
            code: req.code,
            body_length:req.content_length,
            request:req,
        };
        Ok(resp)
    }
    pub async fn read_by_maxsize(&mut self,maxsize:usize)->Result<Vec<u8>,HttpError>{
        /*let res = match self.body_length {
            ContentType::Length(size) if size<=maxsize => {
                self.stream.next(size).await?
            },
            ContentType::Chunked => {
                let mut res=Vec::new();
                loop {
                    let buf =  self.stream.readline().await?;
                    let len = u32::from_str_radix(String::from_utf8(buf.clone())?.as_str(), 16)?;
                    if len == 0 {
                        self.stream.readline().await?;
                        break
                    } else {
                        res.append(&mut self.stream.next(len as usize).await?);
                        self.stream.shift(2).await?;
                    }
                }
                res
            }
            _ => {
                return Err(HttpError::Custom("err content_length".to_string()));
            }
        };

        match self.herder.get("content-encoding") {
            Some(code)=>{
                match code.as_str() {
                    "gzip"=>{
                        let d = GzDecoder::new(&res[..]);
                        let mut r = io::BufReader::new(d);
                        let mut buffer = Vec::new();

                        // read the whole file
                        r.read_to_end(&mut buffer)?;
                        return Ok(buffer)
                    },
                    s=>{
                        return Err(HttpError::Custom(format!("未支持的 content-encoding {}",s)))
                    }
                }
            },
            None=> return Ok(res)
        }*/
        Err(HttpError::UknowError)
    }
    pub async fn writestring(&mut self,str:&str)->Result<(),NetError>{
        /*let r=self.stream;
        r.out.Reset()
        if r.outCode != 0 && httpCode(r.outCode).Bytes() != nil {
            r.out.Write(httpCode(r.outCode).Bytes())
        } else {
            r.out.Write(http1head200)
        }
        r.out.Write(http1nocache)
        if r.outContentType != "" {
            r.out.WriteString("content-Type: ")
            r.out.WriteString(r.outContentType)
            r.out.WriteString("\r\n")
        } else {
            r.out.Write([]byte("content-Type: text/html;charset=utf-8\r\n"))
        }
        r.out1.Reset()
        if len(b) > 9192 && strings.Contains(r.Header("accept-encoding"), "deflate") {
            w := CompressNoContextTakeover(r.out1, 6)
            w.Write(b)
            w.Close()
            r.out.Write(http1deflate)
        } else {
            r.out1.Write(b)
        }
        r.data = r.out1
        r.dataSize = r.out1.Len()
        r.Status = requestStatusNormalWrite
        r.finish()*/
        Ok(())
    }
}
