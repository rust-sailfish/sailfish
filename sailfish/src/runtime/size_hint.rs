use std::sync::atomic::{AtomicUsize, Ordering};

/// Dynamically updated size hint
#[doc(hidden)]
pub struct SizeHint {
    value: AtomicUsize,
}

impl SizeHint {
    pub const fn new() -> SizeHint {
        SizeHint {
            value: AtomicUsize::new(0),
        }
    }

    /// Get the current value
    #[inline]
    pub fn get(&self) -> usize {
        self.value.load(Ordering::Acquire)
    }

    /// Update size hint based on given value.
    ///
    /// There is no guarantee that the value of get() after calling update() is same
    /// as the value passed on update()
    #[inline]
    pub fn update(&self, mut value: usize) {
        value = value + value / 4;
        if self.get() < value {
            self.value.store(value, Ordering::Release);
        }
    }
}
