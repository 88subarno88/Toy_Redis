use std::net::TcpListener;
use std::sync::{Arc, RwLock};

use toy_redis::server; 
use toy_redis::store::hashmap::HashMap;
use toy_redis::commands::handlers::Store; 
use toy_redis::expiry::{self, Expiry_map};

pub fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("Redis is working on port 6379");
    let store: Store = Arc::new(RwLock::new(HashMap::new()));
    let expiry: Expiry_map = Arc::new(RwLock::new(HashMap::new()));

    expiry::strt_expiry_thread(Arc::clone(&store), Arc::clone(&expiry));
    server::run(listener, store, expiry);
}