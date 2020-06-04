use quote::ToTokens;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::*;
use crate::optimizer::Optimizer;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::translator::Translator;
use crate::util::rustfmt_block;

pub struct Compiler {
    delimiter: char,
    escape: bool,
    cache_dir: PathBuf,
}

impl Default for Compiler {
    fn default() -> Self {
        Self {
            delimiter: '%',
            escape: true,
            cache_dir: Path::new(env!("OUT_DIR")).join("cache"),
        }
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn delimiter(mut self, new: char) -> Self {
        self.delimiter = new;
        self
    }

    pub fn escape(mut self, new: bool) -> Self {
        self.escape = new;
        self
    }

    pub fn compile_file(&self, input: &Path, output: &Path) -> Result<(), Error> {
        // TODO: introduce cache system

        let parser = Parser::new().delimiter(self.delimiter);
        let translator = Translator::new().escape(self.escape);
        let resolver = Resolver::new();
        let optimizer = Optimizer::new();

        let compile_file = |input: &Path, output: &Path| -> Result<(), Error> {
            let content = fs::read_to_string(&*input)
                .chain_err(|| format!("Failed to open template file: {:?}", input))?;

            let stream = parser.parse(&*content);
            let mut tsource = translator.translate(stream)?;
            drop(content);

            resolver.resolve(&mut tsource.ast)?;
            optimizer.optimize(&mut tsource.ast);

            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)?;
            }
            if output.exists() {
                fs::remove_file(output)?;
            }

            let string = tsource.ast.into_token_stream().to_string();
            fs::write(output, rustfmt_block(&*string).unwrap_or(string))?;
            Ok(())
        };

        compile_file(&*input, &*output)
            .chain_err(|| "Failed to compile template.")
            .map_err(|mut e| {
                e.source = fs::read_to_string(&*input).ok();
                e.source_file = Some(input.to_owned());
                e
            })?;

        Ok(())
    }
}
