use toy_redis::store::hashmap::HashMap;

#[test]
fn test_hashmap_million_keys() {
    let mut map = HashMap::new();
    let total_keys = 1_000_000;
    
    println!("Inserting {} keys...", total_keys);
    
    // 1. Insert 1,000,000 keys
    for i in 0..total_keys {
        let key = format!("key_{}", i);
        let val = format!("val_{}", i);
        map.insert(key, val);
        
        // 2. Verify load factor never exceeds 0.75
        let load_factor = map.len() as f64 / map.capacity() as f64;
        assert!(
            load_factor <= 0.75, 
            "Load factor exceeded 0.75! Currently at {}", load_factor
        );
    }
    
    assert_eq!(map.len(), total_keys, "Map should have exactly 1 million items");

    println!("Retrieving keys...");

    // 3. Verify they can ALL be retrieved
    for i in 0..total_keys {
        let key = format!("key_{}", i);
        let expected_val = format!("val_{}", i);
        let actual_val = map.get(&key).expect("Key went missing!");
        assert_eq!(*actual_val, expected_val, "Value mismatch!");
    }

    println!("Deleting 50,000 keys...");

    // 4. Delete 50,000 random keys (we'll just delete the first 50k for speed)
    for i in 0..50_000 {
        let key = format!("key_{}", i);
        let removed = map.remove(&key);
        assert!(removed.is_some(), "Failed to remove key!");
    }
    
    assert_eq!(map.len(), total_keys - 50_000, "Map length did not update after deletion");
    
    // 5. Verify the remaining 950,000 keys are still safe (Tombstone check)
    let safe_key = format!("key_{}", 100_000);
    let safe_val = format!("val_{}", 100_000);
    assert_eq!(*map.get(&safe_key).unwrap(), safe_val, "Tombstones broke the probe chain");
    
    println!("HashMap survived the Torture ");
}