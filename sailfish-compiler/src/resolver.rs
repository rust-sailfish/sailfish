use quote::quote;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use syn::visit_mut::VisitMut;
use syn::{Block, Expr, ExprBlock, ExprMacro, LitStr};

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
        mut self,
        new: Arc<dyn 'h + Fn(&Path) -> Result<Block, Error>>,
    ) -> Resolver<'h> {
        self.include_handler = new;
        self
    }

    #[inline]
    pub fn resolve(
        &self,
        input_file: &Path,
        ast: &mut Block,
    ) -> Result<ResolveReport, Error> {
        let mut child = ResolverImpl {
            path_stack: vec![input_file.to_owned()],
            deps: Vec::new(),
            error: None,
            include_handler: Arc::clone(&self.include_handler),
        };
        child.visit_block_mut(ast);

        if let Some(e) = child.error {
            Err(e)
        } else {
            Ok(ResolveReport { deps: child.deps })
        }
    }
}

pub struct ResolveReport {
    pub deps: Vec<PathBuf>,
}

struct ResolverImpl<'h> {
    path_stack: Vec<PathBuf>,
    deps: Vec<PathBuf>,
    error: Option<Error>,
    include_handler: Arc<dyn 'h + Fn(&Path) -> Result<Block, Error>>,
}

impl<'h> ResolverImpl<'h> {
    fn resolve_include(&mut self, i: &ExprMacro) -> Result<Expr, Error> {
        let arg = match syn::parse2::<LitStr>(i.mac.tokens.clone()) {
            Ok(l) => l.value(),
            Err(e) => {
                let mut e = Error::from(e);
                e.chains.push(ErrorKind::AnalyzeError(
                    "invalid arguments for `include` macro".to_owned(),
                ));
                return Err(e);
            }
        };

        // resolve include! for rust file
        if arg.ends_with(".rs") {
            let absolute_path = if Path::new(&*arg).is_absolute() {
                PathBuf::from(&arg[1..])
            } else {
                self.path_stack.last().unwrap().parent().unwrap().join(arg)
            };

            return if let Some(absolute_path_str) = absolute_path.to_str() {
                Ok(syn::parse2(quote! { include!(#absolute_path_str) }).unwrap())
            } else {
                let msg = format!(
                    "cannot include path with non UTF-8 character: {:?}",
                    absolute_path
                );
                Err(make_error!(ErrorKind::AnalyzeError(msg)))
            };
        }

        // resolve the template file path
        // TODO: How should arguments be interpreted on Windows?
        let child_template_file = if Path::new(&*arg).is_absolute() {
            // absolute imclude
            PathBuf::from(&arg[1..])
        } else {
            // relative include
            self.path_stack.last().unwrap().parent().unwrap().join(arg)
        };

        // parse and translate the child template
        let mut blk = (*self.include_handler)(&*child_template_file).chain_err(|| {
            format!("Failed to include {:?}", child_template_file.clone())
        })?;

        self.path_stack.push(child_template_file);
        syn::visit_mut::visit_block_mut(self, &mut blk);

        let child_template_file = self.path_stack.pop().unwrap();
        if self.deps.iter().all(|p| p != &child_template_file) {
            self.deps.push(child_template_file);
        }

        Ok(Expr::Block(ExprBlock {
            attrs: Vec::new(),
            label: None,
            block: blk,
        }))
    }
}

impl<'h> VisitMut for ResolverImpl<'h> {
    fn visit_expr_mut(&mut self, i: &mut Expr) {
        return_if_some!(self.error);
        let em = matches_or_else!(*i, Expr::Macro(ref mut em), em, {
            syn::visit_mut::visit_expr_mut(self, i);
            return;
        });

        // resolve `include`
        if em.mac.path.is_ident("include") {
            match self.resolve_include(em) {
                Ok(e) => *i = e,
                Err(e) => {
                    self.error = Some(e);
                    return;
                }
            }
        }
    }
}
