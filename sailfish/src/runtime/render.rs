use std::path::Path;

use super::buffer::Buffer;
use super::{escape, RenderError};

/// types which can be rendered inside buffer block (`<%= %>`)
///
/// If you want to render the custom data, you must implement this trait and specify
/// the behaviour.
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
    fn render(&self, b: &mut Buffer) -> Result<(), RenderError>;

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> Result<(), RenderError> {
        let mut tmp = Buffer::new();
        self.render(&mut tmp)?;
        b.push_str(tmp.as_str());
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
            _ => b.push(*self),
        }
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
                    use super::integer::Integer;

                    b.reserve(Self::MAX_LEN);

                    unsafe {
                        let ptr = b.as_mut_ptr().add(b.len());
                        let l = self.write_to(ptr);
                        b.set_len(b.len() + l);
                    }
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
        if self.is_finite() {
            unsafe {
                b.reserve(16);
                let ptr = b.as_mut_ptr().add(b.len());
                let l = ryu::raw::format32(*self, ptr);
                b.set_len(b.len() + l);
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
        if self.is_finite() {
            unsafe {
                b.reserve(24);
                let ptr = b.as_mut_ptr().add(b.len());
                let l = ryu::raw::format64(*self, ptr);
                b.set_len(b.len() + l);
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

// private trait for avoiding method name collision in render* macros
#[doc(hidden)]
pub trait RenderInternal {
    fn _sf_r_internal(&self, b: &mut Buffer) -> Result<(), RenderError>;
    fn _sf_re_internal(&self, b: &mut Buffer) -> Result<(), RenderError>;
}

impl<T: Render + ?Sized> RenderInternal for T {
    #[inline]
    fn _sf_r_internal(&self, b: &mut Buffer) -> Result<(), RenderError> {
        self.render(b)
    }

    #[inline]
    fn _sf_re_internal(&self, b: &mut Buffer) -> Result<(), RenderError> {
        self.render_escaped(b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receiver_coercion() {
        let mut b = Buffer::new();
        (&1)._sf_r_internal(&mut b).unwrap();
        (&&1)._sf_r_internal(&mut b).unwrap();
        (&&&1)._sf_r_internal(&mut b).unwrap();
        (&&&&1)._sf_r_internal(&mut b).unwrap();
        assert_eq!(b.as_str(), "1111");
        b.clear();

        let v = 2.0;
        (&v)._sf_r_internal(&mut b).unwrap();
        (&&v)._sf_r_internal(&mut b).unwrap();
        (&&&v)._sf_r_internal(&mut b).unwrap();
        (&&&&v)._sf_r_internal(&mut b).unwrap();
        assert_eq!(b.as_str(), "2.02.02.02.0");
        b.clear();

        let s = "apple";
        (&*s)._sf_re_internal(&mut b).unwrap();
        (&s)._sf_re_internal(&mut b).unwrap();
        (&&s)._sf_re_internal(&mut b).unwrap();
        (&&&s)._sf_re_internal(&mut b).unwrap();
        (&&&&s)._sf_re_internal(&mut b).unwrap();
        assert_eq!(b.as_str(), "appleappleappleappleapple");
        b.clear();

        (&'c')._sf_re_internal(&mut b).unwrap();
        (&&'<')._sf_re_internal(&mut b).unwrap();
        (&&&'&')._sf_re_internal(&mut b).unwrap();
        (&&&&' ')._sf_re_internal(&mut b).unwrap();
        assert_eq!(b.as_str(), "c&lt;&amp; ");
        b.clear();
    }

    #[test]
    fn deref_coercion() {
        use std::path::PathBuf;
        use std::rc::Rc;

        let mut b = Buffer::new();
        (&String::from("a"))._sf_r_internal(&mut b).unwrap();
        (&&PathBuf::from("b"))._sf_r_internal(&mut b).unwrap();
        (&Rc::new(4u32))._sf_re_internal(&mut b).unwrap();
        (&Rc::new(2.3f32))._sf_re_internal(&mut b).unwrap();

        assert_eq!(b.as_str(), "ab42.3");
    }
}
