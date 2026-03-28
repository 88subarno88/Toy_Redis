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
    Keys { pattern: &'a str },
}

pub fn keys_match(pattern:&str,key:&str)->bool{
    glo_match(pattern.as_bytes(),key.as_bytes())
}

fn glo_match(pat:&[u8],s:&[u8])->bool{
    match(pat.first(),s.first()){
        (None,None) =>true,
        (Some(b'*'),_)=> glo_match(&pat[1..], s) || (!s.is_empty() && glo_match(pat, &s[1..])),
        (Some(b'?'),Some(_))=> glo_match(&pat[1..], &s[1..]),
        (Some(p), Some(c)) if p == c => glo_match(&pat[1..], &s[1..]),
        _                  => false,

    }
}

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
        "KEYS" => {
            let pattern = match args.get(1)? {
                RespValue::BulkString(b) => std::str::from_utf8(b).ok()?,
                _ => return None,
            };
            Some(Command::Keys { pattern })
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
        Command::Keys { pattern } => {
            let guard = store.read().unwrap();
            
            let mut matched_keys = Vec::new();
            for (k, _) in guard.iter() {
                if keys_match(pattern, k) {
                    matched_keys.push(k.clone());
                }
            }
            let mut r = format!("*{}\r\n", matched_keys.len()).into_bytes();
            for key in matched_keys {
                let bulk_str = format!("${}\r\n{}\r\n", key.len(), key);
                r.extend_from_slice(bulk_str.as_bytes());
            }
            r
        }
    }
}