use proc_macro2::Span;

use crate::error::*;
use crate::parser::{ParseStream, Token, TokenKind};

use syn::Block;

#[derive(Clone)]
pub struct SourceMapEntry {
    pub original: usize,
    pub new: usize,
    pub length: usize,
}

#[derive(Default)]
pub struct SourceMap {
    entries: Vec<SourceMapEntry>,
}

impl SourceMap {
    #[inline]
    pub fn entries(&self) -> &[SourceMapEntry] {
        &*self.entries
    }

    pub fn reverse_mapping(&self, offset: usize) -> Option<usize> {
        // find entry which satisfies entry.new <= offset < entry.new + entry.length
        let idx = self
            .entries
            .iter()
            .position(|entry| offset < entry.new + entry.length && entry.new <= offset)?;

        let entry = &self.entries[idx];
        debug_assert!(entry.new <= offset);
        debug_assert!(offset < entry.new + entry.length);

        Some(entry.original + offset - entry.new)
    }
}

pub struct TranslatedSource {
    pub ast: Block,
    pub source_map: SourceMap,
}

// translate tokens into Rust code
#[derive(Clone, Debug, Default)]
pub struct Translator {
    escape: bool,
}

impl Translator {
    #[inline]
    pub fn new() -> Self {
        Self { escape: true }
    }

    #[inline]
    pub fn escape(mut self, new: bool) -> Self {
        self.escape = new;
        self
    }

    pub fn translate<'a>(
        &self,
        token_iter: ParseStream<'a>,
    ) -> Result<TranslatedSource, Error> {
        let original_source = token_iter.original_source;

        let mut source = String::with_capacity(original_source.len());
        source.push_str("{\n");
        let mut ps = SourceBuilder {
            escape: self.escape,
            source,
            source_map: SourceMap::default(),
        };
        ps.feed_tokens(&*token_iter.into_vec()?);

        Ok(ps.finalize()?)
    }
}

struct SourceBuilder {
    escape: bool,
    source: String,
    source_map: SourceMap,
}

impl SourceBuilder {
    fn write_token<'a>(&mut self, token: &Token<'a>) {
        let entry = SourceMapEntry {
            original: token.offset(),
            new: self.source.len(),
            length: token.as_str().len(),
        };
        self.source_map.entries.push(entry);
        self.source.push_str(token.as_str());
    }

    fn write_code<'a>(&mut self, token: &Token<'a>) {
        // TODO: automatically add missing tokens (e.g. ';', '{')
        self.write_token(token);
        self.source.push_str("\n");
    }

    fn write_text<'a>(&mut self, token: &Token<'a>) {
        use std::fmt::Write;

        self.source.push_str("sfrt::render_text!(_ctx, ");

        // write text token with Debug::fmt
        write!(self.source, "{:?}", token.as_str()).unwrap();

        self.source.push_str(");\n");
    }

    fn write_buffered_code<'a>(&mut self, token: &Token<'a>, escape: bool) {
        let method = if self.escape && escape {
            "render_escaped"
        } else {
            "render"
        };

        self.source.push_str("sfrt::");
        self.source.push_str(method);
        self.source.push_str("!(_ctx, ");
        self.write_token(token);
        self.source.push_str(");\n");
    }

    pub fn feed_tokens(&mut self, token_iter: &[Token]) {
        let mut it = token_iter.iter().peekable();
        while let Some(token) = it.next() {
            match token.kind() {
                TokenKind::Code => self.write_code(&token),
                TokenKind::Comment => {}
                TokenKind::BufferedCode { escape } => {
                    self.write_buffered_code(&token, escape)
                }
                TokenKind::Text => {
                    // concatenate repeated text token
                    let offset = token.offset();
                    let mut concatenated = String::new();
                    concatenated.push_str(token.as_str());

                    while let Some(next_token) = it.peek() {
                        match next_token.kind() {
                            TokenKind::Text => {
                                concatenated.push_str(next_token.as_str());
                                it.next();
                            }
                            TokenKind::Comment => {
                                it.next();
                            }
                            _ => break,
                        }
                    }

                    let new_token = Token::new(&*concatenated, offset, TokenKind::Text);
                    self.write_text(&new_token);
                }
            }
        }
    }

    pub fn finalize(mut self) -> Result<TranslatedSource, Error> {
        self.source.push_str("\n}");
        match syn::parse_str::<Block>(&*self.source) {
            Ok(ast) => Ok(TranslatedSource {
                ast,
                source_map: self.source_map,
            }),
            Err(synerr) => {
                let span = synerr.span();
                let original_offset = into_offset(&*self.source, span)
                    .and_then(|o| self.source_map.reverse_mapping(o));

                let mut err =
                    make_error!(ErrorKind::RustSyntaxError(synerr), source = self.source);

                err.offset = original_offset;

                Err(err)
            }
        }
    }
}

fn into_offset(source: &str, span: Span) -> Option<usize> {
    let lc = span.start();
    if lc.line > 0 {
        Some(
            source
                .lines()
                .take(lc.line - 1)
                .fold(0, |s, e| s + e.len() + 1)
                + lc.column,
        )
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    #[test]
    fn translate() {
        let src = "<% pub fn sample() { %> <%% <%=//%>\n%><% } %>";
        let lexer = Parser::new();
        let token_iter = lexer.parse(src);
        let mut ps = SourceBuilder {
            escape: true,
            source: String::with_capacity(token_iter.original_source.len()),
            source_map: SourceMap::default(),
        };
        ps.feed_tokens(&token_iter.clone().to_vec().unwrap());
        eprintln!("{}", ps.source);
        Translator::new().translate(token_iter).unwrap();
    }
}
