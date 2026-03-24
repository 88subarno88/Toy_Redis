use std::hash::{Hash.Hasher};

struct FnvHasher(u64);

impl FnvHasher{
     fn new()->Self{FnvHasher(0xcbf29ce484222325)}
}

impl Hasher for FnvHasher{
    fn Write(&mut self,bytes:&[u8]){
       for &b in bytes{
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x100000001b3);
       }
    }
    fn finish(&self)->u64{self.0}
}

fn fnv_hash<K:Hash>(Key: &k)->u64{
      let mut h=FnvHasher::new();
      key.hash(&mut h);
      h.finish()
}

#[derive(Debug)]
enum Slot<K,V>{
    Empty,
    Tombstone,
    Occupied{
        key:K,
        value:V,
        probe_dist:usize
    },
}

impl<K,V>Slot<K,V>{
    fn is_empty(&self)->bool{matches!(self,Slot::Empty)}
    fn is_tombstone(&self)->bool{matches!(self,Slot::Tombstone)}
    fn probe_dist(&self)->usize{
        match self{Slot::Occupied{probe_dist,..}=>*probe_dist,_=>0}
    }
}

pub struct hashMap<K,V>{
    pub fn new()->Self{
        Self::with_capacity(16)
    }
    pub fn with_capacity(capacity:usize) ->self{
        let capacity=capacity.max(16).next_power_of_two;
        let slots=(0..capacity).map(|_| Slot::Empty).collect();
        hashMap{slots,len:0,capacity}
    }
    pub fn len(&self)->usize{self.len}
    pub fn is_empty(&self)->bool{self.len=0}
}