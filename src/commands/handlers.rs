// src/commands/handlers.rs

use crate::protocol::resp::RespValue;
use crate::store::hashmap::HashMap;
use std::sync::{Arc, RwLock};

pub type Store = Arc<RwLock<HashMap<String, String>>>;

pub enum Command<'a> {
    Ping(Option<&'a [u8]>),
    Set { key: &'a str, value: &'a str },
    Get { key: &'a str },
    Del { keys: Vec<&'a str> },
    Exists { key: &'a str },
}

// FIXED: Added <'a> to RespValue so the compiler knows it holds borrowed data
pub fn parse_command<'a>(val: &RespValue<'a>) -> Option<Command<'a>> {
    let args = match val {
        RespValue::Array(a) => a,
        _ => return None,
    };

    let cmd_name = match args.first()? {
        RespValue::BulkString(b) => std::str::from_utf8(b).ok()?,
        _ => return None,
    };

    match cmd_name.to_ascii_uppercase().as_str() {
        "PING" => {
            let msg = args.get(1).and_then(|v| match v {
                 RespValue::BulkString(b) => Some(*b),
                 _ => None,
            });
            Some(Command::Ping(msg))
        }
        "SET" => {
            let key = match args.get(1)? {
                RespValue::BulkString(b) => std::str::from_utf8(b).ok()?,
                _ => return None,
            };
            let val = match args.get(2)? {
                RespValue::BulkString(b) => std::str::from_utf8(b).ok()?,
                _ => return None,
            };
            Some(Command::Set { key, value: val })
        }
        "GET" => {
            let key = match args.get(1)? {
                RespValue::BulkString(b) => std::str::from_utf8(b).ok()?,
                _ => return None,
            };
            Some(Command::Get { key })
        }
        _ => None,
    }
}

pub fn execute(cmd: Command, store: &Store) -> Vec<u8> {
    match cmd {
        Command::Ping(None) => b"+PONG\r\n".to_vec(),
        Command::Ping(Some(m)) => {
            let mut r = format!("${}\r\n", m.len()).into_bytes();
            r.extend_from_slice(m);
            r.extend_from_slice(b"\r\n");
            r
        }
        Command::Set { key, value } => {
            store.write().unwrap().insert(key.to_string(), value.to_string());
            b"+OK\r\n".to_vec()
        }
        Command::Get { key } => {
            let guard = store.read().unwrap();
            match guard.get(&key.to_string()) {
                Some(v) => {
                    let mut r = format!("${}\r\n", v.len()).into_bytes();
                    r.extend_from_slice(v.as_bytes());
                    r.extend_from_slice(b"\r\n");
                    r
                }
                 None => b"$-1\r\n".to_vec(),
            }
        }
        Command::Del { keys } => {
            let mut guard = store.write().unwrap();
            let count = keys.iter()
                 .filter(|k| guard.remove(&k.to_string()).is_some())
                 .count();
            format!(":{}\r\n", count).into_bytes()    
        }
        Command::Exists { key } => {
            let guard = store.read().unwrap();
            let exists = guard.contains_key(&key.to_string()) as u8;
            format!(":{}\r\n", exists).into_bytes()
        }
    }
}