use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::collections::HashMap as StdHashMap;
// Assuming your custom map is exposed here based on your imports!
use toy_redis::store::hashmap::HashMap as CustomHashMap;

fn bench_custom_hashmap(c: &mut Criterion) {
    c.bench_function("Custom HashMap Insert (1000 keys)", |b| {
        b.iter(|| {
            let mut map = CustomHashMap::new();
            
            for i in 0..1000 {
                map.insert(black_box(i), black_box(i * 2));
            }
        })
    });
}

fn bench_std_hashmap(c: &mut Criterion) {
    c.bench_function("Standard HashMap Insert (1000 keys)", |b| {
        b.iter(|| {
            let mut map = StdHashMap::new();
            for i in 0..1000 {
                map.insert(black_box(i), black_box(i * 2));
            }
        })
    });
}


criterion_group!(benches, bench_custom_hashmap, bench_std_hashmap);
criterion_main!(benches);