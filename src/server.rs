use std::io::{Read,Write};
use std::net::TcpListener;

pub fn run(listener:TcpListener) {
    for stream in listener.incoming(){
        let mut stream =stream.unwrap();
        let mut buf=[0u8;512];
        stream.read(&mut buf).unwrap();
        stream.write_all(b"+DICK\r\n").unwrap();

    }
    
}