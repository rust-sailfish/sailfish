use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::fmt;
use std::mem::{align_of, ManuallyDrop};
use std::ops::{Add, AddAssign};
use std::ptr;

/// Buffer for rendered contents
///
/// This struct is quite simular to `String`, but some methods are
/// re-implemented for faster buffering.
pub struct Buffer {
    data: *mut u8,
    len: usize,
    capacity: usize,
}

impl Buffer {
    /// Create an empty buffer
    #[inline]
    pub const fn new() -> Buffer {
        Self {
            data: align_of::<u8>() as *mut u8, // dangling pointer
            len: 0,
            capacity: 0,
        }
    }

    /// Create a empty buffer with a particular capacity
    #[inline]
    pub fn with_capacity(n: usize) -> Buffer {
        if unlikely!(n == 0) {
            Self::new()
        } else {
            Self {
                data: safe_alloc(n),
                len: 0,
                capacity: n,
            }
        }
    }

    /// Extracts a string slice containing the entire buffer
    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe {
            let bytes = std::slice::from_raw_parts(self.data, self.len);
            std::str::from_utf8_unchecked(bytes)
        }
    }

    /// Returns an unsafe mutable pointer to the inner data
    #[inline]
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.data
    }

    /// Returns the length of this buffer in bytes
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns this buffer's capacity in bytes
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    #[inline]
    #[doc(hidden)]
    pub unsafe fn _set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.capacity);
        self.len = new_len;
    }

    /// Increase the length of buffer by `additional` bytes
    ///
    /// # Safety
    ///
    /// - `additional` must be less than or equal to `capacity() - len()`
    /// - The elements at `old_len..old_len + additional` must be initialized
    #[inline]
    pub unsafe fn advance(&mut self, additional: usize) {
        self.len += additional;
    }

    /// Returns `true` if this buffer has a length of zero, and `false` otherwise
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Same as String::reserve
    ///
    /// # Panics
    ///
    /// This method panics if `size` overflows `isize::MAX`.
    #[inline]
    pub fn reserve(&mut self, size: usize) {
        if likely!(size <= self.capacity - self.len) {
            return;
        }
        self.reserve_internal(size);
    }

    /// Same as String::reserve except that undefined behaviour can result if `size`
    /// overflows `isize::MAX`.
    #[inline]
    pub(crate) unsafe fn reserve_small(&mut self, size: usize) {
        debug_assert!(size <= std::isize::MAX as usize);
        if likely!(self.len + size <= self.capacity) {
            return;
        }
        self.reserve_internal(size);
    }

    #[inline]
    #[doc(hidden)]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    /// Converts a `Buffer` into a `String` without copy/realloc operation.
    #[inline]
    pub fn into_string(self) -> String {
        debug_assert!(self.len <= self.capacity);
        let buf = ManuallyDrop::new(self);

        // SAFETY: This operations satisfy all requirements specified in
        // https://doc.rust-lang.org/std/string/struct.String.html#safety
        unsafe { String::from_raw_parts(buf.data, buf.len, buf.capacity) }
    }

    /// Appends a given string slice onto the end of this buffer
    #[inline]
    pub fn push_str(&mut self, data: &str) {
        let size = data.len();

        unsafe {
            // SAFETY: this operation won't overflow because slice cannot exceeds
            // isize::MAX bytes.
            // https://doc.rust-lang.org/reference/behavior-considered-undefined.html
            self.reserve_small(size);

            let p = self.data.add(self.len);
            std::ptr::copy_nonoverlapping(data.as_ptr(), p, size);
            self.len += size;
        }
        debug_assert!(self.len <= self.capacity);
    }

    /// Appends the given `char` to the end of this buffer
    #[inline]
    pub fn push(&mut self, data: char) {
        // Question: Is it safe to pass uninitialized memory to `encode_utf8` function?
        unsafe {
            self.reserve_small(4);
            let bp = self.data.add(self.len) as *mut [u8; 4];
            let result = data.encode_utf8(&mut *bp);
            self.len += result.len();
        }
    }

    #[cfg_attr(feature = "perf-inline", inline)]
    fn reserve_internal(&mut self, size: usize) {
        debug_assert!(size <= std::isize::MAX as usize);

        let new_capacity = std::cmp::max(self.capacity * 2, self.capacity + size);
        debug_assert!(new_capacity > self.capacity);
        self.data = unsafe { safe_realloc(self.data, self.capacity, new_capacity) };
        self.capacity = new_capacity;

        debug_assert!(!self.data.is_null());
        debug_assert!(self.len <= self.capacity);
    }
}

#[inline(never)]
fn safe_alloc(capacity: usize) -> *mut u8 {
    assert!(capacity > 0);
    assert!(
        capacity <= std::isize::MAX as usize,
        "capacity is too large"
    );

    // SAFETY: capacity is non-zero, and always multiple of alignment (1).
    unsafe {
        let layout = Layout::from_size_align_unchecked(capacity, 1);
        let data = alloc(layout);
        if data.is_null() {
            handle_alloc_error(layout);
        }

        data
    }
}

/// # Safety
///
/// - if `capacity > 0`, `capacity` is the same value that was used to allocate the block
/// of memory pointed by `ptr`.
#[cold]
#[inline(never)]
unsafe fn safe_realloc(ptr: *mut u8, capacity: usize, new_capacity: usize) -> *mut u8 {
    assert!(new_capacity > 0);
    assert!(
        new_capacity <= std::isize::MAX as usize,
        "capacity is too large"
    );

    let data = if unlikely!(capacity == 0) {
        let new_layout = Layout::from_size_align_unchecked(new_capacity, 1);
        alloc(new_layout)
    } else {
        let old_layout = Layout::from_size_align_unchecked(capacity, 1);
        realloc(ptr, old_layout, new_capacity)
    };

    if data.is_null() {
        handle_alloc_error(Layout::from_size_align_unchecked(new_capacity, 1));
    }

    data
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        unsafe {
            if self.is_empty() {
                Self::new()
            } else {
                let buf = Self {
                    data: safe_alloc(self.len),
                    len: self.len,
                    capacity: self.len,
                };

                ptr::copy_nonoverlapping(self.data, buf.data, self.len);
                buf
            }
        }
    }
}

impl fmt::Debug for Buffer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.capacity != 0 {
            // SAFETY: when `self.capacity > 0`, `self.capacity` is the same value
            // used for allocate the block of memory pointed by `self.data`.
            unsafe {
                let layout = Layout::from_size_align_unchecked(self.capacity, 1);
                dealloc(self.data, layout);
            }
        }
    }
}

impl fmt::Write for Buffer {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Buffer::push_str(self, s);
        Ok(())
    }
}

impl From<String> for Buffer {
    /// Shrink the data and pass raw pointer directory to buffer
    ///
    /// This operation is `O(1)`
    #[inline]
    fn from(other: String) -> Buffer {
        let bs = other.into_boxed_str();
        let data = Box::leak(bs);
        Buffer {
            data: data.as_mut_ptr(),
            len: data.len(),
            capacity: data.len(),
        }
    }
}

impl From<&str> for Buffer {
    #[inline]
    fn from(other: &str) -> Buffer {
        let mut buf = Buffer::with_capacity(other.len());

        if !other.is_empty() {
            // SAFETY: `Buffer.capacity()` should be same as `other.len()`, so if `other`
            // is not empty, `buf.as_mut_ptr()` is supporsed to point to valid memory.
            unsafe {
                ptr::copy_nonoverlapping(other.as_ptr(), buf.as_mut_ptr(), other.len());
                buf.advance(other.len());
            }
        }

        buf
    }
}

impl Add<&str> for Buffer {
    type Output = Buffer;

    #[inline]
    fn add(mut self, other: &str) -> Buffer {
        self.push_str(other);
        self
    }
}

impl AddAssign<&str> for Buffer {
    #[inline]
    fn add_assign(&mut self, other: &str) {
        self.push_str(other)
    }
}

impl Default for Buffer {
    #[inline]
    fn default() -> Buffer {
        Buffer::new()
    }
}

unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;

    #[test]
    fn push_str() {
        let mut buffer = Buffer::new();
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 0);

        buffer.push_str("apple");
        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.capacity(), 5);

        buffer.push_str("pie");
        assert_eq!(buffer.len(), 8);
        assert_eq!(buffer.capacity(), 10);

        for _ in 0..16 {
            buffer.push_str("zomg");
        }

        assert_eq!(buffer.len(), 72);
        assert_eq!(buffer.capacity(), 80);
    }

    #[test]
    fn with_capacity() {
        let buffer = Buffer::with_capacity(1);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert!(buffer.capacity() >= 1);
    }

    #[test]
    fn string_conversion() {
        // from empty string
        let s = String::with_capacity(2);
        assert!(s.capacity() >= 2);

        let mut buf = Buffer::from(s);
        assert_eq!(buf.as_str(), "");

        // capacity should be shrinked for safety
        assert_eq!(buf.capacity(), 0);

        buf.push_str("abc");
        assert_eq!(buf.as_str(), "abc");

        // into non-empty string
        let mut s = buf.into_string();
        assert_eq!(s, "abc");

        s += "defghijklmn";
        assert_eq!(s, "abcdefghijklmn");

        // from non-empty string
        let mut buf = Buffer::from(s);
        assert_eq!(buf.as_str(), "abcdefghijklmn");
        buf.clear();
        assert_eq!(buf.as_str(), "");

        // into empty string
        let buf = Buffer::default();
        let mut s = buf.into_string();
        assert_eq!(s, "");

        s.push_str("apple");
        assert_eq!(s, "apple");
    }

    #[test]
    fn from_str() {
        let buf = Buffer::from("abcdefgh");
        assert_eq!(buf.as_str(), "abcdefgh");
    }

    #[test]
    fn clone() {
        use std::fmt::Write;

        let mut s1 = Buffer::with_capacity(0);
        let mut s2 = s1.clone();

        s1.push('a');
        s2.push_str("b");

        assert_eq!(s1.as_str(), "a");
        assert_eq!(s2.as_str(), "b");

        let mut s1 = Buffer::from("foo");
        let mut s2 = s1.clone();

        s1 = s1 + "bar";
        write!(s2, "baz").unwrap();

        assert_eq!(s1.as_str(), "foobar");
        assert_eq!(s2.as_str(), "foobaz");

        s2.clear();
        let _ = s2.clone();
    }

    #[test]
    fn push() {
        for initial_capacity in &[0, 4, 16] {
            let mut s = Buffer::with_capacity(*initial_capacity);

            s.push('a');
            s.push('é');
            s.push('Ａ');
            s.push('🄫');

            assert_eq!(s.as_str(), "aéＡ🄫");
        }
    }

    #[test]
    fn multi_thread() {
        const THREADS: usize = 8;
        const ITERS: usize = 100;

        let barrier = Arc::new(Barrier::new(THREADS));
        let buffer = Arc::new(Mutex::new(Buffer::new()));
        let mut handles = Vec::with_capacity(THREADS);

        for _ in 0..THREADS {
            let barrier = barrier.clone();
            let buffer = buffer.clone();

            handles.push(thread::spawn(move || {
                barrier.wait();
                for _ in 0..ITERS {
                    buffer.lock().unwrap().push_str("a");
                }
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(buffer.lock().unwrap().as_str(), "a".repeat(ITERS * THREADS));
    }

    #[test]
    #[should_panic]
    fn reserve_overflow() {
        let mut buf = Buffer::new();
        buf.reserve(std::isize::MAX as usize + 1);
    }

    #[test]
    #[should_panic]
    fn empty_alloc() {
        safe_alloc(0);
    }
}
