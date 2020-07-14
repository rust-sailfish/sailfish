use quote::quote;
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::visit_mut::VisitMut;
use syn::{Block, Expr, ExprMacro, Ident, LitStr, Stmt, Token};

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

fn get_rendertext_value(i: &ExprMacro) -> Option<String> {
    let mut it = i.mac.path.segments.iter();

    if it.next().map_or(false, |s| s.ident == "__sf_rt")
        && it.next().map_or(false, |s| s.ident == "render_text")
        && it.next().is_none()
    {
        let tokens = i.mac.tokens.clone();
        if let Ok(macro_arg) = syn::parse2::<RenderTextMacroArgument>(tokens) {
            return Some(macro_arg.arg.value());
        }
    }

    None
}

struct OptmizerImpl {
    rm_whitespace: bool,
}

impl VisitMut for OptmizerImpl {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        let fl = if let Expr::ForLoop(ref mut fl) = *i {
            fl
        } else {
            syn::visit_mut::visit_expr_mut(self, i);
            return;
        };

        syn::visit_mut::visit_block_mut(self, &mut fl.body);

        let (mf, ml) = match (fl.body.stmts.first(), fl.body.stmts.last()) {
            (
                Some(Stmt::Semi(Expr::Macro(ref mf), ..)),
                Some(Stmt::Semi(Expr::Macro(ref ml), ..)),
            ) => (mf, ml),
            _ => {
                syn::visit_mut::visit_expr_mut(self, i);
                return;
            }
        };

        let (sf, sl) = match (get_rendertext_value(mf), get_rendertext_value(ml)) {
            (Some(sf), Some(sl)) => (sf, sl),
            _ => {
                syn::visit_mut::visit_expr_mut(self, i);
                return;
            }
        };

        let sf_len = sf.len();
        let concat = sl + &*sf;

        fl.body.stmts.remove(0);
        *fl.body.stmts.last_mut().unwrap() = syn::parse2(quote! {
            __sf_rt::render_text!(_ctx, #concat);
        })
        .unwrap();

        let new_expr = syn::parse2(quote! {{
            __sf_rt::render_text!(_ctx, #sf);
            #fl;
            unsafe { _ctx.buf.set_len(_ctx.buf.len() - #sf_len); }
        }})
        .unwrap();

        *i = new_expr;
    }

    fn visit_expr_macro_mut(&mut self, i: &mut ExprMacro) {
        if self.rm_whitespace {
            if let Some(v) = get_rendertext_value(i) {
                let mut buffer = String::new();
                let mut it = v.lines().peekable();

                if let Some(line) = it.next() {
                    if it.peek().is_some() {
                        buffer.push_str(line.trim_end());
                        buffer.push('\n');
                    } else {
                        return;
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
                i.mac.tokens = quote! { _ctx, #buffer };
                return;
            }
        }

        syn::visit_mut::visit_expr_macro_mut(self, i);
    }
}

pub struct Optimizer {
    rm_whitespace: bool,
}

impl Optimizer {
    #[inline]
    pub fn new() -> Self {
        Self {
            rm_whitespace: false,
        }
    }

    #[inline]
    pub fn rm_whitespace(mut self, new: bool) -> Self {
        self.rm_whitespace = new;
        self
    }

    #[inline]
    pub fn optimize(&self, i: &mut Block) {
        OptmizerImpl {
            rm_whitespace: self.rm_whitespace,
        }
        .visit_block_mut(i);
    }
}
