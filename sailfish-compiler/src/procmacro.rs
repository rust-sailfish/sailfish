use proc_macro2::{Span, TokenStream};
use quote::quote;
use std::env;
use std::path::{Path, PathBuf};
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
        move |outer: ParseStream| -> ParseResult<()> {
            let s;
            syn::parenthesized!(s in outer);

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

fn compile(
    input_file: &Path,
    config: Config,
) -> Result<(CompilationReport, String), Error> {
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
    compiler.compile_file(input_file)
}

fn derive_template_impl(tokens: TokenStream) -> Result<TokenStream, syn::Error> {
    let strct = syn::parse2::<ItemStruct>(tokens)?;

    let mut all_options = DeriveTemplateOptions::default();
    for attr in strct.attrs {
        if attr.path.is_ident("template") {
            syn::parse::Parser::parse2(all_options.parser(), attr.tokens)?;
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
    let (report, output_str) = compile(&*input_file, config)
        .map_err(|e| syn::Error::new(Span::call_site(), e))?;

    let input_file_string = input_file
        .to_str()
        .unwrap_or_else(|| panic!("Non UTF-8 file name: {:?}", input_file));

    let mut include_bytes_seq = quote! { include_bytes!(#input_file_string); };
    for dep in report.deps {
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
    let out_tokens: syn::Expr = syn::parse_str(&output_str).expect("Must be an expr; qed");

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
                #include_bytes_seq;

                use sailfish::runtime as __sf_rt;
                let #name { #field_names } = self;
                #out_tokens;

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
