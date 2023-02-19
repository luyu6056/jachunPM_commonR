//封装一个buffer接口，api来自我的go项目
use crate::{Sender,Receiver,Arc,Mutex};
use std::thread::sleep;

const TEST_MAX_BUF_SIZE: usize = 1 * 16; //清理失效数据阈值
const TEST_DEFAULT_BUF_SIZE: usize = 1 * 8; //初始化大小
const MAX_BUF_SIZE: usize = 1024 * 64; //清理失效数据阈值
const DEFAULT_BUF_SIZE: usize = 1024 * 8; //初始化大小

/*##[derive(Debug, Clone)]
pub struct MsgBuffer { //纯Vec实现
    max_buf_size: usize,
    default_buf_size: usize,
    b: Vec<u8>,
    pos: usize, //起点位置
                //l: usize,   //长度以b.len()替代
}

impl Default for MsgBuffer {
    fn default() -> Self {
        new()
    }
}
pub fn new() -> MsgBuffer {
    MsgBuffer {
        max_buf_size: MAX_BUF_SIZE,
        default_buf_size: DEFAULT_BUF_SIZE,
        b: Vec::with_capacity(DEFAULT_BUF_SIZE),
        pos: 0,
    }
}
impl MsgBuffer {

    //从一个vec创建
    pub fn new_from_vec(b: Vec<u8>,max_buf_size:usize,default_buf_size:usize) -> Self {
        MsgBuffer {
            max_buf_size: max_buf_size,
            default_buf_size: default_buf_size,
            b: b,
            pos: 0,
        }
    }

    #[cfg(test)]
    fn test_new() -> Self {
        MsgBuffer {
            max_buf_size: TEST_MAX_BUF_SIZE,
            default_buf_size: TEST_DEFAULT_BUF_SIZE,
            b: Vec::with_capacity(TEST_DEFAULT_BUF_SIZE),
            pos: 0,
        }
    }

    pub fn reset(&mut self) {
        if self.pos > self.max_buf_size {
            #[cfg(test)]
            println!("reset re_size");
            self.re_size();
        }
        self.pos = 0;
        unsafe {
            self.b.set_len(0);
        }
    }

    pub fn make(&mut self, l: usize) -> &mut [u8] {
        if self.pos > self.max_buf_size {
            #[cfg(test)]
            println!("make re_size");
            self.re_size();
        }
        let newlen = self.b.len() + l;

        if self.b.capacity() < newlen {
            self.b.reserve(l);
        }
        unsafe { self.b.set_len(newlen) }
        return &mut self.b[newlen - l..];
    }

    //重新整理内存
    fn re_size(&mut self) {
        unsafe {
            if self.pos >= self.max_buf_size {
                let oldlen = self.b.len() - self.pos; //原始长度
                let dst = self.b.as_mut_ptr();
                std::ptr::copy(self.b[self.pos..].as_ptr(), dst, self.b.len() - self.pos);
                self.b.set_len(oldlen);
                self.b.shrink_to(self.max_buf_size);
                self.pos = 0;
            }
        }
    }

    pub fn write(&mut self, b: &[u8]) {
        self.b.extend_from_slice(b);
    }

    pub fn len(&self) -> usize {
        return self.b.len() - self.pos;
    }
    pub fn shift(&mut self, len: usize) {
        if len <= 0 {
            return;
        }
        //#[cfg(test)]
        //println!("before shift,{},{},{},{}",len,self.pos,self.len(),self.b.capacity());
        if len < self.len() {
            self.pos += len;
            if self.pos > self.max_buf_size {
                #[cfg(test)]
                println!("shift re_size");
                self.re_size();
            }
        } else {
            self.reset();
        }
        //#[cfg(test)]
        //println!(" after shift,{},{},{},{}",len,self.pos,self.len(),self.b.capacity());
    }
    pub fn as_slice(&self) -> &[u8] {
        return &self.b[self.pos..];
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        return &mut self.b[self.pos..];
    }
    pub fn bytes(&self) -> &[u8] {
        return &self.b[self.pos..];
    }
    pub fn write_string(&mut self, s: &str) {
        self.write(s.as_bytes())
    }
    pub fn write_byte(&mut self, s: u8) {
        if self.pos > self.max_buf_size {
            self.re_size();
        }
        self.b.push(s);
    }
    pub fn truncate(&mut self, len: usize) {
        self.b.truncate(self.pos + len);
    }
}*/
//Box<[u8]>实现
pub struct MsgBuffer {
    b: Box<[u8]>, //指向raw或者其他地方的裸指针
    max_buf_size: usize,
    default_buf_size: usize,
    pos: usize,   //起点位置
    l: usize,     //长度以b.len()替代
}
impl Default for MsgBuffer {
    fn default() -> Self {
        new()
    }
}
pub fn new() -> MsgBuffer {
    let mut v: Vec<u8> = vec![0; DEFAULT_BUF_SIZE];
    let mut buf = MsgBuffer {
        b: v.into_boxed_slice(),
        max_buf_size: MAX_BUF_SIZE,
        default_buf_size: DEFAULT_BUF_SIZE,
        pos: 0,
        l: 0,
    };
    buf
}
impl MsgBuffer {



    #[cfg(test)]
    fn test_new() -> Self {
        let mut v: Vec<u8> = vec![0; TEST_DEFAULT_BUF_SIZE];
        let b = v.into_boxed_slice();
        let mut buf = MsgBuffer {
            b: b,
            max_buf_size: TEST_MAX_BUF_SIZE,
            default_buf_size: TEST_DEFAULT_BUF_SIZE,
            pos: 0,
            l: 0,

        };
        buf
    }
    fn grow(&mut self, n: usize) {
        unsafe {

            let len = self.b.len() * 2;
            if len > n {
                let mut v = vec![0; len];
                std::ptr::copy(self.b[self.pos..].as_ptr(), v.as_mut_ptr(), self.b.len() - self.pos);
                self.b = v.into_boxed_slice();
            } else {
                let mut v = vec![0; n];
                std::ptr::copy(self.b[self.pos..].as_ptr(), v.as_mut_ptr(), self.b.len() - self.pos);
                self.b = v.into_boxed_slice();
            }

        }
    }
    pub fn reset(&mut self) {
        if self.pos > self.max_buf_size {
            #[cfg(test)]
            println!("reset re_size");
            self.re_size();
        }
        self.pos = 0;
        self.l = 0;
    }

    pub fn make(&mut self, l: usize) -> &mut [u8] {
        if self.pos > self.max_buf_size {
            self.re_size();
        }
        self.l += l;

        if self.l > self.b.len() {
            self.grow(l);
            //self.b.reserve(l);
        }
        //unsafe { self.b.set_len(newlen) }

        return &mut self.b[self.l - l..self.l];
    }
    pub fn spare(&mut self, l: usize) -> &mut [u8] {
        if self.pos > self.max_buf_size {
            self.re_size();
        }
        let newlen=self.l + l;

        if newlen > self.b.len() {
            self.grow(l);

        }


        return &mut self.b[self.l ..newlen];
    }
    pub fn string(&self)->String{
        unsafe {
            return String::from_utf8_unchecked(self.bytes().to_vec())
        }

    }
    //重新整理内存
    fn re_size(&mut self) {
        unsafe {
            if self.pos >= self.max_buf_size {

                let len = self.l - self.pos; //原始长度
                let newlen = if len > self.max_buf_size {
                    len
                } else {
                    self.max_buf_size
                };
                let mut v = vec![0; newlen];
                let dst = v.as_mut_ptr();
                std::ptr::copy(self.b[self.pos..].as_ptr(), dst, self.b.len() - self.pos);
                self.l = len;
                self.b = v.into_boxed_slice();
                self.pos = 0;

            }
        }
    }

    pub fn write(&mut self, b: &[u8]) {
        if self.pos > self.max_buf_size {
            self.re_size()
        }
        self.l += b.len();
        unsafe {
            if self.l > self.b.len() {
                self.grow(b.len());
            }
            std::ptr::copy(b.as_ptr(), self.b[self.l - b.len()..].as_mut_ptr(), b.len());
        }

        //self.b.extend_from_slice(b);
    }

    pub fn len(&self) -> usize {
        return self.l - self.pos;
    }
    pub fn shift(&mut self, len: usize) {
        if len <= 0 {
            return;
        }
        //#[cfg(test)]
        //println!("before shift,{},{},{},{}",len,self.pos,self.len(),self.b.capacity());
        if len < self.len() {
            self.pos += len;
            if self.pos > self.max_buf_size {
                self.re_size();
            }
            //println!("{}",self.pos);
        } else {
            self.reset();
        }
        //#[cfg(test)]
        //println!(" after shift,{},{},{},{}",len,self.pos,self.len(),self.b.capacity());
    }
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            return &self.b[self.pos..self.l];
        }
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe {
            return &mut self.b[self.pos..self.l];
        }
    }
    pub fn bytes(&self) -> &[u8] {
        unsafe {
            return &self.b[self.pos..self.l];
        }
    }
    pub fn write_string(&mut self, s: &str) {
        self.write(s.as_bytes())
    }
    pub fn write_byte(&mut self, s: u8) {
        if self.pos > self.max_buf_size {
            self.re_size();
        }

        unsafe {
            self.l += 1;
            if self.l > self.b.len() {
                self.grow(1)
            }

            self.b[self.l-1] = s;

        }
    }
    pub fn truncate(&mut self, len: usize) {
        let new = self.pos + len;
        if new <= self.b.len() {
            self.l = new;
        }else{
            self.l=self.b.len()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::Instant;

    #[test]
    fn it_msgbuffer() {
        let mut msg = MsgBuffer::test_new();

        let b = msg.make(1);
        assert_eq!(1, b.len());
        let b = msg.make(8);
        b[7] = 0;
        assert_eq!(8, b.len());
        assert_eq!(9, msg.len());
        msg.shift(1);
        assert_eq!(1, msg.pos);
        assert_eq!(8, msg.len());
        msg.shift(7);
        msg.write(&[1, 2, 3]);
        assert_eq!(8, msg.pos);
        assert_eq!(&[0, 1, 2, 3], msg.bytes());
        msg.shift(5);
        assert_eq!(0, msg.pos);
        assert_eq!(msg.bytes(), &[]);
        msg.write(&[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31,
        ]);
        msg.shift(30);
        assert_eq!(0, msg.pos);
        assert_eq!(&[31], msg.bytes());
        msg.write(&[
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31,
        ]);
        msg.truncate(2);
        println!("{:?}", msg.bytes());
        msg.write_byte(1);
        msg.write_byte(2);
        assert_eq!(&[31, 1, 1, 2], msg.as_mut_slice());
        msg.truncate(100);
        assert_eq!(&[31, 1, 1, 2], msg.as_slice());
        msg.write_string("123");
        assert_eq!(&[31, 1, 1, 2, 49, 50, 51], msg.as_slice());
    }
}
