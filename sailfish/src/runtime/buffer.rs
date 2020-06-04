use std::fmt;
use std::ops::{Add, AddAssign};

#[derive(Clone, Debug)]
pub struct Buffer {
    inner: String,
}

impl Buffer {
    #[inline]
    pub const fn new() -> Buffer {
        Self {
            inner: String::new(),
        }
    }

    #[inline]
    pub fn with_capacity(n: usize) -> Buffer {
        Self {
            inner: String::with_capacity(n),
        }
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        &*self.inner
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    pub unsafe fn set_len(&mut self, new: usize) {
        self.inner.as_mut_vec().set_len(new);
    }

    #[inline]
    pub fn reserve(&mut self, n: usize) {
        if n > self.inner.capacity() - self.inner.len() {
            self.inner.reserve(n);
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        // unsafe { self.inner.set_len(0) };
        self.inner.clear();
    }

    #[inline]
    pub fn into_string(self) -> String {
        self.inner
    }

    #[inline]
    pub fn write_str(&mut self, data: &str) {
        let inner_len = self.inner.len();
        let size = data.len();
        if size > self.inner.capacity() - self.inner.len() {
            self.inner.reserve(size);
        }
        unsafe {
            let p = self.inner.as_mut_ptr().add(self.inner.len());
            std::ptr::copy_nonoverlapping(data.as_ptr(), p, size);
            self.inner.as_mut_vec().set_len(inner_len + size);
        }
    }

    #[inline]
    pub fn write_char(&mut self, data: char) {
        // TODO: do not use standard library
        self.inner.push(data);
    }
}

impl fmt::Write for Buffer {
    #[inline]
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Buffer::write_str(self, s);
        Ok(())
    }
}

impl From<String> for Buffer {
    #[inline]
    fn from(other: String) -> Buffer {
        Buffer { inner: other }
    }
}

impl From<&str> for Buffer {
    #[inline]
    fn from(other: &str) -> Buffer {
        Buffer {
            inner: other.to_owned(),
        }
    }
}

impl Add<&str> for Buffer {
    type Output = Buffer;

    #[inline]
    fn add(mut self, other: &str) -> Buffer {
        self.write_str(other);
        self
    }
}

impl AddAssign<&str> for Buffer {
    #[inline]
    fn add_assign(&mut self, other: &str) {
        self.write_str(other)
    }
}

impl Default for Buffer {
    #[inline]
    fn default() -> Buffer {
        Buffer::new()
    }
}
