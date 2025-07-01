use quote::ToTokens;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use syn::Block;

use crate::analyzer::Analyzer;
use crate::config::Config;
use crate::error::*;
use crate::optimizer::Optimizer;
use crate::parser::Parser;
use crate::resolver::Resolver;
use crate::translator::{TranslatedSource, Translator};
use crate::util::{read_to_string, rustfmt_block};

#[derive(Default)]
pub struct Compiler {
    config: Config,
}

pub struct CompilationReport {
    pub deps: Vec<PathBuf>,
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

        let stream = parser.parse(&content);
        translator.translate(stream)
    }

    pub fn resolve_file(
        &self,
        input: &Path,
    ) -> Result<(TranslatedSource, CompilationReport), Error> {
        let include_handler = Arc::new(|child_file: &Path| -> Result<_, Error> {
            Ok(self.translate_file_contents(child_file)?.ast)
        });

        let resolver = Resolver::new().include_handler(include_handler);
        let mut tsource = self.translate_file_contents(input)?;
        let mut report = CompilationReport { deps: Vec::new() };

        let r = resolver.resolve(input, &mut tsource.ast)?;
        report.deps = r.deps;
        Ok((tsource, report))
    }

    pub fn compile_file(
        &self,
        input: &Path,
        tsource: TranslatedSource,
        output: &Path,
    ) -> Result<(), Error> {
        let analyzer = Analyzer::new();
        let optimizer = Optimizer::new()
            .rm_whitespace(self.config.rm_whitespace)
            .rm_newline(self.config.rm_newline);

        let compile_file = |mut tsource: TranslatedSource,
                            output: &Path|
         -> Result<(), Error> {
            analyzer.analyze(&mut tsource.ast)?;
            optimizer.optimize(&mut tsource.ast);

            if let Some(parent) = output.parent() {
                fs::create_dir_all(parent)
                    .chain_err(|| format!("Failed to save artifacts in {:?}", parent))?;
            }

            let string = tsource.ast.into_token_stream().to_string();

            let mut f = fs::File::create(output)
                .chain_err(|| format!("Failed to create artifact: {:?}", output))?;
            writeln!(f, "// Template compiled from: {}", input.display())
                .chain_err(|| format!("Failed to write artifact into {:?}", output))?;
            writeln!(f, "{}", rustfmt_block(&string).unwrap_or(string))
                .chain_err(|| format!("Failed to write artifact into {:?}", output))?;
            drop(f);

            Ok(())
        };

        compile_file(tsource, output)
            .chain_err(|| "Failed to compile template.")
            .map_err(|mut e| {
                e.source = fs::read_to_string(input).ok();
                e.source_file = Some(input.to_owned());
                e
            })
    }

    pub fn compile_str(&self, input: &str) -> Result<String, Error> {
        let dummy_path = Path::new(env!("CARGO_MANIFEST_DIR"));

        let include_handler = Arc::new(|_: &Path| -> Result<Block, Error> {
            Err(make_error!(
                ErrorKind::AnalyzeError(
                    "include! macro is not allowed in inline template".to_owned()
                ),
                source = input.to_owned()
            ))
        });

        let parser = Parser::new().delimiter(self.config.delimiter);
        let translator = Translator::new().escape(self.config.escape);
        let resolver = Resolver::new().include_handler(include_handler);
        let optimizer = Optimizer::new()
            .rm_whitespace(self.config.rm_whitespace)
            .rm_newline(self.config.rm_newline);

        let compile = || -> Result<String, Error> {
            let stream = parser.parse(input);
            let mut tsource = translator.translate(stream)?;
            resolver.resolve(dummy_path, &mut tsource.ast)?;

            optimizer.optimize(&mut tsource.ast);
            Ok(tsource.ast.into_token_stream().to_string())
        };

        compile()
            .chain_err(|| "Failed to compile template.")
            .map_err(|mut e| {
                e.source = Some(input.to_owned());
                e
            })
    }
}
