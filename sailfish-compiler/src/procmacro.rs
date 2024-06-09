use proc_macro2::{Span, TokenStream};
use quote::{quote, TokenStreamExt};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::io::Write;
use std::iter;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::{env, thread};
use syn::parse::{ParseStream, Parser, Result as ParseResult};
use syn::punctuated::Punctuated;
use syn::{Fields, Ident, ItemStruct, LitBool, LitChar, LitStr, Token};

use crate::compiler::Compiler;
use crate::config::Config;
use crate::error::*;
use crate::util::filetime;

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
    fn parser(&mut self) -> impl Parser + '_ {
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
    let mut hasher = DefaultHasher::new();
    config.hash(&mut hasher);
    let config_hash = hasher.finish();

    path.hash(&mut hasher);
    let path_hash = hasher.finish();

    format!("{:016x}-{:016x}", config_hash, path_hash)
}

fn with_compiler<T, F: FnOnce(Compiler) -> Result<T, Error>>(
    config: Config,
    apply: F,
) -> Result<T, Error> {
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
    apply(compiler)
}

fn derive_template_common_impl(
    tokens: TokenStream,
) -> Result<(ItemStruct, TokenStream, String), syn::Error> {
    let strct = syn::parse2::<ItemStruct>(tokens)?;

    let mut all_options = DeriveTemplateOptions::default();
    for attr in &strct.attrs {
        if attr.path().is_ident("template") {
            attr.parse_args_with(all_options.parser())?;
        }
    }

    #[cfg(feature = "config")]
    let mut config = {
        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect(
            "Internal error: environmental variable `CARGO_MANIFEST_DIR` is not set.",
        ));

        Config::search_file_and_read(&manifest_dir)
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
        resolve_template_file(&path.value(), &config.template_dirs)
            .and_then(|path| path.canonicalize().ok())
            .ok_or_else(|| {
                syn::Error::new(
                    path.span(),
                    format!("Template file {:?} not found", path.value()),
                )
            })?
    };

    merge_config_options(&mut config, &all_options);

    // Template compilation through this proc-macro uses a caching mechanism. Output file
    // names include a hash calculated from input file contents and compiler
    // configuration. This way, existing files never need updating and can simply be
    // re-used if they exist.
    let mut output_file = PathBuf::from(env!("OUT_DIR"));
    output_file.push("templates");
    output_file.push(filename_hash(&input_file, &config));

    std::fs::create_dir_all(output_file.parent().unwrap()).unwrap();

    // This makes sure max 1 process creates a new file, "create_new" check+create is an
    // atomic operation. Cargo sometimes runs multiple macro invocations for the same
    // file in parallel, so that's important to prevent a race condition.
    struct Lock<'path> {
        path: &'path Path,
    }

    impl<'path> Lock<'path> {
        fn new(path: &'path Path) -> std::io::Result<Self> {
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(path)
                .map(|_| Lock { path })
        }
    }

    impl<'path> Drop for Lock<'path> {
        fn drop(&mut self) {
            std::fs::remove_file(self.path)
                .expect("Failed to clean up lock file {}. Delete it manually, or run `cargo clean`.");
        }
    }

    let deps = with_compiler(config, |compiler| {
        let dep_path = output_file.with_extension("deps");
        let lock_path = output_file.with_extension("lock");
        let lock = Lock::new(&lock_path);
        match lock {
            Ok(lock) => {
                let (tsource, report) = compiler.resolve_file(&input_file)?;

                let output_filetime = filetime(&output_file);
                let input_filetime = iter::once(&input_file)
                    .chain(&report.deps)
                    .map(|path| filetime(path))
                    .max()
                    .expect("Iterator contains at least `input_file`");

                // Recompile template if any included templates were changed
                // since the last time we compiled.
                if input_filetime > output_filetime {
                    compiler.compile_file(&input_file, tsource, &output_file)?;

                    // Write access to `dep_path` is serialized by `lock`.
                    let mut dep_file = std::fs::OpenOptions::new()
                        .write(true)
                        .create(true)
                        .truncate(true)
                        .open(&dep_path)
                        .unwrap_or_else(|e| {
                            panic!("Failed to open {:?}: {}", dep_path, e)
                        });

                    // Write out dependencies for concurrent processes to reuse.
                    for dep in &report.deps {
                        writeln!(&mut dep_file, "{}", dep.to_str().unwrap()).unwrap();
                    }

                    // Prevent output file from being tracked by Cargo. Without this hack,
                    // every change to a template causes two recompilations:
                    //
                    // 1. Change a template at timestamp t.
                    // 2. Cargo detects template change due to `include_bytes!` macro below.
                    // 3. Sailfish compiler generates an output file with a later timestamp t'.
                    // 4. Build finishes with timestamp t.
                    // 5. Next cargo build detects output file with timestamp t' > t and rebuilds.
                    // 6. Sailfish compiler does not regenerate output due to timestamp logic above.
                    // 7. Build finishes with timestamp t'.
                    let _ = filetime::set_file_times(
                        &output_file,
                        input_filetime,
                        input_filetime,
                    );
                }

                drop(lock);
                Ok(report.deps)
            }
            // Lock file exists, template is already (currently being?) compiled.
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                let mut load_attempts = 0;
                while lock_path.exists() {
                    load_attempts += 1;
                    if load_attempts > 100 {
                        panic!("Lock file {:?} is stuck. Try deleting it.", lock_path);
                    }
                    thread::sleep(Duration::from_millis(10));
                }

                Ok(std::fs::read_to_string(&dep_path)
                    .unwrap()
                    .trim()
                    .lines()
                    .map(PathBuf::from)
                    .collect())
            }
            Err(e) => panic!("{:?}: {}. Maybe try `cargo clean`?", lock_path, e),
        }
    })
    .map_err(|e| syn::Error::new(Span::call_site(), e))?;

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

    Ok((strct, include_bytes_seq, output_file_string.to_string()))
}

fn derive_template_once_only_impl(
    strct: &ItemStruct,
    include_bytes_seq: &TokenStream,
    output_file_string: &String,
) -> TokenStream {
    let name = &strct.ident;
    let (impl_generics, ty_generics, where_clause) = strct.generics.split_for_impl();

    // render_once method always results in the same code.
    // This method can be implemented in `sailfish` crate, but I found that performance
    // drops when the implementation is written in `sailfish` crate.
    quote! {
        impl #impl_generics sailfish::TemplateOnce for #name #ty_generics #where_clause {
            fn render_once(mut self) -> sailfish::RenderResult {
                use sailfish::runtime::{Buffer, SizeHint};
                static SIZE_HINT: SizeHint = SizeHint::new();

                let mut buf = Buffer::with_capacity(SIZE_HINT.get());
                self.render_once_to(&mut buf)?;
                SIZE_HINT.update(buf.len());

                Ok(buf.into_string())
            }

            fn render_once_to(mut self, __sf_buf: &mut sailfish::runtime::Buffer) -> std::result::Result<(), sailfish::runtime::RenderError> {
                // This line is required for cargo to track child templates
                #include_bytes_seq;

                use sailfish::runtime as __sf_rt;
                include!(#output_file_string);

                Ok(())
            }
        }
    }
}

fn derive_template_mut_only_impl(
    strct: &ItemStruct,
    include_bytes_seq: &TokenStream,
    output_file_string: &String,
) -> TokenStream {
    let name = &strct.ident;
    let (impl_generics, ty_generics, where_clause) = strct.generics.split_for_impl();

    // This method can be implemented in `sailfish` crate, but I found that performance
    // drops when the implementation is written in `sailfish` crate.
    quote! {
        impl #impl_generics sailfish::TemplateMut for #name #ty_generics #where_clause {
            fn render_mut(&mut self) -> sailfish::RenderResult {
                use sailfish::runtime::{Buffer, SizeHint};
                static SIZE_HINT: SizeHint = SizeHint::new();

                let mut buf = Buffer::with_capacity(SIZE_HINT.get());
                self.render_mut_to(&mut buf)?;
                SIZE_HINT.update(buf.len());

                Ok(buf.into_string())
            }

            fn render_mut_to(&mut self, __sf_buf: &mut sailfish::runtime::Buffer) -> std::result::Result<(), sailfish::runtime::RenderError> {
                // This line is required for cargo to track child templates
                #include_bytes_seq;

                use sailfish::runtime as __sf_rt;
                include!(#output_file_string);

                Ok(())
            }
        }
    }
}

fn derive_template_only_impl(
    strct: &ItemStruct,
    include_bytes_seq: &TokenStream,
    output_file_string: &String,
) -> TokenStream {
    let name = &strct.ident;
    let (impl_generics, ty_generics, where_clause) = strct.generics.split_for_impl();

    // This method can be implemented in `sailfish` crate, but I found that performance
    // drops when the implementation is written in `sailfish` crate.
    quote! {
        impl #impl_generics sailfish::Template for #name #ty_generics #where_clause {
            fn render(&self) -> sailfish::RenderResult {
                use sailfish::runtime::{Buffer, SizeHint};
                static SIZE_HINT: SizeHint = SizeHint::new();

                let mut buf = Buffer::with_capacity(SIZE_HINT.get());
                self.render_to(&mut buf)?;
                SIZE_HINT.update(buf.len());

                Ok(buf.into_string())
            }

            fn render_to(&self, __sf_buf: &mut sailfish::runtime::Buffer) -> std::result::Result<(), sailfish::runtime::RenderError> {
                // This line is required for cargo to track child templates
                #include_bytes_seq;

                use sailfish::runtime as __sf_rt;
                include!(#output_file_string);

                Ok(())
            }
        }
    }
}

fn derive_template_once_impl(tokens: TokenStream) -> Result<TokenStream, syn::Error> {
    let (strct, include_bytes_seq, output_file_string) =
        derive_template_common_impl(tokens)?;

    let mut output = TokenStream::new();

    output.append_all(derive_template_once_only_impl(
        &strct,
        &include_bytes_seq,
        &output_file_string,
    ));

    Ok(output)
}

fn derive_template_mut_impl(tokens: TokenStream) -> Result<TokenStream, syn::Error> {
    let (strct, include_bytes_seq, output_file_string) =
        derive_template_common_impl(tokens)?;

    let mut output = TokenStream::new();

    output.append_all(derive_template_once_only_impl(
        &strct,
        &include_bytes_seq,
        &output_file_string,
    ));

    output.append_all(derive_template_mut_only_impl(
        &strct,
        &include_bytes_seq,
        &output_file_string,
    ));

    Ok(output)
}

fn derive_template_impl(tokens: TokenStream) -> Result<TokenStream, syn::Error> {
    let (strct, include_bytes_seq, output_file_string) =
        derive_template_common_impl(tokens)?;

    let mut output = TokenStream::new();

    output.append_all(derive_template_once_only_impl(
        &strct,
        &include_bytes_seq,
        &output_file_string,
    ));

    output.append_all(derive_template_mut_only_impl(
        &strct,
        &include_bytes_seq,
        &output_file_string,
    ));

    output.append_all(derive_template_only_impl(
        &strct,
        &include_bytes_seq,
        &output_file_string,
    ));

    Ok(output)
}

fn derive_template_simple_impl(tokens: TokenStream) -> Result<TokenStream, syn::Error> {
    let (strct, include_bytes_seq, output_file_string) =
        derive_template_common_impl(tokens)?;

    let name = &strct.ident;

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
                "You cannot derive `TemplateSimple` for tuple struct",
            ));
        }
    };

    let (impl_generics, ty_generics, where_clause) = strct.generics.split_for_impl();

    // render_once method always results in the same code.
    // This method can be implemented in `sailfish` crate, but I found that performance
    // drops when the implementation is written in `sailfish` crate.
    Ok(quote! {
        impl #impl_generics sailfish::TemplateSimple for #name #ty_generics #where_clause {
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
    })
}

pub fn derive_template_once(tokens: TokenStream) -> TokenStream {
    derive_template_once_impl(tokens).unwrap_or_else(|e| e.to_compile_error())
}

pub fn derive_template_mut(tokens: TokenStream) -> TokenStream {
    derive_template_mut_impl(tokens).unwrap_or_else(|e| e.to_compile_error())
}

pub fn derive_template(tokens: TokenStream) -> TokenStream {
    derive_template_impl(tokens).unwrap_or_else(|e| e.to_compile_error())
}

pub fn derive_template_simple(tokens: TokenStream) -> TokenStream {
    derive_template_simple_impl(tokens).unwrap_or_else(|e| e.to_compile_error())
}
