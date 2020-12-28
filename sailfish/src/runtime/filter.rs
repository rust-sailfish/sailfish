//! Build-in filters

use std::fmt;
use std::ptr;

use super::{Buffer, Render, RenderError};

/// Helper struct for 'display' filter
pub struct Display<'a, T: ?Sized>(&'a T);

impl<'a, T: fmt::Display + ?Sized> Render for Display<'a, T> {
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        use fmt::Write;

        write!(b, "{}", self.0).map_err(|e| RenderError::from(e))
    }
}

/// render using `std::fmt::Display` trait
///
/// # Examples
///
/// ```text
/// filename: <%= filename.display() | disp %>
/// ```
#[inline]
pub fn disp<T: fmt::Display + ?Sized>(expr: &T) -> Display<T> {
    Display(expr)
}

/// Helper struct for 'dbg' filter
pub struct Debug<'a, T: ?Sized>(&'a T);

impl<'a, T: fmt::Debug + ?Sized> Render for Debug<'a, T> {
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        use fmt::Write;

        write!(b, "{:?}", self.0).map_err(|e| RenderError::from(e))
    }
}

/// render using `std::fmt::Debug` trait
///
/// # Examples
///
/// The following examples produce exactly same results, but former is a bit faster
///
/// ```text
/// table content: <%= table | dbg %>
/// ```
///
/// ```text
/// table content: <%= format!("{:?}", table) %>
/// ```
#[inline]
pub fn dbg<T: fmt::Debug + ?Sized>(expr: &T) -> Debug<T> {
    Debug(expr)
}

/// Helper struct for 'upper' filter
pub struct Upper<'a, T: ?Sized>(&'a T);

impl<'a, T: Render + ?Sized> Render for Upper<'a, T> {
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let old_len = b.len();
        self.0.render(b)?;

        let content = b
            .as_str()
            .get(old_len..)
            .ok_or_else(|| RenderError::BufSize)?;
        let s = content.to_uppercase();
        unsafe { b._set_len(old_len) };
        b.push_str(&*s);
        Ok(())
    }
}

/// convert the rendered contents to uppercase
///
/// # Examples
///
/// ```text
/// <%= "tschüß" | upper %>
/// ```
///
/// result:
///
/// ```text
/// TSCHÜSS
/// ```
#[inline]
pub fn upper<T: Render + ?Sized>(expr: &T) -> Upper<T> {
    Upper(expr)
}

/// Helper struct for 'lower' filter
pub struct Lower<'a, T: ?Sized>(&'a T);

impl<'a, T: Render + ?Sized> Render for Lower<'a, T> {
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let old_len = b.len();
        self.0.render(b)?;

        let content = b
            .as_str()
            .get(old_len..)
            .ok_or_else(|| RenderError::BufSize)?;
        let s = content.to_lowercase();
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
///
/// # Examples
///
/// ```text
/// <%= "ὈΔΥΣΣΕΎΣ" | lower %>
/// ```
///
/// result:
///
/// ```text
/// ὀδυσσεύς
/// ```
#[inline]
pub fn lower<T: Render + ?Sized>(expr: &T) -> Lower<T> {
    Lower(expr)
}

/// Helper struct for 'trim' filter
pub struct Trim<'a, T: ?Sized>(&'a T);

impl<'a, T: Render + ?Sized> Render for Trim<'a, T> {
    #[inline]
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let old_len = b.len();
        self.0.render(b)?;
        trim_impl(b, old_len)
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let old_len = b.len();
        self.0.render_escaped(b)?;
        trim_impl(b, old_len)
    }
}

fn trim_impl(b: &mut Buffer, old_len: usize) -> Result<(), RenderError> {
    let new_contents = b
        .as_str()
        .get(old_len..)
        .ok_or_else(|| RenderError::BufSize)?;

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

        // SAFETY: `new_contents.len() = b.len() - old_len` and
        // `trimmed_len < new_contents.len()`, so `old_len + trimmed_len < b.len()`.
        unsafe {
            b._set_len(old_len + trimmed_len);
        }
    }

    Ok(())
}

/// Remove leading and trailing writespaces from rendered results
///
/// # Examples
///
/// ```text
/// <%= " Hello world\n" | trim %>
/// ```
///
/// result:
///
/// ```text
/// Hello world
/// ```
#[inline]
pub fn trim<T: Render + ?Sized>(expr: &T) -> Trim<T> {
    Trim(expr)
}

/// Helper struct for 'truncate' filter
pub struct Truncate<'a, T: ?Sized>(&'a T, usize);

impl<'a, T: Render + ?Sized> Render for Truncate<'a, T> {
    #[inline]
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let old_len = b.len();
        self.0.render(b)?;
        truncate_impl(b, old_len, self.1)
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let old_len = b.len();
        self.0.render_escaped(b)?;
        truncate_impl(b, old_len, self.1)
    }
}

fn truncate_impl(
    b: &mut Buffer,
    old_len: usize,
    limit: usize,
) -> Result<(), RenderError> {
    let new_contents = b
        .as_str()
        .get(old_len..)
        .ok_or_else(|| RenderError::BufSize)?;

    if let Some(idx) = new_contents.char_indices().nth(limit).map(|(i, _)| i) {
        unsafe { b._set_len(old_len.wrapping_add(idx)) };
        b.push_str("...");
    }

    Ok(())
}

/// Limit length of rendered contents, appends '...' if truncated
///
/// # Examples
///
/// The following example renders the first 20 characters of `message`
///
/// ```test
/// <%= "Hello, world!" | truncate(5) %>
/// ```
///
/// result:
///
/// ```text
/// Hello...
/// ```
#[inline]
pub fn truncate<T: Render + ?Sized>(expr: &T, limit: usize) -> Truncate<T> {
    Truncate(expr, limit)
}

cfg_json! {
    /// Helper struct for 'json' filter
    pub struct Json<'a, T: ?Sized>(&'a T);

    impl<'a, T: serde::Serialize + ?Sized> Render for Json<'a, T> {
        #[inline]
        fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
            struct Writer<'a>(&'a mut Buffer);

            impl<'a> std::io::Write for Writer<'a> {
                #[inline]
                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                    let buf = unsafe { std::str::from_utf8_unchecked(buf) };
                    self.0.push_str(buf);
                    Ok(buf.len())
                }

                #[inline]
                fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
                    self.write(buf).map(|_| {})
                }

                #[inline]
                fn flush(&mut self) -> std::io::Result<()> {
                    Ok(())
                }
            }

            serde_json::to_writer(Writer(b), self.0)
                .map_err(|e| RenderError::new(&e.to_string()))
        }

        #[inline]
        fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
            use super::escape::escape_to_buf;

            struct Writer<'a>(&'a mut Buffer);

            impl<'a> std::io::Write for Writer<'a> {
                #[inline]
                fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
                    let buf = unsafe { std::str::from_utf8_unchecked(buf) };
                    escape_to_buf(buf, self.0);
                    Ok(buf.len())
                }

                #[inline]
                fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
                    self.write(buf).map(|_| {})
                }

                #[inline]
                fn flush(&mut self) -> std::io::Result<()> {
                    Ok(())
                }
            }

            serde_json::to_writer(Writer(b), self.0)
                .map_err(|e| RenderError::new(&e.to_string()))
        }
    }

    /// Serialize the given data structure as JSON into the buffer
    ///
    /// # Examples
    ///
    /// ```text
    /// {
    ///     "name": "JSON example",
    ///     "data": <%- data | json %>
    /// }
    /// ```
    #[inline]
    pub fn json<T: serde::Serialize + ?Sized>(expr: &T) -> Json<T> {
        Json(expr)
    }
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
