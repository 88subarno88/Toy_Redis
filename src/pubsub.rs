use std::collections::HashMap;
use std::sync::{Arc, RwLock, mpsc::Sender};

#[derive(Clone)]
pub struct PubSub {
    // Maps a channel name (like "news") to a list of Transmitters connected to clients
    channels: Arc<RwLock<HashMap<String, Vec<Sender<String>>>>>,
}

impl PubSub {
    pub fn new() -> Self {
        PubSub {
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // When a client types SUBSCRIBE, we add their network transmitter to the list
    pub fn subscribe(&self, channel: &str, tx: Sender<String>) {
        let mut map = self.channels.write().unwrap();
        let subscribers = map.entry(channel.to_string()).or_insert_with(Vec::new);
        subscribers.push(tx);
    }

    // When a client types PUBLISH, we shoot the message down all transmitters
    pub fn publish(&self, channel: &str, message: &str) -> usize {
        let mut map = self.channels.write().unwrap();
        if let Some(subscribers) = map.get_mut(channel) {
            // Retain only keeps active connections. If a client disconnected, it removes them!
            subscribers.retain(|tx| tx.send(message.to_string()).is_ok());
            return subscribers.len();
        }
        0 // Nobody is listening
    }
}