

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use crate::store::hashmap::HashMap;
use crate::commands::{execute, parse_command, Store};
use crate::protocol::resp::parse;

pub fn run(listener: TcpListener) {
    // FIXED: Changed :: to = for variable assignment
    let store: Store = Arc::new(RwLock::new(HashMap::new()));

    for stream in listener.incoming() {
        // FIXED: Safely handle the connection result to avoid panics
        match stream {
            Ok(valid_stream) => {
                let store = Arc::clone(&store);
                std::thread::spawn(move || {
                    handle_connection(valid_stream, store);
                });
            }
            Err(e) => {
                eprintln!("Failed to establish connection: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, store: Store) {
    let mut buf = vec![0u8; 4096];
    loop {
        let n = match stream.read(&mut buf) {
            Ok(0) | Err(_) => return, // Connection closed or error
            Ok(n) => n,
        };
        
        match parse(&buf[..n]) {
            Ok((resp_val, _consumed)) => {
                if let Some(cmd) = parse_command(&resp_val) {
                    let response = execute(cmd, &store);
                    stream.write_all(&response).unwrap();
                } else {
                    stream.write_all(b"-ERR unknown command\r\n").unwrap();
                }
            }
            Err(_) => {
                stream.write_all(b"-ERR parse error\r\n").unwrap();
            }
        }
    }
}