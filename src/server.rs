use std::{io::Read, net::TcpStream};
use std::net::TcpListener;
use std::thread;
use std::sync::{Arc, RwLock};
use crate::expiry::Expiry_map;
// use std::collections::HashMap;
// use crate::commands::Store; 
use crate::store::hashmap::HashMap;
use std::time::Instant;
use crate::aof::Aof;


use crate::{
    commands::{parse_command, Command, Store,keys_match},   
    protocol::{resp::parse, writer::RespWriter}, 
};

pub fn run(
    listener: std::net::TcpListener, 
    store: crate::commands::handlers::Store, 
    expiry: crate::expiry::Expiry_map, 
    aof: crate::aof::Aof 
) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let store = std::sync::Arc::clone(&store);
                let expiry = std::sync::Arc::clone(&expiry);
                let aof = aof.clone(); 

                std::thread::spawn(move || {
                    handle_connection(stream, store, expiry, aof);
                });
            }
            Err(e) => { println!("Connection failed: {}", e); }
        }
    }
}

pub fn handle_connection(mut stream: std::net::TcpStream, 
    store: crate::commands::handlers::Store, 
    expiry: crate::expiry::Expiry_map, 
    aof: crate::aof::Aof ){

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
                       store.insert(key.to_string(), value.to_string());
                        let log = format!("*3\r\n$3\r\nSET\r\n${}\r\n{}\r\n${}\r\n{}\r\n", 
                            key.len(), key, value.len(), value);
                        aof.log_command(&log);
                        
                    
                        writer.write_simple_string(b"OK").unwrap();
                    }
                    Some(Command::Get {key})=>{
                        //    let guard = store.read().unwrap();
                            match store.get(&key.to_string()) {
                            Some(v) => writer.write_bulk_string(v.as_bytes()).unwrap(),
                            None    => writer.write_null().unwrap(),
                        }
                    }
                    Some(Command::Del { keys }) => {
                        let count = keys.iter()
                                     .filter(|k| store.remove(&k.to_string()).is_some())
                                     .count();
                                     
                        if count > 0 {
                            let mut log = format!("*{}\r\n$3\r\nDEL\r\n", keys.len() + 1);
                            for k in &keys {
                                log.push_str(&format!("${}\r\n{}\r\n", k.len(), k));
                            }
                            aof.log_command(&log);
                        }
                        
                        writer.write_int(count as i64).unwrap();
                    }
                    Some(Command::Exists { key }) => {
                        let exists = store.contains_key(&key.to_string());
                        writer.write_int(exists as i64).unwrap();
                    }
                    Some(Command::Keys { pattern }) => {
                        // let guard = store.read().unwrap();
                        let all_keys = store.get_all_keys();
                        let mut matched_keys = Vec::new();
                        
                        for k in all_keys {
                            if keys_match(pattern, &k) {
                                matched_keys.push(k.clone());
                            }
                        }
                        writer.write_array_header(matched_keys.len()).unwrap();
                        for key in matched_keys {
                            writer.write_bulk_string(key.as_bytes()).unwrap();
                        }
                    }
                    Some(Command::Expire { key, seconds }) => {
                        let exists = store.contains_key(&key.to_string());
        
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
                        } else if store.contains_key(&key.to_string()) {
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