use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::env;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::punctuated::Punctuated;
use syn::{Fields, Ident, ItemStruct, LitBool, LitChar, LitStr, Token};

use crate::compiler::{CompilationReport, Compiler};
use crate::config::Config;
use crate::error::*;

// options for `template` attributes
#[derive(Default)]
struct DeriveTemplateOptions {
    path: Option<LitStr>,
    delimiter: Option<LitChar>,
    escape: Option<LitBool>,
    rm_whitespace: Option<LitBool>,
    type_: Option<LitStr>,
}

impl Parse for DeriveTemplateOptions {
    fn parse(outer: ParseStream) -> ParseResult<Self> {
        let s;
        syn::parenthesized!(s in outer);

        let mut options = Self::default();
        let mut found_keys = Vec::new();

        while !s.is_empty() {
            let key = s.parse::<Ident>()?;
            s.parse::<Token![=]>()?;

            // check if argument is repeated
            if found_keys.iter().any(|e| *e == key) {
                return Err(syn::Error::new(
                    key.span(),
                    format!("Argument `{}` was repeated.", key),
                ));
            }

            if key == "path" {
                options.path = Some(s.parse::<LitStr>()?);
            } else if key == "delimiter" {
                options.delimiter = Some(s.parse::<LitChar>()?);
            } else if key == "escape" {
                options.escape = Some(s.parse::<LitBool>()?);
            } else if key == "rm_whitespace" {
                options.rm_whitespace = Some(s.parse::<LitBool>()?);
            } else if key == "type" {
                options.type_ = Some(s.parse::<LitStr>()?);
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    format!("Unknown option: `{}`", key),
                ));
            }

            found_keys.push(key);

            // consume comma token
            if s.is_empty() {
                break;
            } else {
                s.parse::<Token![,]>()?;
            }
        }

        Ok(options)
    }
}

impl DeriveTemplateOptions {
    fn merge(&mut self, other: DeriveTemplateOptions) -> Result<(), syn::Error> {
        fn merge_single<T: ToTokens>(
            lhs: &mut Option<T>,
            rhs: Option<T>,
        ) -> Result<(), syn::Error> {
            if lhs.is_some() {
                if let Some(rhs) = rhs {
                    Err(syn::Error::new_spanned(rhs, "keyword argument repeated."))
                } else {
                    Ok(())
                }
            } else {
                *lhs = rhs;
                Ok(())
            }
        }

        merge_single(&mut self.path, other.path)?;
        merge_single(&mut self.delimiter, other.delimiter)?;
        merge_single(&mut self.escape, other.escape)?;
        merge_single(&mut self.rm_whitespace, other.rm_whitespace)?;
        merge_single(&mut self.type_, other.type_)?;
        Ok(())
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

fn filename_hash(path: &Path) -> String {
    use std::fmt::Write;

    const FNV_PRIME: u64 = 1_099_511_628_211;
    const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;

    let mut hash = String::with_capacity(16);

    if let Some(n) = path.file_name() {
        let mut filename = &*n.to_string_lossy();
        if let Some(p) = filename.find('.') {
            filename = &filename[..p];
        }
        hash.push_str(filename);
        hash.push('-');
    }

    // calculate 64bit hash
    let mut h = FNV_OFFSET_BASIS;
    for b in (&*path.to_string_lossy()).bytes() {
        h = h.wrapping_mul(FNV_PRIME);
        h ^= b as u64;
    }

    // convert 64bit hash into ascii
    let _ = write!(hash, "{:016x}", h);

    hash
}

fn compile(
    input_file: &Path,
    output_file: &Path,
    config: Config,
) -> Result<CompilationReport, Error> {
    let compiler = Compiler::with_config(config);
    compiler.compile_file(input_file, &*output_file)
}

fn derive_template_impl(tokens: TokenStream) -> Result<TokenStream, syn::Error> {
    let strct = syn::parse2::<ItemStruct>(tokens)?;

    let mut all_options = DeriveTemplateOptions::default();
    for attr in strct.attrs {
        if attr.path.is_ident("template") {
            let opt = syn::parse2::<DeriveTemplateOptions>(attr.tokens)?;
            all_options.merge(opt)?;
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

    let out_dir = PathBuf::from(env!("OUT_DIR"));
    let mut output_file = out_dir.clone();
    output_file.push("templates");
    output_file.push(filename_hash(&*input_file));

    merge_config_options(&mut config, &all_options);
    let report = compile(&*input_file, &*output_file, config)
        .map_err(|e| syn::Error::new(Span::call_site(), e))?;

    let input_file_string = input_file.to_string_lossy();
    let output_file_string = output_file.to_string_lossy();

    let mut include_bytes_seq = quote! { include_bytes!(#input_file_string); };
    for dep in report.deps {
        let dep_string = dep.to_string_lossy();
        include_bytes_seq.extend(quote! { include_bytes!(#dep_string); });
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

    let tokens = quote! {
        impl #impl_generics sailfish::TemplateOnce for #name #ty_generics #where_clause {
            fn render_once_to_string(self, buf: &mut String) -> Result<(), sailfish::runtime::RenderError> {
                #include_bytes_seq;

                use sailfish::runtime as __sf_rt;

                static SIZE_HINT: __sf_rt::SizeHint = __sf_rt::SizeHint::new();

                let mut _ctx = __sf_rt::Context {
                    buf: __sf_rt::Buffer::from(std::mem::take(buf))
                };

                let _size_hint = SIZE_HINT.get();
                _ctx.buf.reserve(_size_hint);

                let #name { #field_names } = self;
                include!(#output_file_string);

                SIZE_HINT.update(_ctx.buf.len());
                *buf = _ctx.buf.into_string();
                Ok(())
            }
        }
    };

    Ok(tokens)
}

pub fn derive_template(tokens: TokenStream) -> TokenStream {
    derive_template_impl(tokens).unwrap_or_else(|e| e.to_compile_error())
}
