use std::hash::{Hash,Hasher};

struct FnvHasher(u64);

impl FnvHasher{
     fn new()->Self{FnvHasher(0xcbf29ce484222325)}
}

impl Hasher for FnvHasher{
    fn write(&mut self,bytes:&[u8]){
       for &b in bytes{
            self.0 ^= b as u64;
            self.0 = self.0.wrapping_mul(0x100000001b3);
       }
    }
    fn finish(&self)->u64{self.0}
}

fn fnv_hash<K: Hash>(key: &K) -> u64 {
    let mut h = FnvHasher::new();
    key.hash(&mut h);
    let mut hash = h.finish();
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xff51afd7ed558ccd);
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(0xc4ceb9fe1a85ec53);
    hash ^= hash >> 33;
    
    hash
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


pub struct HashMap<K, V> {
    slots:    Vec<Slot<K, V>>,
    len:      usize,   
    capacity: usize,   
}

const LF: f64 = 0.75;

impl<K: Eq + Hash, V> HashMap<K, V>{
    pub fn new()->Self{
        Self::with_capacity(16)
    }
    pub fn with_capacity(capacity:usize) ->Self{
        let capacity=capacity.max(16).next_power_of_two();
        let slots=(0..capacity).map(|_| Slot::Empty).collect();
        HashMap{slots,len:0,capacity}
    }
    pub fn len(&self)->usize{self.len}
    pub fn is_empty(&self)->bool{self.len==0}
    fn ideal(&self,key: &K)->usize{
        (fnv_hash(key) as usize)&(self.capacity-1)
    }
    fn resize(&mut self){
       let new_cap=self.capacity*2;
       let old_slots=std::mem::replace(
        &mut self.slots,(0..new_cap).map(|_|Slot::Empty).collect(),
       );
       self.capacity=new_cap;
       self.len=0;
       for slot in old_slots{
        if let Slot::Occupied{key,value,..}=slot{
            self.insert(key,value);
        }
       }
    }
    

    pub fn insert(&mut self, mut key:K,mut value:V)->Option<V>{
        if (self.len+1) as f64>self.capacity as f64*LF{
            self.resize();
        }
        let mut indx=self.ideal(&key);
        let mut dist=0usize;
        loop{
            if self.slots[indx].is_empty() || self.slots[indx].is_tombstone() {
                 self.slots[indx] = Slot::Occupied { key, value, probe_dist: dist };
                 self.len += 1;
                 return None;
             }

             if let Slot::Occupied { key: ref k, .. } = self.slots[indx] {
                 if k == &key { 
                     if let Slot::Occupied { value: ref mut v, .. } = self.slots[indx] {
                         return Some(std::mem::replace(v, value));
                     }
                     unreachable!()
              }
             }
             let incumbent_dist = self.slots[indx].probe_dist();
             if dist > incumbent_dist {
                 let old = std::mem::replace(
                     &mut self.slots[indx],
                     Slot::Occupied { key, value, probe_dist: dist }
                 );
                 if let Slot::Occupied { key: k, value: v, probe_dist: d } = old {
                     key = k;
                    value = v;
                     dist = d;
                 } else {
                     unreachable!();
                 }
            }
            indx=(indx+1)&(self.capacity-1);
            dist+=1;
        }
    }
    fn find_slot(&self,key:&K)->Option<usize>{
        let mut indx=self.ideal(&key);
        let mut dist=0usize;
        loop{
            match &self.slots[indx]{
                Slot::Empty=>return None,
                Slot::Tombstone=>{},
                Slot::Occupied{key:k,probe_dist,..}=>{
                    if k==key{
                        return Some(indx);
                    }
                    if dist>*probe_dist{
                        return None;
                    }
                }
            }
            indx=(indx+1)&(self.capacity-1);
            dist+=1;
        }
    }

    pub fn get(&self,key:&K)->Option<&V>{
         let indx=self.find_slot(key)?;
         match &self.slots[indx]{
            Slot::Occupied{value,..}=>Some(value),
            _ =>None,
         }
    }

    pub fn get_mut(&mut self,key:&K)->Option<&mut V>{
         let indx=self.find_slot(key)?;
         match &mut self.slots[indx]{
            Slot::Occupied{value,..}=>Some(value),
            _ =>None,
         }
    }
    pub fn contains_key(&self,key:&K)->bool{
        self.find_slot(key).is_some()
    }
    pub fn remove(&mut self,key:&K)->Option<V>{
        let indx=self.find_slot(key)?;
        let old=std::mem::replace(&mut self.slots[indx],Slot::Tombstone);
        self.len-=1;
        match old{
            Slot::Occupied{value,..}=>Some(value),
             _ => unreachable!(),
        }
    }
    pub fn iter(&self)->impl Iterator<Item=(&K,&V)>{
        self.slots.iter().filter_map(|s| match s{
            Slot::Occupied{key,value,..}=>Some((key,value)),
            _ =>None,
        })
    }
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_set_get() {
        let mut m: HashMap<String, String> = HashMap::new();
        m.insert("hello".into(), "world".into());
        assert_eq!(m.get(&"hello".into()), Some(&"world".into()));
        assert_eq!(m.get(&"nope".into()), None);
    }

    #[test]
    fn remove_and_tombstone() {
        let mut m: HashMap<String, i32> = HashMap::new();
        m.insert("a".into(), 1);
        m.insert("b".into(), 2);
        m.remove(&"a".into());
        assert!(!m.contains_key(&"a".into()));
        assert!(m.contains_key(&"b".into()));
    }

    #[test]
    fn load_factor_respected() {
        let mut m: HashMap<i32, i32> = HashMap::new();
        for i in 0..100 {
            m.insert(i, i * 2);
        }
        let load = m.len() as f64 / m.capacity as f64;
        assert!(load <= 0.75, "load factor exceeded: {:.2}", load);
    }
}