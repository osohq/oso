use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use crate::numerics::MOST_POSITIVE_EXACT_FLOAT;

const MAX_ID: u64 = (MOST_POSITIVE_EXACT_FLOAT - 1) as u64;

#[derive(Clone)]
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
        if self.next.compare_and_swap(MAX_ID, 1, Ordering::SeqCst) == MAX_ID {
            MAX_ID
        } else {
            self.next.fetch_add(1, Ordering::SeqCst)
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
