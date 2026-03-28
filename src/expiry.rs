use std::sync::{Arc,RwLock};
use std::time::{Duration,Instant};
use crate::store::hashmap::HashMap;

pub type Expiry_map=Arc<RwLock<HashMap<String,Instant>>>;

pub fn  strt_expiry_thread(
    store:Arc<RwLock<HashMap<String,String>>>,
    expiry:Expiry_map,
){
    std::thread::spawn(move||{
        loop{
            std::thread::sleep(Duration::from_millis(100));
            let now=Instant::now();
            let mut expired_keys:Vec<String>=Vec::new();
            {
                let guard=expiry.read().unwrap();
                for(k,deadline) in guard.iter(){
                    if  now>=*deadline{
                        expired_keys.push(k.clone());
                    }
                }

            }
            if !expired_keys.is_empty(){
                let mut store=store.write().unwrap();
                let mut expire=expiry.write().unwrap();
                for k in expired_keys{
                    store.remove(&k);
                    expire.remove(&k);
                }
            }
        }
    });
}