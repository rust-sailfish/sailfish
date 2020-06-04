use syn::Block;

use crate::error::*;

#[derive(Clone, Debug, Default)]
pub struct Resolver {}

impl Resolver {
    #[inline]
    pub fn new() -> Self {
        Self {}
    }

    #[inline]
    pub fn resolve(&self, _ast: &mut Block) -> Result<(), Error> {
        // not implemented yet
        Ok(())
    }
}
