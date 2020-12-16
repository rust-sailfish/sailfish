use std::env;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[non_exhaustive]
#[derive(Debug)]
pub enum ErrorKind {
    FmtError(fmt::Error),
    IoError(io::Error),
    RustSyntaxError(syn::Error),
    ConfigError(String),
    ParseError(String),
    AnalyzeError(String),
    Unimplemented(String),
    Other(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ErrorKind::FmtError(ref e) => e.fmt(f),
            ErrorKind::IoError(ref e) => e.fmt(f),
            ErrorKind::RustSyntaxError(ref e) => write!(f, "Rust Syntax Error ({})", e),
            ErrorKind::ConfigError(ref e) => write!(f, "Invalid configuration ({})", e),
            ErrorKind::ParseError(ref msg) => write!(f, "Parse error ({})", msg),
            ErrorKind::AnalyzeError(ref msg) => write!(f, "Analyzation error ({})", msg),
            ErrorKind::Unimplemented(ref msg) => f.write_str(&**msg),
            ErrorKind::Other(ref msg) => f.write_str(&**msg),
        }
    }
}

macro_rules! impl_errorkind_conversion {
    ($source:ty, $kind:ident, $conv:expr, [ $($lifetimes:tt),* ]) => {
        impl<$($lifetimes),*> From<$source> for ErrorKind {
            #[inline]
            fn from(other: $source) -> Self {
                ErrorKind::$kind($conv(other))
            }
        }
    };
    ($source:ty, $kind:ident) => {
        impl_errorkind_conversion!($source, $kind, std::convert::identity, []);
    }
}

impl_errorkind_conversion!(fmt::Error, FmtError);
impl_errorkind_conversion!(io::Error, IoError);
impl_errorkind_conversion!(syn::Error, RustSyntaxError);
impl_errorkind_conversion!(String, Other);
impl_errorkind_conversion!(&'a str, Other, |s: &str| s.to_owned(), ['a]);

#[derive(Debug, Default)]
pub struct Error {
    pub(crate) source_file: Option<PathBuf>,
    pub(crate) source: Option<String>,
    pub(crate) offset: Option<usize>,
    pub(crate) chains: Vec<ErrorKind>,
}

impl Error {
    pub fn from_kind(kind: ErrorKind) -> Self {
        Self {
            chains: vec![kind],
            ..Self::default()
        }
    }

    pub fn kind(&self) -> &ErrorKind {
        self.chains.last().unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ErrorKind> {
        self.chains.iter().rev()
    }
}

impl<T> From<T> for Error
where
    ErrorKind: From<T>,
{
    fn from(other: T) -> Self {
        Self::from_kind(ErrorKind::from(other))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let source = match (self.source.as_ref(), self.source_file.as_deref()) {
            (Some(s), _) => Some(s.to_owned()),
            (None, Some(f)) => fs::read_to_string(f).ok(),
            (None, None) => None,
        };

        writeln!(f, "{}", self.chains.last().unwrap())?;

        for e in self.chains.iter().rev().skip(1) {
            writeln!(f, "caused by: {}", e)?;
        }

        f.write_str("\n")?;

        if let Some(ref source_file) = self.source_file {
            let source_file =
                if env::var("SAILFISH_INTEGRATION_TESTS").map_or(false, |s| s == "1") {
                    match source_file.file_name() {
                        Some(f) => Path::new(f),
                        None => Path::new(""),
                    }
                } else {
                    source_file
                };
            writeln!(f, "file: {}", source_file.display())?;
        }

        if let (Some(ref source), Some(offset)) = (source, self.offset) {
            let (lineno, colno) = into_line_column(source, offset);
            writeln!(f, "position: line {}, column {}\n", lineno, colno)?;

            // TODO: display adjacent lines
            let line = source.lines().nth(lineno - 1).unwrap();
            let lpad = count_digits(lineno);

            writeln!(f, "{:<lpad$} |", "", lpad = lpad)?;
            writeln!(f, "{} | {}", lineno, line)?;
            writeln!(
                f,
                "{:<lpad$} | {:<rpad$}^",
                "",
                "",
                lpad = lpad,
                rpad = colno - 1
            )?;
        }

        Ok(())
    }
}

impl std::error::Error for Error {}

pub trait ResultExt<T> {
    fn chain_err<F, EK>(self, kind: F) -> Result<T, Error>
    where
        F: FnOnce() -> EK,
        EK: Into<ErrorKind>;
}

impl<T> ResultExt<T> for Result<T, Error> {
    fn chain_err<F, EK>(self, kind: F) -> Result<T, Error>
    where
        F: FnOnce() -> EK,
        EK: Into<ErrorKind>,
    {
        self.map_err(|mut e| {
            e.chains.push(kind().into());
            e
        })
    }
}

impl<T, E: Into<ErrorKind>> ResultExt<T> for Result<T, E> {
    fn chain_err<F, EK>(self, kind: F) -> Result<T, Error>
    where
        F: FnOnce() -> EK,
        EK: Into<ErrorKind>,
    {
        self.map_err(|e| {
            let mut e = Error::from(e.into());
            e.chains.push(kind().into());
            e
        })
    }
}

fn into_line_column(source: &str, offset: usize) -> (usize, usize) {
    assert!(
        offset <= source.len(),
        "Internal error: error position offset overflow (error code: 56066)"
    );
    let mut lineno = 1;
    let mut colno = 1;
    let mut current = 0;

    for line in source.lines() {
        let end = current + line.len() + 1;
        if offset < end {
            colno = offset - current + 1;
            break;
        }

        lineno += 1;
        current = end;
    }

    (lineno, colno)
}

fn count_digits(n: usize) -> usize {
    let mut current = 10;
    let mut digits = 1;

    while current <= n {
        current *= 10;
        digits += 1;
    }

    digits
}

macro_rules! make_error {
    ($kind:expr) => {
        $crate::Error::from_kind($kind)
    };
    ($kind:expr, $($remain:tt)*) => {{
        #[allow(unused_mut)]
        let mut err = $crate::Error::from_kind($kind);
        make_error!(@opt err $($remain)*);
        err
    }};
    (@opt $var:ident $key:ident = $value:expr, $($remain:tt)*) => {
        $var.$key = Some($value.into());
        make_error!(@opt $var $($remain)*);
    };
    (@opt $var:ident $key:ident = $value:expr) => {
        $var.$key = Some($value.into());
    };
    (@opt $var:ident $key:ident, $($remain:tt)*) => {
        $var.$key = Some($key);
        make_error!(@opt $var $($remain)*);
    };
    (@opt $var:ident $key:ident) => {
        $var.$key = Some($key);
    };
    (@opt $var:ident) => {};
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn display_error() {
        let mut err = make_error!(
            ErrorKind::AnalyzeError("mismatched types".to_owned()),
            source_file = PathBuf::from("apple.rs"),
            source = "fn func() {\n    1\n}".to_owned(),
            offset = 16usize
        );
        err.chains.push(ErrorKind::Other("some error".to_owned()));
        assert!(matches!(err.kind(), &ErrorKind::Other(_)));
        assert_eq!(
            err.to_string(),
            r#"some error
caused by: Analyzation error (mismatched types)

file: apple.rs
position: line 2, column 5

  |
2 |     1
  |     ^
"#
        );
    }
}
