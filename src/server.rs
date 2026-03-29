use std::{io::Read, net::TcpStream};
use std::net::TcpListener;
use std::thread;
use std::sync::{Arc, RwLock};
use crate::expiry::Expiry_map;
// use std::collections::HashMap;
// use crate::commands::Store; 
use crate::store::hashmap::HashMap;
use std::time::Instant;

use crate::{
    commands::{parse_command, Command, Store,keys_match},   
    protocol::{resp::parse, writer::RespWriter}, 
};

pub fn run(listener: TcpListener, store: Store, expiry: Expiry_map) {

    println!("Waiting for clients to connect...");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store_clone = Arc::clone(&store);
                let expiry_clone = Arc::clone(&expiry);
                thread::spawn(move || {
                    handle_connection(stream, store_clone,expiry_clone);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

pub fn handle_connection(mut stream:TcpStream,store:Store,expiry: Expiry_map){
    let mut buffer: Vec<u8> = Vec::new();
    let mut buf=vec![0u8;4096];
   loop {

        let n = match stream.read(&mut buf) {
            Ok(0) | Err(_) => return, 
            Ok(n) => n,
        };

        buffer.extend_from_slice(&buf[..n]);

        let mut writer = RespWriter::new(&mut stream);

    
        loop {
            if buffer.is_empty() {
                break;
            }
        match parse(&buffer){
            Ok((resp_val, bytes_consm))=>{
                  match parse_command(&resp_val) {
                    Some(Command::Ping(None))=>{writer.write_simple_string(b"PONG").unwrap();}
                    Some(Command::Ping(Some(m))) => {writer.write_bulk_string(m).unwrap();}
                    Some(Command::Set { key, value }) => {
                        store.write().unwrap().insert(key.to_string(), value.to_string());
                        writer.write_simple_string(b"OK").unwrap();
                    }
                    Some(Command::Get {key})=>{
                           let guard = store.read().unwrap();
                            match guard.get(&key.to_string()) {
                            Some(v) => writer.write_bulk_string(v.as_bytes()).unwrap(),
                            None    => writer.write_null().unwrap(),
                        }
                    }
                    Some(Command::Del { keys })=>{
                        let mut guard=store.write().unwrap();
                        let count=keys.iter()
                                     .filter(|k| guard.remove(&k.to_string()).is_some())
                                     .count();
                        writer.write_int(count as i64).unwrap();
                    }
                    Some(Command::Exists { key }) => {
                        let exists = store.read().unwrap().contains_key(&key.to_string());
                        writer.write_int(exists as i64).unwrap();
                    }
                    Some(Command::Keys { pattern }) => {
                        let guard = store.read().unwrap();
                        let mut matched_keys = Vec::new();
                        
                        for (k, _) in guard.iter() {
                            if keys_match(pattern, k) {
                                matched_keys.push(k.clone());
                            }
                        }
                        writer.write_array_header(matched_keys.len()).unwrap();
                        for key in matched_keys {
                            writer.write_bulk_string(key.as_bytes()).unwrap();
                        }
                    }
                    Some(Command::Expire { key, seconds }) => {
                        let exists = store.read().unwrap().contains_key(&key.to_string());
        
                        if exists {
                            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(seconds);
                            expiry.write().unwrap().insert(key.to_string(), deadline);
                            writer.write_int(1).unwrap();
                        } else {
                            writer.write_int(0).unwrap();
                        }
                    }
                    
                    Some(Command::Ttl { key }) => {
                        let guard = expiry.read().unwrap();
                        
                        if let Some(deadline) = guard.get(&key.to_string()) {
                            let now = std::time::Instant::now();
                            if *deadline > now {
                                let secs = deadline.duration_since(now).as_secs();
                                writer.write_int(secs as i64).unwrap();
                            } else {
                                writer.write_int(-2).unwrap(); 
                            }
                        } else if store.read().unwrap().contains_key(&key.to_string()) {
                            writer.write_int(-1).unwrap();
                        } else {
                            writer.write_int(-2).unwrap(); 
                        }
                    }
                    None => { writer.write_error(b"ERR unknown command").unwrap(); }
                    
                }
                buffer.drain(..bytes_consm);


            }
              Err(_) => { 
                if buffer.len()>1024*1024{
                    buffer.clear();
                    let _=writer.write_error(b"ERROR payload is too large");
                }
                break;
               }

        }
    }
}
}