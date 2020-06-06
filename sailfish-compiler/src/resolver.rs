use std::sync::Arc;
use syn::visit_mut::VisitMut;
use syn::{Block, Expr, ExprBlock, LitStr};

use crate::error::*;

macro_rules! matches_or_else {
    ($val:expr, $p:pat, $ok:expr, $else:expr) => {
        match $val {
            $p => $ok,
            _ => $else,
        }
    };
}

macro_rules! return_if_some {
    ($val:expr) => {
        if $val.is_some() {
            return;
        }
    };
}

fn empty_block() -> Block {
    Block {
        brace_token: Default::default(),
        stmts: Vec::new(),
    }
}

struct ResolverImpl<'h> {
    deps: Vec<String>,
    error: Option<Error>,
    include_handler: Arc<dyn 'h + Fn(&str) -> Result<Block, Error>>,
}

impl<'h> VisitMut for ResolverImpl<'h> {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        return_if_some!(self.error);
        let em = matches_or_else!(*i, Expr::Macro(ref em), em, {
            syn::visit_mut::visit_expr_mut(self, i);
            return;
        });

        // check if path is `include`
        if !em.mac.path.is_ident("include") {
            syn::visit_mut::visit_expr_mut(self, i);
            return;
        }

        let arg = match syn::parse2::<LitStr>(em.mac.tokens.clone()) {
            Ok(l) => l.value(),
            Err(e) => {
                let mut e = Error::from(e);
                e.chains.push(ErrorKind::AnalyzeError(
                    "invalid arguments for `include` macro".to_owned(),
                ));
                self.error = Some(e);
                return;
            }
        };

        // parse and translate the child template
        let mut blk = match (*self.include_handler)(&arg) {
            Ok(blk) => blk,
            Err(mut e) => {
                e.chains
                    .push(ErrorKind::Other(format!("Failed to include {}", arg)));
                self.error = Some(e);
                return;
            }
        };

        self.deps.push(arg);
        syn::visit_mut::visit_block_mut(self, &mut blk);

        *i = Expr::Block(ExprBlock {
            attrs: Vec::new(),
            label: None,
            block: blk,
        });
    }
}

#[derive(Clone)]
pub struct Resolver<'h> {
    include_handler: Arc<dyn 'h + Fn(&str) -> Result<Block, Error>>,
}

impl<'h> Resolver<'h> {
    pub fn new() -> Self {
        Self {
            include_handler: Arc::new(|_| {
                Err(make_error!(ErrorKind::AnalyzeError(
                    "You cannot use `include` macro inside templates".to_owned()
                )))
            }),
        }
    }

    #[inline]
    pub fn include_handler(
        self,
        new: Arc<dyn 'h + Fn(&str) -> Result<Block, Error>>,
    ) -> Resolver<'h> {
        Self {
            include_handler: new,
        }
    }

    #[inline]
    pub fn resolve(&self, ast: &mut Block) -> Result<(), Error> {
        ResolverImpl {
            deps: Vec::new(),
            error: None,
            include_handler: Arc::clone(&self.include_handler)
        }.visit_block_mut(ast);
        Ok(())
    }
}
