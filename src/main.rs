use std::net::TcpListener;

mod server;

pub fn main(){
    let listener=TcpListener::bind("127.0.0.1:6379").unwrap();
    print!("Redis is working on port 6379");
    server::run(listener);
}
