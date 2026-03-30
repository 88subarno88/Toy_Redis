use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::{self, Sender};
use std::thread;
use crate::commands::handlers::Store; 

#[derive(Clone)]
pub struct Aof {
    sender: Sender<Vec<u8>>,
}

impl Aof {
    pub fn new(path: &str) -> Self {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();

        let (tx, rx) = mpsc::channel::<Vec<u8>>();

        thread::spawn(move || {
            for log in rx {
                file.write_all(&log).unwrap();
                file.sync_data().unwrap(); 
            }
        });

        Aof { sender: tx }
    }

    pub fn log_command(&self, command: &str) {
        self.sender.send(command.as_bytes().to_vec()).unwrap();
    }
}

pub fn restore_from_aof(path: &str, store: &Store) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => {
            println!("No existing AOF file found. Starting fresh.");
            return;
        }
    };

    println!("Loading database from AOF...");
    let reader = BufReader::new(file);
    let mut lines = reader.lines().filter_map(|l| l.ok());

    while let Some(line) = lines.next() {
        let clean_line = line.trim();
        
        if clean_line.starts_with('*') {
            lines.next(); 
            let cmd = lines.next().unwrap_or_default().trim().to_uppercase();
            if cmd == "SET" {
                lines.next(); 
                let key = lines.next().unwrap_or_default().trim().to_string();
                lines.next(); 
                let value = lines.next().unwrap_or_default().trim().to_string();
                store.insert(key, value);
            } 
            else if cmd == "DEL" {
                lines.next(); 
                let key = lines.next().unwrap_or_default().trim().to_string();
                
                store.remove(&key);
            }
        }
    }
    println!("AOF restore complete!");
}