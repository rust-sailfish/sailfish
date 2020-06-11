use quote::quote;
use std::path::{Path, PathBuf};
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

struct ResolverImpl<'s, 'h> {
    template_dir: &'s Path,
    path_stack: Vec<PathBuf>,
    deps: Vec<String>,
    error: Option<Error>,
    include_handler: Arc<dyn 'h + Fn(&Path) -> Result<Block, Error>>,
}

impl<'s, 'h> VisitMut for ResolverImpl<'s, 'h> {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        return_if_some!(self.error);
        let em = matches_or_else!(*i, Expr::Macro(ref mut em), em, {
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

        // resolve include! for rust file
        if arg.ends_with(".rs") {
            if !arg.starts_with('/') {
                let absolute_path =
                    self.path_stack.last().unwrap().parent().unwrap().join(arg);
                let absolute_path_str = absolute_path.to_string_lossy();
                *i = syn::parse2(quote! { include!(#absolute_path_str) }).unwrap();
            }
            return;
        }

        // resolve the template file path
        // TODO: How should arguments be interpreted on Windows?
        let input_file = if arg.starts_with('/') {
            // absolute imclude
            self.template_dir.join(&arg[1..])
        } else {
            // relative include
            self.path_stack
                .last()
                .unwrap()
                .parent()
                .unwrap()
                .join(arg.clone())
        };

        // parse and translate the child template
        let mut blk = match (*self.include_handler)(&*input_file) {
            Ok(blk) => blk,
            Err(mut e) => {
                e.chains
                    .push(ErrorKind::Other(format!("Failed to include {}", arg)));
                self.error = Some(e);
                return;
            }
        };

        self.path_stack.push(input_file);
        self.deps.push(arg);
        syn::visit_mut::visit_block_mut(self, &mut blk);
        self.path_stack.pop();

        *i = Expr::Block(ExprBlock {
            attrs: Vec::new(),
            label: None,
            block: blk,
        });
    }
}

#[derive(Clone)]
pub struct Resolver<'h> {
    include_handler: Arc<dyn 'h + Fn(&Path) -> Result<Block, Error>>,
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
        new: Arc<dyn 'h + Fn(&Path) -> Result<Block, Error>>,
    ) -> Resolver<'h> {
        Self {
            include_handler: new,
        }
    }

    #[inline]
    pub fn resolve(
        &self,
        template_dir: &Path,
        input_file: &Path,
        ast: &mut Block,
    ) -> Result<(), Error> {
        let mut child = ResolverImpl {
            template_dir,
            path_stack: vec![input_file.to_owned()],
            deps: Vec::new(),
            error: None,
            include_handler: Arc::clone(&self.include_handler),
        };
        child.visit_block_mut(ast);

        if let Some(e) = child.error {
            Err(e)
        } else {
            Ok(())
        }
    }
}
