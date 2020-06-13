use quote::ToTokens;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::Config;
use crate::error::*;
use crate::optimizer::Optimizer;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::translator::{TranslatedSource, Translator};
use crate::util::{read_to_string, rustfmt_block};

pub struct CompilationReport {
    pub deps: Vec<PathBuf>,
}

#[derive(Default)]
pub struct Compiler {
    config: Config,
}

impl Compiler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: Config) -> Self {
        Self { config }
    }

    fn translate_file_contents(&self, input: &Path) -> Result<TranslatedSource, Error> {
        let parser = Parser::new().delimiter(self.config.delimiter);
        let translator = Translator::new().escape(self.config.escape);
        let content = read_to_string(input)
            .chain_err(|| format!("Failed to open template file: {:?}", input))?;

        let stream = parser.parse(&*content);
        translator.translate(stream)
    }

    pub fn compile_file(
        &self,
        template_dir: &Path,
        input: &Path,
        output: &Path,
    ) -> Result<CompilationReport, Error> {
        // TODO: introduce cache system

        let input = input.canonicalize()?;

        let include_handler = Arc::new(|child_file: &Path| -> Result<_, Error> {
            Ok(self.translate_file_contents(&*child_file)?.ast)
        });

        let resolver = Resolver::new().include_handler(include_handler);
        let optimizer = Optimizer::new().rm_whitespace(self.config.rm_whitespace);

        let compile_file =
            |input: &Path, output: &Path| -> Result<CompilationReport, Error> {
                let mut tsource = self.translate_file_contents(input)?;
                let mut report = CompilationReport { deps: Vec::new() };

                let r = resolver.resolve(template_dir, &*input, &mut tsource.ast)?;
                report.deps = r.deps;

                optimizer.optimize(&mut tsource.ast);

                if let Some(parent) = output.parent() {
                    fs::create_dir_all(parent)?;
                }
                if output.exists() {
                    fs::remove_file(output)?;
                }

                let string = tsource.ast.into_token_stream().to_string();
                fs::write(output, rustfmt_block(&*string).unwrap_or(string))?;
                Ok(report)
            };

        compile_file(&*input, &*output)
            .chain_err(|| "Failed to compile template.")
            .map_err(|mut e| {
                e.source = fs::read_to_string(&*input).ok();
                e.source_file = Some(input.to_owned());
                e
            })
    }
}
