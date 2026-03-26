use::std::mem::MaybeUninit;

#[derive(Debug,Clone)]
pub struct AllocStats{
    pub total_cap:usize,
    pub used:usize,
    pub slabs_alloc:usize,
}

pub struct slabAllocator<T,const SLAB_SIZE: usize=1024>{
    slabs:Vec<Box<[MaybeUninit<T>;SLAB_SIZE]>>,
    free_list:Vec<*mut T>,
    used:usize,
}

unsafe impl<T:Send,const N:usize> Send for slabAllocator<T,N>{}
unsafe impl<T:Send,const N:usize> Sync for slabAllocator<T,N>{}

impl<T,const SLAB_SIZE: usize>slabAllocator<T,SLAB_SIZE>{
    pub fn new() -> Self{
        Self{
            slabs: Vec::new(),
            free_list: Vec::new(),
            used:  0,
        }
    }

    fn grow(&mut self){
      let mut slab: Box<[MaybeUninit<T>; SLAB_SIZE]> = 
    Box::new([const { MaybeUninit::uninit() }; SLAB_SIZE]);
        //     unsafe {
        //         MaybeUninit::uninit().assume_init()
        //     }
        // );

        let base_ptr=slab.as_mut_ptr() as *mut MaybeUninit<T>;
        for i in (0..SLAB_SIZE).rev(){
            let slot_ptr= unsafe {
                base_ptr.add(i) as *mut T
            };
            self.free_list.push(slot_ptr);
        }
       self.slabs.push(slab);
    }

    pub fn alloc(&mut self)->*mut T{
        if self.free_list.is_empty(){
            self.grow();
        }
        self.used+=1;
        self.free_list.pop().unwrap()
    }

    pub fn dealloc(&mut self,ptr:*mut T){
        debug_assert!(!ptr.is_null());
        self.used-=1;
        self.free_list.push(ptr);
    }

    pub fn stats(&self)->AllocStats{
        AllocStats { total_cap: self.slabs.len() * SLAB_SIZE,
             used:             self.used, 
             slabs_alloc: self.slabs.len(),
             }
    }
}

impl<T,const SLAB_SIZE:usize>Drop for slabAllocator<T,SLAB_SIZE>{
    fn drop(&mut self) {
        #[cfg(debug_assertions)]
        if self.used > 0 {
            eprintln!(
                "WARNng: SlabAllocator dropped with {} live allocations — possible leak",
                self.used
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test 1: basic alloc/dealloc cycle
    #[test]
    fn alloc_dealloc() {
        let mut slab: slabAllocator<u64, 8> = slabAllocator::new();
        let ptr = slab.alloc();
        unsafe { ptr.write(42u64) };
        assert_eq!(unsafe { *ptr }, 42);
        unsafe { std::ptr::drop_in_place(ptr) };
        slab.dealloc(ptr);
        assert_eq!(slab.stats().used, 0);
    }

    // Test 2: slab grows when full
    #[test]
    fn grows_across_slabs() {
        let mut slab: slabAllocator<u32, 4> = slabAllocator::new();
        let mut ptrs = vec![];
        for i in 0..12u32 {           // 3 slabs of 4
            let p = slab.alloc();
            unsafe { p.write(i) };
            ptrs.push(p);
        }
        assert_eq!(slab.stats().slabs_alloc, 3);
        assert_eq!(slab.stats().used, 12);

        for p in ptrs {
            unsafe { std::ptr::drop_in_place(p) };
            slab.dealloc(p);
        }
        assert_eq!(slab.stats().used, 0);
    }

    // Test 3: spatial locality — items in same slab are close in memory
    #[test]
    fn cache_locality() {
        let mut slab: slabAllocator<u64, 16> = slabAllocator::new();
        let p1 = slab.alloc();
        let p2 = slab.alloc();
        let distance = (p1 as isize - p2 as isize).unsigned_abs();
        // Both from same slab — must be within slab bounds
        assert!(distance < 16 * std::mem::size_of::<u64>(),
            "too far apart: {} bytes", distance);
        slab.dealloc(p1);
        slab.dealloc(p2);
    }

    // Test 4: 500k alloc/dealloc (the assignment's required test)
    #[test]
    fn stress_500k() {
        let mut slab: slabAllocator<u64, 1024> = slabAllocator::new();
        let mut ptrs = Vec::with_capacity(500_000);

        for i in 0..500_000u64 {
            let p = slab.alloc();
            unsafe { p.write(i) };
            ptrs.push(p);
        }
        assert_eq!(slab.stats().used, 500_000);

        for p in ptrs {
            unsafe { std::ptr::drop_in_place(p) };
            slab.dealloc(p);
        }
        assert_eq!(slab.stats().used, 0);
    }
}