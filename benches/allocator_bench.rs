use criterion::{criterion_group,criterion_main,Criterion};
use toy_redis::store::allocator::slabAllocator;

fn bench_slab(c:&mut Criterion){
    c.bench_function("slab alloc+dealloc",|b|{
        let mut slab:slabAllocator<String,1024>=slabAllocator::new();
        b.iter(||{
          let ptr=slab.alloc();
          unsafe{
            ptr.write("hello".to_string())
          };
          unsafe {
            std::ptr::drop_in_place(ptr);
          }
          slab.dealloc(ptr);
        });
    });
}

fn bench_box(c:&mut Criterion){
    c.bench_function("Box alloc+dealloc",|b|{
        b.iter(||{
            let b=Box::new("hello".to_string());
            drop(b);
        });

    });
}
criterion_group!(benches, bench_slab, bench_box);
criterion_main!(benches);

