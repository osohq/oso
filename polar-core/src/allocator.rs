use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static NESTED_ALLOCS: AtomicUsize = AtomicUsize::new(0);

pub fn get_allocated() -> usize {
    ALLOCATED.load(Ordering::SeqCst)
}

struct Allocator {}

#[global_allocator]
static GLOBAL: Allocator = Allocator {};

unsafe impl GlobalAlloc for Allocator {
    #[track_caller]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let in_allocator = NESTED_ALLOCS.fetch_add(1, Ordering::SeqCst);

        let size = layout.size();
        let ptr = System.alloc(layout);
        // #[cfg(nightly)]
        if in_allocator == 0 {
            ALLOCATED.fetch_add(size, Ordering::Relaxed);
            let bt = std::backtrace::Backtrace::force_capture().to_string();

            eprintln!("A {:x} {}", ptr as usize, size);
            let frame = bt
                .split('\n')
                .skip_while(|f| {
                    !(f.contains("polar_core::") || f.contains("polar::"))
                        || f.contains("polar_core::allocator")
                })
                .take(2)
                .collect::<Vec<&str>>()
                .join("\n");
            eprintln!("{}", frame);
            // eprintln!("{}", bt);
        }
        NESTED_ALLOCS.fetch_sub(1, Ordering::SeqCst);
        ptr
    }

    #[track_caller]
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        let in_allocator = NESTED_ALLOCS.load(Ordering::SeqCst);
        if in_allocator == 0 {
            eprintln!("D {:x} {}", ptr as usize, size);
            ALLOCATED.fetch_sub(size, Ordering::Relaxed);
        }

        System.dealloc(ptr, layout)
    }
}
