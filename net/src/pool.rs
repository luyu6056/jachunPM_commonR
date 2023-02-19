use crate::{Arc, Future, Mutex, Pin};
use std::sync::atomic::{AtomicUsize, Ordering};

pub type PoolNewFn<T> = Box<dyn FnMut() -> PoolNewResult<T> + Send + Sync>;
pub type PoolNewResult<T> = Pin<Box<dyn Future<Output = T> + Send>>;

pub struct Pool<T> {
    vec_pool: Vec<T>,

    //最大容量
    max_buf_num: usize,
    //new_fn: PoolNewFn<T>,
}
pub fn new<T:Default>() -> Pool<T>
{
    let mut pool = Pool {
        vec_pool: Vec::new(),
        max_buf_num: 4096,
        //new_fn: Box::new(newfn),
    };
    pool
}
impl<T:Default> Pool<T> {

    fn with_max_buf_num(mut self,size:usize)->Self {
        self.max_buf_num=size;
        self
    }
    pub fn get(&mut self) -> T {
        match self.vec_pool.pop() {
            Some(inner) => inner,
            None => T::default(),
        }
    }
    pub fn put(&mut self, inner: T) {
        if self.vec_pool.len()<self.max_buf_num{
            self.vec_pool.push(inner)
        }
    }
}
