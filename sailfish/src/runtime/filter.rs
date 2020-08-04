//! Build-in filters

// TODO: performance improvement

use std::fmt;
use std::ptr;

use super::{Buffer, Render, RenderError};

pub struct Display<'a, T>(&'a T);

impl<'a, T: fmt::Display> Render for Display<'a, T> {
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        use fmt::Write;

        write!(b, "{}", self.0).map_err(|e| RenderError::from(e))
    }
}

/// render using `std::fmt::Display` trait
#[inline]
pub fn disp<T: fmt::Display>(expr: &T) -> Display<T> {
    Display(expr)
}

pub struct Debug<'a, T>(&'a T);

impl<'a, T: fmt::Debug> Render for Debug<'a, T> {
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        use fmt::Write;

        write!(b, "{:?}", self.0).map_err(|e| RenderError::from(e))
    }
}

/// render using `std::fmt::Debug` trait
#[inline]
pub fn dbg<T: fmt::Debug>(expr: &T) -> Debug<T> {
    Debug(expr)
}

pub struct Upper<'a, T>(&'a T);

impl<'a, T: Render> Render for Upper<'a, T> {
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let old_len = b.len();
        self.0.render(b)?;

        let s = b.as_str()[old_len..].to_uppercase();
        unsafe { b._set_len(old_len) };
        b.push_str(&*s);
        Ok(())
    }
}

/// convert the rendered contents to uppercase
#[inline]
pub fn upper<T: Render>(expr: &T) -> Upper<T> {
    Upper(expr)
}

pub struct Lower<'a, T>(&'a T);

impl<'a, T: Render> Render for Lower<'a, T> {
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let old_len = b.len();
        self.0.render(b)?;

        let s = b.as_str()[old_len..].to_lowercase();
        unsafe { b._set_len(old_len) };
        b.push_str(&*s);
        Ok(())
    }

    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let old_len = b.len();
        self.0.render_escaped(b)?;

        let s = b.as_str()[old_len..].to_lowercase();
        unsafe { b._set_len(old_len) };
        b.push_str(&*s);
        Ok(())
    }
}

/// convert the rendered contents to lowercase
#[inline]
pub fn lower<T: Render>(expr: &T) -> Lower<T> {
    Lower(expr)
}

pub struct Trim<'a, T>(&'a T);

impl<'a, T: Render> Render for Trim<'a, T> {
    #[inline]
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let old_len = b.len();
        self.0.render(b)?;
        trim_impl(b, old_len);
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let old_len = b.len();
        self.0.render_escaped(b)?;
        trim_impl(b, old_len);
        Ok(())
    }
}

fn trim_impl(b: &mut Buffer, old_len: usize) {
    debug_assert!(b.len() >= old_len);
    let new_contents = &b.as_str()[old_len..];
    let trimmed = new_contents.trim();
    let trimmed_len = trimmed.len();

    if new_contents.len() != trimmed_len {
        // performs inplace trimming

        if new_contents.as_ptr() != trimmed.as_ptr() {
            debug_assert!(new_contents.as_ptr() < trimmed.as_ptr());
            let offset = trimmed.as_ptr() as usize - new_contents.as_ptr() as usize;
            unsafe {
                ptr::copy(
                    b.as_mut_ptr().add(old_len + offset),
                    b.as_mut_ptr().add(old_len),
                    trimmed_len,
                );
            }
        }

        debug_assert!(b.capacity() >= old_len + trimmed_len);
        unsafe {
            b._set_len(old_len + trimmed_len);
        }
    }
}

/// convert the rendered contents to lowercase
#[inline]
pub fn trim<T: Render>(expr: &T) -> Trim<T> {
    Trim(expr)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case() {
        let mut buf = Buffer::new();
        upper(&"hElLO, WOrLd!").render(&mut buf).unwrap();
        assert_eq!(buf.as_str(), "HELLO, WORLD!");

        buf.clear();
        lower(&"hElLO, WOrLd!").render(&mut buf).unwrap();
        assert_eq!(buf.as_str(), "hello, world!");

        buf.clear();
        lower(&"<h1>TITLE</h1>").render_escaped(&mut buf).unwrap();
        assert_eq!(buf.as_str(), "&lt;h1&gt;title&lt;/h1&gt;");
    }

    #[test]
    fn trim_test() {
        let mut buf = Buffer::new();
        trim(&" hello  ").render(&mut buf).unwrap();
        trim(&"hello ").render(&mut buf).unwrap();
        trim(&"   hello").render(&mut buf).unwrap();
        assert_eq!(buf.as_str(), "hellohellohello");

        let mut buf = Buffer::new();
        trim(&"hello ").render(&mut buf).unwrap();
        trim(&" hello").render(&mut buf).unwrap();
        trim(&"hello").render(&mut buf).unwrap();
        assert_eq!(buf.as_str(), "hellohellohello");

        let mut buf = Buffer::new();
        trim(&" hello").render(&mut buf).unwrap();
        assert_eq!(buf.as_str(), "hello");
    }
}
