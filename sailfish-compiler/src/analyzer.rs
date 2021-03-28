use crate::error::*;
use syn::visit_mut::VisitMut;
use syn::Block;

#[derive(Clone)]
pub struct Analyzer {}

impl Analyzer {
    pub fn new() -> Self {
        Self {}
    }

    #[inline]
    pub fn analyze(&self, ast: &mut Block) -> Result<(), Error> {
        let mut child = AnalyzerImpl { error: None };
        child.visit_block_mut(ast);

        if let Some(e) = child.error {
            Err(e)
        } else {
            Ok(())
        }
    }
}

struct AnalyzerImpl {
    error: Option<Error>,
}

impl VisitMut for AnalyzerImpl {
    // write code here
}
