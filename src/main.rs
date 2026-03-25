mod server;
mod store;
mod commands;
mod protocol;

use std::net::TcpListener;




pub fn main(){
    let listener=TcpListener::bind("127.0.0.1:6379").unwrap();
    println!("Redis is working on port 6379");
    server::run(listener);
}
