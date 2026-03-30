use std::net::TcpListener;
use std::sync::{Arc, RwLock};

use toy_redis::server; 
use toy_redis::store::hashmap::HashMap;
use toy_redis::commands::handlers::Store; 
use toy_redis::expiry::{self, Expiry_map};
use toy_redis::store::sharded::ShardedStore;
use toy_redis::aof::Aof;
use toy_redis::aof::{ restore_from_aof};
use toy_redis::pubsub::PubSub;
pub fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("Redis is working on port 6379");
    let store = Arc::new(ShardedStore::new());
    // let store: Store = Arc::new(RwLock::new(HashMap::new()));
    let expiry: Expiry_map = Arc::new(RwLock::new(HashMap::new()));
  restore_from_aof("appendonly.aof", &store);
    let aof = Aof::new("appendonly.aof");   
    let pubsub = PubSub::new();

    expiry::strt_expiry_thread(Arc::clone(&store), Arc::clone(&expiry));
    server::run(listener, store, expiry,aof,pubsub);
}