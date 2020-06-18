use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::fmt;
use std::mem::{align_of, ManuallyDrop};
use std::ops::{Add, AddAssign};
use std::ptr;

#[cfg(sailfish_nightly)]
macro_rules! unlikely {
    ($val:expr) => {
        std::intrinsics::unlikely($val)
    };
}

#[cfg(not(sailfish_nightly))]
macro_rules! unlikely {
    ($val:expr) => {
        $val
    };
}

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
    #[inline]
    pub const fn new() -> Buffer {
        Self {
            data: align_of::<u8>() as *mut u8, // dangling pointer
            len: 0,
            capacity: 0,
        }
    }

    #[inline]
    pub fn with_capacity(n: usize) -> Buffer {
        unsafe {
            if n == 0 {
                Self::new()
            } else {
                let layout = Layout::from_size_align_unchecked(n, 1);
                let data = alloc(layout);
                if data.is_null() {
                    handle_alloc_error(layout);
                }

                Self {
                    data,
                    len: 0,
                    capacity: n,
                }
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

    #[inline]
    pub fn reserve(&mut self, size: usize) {
        if unlikely!(self.len + size > self.capacity) {
            self.reserve_internal(size);
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.len = 0;
    }

    #[inline]
    pub fn into_string(self) -> String {
        let buf = ManuallyDrop::new(self);
        unsafe { String::from_raw_parts(buf.data, buf.len, buf.capacity) }
    }

    #[inline]
    pub fn push_str(&mut self, data: &str) {
        let size = data.len();
        self.reserve(size);
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

    #[inline]
    fn reserve_internal(&mut self, size: usize) {
        unsafe {
            let new_capacity = std::cmp::max(self.capacity * 2, self.len + size);
            self.data = self.realloc(new_capacity);
            self.capacity = new_capacity;
        }
    }

    #[inline]
    unsafe fn realloc(&self, cap: usize) -> *mut u8 {
        let data = if unlikely!(self.capacity == 0) {
            let new_layout = Layout::from_size_align_unchecked(cap, 1);
            alloc(new_layout)
        } else {
            debug_assert!(cap <= std::usize::MAX / 2, "capacity is too large");
            let old_layout = Layout::from_size_align_unchecked(self.capacity, 1);
            realloc(self.data, old_layout, cap)
        };

        if data.is_null() {
            handle_alloc_error(Layout::from_size_align_unchecked(cap, 1));
        }

        data
    }
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        unsafe {
            if self.capacity == 0 {
                Self::new()
            } else {
                let layout = Layout::from_size_align_unchecked(self.len, 1);
                let data = alloc(layout);
                if data.is_null() {
                    handle_alloc_error(layout);
                }

                let buf = Self {
                    data,
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
    #[inline]
    fn from(other: String) -> Buffer {
        let mut other = ManuallyDrop::new(other);
        Buffer {
            data: other.as_mut_ptr(),
            len: other.len(),
            capacity: other.capacity(),
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
        assert_eq!(buffer.len(), 0);
        assert_eq!(buffer.capacity(), 0);

        buffer.push_str("apple");
        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.capacity(), 5);

        buffer.push_str("pie");
        assert_eq!(buffer.len(), 8);
        assert_eq!(buffer.capacity(), 10);
    }

    #[test]
    fn test2() {
        let mut buffer = Buffer::with_capacity(1);
        assert!(buffer.is_empty());
        assert_eq!(buffer.len(), 0);
        assert!(buffer.capacity() >= 1);

        buffer += "pie";
        assert!(!buffer.is_empty());
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
