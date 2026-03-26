use std::string;
use std::sync::{Arc, Mutex, RwLock};
use crate::store::hashmap::HashMap;
use crate::store::allocator::slabAllocator;

pub mod hashmap;
pub mod allocator;

// pub type Value=SlabBox<String>;
pub type Allocator=Arc<Mutex<slabAllocator<String, 1024>>>;

pub struct KvStore{
    pub map:HashMap<String,*mut String>,
    pub allocator:Allocator,
}

impl KvStore {
    pub fn new() -> Self {
    Self {
        map: HashMap::new(),
        allocator: Arc::new(Mutex::new(slabAllocator::new())),
    }
}
    pub fn set(&mut self,key:String,Value:String){
        if let Some(old_ptr) = self.map.remove(&key){
            unsafe {
                std::ptr::drop_in_place(old_ptr);
            }
            self.allocator.lock().unwrap().dealloc(old_ptr);
        }
        let ptr=self.allocator.lock().unwrap().alloc();
        unsafe {
            ptr.write(Value)
        };
        self.map.insert(key,ptr);
    }

    pub fn get(&self,key:&str)->Option<&str>{
        self.map.get(&key.to_string())
            .map(|ptr| unsafe {
                &**ptr as &str
            })
    }

    pub fn del(&mut self,key:&str)->bool{
        if let Some(ptr)= self.map.remove(&key.to_string()){
            unsafe {
                std::ptr::drop_in_place(ptr);
            }
             self.allocator.lock().unwrap().dealloc(ptr);
            true
        } else {
            false
        }
    }
     pub fn stats(&self) -> crate::store::allocator::AllocStats {
        self.allocator.lock().unwrap().stats()
    }
}