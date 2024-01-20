use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::numerics::MOST_POSITIVE_EXACT_FLOAT;

const MAX_ID: u64 = (MOST_POSITIVE_EXACT_FLOAT - 1) as u64;

/// Note about memory ordering: 
/// 
/// Here 'next' is just a global counter between threads and doesn't synchronize with other 
/// variables. Therefore, `Relaxed` can be used in both single-threaded and  multi-threaded 
/// environments.
/// 
/// While atomic operations using 'Relaxed' memory ordering do not provide any happens-before 
/// relationship, they do guarantee a total modification order of the 'next' atomic variable. 
/// This means that all modifications of the 'next' atomic variable happen in an order that 
/// is the same from the perspective of every single thread. 
#[derive(Clone, Debug)]
pub struct Counter {
    next: Arc<AtomicU64>,
}

impl Default for Counter {
    fn default() -> Self {
        Self {
            next: Arc::new(AtomicU64::new(1)),
        }
    }
}

impl Counter {
    /// Create a new counter starting at `start`.
    #[cfg(test)]
    pub fn with_start(start: u64) -> Self {
        Self {
            next: Arc::new(AtomicU64::new(start)),
        }
    }

    /// Return a monotonically increasing integer ID.
    ///
    /// Wraps around at 52 bits of precision so that it can be safely
    /// coerced to an IEEE-754 double-float (f64).
    pub fn next(&self) -> u64 {
        if self
            .next
            .compare_exchange(MAX_ID, 1, Ordering::Relaxed, Ordering::Relaxed)
            == Ok(MAX_ID)
        {
            MAX_ID
        } else {
            self.next.fetch_add(1, Ordering::Relaxed)
        }
    }
}

#[test]
fn test_id_wrapping() {
    let counter = Counter::with_start(MAX_ID - 1);

    assert_eq!(MAX_ID - 1, counter.next());
    assert_eq!(MAX_ID, counter.next());
    assert_eq!(1, counter.next());
    assert_eq!(2, counter.next());
}
