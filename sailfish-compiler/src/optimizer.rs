use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::visit_mut::VisitMut;
use syn::{
    Block, Expr, ExprBreak, ExprContinue, ExprMacro, Ident, LitStr, Macro, Stmt,
    StmtMacro, Token,
};

pub struct Optimizer {
    rm_whitespace: bool,
    rm_newline: bool,
}

impl Optimizer {
    #[inline]
    pub fn new() -> Self {
        Self {
            rm_whitespace: false,
            rm_newline: false,
        }
    }

    #[inline]
    pub fn rm_whitespace(mut self, new: bool) -> Self {
        self.rm_whitespace = new;
        self
    }

    #[inline]
    pub fn rm_newline(mut self, new: bool) -> Self {
        self.rm_newline = new;
        self
    }

    #[inline]
    pub fn optimize(&self, i: &mut Block) {
        OptmizerImpl {
            rm_whitespace: self.rm_whitespace,
            rm_newline: self.rm_newline,
        }
        .visit_block_mut(i);
    }
}

struct OptmizerImpl {
    rm_whitespace: bool,
    rm_newline: bool,
}

impl OptmizerImpl {
    fn apply_optimizations(&self, v: String) -> Option<TokenStream> {
        let mut optimized = v.to_string();

        if self.rm_whitespace {
            optimized = remove_whitespace(&optimized);
        }
        if self.rm_newline {
            optimized = remove_newlines(&optimized);
        }

        // Only return a token stream if the string was actually modified
        if optimized != v {
            Some(quote! { __sf_buf, #optimized })
        } else {
            None
        }
    }
}

impl VisitMut for OptmizerImpl {
    fn visit_block_mut(&mut self, i: &mut Block) {
        let mut results = Vec::with_capacity(i.stmts.len());

        for mut stmt in i.stmts.drain(..) {
            // process whole statement in advance
            syn::visit_mut::visit_stmt_mut(self, &mut stmt);

            // check if statement is for loop
            let fl = match stmt {
                Stmt::Expr(Expr::ForLoop(ref mut fl), _) => fl,
                _ => {
                    results.push(stmt);
                    continue;
                }
            };

            // check if for loop contains 2 or more statements
            if fl.body.stmts.len() <= 1 {
                results.push(stmt);
                continue;
            }

            // check if for loop contains continue or break statement
            if block_has_continue_or_break(&mut fl.body) {
                results.push(stmt);
                continue;
            }

            // check if first and last statement inside for loop is render_text! macro
            let (sf, sl) = match (
                fl.body
                    .stmts
                    .first()
                    .and_then(get_rendertext_value_from_stmt),
                fl.body
                    .stmts
                    .last()
                    .and_then(get_rendertext_value_from_stmt),
            ) {
                (Some(sf), Some(sl)) => (sf, sl),
                _ => {
                    results.push(stmt);
                    continue;
                }
            };

            let sf_len = sf.len();

            // optimize for loop contents
            let mut concat = sl;
            concat += sf.as_str();

            let mut previous;
            if let Some(prev) = results.last().and_then(get_rendertext_value_from_stmt) {
                results.pop();
                previous = prev;
                previous += sf.as_str();
            } else {
                previous = sf;
            }

            fl.body.stmts.remove(0);
            *fl.body.stmts.last_mut().unwrap() = syn::parse2(quote! {
                __sf_rt::render_text!(__sf_buf, #concat);
            })
            .unwrap();

            let mut new_stmts = syn::parse2::<Block>(quote! {{
                __sf_rt::render_text!(__sf_buf, #previous);
                #stmt
                unsafe { __sf_buf._set_len(__sf_buf.len() - #sf_len); }
            }})
            .unwrap();

            results.append(&mut new_stmts.stmts)
        }

        i.stmts = results;
    }

    fn visit_stmt_macro_mut(&mut self, i: &mut StmtMacro) {
        if let Some(v) = get_rendertext_value(&i.mac) {
            if let Some(ts) = self.apply_optimizations(v) {
                i.mac.tokens = ts;
                return;
            }
        }

        syn::visit_mut::visit_stmt_macro_mut(self, i);
    }

    fn visit_expr_macro_mut(&mut self, i: &mut ExprMacro) {
        if let Some(v) = get_rendertext_value(&i.mac) {
            if let Some(ts) = self.apply_optimizations(v) {
                i.mac.tokens = ts;
                return;
            }
        }

        syn::visit_mut::visit_expr_macro_mut(self, i);
    }
}

fn remove_whitespace(v: &str) -> String {
    let mut buffer = String::new();
    let mut it = v.lines().peekable();
    if let Some(line) = it.next() {
        if it.peek().is_some() {
            buffer.push_str(line.trim_end());
            buffer.push('\n');
        } else {
            return v.to_string();
        }
    }
    while let Some(line) = it.next() {
        if it.peek().is_some() {
            if !line.is_empty() {
                buffer.push_str(line.trim());
                buffer.push('\n');
            } else {
                // ignore empty line
            }
        } else {
            // last line
            buffer.push_str(line.trim_start());
        }
    }
    buffer
}

fn remove_newlines(v: &str) -> String {
    v.replace(['\n', '\r'], "")
}

fn get_rendertext_value(mac: &Macro) -> Option<String> {
    struct RenderTextMacroArgument {
        #[allow(dead_code)]
        context: Ident,
        arg: LitStr,
    }

    impl Parse for RenderTextMacroArgument {
        fn parse(s: ParseStream) -> ParseResult<Self> {
            let context = s.parse()?;
            s.parse::<Token![,]>()?;
            let arg = s.parse()?;

            Ok(Self { context, arg })
        }
    }

    let mut it = mac.path.segments.iter();

    if it.next().map_or(false, |s| s.ident == "__sf_rt")
        && it.next().map_or(false, |s| s.ident == "render_text")
        && it.next().is_none()
    {
        let tokens = mac.tokens.clone();
        if let Ok(macro_arg) = syn::parse2::<RenderTextMacroArgument>(tokens) {
            return Some(macro_arg.arg.value());
        }
    }

    None
}

fn get_rendertext_value_from_stmt(stmt: &Stmt) -> Option<String> {
    let em = match stmt {
        Stmt::Expr(Expr::Macro(ref mac), Some(_)) => mac,
        _ => return None,
    };

    get_rendertext_value(&em.mac)
}

fn block_has_continue_or_break(i: &mut Block) -> bool {
    #[derive(Default)]
    struct ContinueBreakFinder {
        found: bool,
    }

    impl VisitMut for ContinueBreakFinder {
        fn visit_expr_continue_mut(&mut self, _: &mut ExprContinue) {
            self.found = true;
        }

        fn visit_expr_break_mut(&mut self, _: &mut ExprBreak) {
            self.found = true;
        }
    }

    let mut finder = ContinueBreakFinder::default();
    finder.visit_block_mut(i);
    finder.found
}
