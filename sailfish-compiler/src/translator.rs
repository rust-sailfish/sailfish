use proc_macro2::{Span, TokenStream};
use quote::ToTokens;
use syn::parse::{Parse, ParseStream as SynParseStream, Result as ParseResult};
use syn::{BinOp, Block, Expr};

use crate::error::*;
use crate::parser::{ParseStream, Token, TokenKind};

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

        let mut ps = SourceBuilder::new(self.escape);
        ps.reserve(original_source.len());
        ps.feed_tokens(token_iter)?;

        Ok(ps.finalize()?)
    }
}

pub struct TranslatedSource {
    pub ast: Block,
    pub source_map: SourceMap,
}

#[derive(Default)]
pub struct SourceMap {
    entries: Vec<SourceMapEntry>,
}

#[derive(Clone)]
pub struct SourceMapEntry {
    pub original: usize,
    pub new: usize,
    pub length: usize,
}

impl SourceMap {
    // #[inline]
    // pub fn entries(&self) -> &[SourceMapEntry] {
    //     &*self.entries
    // }

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

struct SourceBuilder {
    escape: bool,
    source: String,
    source_map: SourceMap,
}

impl SourceBuilder {
    fn new(escape: bool) -> SourceBuilder {
        SourceBuilder {
            escape,
            source: String::from("{\n"),
            source_map: SourceMap::default(),
        }
    }

    fn reserve(&mut self, additional: usize) {
        self.source.reserve(additional);
    }

    fn write_token<'a>(&mut self, token: &Token<'a>) {
        let entry = SourceMapEntry {
            original: token.offset(),
            new: self.source.len(),
            length: token.as_str().len(),
        };
        self.source_map.entries.push(entry);
        self.source.push_str(token.as_str());
    }

    fn write_code<'a>(&mut self, token: &Token<'a>) -> Result<(), Error> {
        // TODO: automatically add missing tokens (e.g. ';', '{')
        self.write_token(token);
        self.source.push('\n');
        Ok(())
    }

    fn write_text<'a>(&mut self, token: &Token<'a>) -> Result<(), Error> {
        use std::fmt::Write;

        // if error has occured at the first byte of `render_text!` macro, it
        // will be mapped to the first byte of text
        self.source_map.entries.push(SourceMapEntry {
            original: token.offset(),
            new: self.source.len(),
            length: 1,
        });

        self.source.push_str("__sf_rt::render_text!(__sf_buf, ");
        // write text token with Debug::fmt
        write!(self.source, "{:?}", token.as_str()).unwrap();
        self.source.push_str(");\n");
        Ok(())
    }

    fn write_buffered_code<'a>(
        &mut self,
        token: &Token<'a>,
        escape: bool,
    ) -> Result<(), Error> {
        self.write_buffered_code_with_suffix(token, escape, "")
    }

    fn write_buffered_code_with_suffix<'a>(
        &mut self,
        token: &Token<'a>,
        escape: bool,
        suffix: &str,
    ) -> Result<(), Error> {
        // parse and split off filter
        let code_block = syn::parse_str::<CodeBlock>(token.as_str()).map_err(|e| {
            let span = e.span();
            let mut err = make_error!(ErrorKind::RustSyntaxError(e));
            err.offset = into_offset(token.as_str(), span).map(|p| token.offset() + p);
            err
        })?;
        let method = if self.escape && escape {
            "render_escaped"
        } else {
            "render"
        };

        self.source.push_str("__sf_rt::");
        self.source.push_str(method);
        self.source.push_str("!(__sf_buf, ");

        if let Some(filter) = code_block.filter {
            let expr_str = format!("{}{}", code_block.expr.into_token_stream(), suffix);
            let (name, extra_args) = match filter {
                Filter::Ident(i) => (i.to_string(), None),
                Filter::Call(c) => (
                    c.func.into_token_stream().to_string(),
                    Some(c.args.into_token_stream().to_string()),
                ),
            };

            self.source.push_str("sailfish::runtime::filter::");
            self.source.push_str(&*name);
            self.source.push('(');

            // arguments to filter function
            {
                self.source.push_str("&(");
                let entry = SourceMapEntry {
                    original: token.offset(),
                    new: self.source.len(),
                    length: expr_str.len(),
                };
                self.source_map.entries.push(entry);
                self.source.push_str(&expr_str);
                self.source.push(')');

                if let Some(extra_args) = extra_args {
                    self.source.push_str(", ");
                    self.source.push_str(&*extra_args);
                }
            }

            self.source.push(')');
        } else {
            self.write_token(token);
            self.source.push_str(suffix);
        }

        self.source.push_str(");\n");

        Ok(())
    }

    pub fn feed_tokens<'a>(&mut self, token_iter: ParseStream<'a>) -> Result<(), Error> {
        let mut it = token_iter.peekable();
        while let Some(token) = it.next() {
            let token = token?;
            match token.kind() {
                TokenKind::Code => self.write_code(&token)?,
                TokenKind::Comment => {}
                TokenKind::BufferedCode { escape } => {
                    self.write_buffered_code(&token, escape)?
                }
                TokenKind::NestedTemplateOnce => self.write_buffered_code_with_suffix(
                    &token,
                    false,
                    ".render_once()?",
                )?,
                TokenKind::Text => {
                    // concatenate repeated text token
                    let offset = token.offset();
                    let mut concatenated = String::new();
                    concatenated.push_str(token.as_str());

                    while let Some(&Ok(ref next_token)) = it.peek() {
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
                    self.write_text(&new_token)?;
                }
            }
        }

        Ok(())
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

enum Filter {
    Ident(syn::Ident),
    Call(syn::ExprCall),
}

impl ToTokens for Filter {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Filter::Ident(ident) => ident.to_tokens(tokens),
            Filter::Call(call) => call.to_tokens(tokens),
        }
    }
}

struct CodeBlock {
    #[allow(dead_code)]
    expr: Box<Expr>,
    filter: Option<Filter>,
}

impl Parse for CodeBlock {
    fn parse(s: SynParseStream) -> ParseResult<Self> {
        let main = s.parse::<Expr>()?;

        let code_block = match main {
            Expr::Binary(b) if matches!(b.op, BinOp::BitOr(_)) => {
                match *b.right {
                    Expr::Call(c) => {
                        if let Expr::Path(ref p) = *c.func {
                            if p.path.get_ident().is_some() {
                                CodeBlock {
                                    expr: b.left,
                                    filter: Some(Filter::Call(c)),
                                }
                            } else {
                                return Err(syn::Error::new_spanned(
                                    p,
                                    "Invalid filter name",
                                ));
                            }
                        } else {
                            // if function in right side is not a path, fallback to
                            // normal evaluation block
                            CodeBlock {
                                expr: b.left,
                                filter: None,
                            }
                        }
                    }
                    Expr::Path(p) => {
                        if let Some(i) = p.path.get_ident() {
                            CodeBlock {
                                expr: b.left,
                                filter: Some(Filter::Ident(i.clone())),
                            }
                        } else {
                            return Err(syn::Error::new_spanned(
                                p,
                                "Invalid filter name",
                            ));
                        }
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(b, "Expected filter"));
                    }
                }
            }
            _ => CodeBlock {
                expr: Box::new(main),
                filter: None,
            },
        };

        Ok(code_block)
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
    #[ignore]
    fn translate() {
        let src = "<% pub fn sample() { %> <%% <%=//%>\n1%><% } %>";
        let lexer = Parser::new();
        let token_iter = lexer.parse(src);
        let mut ps = SourceBuilder {
            escape: true,
            source: String::with_capacity(token_iter.original_source.len()),
            source_map: SourceMap::default(),
        };
        ps.feed_tokens(token_iter.clone()).unwrap();
        Translator::new().translate(token_iter).unwrap();
    }

    #[test]
    fn translate_nested_render_once() {
        let src = r#"outer <%+ inner %> outer"#;
        let lexer = Parser::new();
        let token_iter = lexer.parse(src);
        let mut ps = SourceBuilder {
            escape: true,
            source: String::with_capacity(token_iter.original_source.len()),
            source_map: SourceMap::default(),
        };
        ps.feed_tokens(token_iter.clone()).unwrap();
        assert_eq!(
            &Translator::new()
                .translate(token_iter)
                .unwrap()
                .ast
                .into_token_stream()
                .to_string(),
            r#"{ __sf_rt :: render_text ! (__sf_buf , "outer ") ; __sf_rt :: render ! (__sf_buf , inner . render_once () ?) ; __sf_rt :: render_text ! (__sf_buf , " outer") ; }"#
        );
    }

    #[test]
    fn translate_nested_render_once_with_filter() {
        let src = r#"outer <%+ inner|upper %> outer"#;
        let lexer = Parser::new();
        let token_iter = lexer.parse(src);
        let mut ps = SourceBuilder {
            escape: true,
            source: String::with_capacity(token_iter.original_source.len()),
            source_map: SourceMap::default(),
        };
        ps.feed_tokens(token_iter.clone()).unwrap();
        assert_eq!(
            &Translator::new()
                .translate(token_iter)
                .unwrap()
                .ast
                .into_token_stream()
                .to_string(),
            r#"{ __sf_rt :: render_text ! (__sf_buf , "outer ") ; __sf_rt :: render ! (__sf_buf , sailfish :: runtime :: filter :: upper (& (inner . render_once () ?))) ; __sf_rt :: render_text ! (__sf_buf , " outer") ; }"#
        );
    }
}
