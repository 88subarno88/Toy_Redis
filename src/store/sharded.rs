use std::sync::RwLock;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use crate::store::hashmap::HashMap; 

const NUM_SHARDS: usize = 16;

pub struct ShardedStore {
    shards: [RwLock<HashMap<String, String>>; NUM_SHARDS],
}

impl ShardedStore {
    pub fn new() -> Self {
        let shards = std::array::from_fn(|_| RwLock::new(HashMap::new()));
        ShardedStore { shards }
    }

    fn get_shard_index(&self, key: &str) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() as usize) % NUM_SHARDS
    }


    pub fn insert(&self, key: String, value: String) {
        let idx = self.get_shard_index(&key);
        let mut shard = self.shards[idx].write().unwrap();
        shard.insert(key, value);
    }
    pub fn get(&self, key: &str) -> Option<String> {
        let idx = self.get_shard_index(key);
        let shard = self.shards[idx].read().unwrap();
        shard.get(&key.to_string()).cloned() 
    }

    pub fn remove(&self, key: &str) -> Option<String> {
        let idx = self.get_shard_index(key);
        let mut shard = self.shards[idx].write().unwrap();
        shard.remove(&key.to_string())
    }

    pub fn contains_key(&self, key: &str) -> bool {
        let idx = self.get_shard_index(key);
        let shard = self.shards[idx].read().unwrap();
        shard.contains_key(&key.to_string())
    }

    pub fn get_all_keys(&self) -> Vec<String> {
        let mut all_keys = Vec::new();
        for shard in &self.shards {
            let guard = shard.read().unwrap();
            for (k, _) in guard.iter() {
                all_keys.push(k.clone());
            }
        }
        all_keys
    }
}