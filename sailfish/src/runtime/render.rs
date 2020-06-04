use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

use super::buffer::Buffer;
use super::escape;

pub trait Render {
    fn render(&self, b: &mut Buffer) -> fmt::Result;

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> fmt::Result {
        let mut tmp = Buffer::new();
        self.render(&mut tmp)?;
        b.write_str(tmp.as_str());
        Ok(())
    }
}

/// Autoref-based stable specialization
///
/// Explanation can be found [here](https://github.com/dtolnay/case-studies/blob/master/autoref-specialization/README.md)
impl<T: Display> Render for &T {
    fn render(&self, b: &mut Buffer) -> fmt::Result {
        fmt::write(b, format_args!("{}", self))
    }

    fn render_escaped(&self, b: &mut Buffer) -> fmt::Result {
        struct Wrapper<'a>(&'a mut Buffer);

        impl<'a> fmt::Write for Wrapper<'a> {
            #[inline]
            fn write_str(&mut self, s: &str) -> fmt::Result {
                escape::escape_to_buf(s, self.0);
                Ok(())
            }
        }

        fmt::write(&mut Wrapper(b), format_args!("{}", self))
    }
}

impl Render for str {
    #[inline]
    fn render(&self, b: &mut Buffer) -> fmt::Result {
        b.write_str(self);
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> fmt::Result {
        escape::escape_to_buf(self, b);
        Ok(())
    }
}

impl<'a> Render for &'a str {
    #[inline]
    fn render(&self, b: &mut Buffer) -> fmt::Result {
        b.write_str(self);
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> fmt::Result {
        // escape string
        escape::escape_to_buf(self, b);
        Ok(())
    }
}

impl Render for String {
    #[inline]
    fn render(&self, b: &mut Buffer) -> fmt::Result {
        b.write_str(self);
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> fmt::Result {
        // escape string
        escape::escape_to_buf(self, b);
        Ok(())
    }
}

impl Render for char {
    #[inline]
    fn render(&self, b: &mut Buffer) -> fmt::Result {
        b.write_char(*self);
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> fmt::Result {
        match *self {
            '\"' => b.write_str("&quot;"),
            '&' => b.write_str("&amp;"),
            '<' => b.write_str("&lt;"),
            '>' => b.write_str("&gt;"),
            _ => b.write_char(*self),
        }
        Ok(())
    }
}

impl<'a> Render for &'a Path {
    #[inline]
    fn render(&self, b: &mut Buffer) -> fmt::Result {
        // TODO: speed up on Windows using OsStrExt
        b.write_str(&*self.to_string_lossy());
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> fmt::Result {
        escape::escape_to_buf(&*self.to_string_lossy(), b);
        Ok(())
    }
}

impl Render for PathBuf {
    #[inline]
    fn render(&self, b: &mut Buffer) -> fmt::Result {
        b.write_str(&*self.to_string_lossy());
        Ok(())
    }

    #[inline]
    fn render_escaped(&self, b: &mut Buffer) -> fmt::Result {
        // escape string
        escape::escape_to_buf(&*self.to_string_lossy(), b);

        Ok(())
    }
}

// impl Render for [u8] {
//     #[inline]
//     fn render(&self, b: &mut Buffer) -> fmt::Result {
//         b.write_bytes(self);
//         Ok(())
//     }
// }
//
// impl<'a> Render for &'a [u8] {
//     #[inline]
//     fn render(&self, b: &mut Buffer) -> fmt::Result {
//         b.write_bytes(self);
//         Ok(())
//     }
// }
//
// impl Render for Vec<u8> {
//     #[inline]
//     fn render(&self, b: &mut Buffer) -> fmt::Result {
//         b.write_bytes(&**self);
//         Ok(())
//     }
// }

macro_rules! render_int {
    ($($int:ty),*) => {
        $(
            impl Render for $int {
                #[inline]
                fn render(&self, b: &mut Buffer) -> fmt::Result {
                    let mut buffer = itoa::Buffer::new();
                    let s = buffer.format(*self);
                    b.write_str(s);
                    Ok(())
                }

                #[inline]
                fn render_escaped(&self, b: &mut Buffer) -> fmt::Result {
                    // write_str without escape
                    self.render(b)
                }
            }
        )*
    }
}

render_int!(u8, u16, u32, u64, i8, i16, i32, i64, usize, isize);

macro_rules! render_float {
    ($($float:ty),*) => {
        $(
            impl Render for $float {
                #[inline]
                fn render(&self, b: &mut Buffer) -> fmt::Result {
                    let mut buffer = ryu::Buffer::new();
                    let s = buffer.format(*self);
                    b.write_str(s);
                    Ok(())
                }

                #[inline]
                fn render_escaped(&self, b: &mut Buffer) -> fmt::Result {
                    // escape string
                    self.render(b)
                }
            }
        )*
    }
}

render_float!(f32, f64);
