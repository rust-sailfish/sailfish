// TODO: Better error message (unbalanced rust delimiter, etc.)
// TODO: disallow '<%' token inside code block

use memchr::{memchr, memchr2, memchr3};
use std::convert::TryInto;
use std::rc::Rc;

use crate::{Error, ErrorKind};

macro_rules! unwrap_or_break {
    ($val:expr) => {
        match $val {
            Some(t) => t,
            None => break,
        }
    };
}

#[derive(Clone, Debug)]
pub struct Parser {
    delimiter: char,
}

impl Parser {
    pub fn new() -> Self {
        Self::default()
    }

    /// change delimiter
    pub fn delimiter(mut self, new: char) -> Self {
        self.delimiter = new;
        self
    }

    /// parse source string
    pub fn parse<'a>(&self, source: &'a str) -> ParseStream<'a> {
        let block_delimiter = Rc::new((
            format!("<{}", self.delimiter),
            format!("{}>", self.delimiter),
        ));

        ParseStream {
            block_delimiter,
            original_source: source,
            source,
            delimiter: self.delimiter,
        }
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self { delimiter: '%' }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    BufferedCode { escape: bool },
    Code,
    Comment,
    Text,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token<'a> {
    content: &'a str,
    offset: usize,
    kind: TokenKind,
}

impl<'a> Token<'a> {
    #[inline]
    pub fn new(content: &'a str, offset: usize, kind: TokenKind) -> Token<'a> {
        Token {
            content,
            offset,
            kind,
        }
    }

    #[inline]
    pub fn as_str(&self) -> &'a str {
        self.content
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    pub fn kind(&self) -> TokenKind {
        self.kind
    }
}

#[derive(Clone, Debug)]
pub struct ParseStream<'a> {
    block_delimiter: Rc<(String, String)>,
    pub(crate) original_source: &'a str,
    source: &'a str,
    delimiter: char,
}

impl<'a> ParseStream<'a> {
    // /// Returns an empty `ParseStream` containing no tokens
    // pub fn new() -> Self {
    //     Self::default()
    // }
    //
    // pub fn is_empty(&self) -> bool {
    //     self.source.is_empty()
    // }

    pub fn into_vec(self) -> Result<Vec<Token<'a>>, Error> {
        let mut vec = Vec::new();
        for token in self {
            vec.push(token?);
        }

        Ok(vec)
    }

    fn error(&self, msg: &str) -> Error {
        let offset = self.original_source.len() - self.source.len();
        make_error!(
            ErrorKind::ParseError(msg.to_owned()),
            source = self.original_source.to_owned(),
            offset
        )
    }

    fn offset(&self) -> usize {
        self.original_source.len() - self.source.len()
    }

    fn take_n(&mut self, n: usize) -> &'a str {
        let (l, r) = self.source.split_at(n);
        self.source = r;
        l
    }

    fn tokenize_code(&mut self) -> Result<Token<'a>, Error> {
        debug_assert!(self.source.starts_with(&*self.block_delimiter.0));

        let mut start = self.block_delimiter.0.len();
        let mut token_kind = TokenKind::Code;

        // read flags
        match self.source.as_bytes().get(start).copied() {
            Some(b'#') => {
                token_kind = TokenKind::Comment;
                start += 1;
            }
            Some(b'=') => {
                token_kind = TokenKind::BufferedCode { escape: true };
                start += 1;
            }
            Some(b'-') => {
                token_kind = TokenKind::BufferedCode { escape: false };
                start += 1;
            }
            _ => {}
        }

        // skip whitespaces
        for ch in self.source.bytes().skip(start) {
            match ch {
                b' ' | b'\t' | b'\n'..=b'\r' => {
                    start += 1;
                }
                _ => break,
            }
        }

        if token_kind == TokenKind::Comment {
            let pos = self.source[start..]
                .find(&*self.block_delimiter.1)
                .ok_or_else(|| self.error("Unterminated comment block"))?;

            self.take_n(start);
            let token = Token {
                content: self.source[..pos].trim_end(),
                offset: self.offset(),
                kind: token_kind,
            };

            self.take_n(pos + self.block_delimiter.1.len());
            return Ok(token);
        }

        // find closing bracket
        if let Some(pos) = find_block_end(&self.source[start..], &*self.block_delimiter.1)
        {
            // closing bracket was found
            self.take_n(start);
            let s = &self.source[..pos - self.block_delimiter.1.len()].trim_end_matches(
                |c| matches!(c, ' ' | '\t' | '\r' | '\u{000B}' | '\u{000C}'),
            );
            let token = Token {
                content: s,
                offset: self.offset(),
                kind: token_kind,
            };
            self.take_n(pos);
            Ok(token)
        } else {
            Err(self.error("Unterminated code block"))
        }
    }

    fn tokenize_text(&mut self) -> Result<Token<'a>, Error> {
        // TODO: allow buffer block inside code block
        let offset = self.offset();
        let end = self
            .source
            .find(&*self.block_delimiter.0)
            .unwrap_or_else(|| self.source.len());
        let token = Token {
            content: self.take_n(end),
            offset,
            kind: TokenKind::Text,
        };
        Ok(token)
    }
}

impl<'a> Default for ParseStream<'a> {
    fn default() -> Self {
        Self {
            block_delimiter: Rc::new(("<%".to_owned(), "%>".to_owned())),
            original_source: "",
            source: "",
            delimiter: '%',
        }
    }
}

impl<'a> Iterator for ParseStream<'a> {
    type Item = Result<Token<'a>, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.source.is_empty() {
            return None;
        }

        let token = if self.source.starts_with(&*self.block_delimiter.0) {
            if !self.source[self.block_delimiter.0.len()..].starts_with(self.delimiter) {
                self.tokenize_code()
            } else {
                debug_assert_eq!(
                    &self.source[..self.delimiter.len_utf8() * 2 + 1],
                    format!("<{0}{0}", self.delimiter)
                );

                // Escape '<%%' token
                let token = Token {
                    content: &self.source[..self.block_delimiter.0.len()],
                    offset: self.offset(),
                    kind: TokenKind::Text,
                };
                self.take_n(self.block_delimiter.0.len() * 2 - 1);
                Ok(token)
            }
        } else {
            self.tokenize_text()
        };

        Some(token)
    }
}

impl<'a> TryInto<Vec<Token<'a>>> for ParseStream<'a> {
    type Error = crate::Error;

    fn try_into(self) -> Result<Vec<Token<'a>>, Error> {
        self.into_vec()
    }
}

fn find_block_end(haystack: &str, delimiter: &str) -> Option<usize> {
    let mut remain = haystack;

    'outer: while let Some(pos) =
        memchr3(b'/', b'\"', delimiter.as_bytes()[0], remain.as_bytes())
    {
        let skip_num = match remain.as_bytes()[pos] {
            b'/' => match remain.as_bytes().get(pos + 1).copied() {
                Some(b'/') => unwrap_or_break!(find_comment_end(&remain[pos..])),
                Some(b'*') => unwrap_or_break!(find_block_comment_end(&remain[pos..])),
                _ => 1,
            },
            b'\"' => {
                // check if the literal is a raw string
                for (i, byte) in remain[..pos].as_bytes().iter().enumerate().rev() {
                    match byte {
                        b'#' => {}
                        b'r' => {
                            let skip_num =
                                unwrap_or_break!(find_raw_string_end(&remain[i..]));
                            remain = &remain[i + skip_num..];
                            continue 'outer;
                        }
                        _ => break,
                    }
                }
                unwrap_or_break!(find_string_end(&remain[pos..]))
            }
            _ => {
                if remain[pos..].starts_with(delimiter) {
                    return Some(haystack.len() - remain.len() + pos + delimiter.len());
                } else {
                    1
                }
            }
        };

        remain = &remain[pos + skip_num..];
    }

    None
}

fn find_comment_end(haystack: &str) -> Option<usize> {
    debug_assert!(haystack.starts_with("//"));
    memchr(b'\n', haystack.as_bytes()).map(|p| p + 1)
}

fn find_block_comment_end(haystack: &str) -> Option<usize> {
    debug_assert!(haystack.starts_with("/*"));

    let mut remain = &haystack[2..];
    let mut depth = 1;

    while let Some(p) = memchr2(b'*', b'/', remain.as_bytes()) {
        let c = remain.as_bytes()[p];
        let next = remain.as_bytes().get(p + 1);

        match (c, next) {
            (b'*', Some(b'/')) => {
                if depth == 1 {
                    let offset = haystack.len() - (remain.len() - (p + 2));
                    return Some(offset);
                }
                depth -= 1;
                remain = &remain[p + 2..];
            }
            (b'/', Some(b'*')) => {
                depth += 1;
                remain = &remain[p + 2..];
            }
            _ => {
                remain = &remain[p + 1..];
            }
        }
    }

    None
}

fn find_string_end(haystack: &str) -> Option<usize> {
    debug_assert!(haystack.starts_with('\"'));
    let mut bytes = &haystack.as_bytes()[1..];

    while let Some(p) = memchr2(b'"', b'\\', bytes) {
        if bytes[p] == b'\"' {
            // string terminator found
            return Some(haystack.len() - (bytes.len() - p) + 1);
        } else if p + 2 < bytes.len() {
            // skip escape
            bytes = &bytes[p + 2..];
        } else {
            break;
        }
    }

    None
}

fn find_raw_string_end(haystack: &str) -> Option<usize> {
    debug_assert!(haystack.starts_with('r'));
    let mut terminator = String::from("\"");
    for ch in haystack[1..].bytes() {
        match ch {
            b'#' => terminator.push('#'),
            b'"' => break,
            _ => {
                // is not a raw string literal
                return Some(1);
            }
        }
    }

    haystack[terminator.len() + 1..]
        .find(&terminator)
        .map(|p| p + terminator.len() * 2 + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn non_ascii_delimiter() {
        let src = r##"foo <🍣# This is a comment 🍣> bar <🍣= r"🍣>" 🍣> baz <🍣🍣"##;
        let parser = Parser::new().delimiter('🍣');
        let tokens = parser.parse(src).into_vec().unwrap();
        assert_eq!(
            &tokens,
            &[
                Token {
                    content: "foo ",
                    offset: 0,
                    kind: TokenKind::Text
                },
                Token {
                    content: "This is a comment",
                    offset: 11,
                    kind: TokenKind::Comment
                },
                Token {
                    content: " bar ",
                    offset: 34,
                    kind: TokenKind::Text
                },
                Token {
                    content: "r\"🍣>\"",
                    offset: 46,
                    kind: TokenKind::BufferedCode { escape: true }
                },
                Token {
                    content: " baz ",
                    offset: 60,
                    kind: TokenKind::Text
                },
                Token {
                    content: "<🍣",
                    offset: 65,
                    kind: TokenKind::Text
                },
            ]
        );
    }

    #[test]
    fn comment_inside_block() {
        let src = "<% // %>\n %><%= /* %%>*/ 1 %>";
        let parser = Parser::new();
        let tokens = parser.parse(src).into_vec().unwrap();
        assert_eq!(
            &tokens,
            &[
                Token {
                    content: "// %>\n",
                    offset: 3,
                    kind: TokenKind::Code
                },
                Token {
                    content: "/* %%>*/ 1",
                    offset: 16,
                    kind: TokenKind::BufferedCode { escape: true }
                },
            ]
        );
    }
}
