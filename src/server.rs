use std::{io::Read, net::TcpStream};

use crate::{
    commands::{parse_command, Command, Store},   
    protocol::{resp::parse, writer::RespWriter}, 
};

pub fn handle_connection(mut stream:TcpStream,store:Store){
    let mut buf=vec![0u8;4096];
    loop {
        let n=match stream.read(&mut buf) {
            Ok(0) | Err(_) => return,
            Ok(n) => n,
        };

        let mut writer=RespWriter::new(&mut stream);

        match parse(&buf[..n]){
            Ok((resp_val, _))=>{
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
                    None => { writer.write_error(b"ERR unknown command").unwrap(); }
                    
                }


            }
              Err(_) => { writer.write_error(b"ERR parse error").unwrap(); }

        }
    }
}