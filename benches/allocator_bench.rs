use criterion::{black_box,criterion_group,criterion_main,Criterion};
use toy_redis::store::allocator::slabAllocator;

struct DummyValue {
    _data: [u8; 64],
}

fn bench_slab(c:&mut Criterion){
    c.bench_function("slab alloc+dealloc",|b|{
        let mut slab:slabAllocator<DummyValue,1024>=slabAllocator::new();
        b.iter(||{
          let ptr=black_box(slab.alloc());
          unsafe{
            ptr.write(DummyValue { _data: [0; 64] });
           std::ptr::drop_in_place(ptr);
          };
          slab.dealloc(black_box(ptr));
        });
    });
}

fn bench_box(c:&mut Criterion){
    c.bench_function("Box alloc+dealloc",|b|{
        b.iter(||{
            let b=black_box(Box::new(DummyValue { _data: [0; 64]}));
            drop(black_box(b));
        });

    });
}
criterion_group!(benches, bench_slab, bench_box);
criterion_main!(benches);

