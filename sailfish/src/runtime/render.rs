use std::borrow::Cow;
use std::cell::{Ref, RefMut};
use std::fmt;
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize,
    NonZeroU128, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize, Wrapping,
};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, MutexGuard, RwLockReadGuard, RwLockWriteGuard};

use super::buffer::Buffer;
use super::escape;

/// types which can be rendered inside buffer block (`<%= %>`)
///
/// If you want to render the custom data, you must implement this trait and specify
/// the behaviour.
///
/// # Safety
///
/// This trait allows modifying the previously-rendered contents or even decreasing the
/// buffer size. However, such an operation easily cause unexpected rendering results.
/// In order to avoid this, implementors should ensure that the contents which is already
/// rendered won't be changed during `render` or `render_escaped` method is called.
///
/// # Examples
///
/// ```
/// use sailfish::runtime::{Buffer, Render, RenderError};
///
/// struct MyU64(u64);
///
/// impl Render for MyU64 {
///     #[inline]
///     fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
///         self.0.render(b)
///     }
/// }
/// ```
pub trait Render {
    /// render to `Buffer` without escaping
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError>;

    /// render to `Buffer` with HTML escaping
    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let mut tmp = Buffer::new();
        self.render(&mut tmp)?;
        escape::escape_to_buf(tmp.as_str(), b);
        Ok(())
    }
}

// /// Autoref-based stable specialization
// ///
// /// Explanation can be found [here](https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md)
// impl<T: Display> Render for &T {
//     fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
//         fmt::write(b, format_args!("{}", self))
//     }
//
//     fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
//         struct Wrapper<'a>(&'a mut Buffer);
//
//         impl<'a> fmt::Write for Wrapper<'a> {
//             #[inline]
//             fn push_str(&mut self, s: &str) -> Result<(), RenderError> {
//                 escape::escape_to_buf(s, self.0);
//                 Ok(())
//             }
//         }
//
//         fmt::write(&mut Wrapper(b), format_args!("{}", self))
//     }
// }

impl Render for String {
    #[inline]
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        b.push_str(&**self);
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        escape::escape_to_buf(&**self, b);
        Ok(())
    }
}

impl Render for str {
    #[inline]
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        b.push_str(self);
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        escape::escape_to_buf(self, b);
        Ok(())
    }
}

impl Render for char {
    #[inline]
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        b.push(*self);
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        match *self {
            '\"' => b.push_str("&quot;"),
            '&' => b.push_str("&amp;"),
            '<' => b.push_str("&lt;"),
            '>' => b.push_str("&gt;"),
            '\'' => b.push_str("&#039;"),
            _ => b.push(*self),
        }
        Ok(())
    }
}

impl Render for PathBuf {
    #[inline]
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        // TODO: speed up on Windows using OsStrExt
        b.push_str(&*self.to_string_lossy());
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        escape::escape_to_buf(&*self.to_string_lossy(), b);
        Ok(())
    }
}

impl Render for Path {
    #[inline]
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        // TODO: speed up on Windows using OsStrExt
        b.push_str(&*self.to_string_lossy());
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        escape::escape_to_buf(&*self.to_string_lossy(), b);
        Ok(())
    }
}

// impl Render for [u8] {
//     #[inline]
//     fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
//         b.write_bytes(self);
//         Ok(())
//     }
// }
//
// impl<'a> Render for &'a [u8] {
//     #[inline]
//     fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
//         b.write_bytes(self);
//         Ok(())
//     }
// }
//
// impl Render for Vec<u8> {
//     #[inline]
//     fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
//         b.write_bytes(&**self);
//         Ok(())
//     }
// }

impl Render for bool {
    #[inline]
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let s = if *self { "true" } else { "false" };
        b.push_str(s);
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        self.render(b)
    }
}

macro_rules! render_int {
    ($($int:ty),*) => {
        $(
            impl Render for $int {
                #[cfg_attr(feature = "perf-inline", inline)]
                fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
                    use itoap::Integer;

                    // SAFETY: `MAX_LEN < 40` and then does not overflows `isize::MAX`.
                    // Also `b.len()` should be always less than or equal to `isize::MAX`.
                    unsafe {
                        b.reserve_small(Self::MAX_LEN);
                        let ptr = b.as_mut_ptr().add(b.len());

                        // SAFETY: `MAX_LEN` is always greater than zero, so
                        // `b.as_mut_ptr()` always point to valid block of memory
                        let l = itoap::write_to_ptr(ptr, *self);
                        b.advance(l);
                    }
                    debug_assert!(b.len() <= b.capacity());
                    Ok(())
                }

                #[inline]
                fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
                    // push_str without escape
                    self.render(b)
                }
            }
        )*
    }
}

render_int!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize);

impl Render for f32 {
    #[cfg_attr(feature = "perf-inline", inline)]
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        if likely!(self.is_finite()) {
            unsafe {
                b.reserve_small(16);
                let ptr = b.as_mut_ptr().add(b.len());
                let l = ryu::raw::format32(*self, ptr);
                b.advance(l);
                debug_assert!(b.len() <= b.capacity());
            }
        } else if self.is_nan() {
            b.push_str("NaN");
        } else if *self > 0.0 {
            b.push_str("inf");
        } else {
            b.push_str("-inf");
        }

        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        // escape string
        self.render(b)
    }
}

impl Render for f64 {
    #[cfg_attr(feature = "perf-inline", inline)]
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        if likely!(self.is_finite()) {
            unsafe {
                b.reserve_small(24);
                let ptr = b.as_mut_ptr().add(b.len());
                let l = ryu::raw::format64(*self, ptr);
                b.advance(l);
                debug_assert!(b.len() <= b.capacity());
            }
        } else if self.is_nan() {
            b.push_str("NaN");
        } else if *self > 0.0 {
            b.push_str("inf");
        } else {
            b.push_str("-inf");
        }

        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        // escape string
        self.render(b)
    }
}

macro_rules! render_deref {
    (
        $(#[doc = $doc:tt])*
        [$($bounds:tt)+] $($desc:tt)+
    ) => {
        $(#[doc = $doc])*
        impl <$($bounds)+> Render for $($desc)+ {
            #[inline]
            fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
                (**self).render(b)
            }

            #[inline]
            fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
                (**self).render_escaped(b)
            }
        }
    };
}

render_deref!(['a, T: Render + ?Sized] &'a T);
render_deref!(['a, T: Render + ?Sized] &'a mut T);
render_deref!([T: Render + ?Sized] Box<T>);
render_deref!([T: Render + ?Sized] Rc<T>);
render_deref!([T: Render + ?Sized] Arc<T>);
render_deref!(['a, T: Render + ToOwned + ?Sized] Cow<'a, T>);
render_deref!(['a, T: Render + ?Sized] Ref<'a, T>);
render_deref!(['a, T: Render + ?Sized] RefMut<'a, T>);
render_deref!(['a, T: Render + ?Sized] MutexGuard<'a, T>);
render_deref!(['a, T: Render + ?Sized] RwLockReadGuard<'a, T>);
render_deref!(['a, T: Render + ?Sized] RwLockWriteGuard<'a, T>);

macro_rules! render_nonzero {
    ($($type:ty,)*) => {
        $(
            impl Render for $type {
                #[inline]
                fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
                    self.get().render(b)
                }

                #[inline]
                fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
                    self.get().render_escaped(b)
                }
            }
        )*
    }
}

render_nonzero!(
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroIsize,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroUsize,
);

impl<T: Render> Render for Wrapping<T> {
    #[inline]
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError> {
        self.0.render(b)
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        self.0.render_escaped(b)
    }
}

/// The error type which is returned from template function
#[derive(Clone, Debug)]
pub enum RenderError {
    /// Custom error message
    Msg(String),
    /// fmt::Error was raised during rendering
    Fmt(fmt::Error),
    /// Buffer size shrinked during rendering
    ///
    /// This method won't be raised unless you implement `Render` trait for custom type.
    ///
    /// Also there is no guarentee that this error will be returned whenever the buffer
    /// size shrinked.
    BufSize,
}

impl RenderError {
    /// Construct a new error with custom message
    pub fn new(msg: &str) -> Self {
        RenderError::Msg(msg.to_owned())
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RenderError::Msg(ref s) => f.pad(&**s),
            RenderError::Fmt(ref e) => fmt::Display::fmt(e, f),
            RenderError::BufSize => f.pad("buffer size shrinked while rendering"),
        }
    }
}

impl std::error::Error for RenderError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RenderError::Msg(_) | RenderError::BufSize => None,
            RenderError::Fmt(ref e) => Some(e),
        }
    }
}

impl From<fmt::Error> for RenderError {
    #[inline]
    fn from(other: fmt::Error) -> Self {
        RenderError::Fmt(other)
    }
}

/// Result type returned from `TemplateOnce::render_once` method
pub type RenderResult = Result<String, RenderError>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn receiver_coercion() {
        let mut b = Buffer::new();
        Render::render(&1, &mut b).unwrap();
        Render::render(&&1, &mut b).unwrap();
        Render::render(&&&1, &mut b).unwrap();
        Render::render(&&&&1, &mut b).unwrap();
        assert_eq!(b.as_str(), "1111");
        b.clear();

        Render::render(&true, &mut b).unwrap();
        Render::render(&&false, &mut b).unwrap();
        Render::render_escaped(&&&true, &mut b).unwrap();
        Render::render_escaped(&&&&false, &mut b).unwrap();
        assert_eq!(b.as_str(), "truefalsetruefalse");
        b.clear();

        let s = "apple";
        Render::render_escaped(&s, &mut b).unwrap();
        Render::render_escaped(&s, &mut b).unwrap();
        Render::render_escaped(&&s, &mut b).unwrap();
        Render::render_escaped(&&&s, &mut b).unwrap();
        Render::render_escaped(&&&&s, &mut b).unwrap();
        assert_eq!(b.as_str(), "appleappleappleappleapple");
        b.clear();

        Render::render_escaped(&'c', &mut b).unwrap();
        Render::render_escaped(&&'<', &mut b).unwrap();
        Render::render_escaped(&&&'&', &mut b).unwrap();
        Render::render_escaped(&&&&' ', &mut b).unwrap();
        assert_eq!(b.as_str(), "c&lt;&amp; ");
        b.clear();
    }

    #[test]
    fn deref_coercion() {
        use std::path::{Path, PathBuf};
        use std::rc::Rc;

        let mut b = Buffer::new();
        Render::render(&String::from("a"), &mut b).unwrap();
        Render::render(&&PathBuf::from("b"), &mut b).unwrap();
        Render::render_escaped(&Rc::new(4u32), &mut b).unwrap();
        Render::render_escaped(&Rc::new(2.3f32), &mut b).unwrap();
        Render::render_escaped(Path::new("<"), &mut b).unwrap();
        Render::render_escaped(&Path::new("d"), &mut b).unwrap();

        assert_eq!(b.as_str(), "ab42.3&lt;d");
    }

    #[test]
    fn float() {
        let mut b = Buffer::new();

        Render::render_escaped(&0.0f64, &mut b).unwrap();
        Render::render_escaped(&std::f64::INFINITY, &mut b).unwrap();
        Render::render_escaped(&std::f64::NEG_INFINITY, &mut b).unwrap();
        Render::render_escaped(&std::f64::NAN, &mut b).unwrap();
        assert_eq!(b.as_str(), "0.0inf-infNaN");
        b.clear();

        Render::render_escaped(&0.0f32, &mut b).unwrap();
        Render::render_escaped(&std::f32::INFINITY, &mut b).unwrap();
        Render::render_escaped(&std::f32::NEG_INFINITY, &mut b).unwrap();
        Render::render_escaped(&std::f32::NAN, &mut b).unwrap();
        assert_eq!(b.as_str(), "0.0inf-infNaN");
    }

    #[test]
    fn test_char() {
        let mut b = Buffer::new();

        let funcs: Vec<fn(&char, &mut Buffer) -> Result<(), RenderError>> =
            vec![Render::render, Render::render_escaped];

        for func in funcs {
            func(&'a', &mut b).unwrap();
            func(&'b', &mut b).unwrap();
            func(&'c', &mut b).unwrap();
            func(&'d', &mut b).unwrap();

            assert_eq!(b.as_str(), "abcd");
            b.clear();

            func(&'あ', &mut b).unwrap();
            func(&'い', &mut b).unwrap();
            func(&'う', &mut b).unwrap();
            func(&'え', &mut b).unwrap();

            assert_eq!(b.as_str(), "あいうえ");
            b.clear();
        }
    }

    #[test]
    fn test_nonzero() {
        let mut b = Buffer::with_capacity(2);
        Render::render(&NonZeroU8::new(10).unwrap(), &mut b).unwrap();
        Render::render_escaped(&NonZeroI16::new(-20).unwrap(), &mut b).unwrap();
        assert_eq!(b.as_str(), "10-20");
    }

    #[test]
    fn render_error() {
        let err = RenderError::new("custom error");
        assert!(err.source().is_none());
        assert_eq!(format!("{}", err), "custom error");

        let err = RenderError::from(std::fmt::Error::default());
        assert!(err.source().is_some());
        assert_eq!(
            format!("{}", err),
            format!("{}", std::fmt::Error::default())
        );

        let err = RenderError::BufSize;
        assert!(err.source().is_none());
        format!("{}", err);
    }
}
