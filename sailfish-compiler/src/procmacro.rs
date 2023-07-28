use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, thread};
use syn::parse::{ParseStream, Parser, Result as ParseResult};
use syn::punctuated::Punctuated;
use syn::{Fields, Ident, ItemStruct, LitBool, LitChar, LitStr, Token};

use crate::compiler::{CompilationReport, Compiler};
use crate::config::Config;
use crate::error::*;

// options for `template` attributes
#[derive(Default)]
struct DeriveTemplateOptions {
    found_keys: Vec<Ident>,
    path: Option<LitStr>,
    delimiter: Option<LitChar>,
    escape: Option<LitBool>,
    rm_whitespace: Option<LitBool>,
}

impl DeriveTemplateOptions {
    fn parser<'s>(&'s mut self) -> impl Parser + 's {
        move |s: ParseStream| -> ParseResult<()> {
            while !s.is_empty() {
                let key = s.parse::<Ident>()?;
                s.parse::<Token![=]>()?;

                // check if argument is repeated
                if self.found_keys.iter().any(|e| *e == key) {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Argument `{}` was repeated.", key),
                    ));
                }

                if key == "path" {
                    self.path = Some(s.parse::<LitStr>()?);
                } else if key == "delimiter" {
                    self.delimiter = Some(s.parse::<LitChar>()?);
                } else if key == "escape" {
                    self.escape = Some(s.parse::<LitBool>()?);
                } else if key == "rm_whitespace" {
                    self.rm_whitespace = Some(s.parse::<LitBool>()?);
                } else {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown option: `{}`", key),
                    ));
                }

                self.found_keys.push(key);

                // consume comma token
                if s.is_empty() {
                    break;
                } else {
                    s.parse::<Token![,]>()?;
                }
            }

            Ok(())
        }
    }
}

fn merge_config_options(config: &mut Config, options: &DeriveTemplateOptions) {
    if let Some(ref delimiter) = options.delimiter {
        config.delimiter = delimiter.value();
    }
    if let Some(ref escape) = options.escape {
        config.escape = escape.value;
    }
    if let Some(ref rm_whitespace) = options.rm_whitespace {
        config.rm_whitespace = rm_whitespace.value;
    }
}

fn resolve_template_file(path: &str, template_dirs: &[PathBuf]) -> Option<PathBuf> {
    for template_dir in template_dirs.iter().rev() {
        let p = template_dir.join(path);
        if p.is_file() {
            return Some(p);
        }
    }

    let mut fallback = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect(
        "Internal error: environmental variable `CARGO_MANIFEST_DIR` is not set.",
    ));
    fallback.push("templates");
    fallback.push(path);

    if fallback.is_file() {
        return Some(fallback);
    }

    None
}

fn filename_hash(path: &Path, config: &Config) -> String {
    use std::fmt::Write;

    let mut path_with_hash = String::with_capacity(16);

    if let Some(n) = path.file_name() {
        let mut filename = &*n.to_string_lossy();
        if let Some(p) = filename.find('.') {
            filename = &filename[..p];
        }
        path_with_hash.push_str(filename);
        path_with_hash.push('-');
    }

    let input_bytes = std::fs::read(path).unwrap();

    let mut hasher = DefaultHasher::new();
    input_bytes.hash(&mut hasher);
    config.hash(&mut hasher);
    let hash = hasher.finish();
    let _ = write!(path_with_hash, "{:016x}", hash);

    path_with_hash
}

fn compile(
    input_file: &Path,
    output_file: &Path,
    config: Config,
) -> Result<CompilationReport, Error> {
    struct FallbackScope {}

    impl FallbackScope {
        fn new() -> Self {
            // SAFETY:
            // Any token or span constructed after `proc_macro2::fallback::force()` must
            // not outlive after `unforce()` because it causes span mismatch error. In
            // this case, we must ensure that `compile_file` does not return any token or
            // span.
            proc_macro2::fallback::force();
            FallbackScope {}
        }
    }

    impl Drop for FallbackScope {
        fn drop(&mut self) {
            proc_macro2::fallback::unforce();
        }
    }

    let compiler = Compiler::with_config(config);

    let _scope = FallbackScope::new();
    compiler.compile_file(input_file, &*output_file)
}

fn derive_template_impl(tokens: TokenStream) -> Result<TokenStream, syn::Error> {
    let strct = syn::parse2::<ItemStruct>(tokens)?;

    let mut all_options = DeriveTemplateOptions::default();
    for attr in strct.attrs {
        if attr.path().is_ident("template") {
            attr.parse_args_with(all_options.parser())?;
        }
    }

    #[cfg(feature = "config")]
    let mut config = {
        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect(
            "Internal error: environmental variable `CARGO_MANIFEST_DIR` is not set.",
        ));

        Config::search_file_and_read(&*manifest_dir)
            .map_err(|e| syn::Error::new(Span::call_site(), e))?
    };

    #[cfg(not(feature = "config"))]
    let mut config = Config::default();

    if env::var("SAILFISH_INTEGRATION_TESTS").map_or(false, |s| s == "1") {
        let template_dir = env::current_dir()
            .unwrap()
            .ancestors()
            .find(|p| p.join("LICENSE").exists())
            .unwrap()
            .join("sailfish-tests")
            .join("integration-tests")
            .join("tests")
            .join("fails")
            .join("templates");
        config.template_dirs.push(template_dir);
    }

    let input_file = {
        let path = all_options.path.as_ref().ok_or_else(|| {
            syn::Error::new(Span::call_site(), "`path` option must be specified.")
        })?;
        resolve_template_file(&*path.value(), &*config.template_dirs).ok_or_else(
            || {
                syn::Error::new(
                    path.span(),
                    format!("Template file {:?} not found", path.value()),
                )
            },
        )?
    };

    merge_config_options(&mut config, &all_options);

    // Template compilation through this proc-macro uses a caching mechanism. Output file
    // names include a hash calculated from input file contents and compiler
    // configuration. This way, existing files never need updating and can simply be
    // re-used if they exist.
    let mut output_file = PathBuf::from(env!("OUT_DIR"));
    output_file.push("templates");
    output_file.push(filename_hash(&*input_file, &config));

    std::fs::create_dir_all(&output_file.parent().unwrap()).unwrap();

    const DEPS_END_MARKER: &str = "=--end-of-deps--=";
    let dep_file = output_file.with_extension("deps");

    // This makes sure max 1 process creates a new file, "create_new" check+create is an
    // atomic operation. Cargo sometimes runs multiple macro invocations for the same
    // file in parallel, so that's important to prevent a race condition.
    let dep_file_status = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&dep_file);

    let deps = match dep_file_status {
        Ok(mut file) => {
            // Successfully created new .deps file. Now template needs to be compiled.
            let report = compile(&*input_file, &*output_file, config)
                .map_err(|e| syn::Error::new(Span::call_site(), e))?;

            for dep in &report.deps {
                writeln!(file, "{}", dep.to_str().unwrap()).unwrap();
            }
            writeln!(file, "{}", DEPS_END_MARKER).unwrap();

            report.deps
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            // .deps file exists, template is already (currently being?) compiled.
            let mut load_attempts = 0;
            loop {
                let dep_file_content = std::fs::read_to_string(&dep_file).unwrap();
                let mut lines_reversed = dep_file_content.rsplit_terminator('\n');
                if lines_reversed.next() == Some(DEPS_END_MARKER) {
                    // .deps file is complete, so we can continue.
                    break lines_reversed.map(PathBuf::from).collect();
                }

                // .deps file exists, but appears incomplete. Wait a bit and try again.
                load_attempts += 1;
                if load_attempts > 100 {
                    panic!("file {:?} is incomplete. Try deleting it.", dep_file);
                }

                thread::sleep(Duration::from_millis(10));
            }
        }
        Err(e) => panic!("{:?}: {}. Maybe try `cargo clean`?", dep_file, e),
    };

    let input_file_string = input_file
        .to_str()
        .unwrap_or_else(|| panic!("Non UTF-8 file name: {:?}", input_file));
    let output_file_string = output_file
        .to_str()
        .unwrap_or_else(|| panic!("Non UTF-8 file name: {:?}", output_file));

    let mut include_bytes_seq = quote! { include_bytes!(#input_file_string); };
    for dep in deps {
        if let Some(dep_string) = dep.to_str() {
            include_bytes_seq.extend(quote! { include_bytes!(#dep_string); });
        }
    }

    // Generate tokens

    let name = strct.ident;

    let field_names: Punctuated<Ident, Token![,]> = match strct.fields {
        Fields::Named(fields) => fields
            .named
            .into_iter()
            .map(|f| {
                f.ident.expect(
                    "Internal error: Failed to get field name (error code: 73621)",
                )
            })
            .collect(),
        Fields::Unit => Punctuated::new(),
        _ => {
            return Err(syn::Error::new(
                Span::call_site(),
                "You cannot derive `Template` or `TemplateOnce` for tuple struct",
            ));
        }
    };

    let (impl_generics, ty_generics, where_clause) = strct.generics.split_for_impl();

    // render_once method always results in the same code.
    // This method can be implemented in `sailfish` crate, but I found that performance
    // drops when the implementation is written in `sailfish` crate.
    let tokens = quote! {
        impl #impl_generics sailfish::TemplateOnce for #name #ty_generics #where_clause {
            fn render_once(self) -> sailfish::RenderResult {
                use sailfish::runtime::{Buffer, SizeHint};
                static SIZE_HINT: SizeHint = SizeHint::new();

                let mut buf = Buffer::with_capacity(SIZE_HINT.get());
                self.render_once_to(&mut buf)?;
                SIZE_HINT.update(buf.len());

                Ok(buf.into_string())
            }

            fn render_once_to(self, __sf_buf: &mut sailfish::runtime::Buffer) -> std::result::Result<(), sailfish::runtime::RenderError> {
                // This line is required for cargo to track child templates
                #include_bytes_seq;

                use sailfish::runtime as __sf_rt;
                let #name { #field_names } = self;
                include!(#output_file_string);

                Ok(())
            }
        }

        impl #impl_generics sailfish::private::Sealed for #name #ty_generics #where_clause {}
    };

    Ok(tokens)
}

pub fn derive_template(tokens: TokenStream) -> TokenStream {
    derive_template_impl(tokens).unwrap_or_else(|e| e.to_compile_error())
}
