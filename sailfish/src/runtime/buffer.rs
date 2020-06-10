use std::alloc::{alloc, dealloc, realloc, Layout};
use std::fmt;
use std::mem::ManuallyDrop;
use std::ops::{Add, AddAssign};
use std::ptr;

const MEMORY_LAYOUT: Layout = unsafe { Layout::from_size_align_unchecked(1, 1) };

/// Buffer for rendered contents
///
/// This struct is quite simular to `String`, but some methods are
/// re-implemented for faster buffering.
#[derive(Clone, Debug)]
pub struct Buffer {
    data: *mut u8,
    len: usize,
    capacity: usize,
}

impl Buffer {
    #[inline]
    pub const fn new() -> Buffer {
        Self {
            data: ptr::null_mut(),
            len: 0,
            capacity: 0,
        }
    }

    #[inline]
    pub fn with_capacity(n: usize) -> Buffer {
        unsafe {
            let layout = Layout::from_size_align_unchecked(n, 1);
            let data = alloc(layout);
            Self {
                data,
                len: 0,
                capacity: n,
            }
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        unsafe {
            let bytes = std::slice::from_raw_parts(self.data, self.len);
            std::str::from_utf8_unchecked(bytes)
        }
    }

    #[inline]
    pub fn as_mut_ptr(&self) -> *mut u8 {
        self.data
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Force the length of buffer to `new_len`
    ///
    /// # Safety
    ///
    /// - `new_len` must be less than or equal to `capacity()`
    /// - The elements at `old_len..new_len` must be initialized
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        self.len = new_len;
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn reserve(&mut self, size: usize) {
        if size > self.capacity - self.len {
            unsafe {
                let new_capacity = std::cmp::max(self.capacity * 2, self.len + size);
                self.realloc(new_capacity);
                self.capacity = new_capacity;
            }
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    pub fn into_string(self) -> String {
        if self.capacity == 0 {
            std::mem::forget(self);
            String::new()
        } else {
            let buf = ManuallyDrop::new(self);
            unsafe { String::from_raw_parts(buf.data, buf.len, buf.capacity) }
        }
    }

    #[inline]
    pub fn push_str(&mut self, data: &str) {
        let size = data.len();
        if size > self.capacity - self.len {
            self.reserve(size);
        }
        unsafe {
            let p = self.data.add(self.len);
            std::ptr::copy_nonoverlapping(data.as_ptr(), p, size);
            self.len += size;
        }
    }

    #[inline]
    pub fn push(&mut self, data: char) {
        let mut buf = [0u8; 4];
        self.push_str(data.encode_utf8(&mut buf));
    }

    unsafe fn realloc(&mut self, cap: usize) {
        if self.data.is_null() {
            let new_layout = Layout::from_size_align_unchecked(cap, 1);
            self.data = alloc(new_layout);
        } else {
            let old_layout = Layout::from_size_align_unchecked(self.capacity, 1);
            self.data = realloc(self.data, old_layout, cap);
        }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if !self.data.is_null() {
            unsafe {
                dealloc(self.data, MEMORY_LAYOUT);
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
    #[inline]
    fn from(other: String) -> Buffer {
        if other.capacity() > 0 {
            let mut other = ManuallyDrop::new(other);
            Buffer {
                data: other.as_mut_ptr(),
                len: other.len(),
                capacity: other.capacity(),
            }
        } else {
            Buffer::new()
        }
    }
}

impl From<&str> for Buffer {
    #[inline]
    fn from(other: &str) -> Buffer {
        Buffer::from(other.to_owned())
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

#[cfg(test)]
mod tests {
    use super::Buffer;

    #[test]
    fn test1() {
        let mut buffer = Buffer::new();
        assert!(buffer.data.is_null());
        assert_eq!(buffer.len, 0);
        assert_eq!(buffer.capacity, 0);

        buffer.push_str("apple");
        assert!(!buffer.data.is_null());
        assert_eq!(buffer.len, 5);
        assert_eq!(buffer.capacity, 5);

        buffer.push_str("pie");
        assert!(!buffer.data.is_null());
        assert_eq!(buffer.len, 8);
        assert_eq!(buffer.capacity, 10);
    }

    #[test]
    fn string_conversion() {
        // from empty string
        let s = String::new();
        let mut buf = Buffer::from(s);
        assert_eq!(buf.as_str(), "");
        buf.push_str("abc");
        assert_eq!(buf.as_str(), "abc");

        // into non-empty string
        let mut s = buf.into_string();
        assert_eq!(s, "abc");

        s.push_str("defghijklmn");
        assert_eq!(s, "abcdefghijklmn");

        // from non-empty string
        let mut buf = Buffer::from(s);
        assert_eq!(buf.as_str(), "abcdefghijklmn");
        buf.clear();
        assert_eq!(buf.as_str(), "");

        // into empty string
        let buf = Buffer::new();
        let mut s = buf.into_string();
        assert_eq!(s, "");

        s.push_str("apple");
        assert_eq!(s, "apple");
    }
}
