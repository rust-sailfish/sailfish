use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use std::fs;
use std::path::{Path, PathBuf};
use syn::parse::{Parse, ParseStream, Result as ParseResult};
use syn::punctuated::Punctuated;
use syn::{Fields, Ident, ItemStruct, Lifetime, LitBool, LitChar, LitStr, Token};

use crate::config::Config;
use crate::compiler::Compiler;
use crate::error::*;

enum GenericParamName {
    Ident(Ident),
    LifeTime(Lifetime),
}

impl ToTokens for GenericParamName {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            GenericParamName::Ident(ref i) => i.to_tokens(tokens),
            GenericParamName::LifeTime(ref l) => l.to_tokens(tokens),
        }
    }
}

// arguments for include_template* macros
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

struct TemplateStruct {
    options: DeriveTemplateOptions,
}

fn compile(
    template_dir: &Path,
    input_file: &Path,
    output_file: &Path,
    options: &DeriveTemplateOptions,
) -> Result<(), Error> {
    let mut config = Config::default();
    if let Some(ref delimiter) = options.delimiter {
        config.delimiter = delimiter.value();
    }
    if let Some(ref escape) = options.escape {
        config.escape = escape.value;
    }
    if let Some(ref rm_whitespace) = options.rm_whitespace {
        config.rm_whitespace = rm_whitespace.value;
    }

    let compiler = Compiler::with_config(config);
    compiler.compile_file(template_dir, input_file, &*output_file)
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

    let mut template_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect(
        "Internal error: environmental variable `CARGO_MANIFEST_DIR` is not set.",
    ));
    template_dir.push("templates");

    let input_file = match all_options.path {
        Some(ref path) => template_dir.join(path.value()),
        None => {
            return Err(syn::Error::new(
                Span::call_site(),
                "`path` option must be specified.",
            )
            .into())
        }
    };

    let filename = match input_file.file_name() {
        Some(f) => f,
        None => {
            return Err(syn::Error::new(
                Span::call_site(),
                format!("Invalid file name: {:?}", input_file),
            ))
        }
    };

    let out_dir = match std::env::var("SAILFISH_OUTPUT_DIR") {
        Ok(dir) => {
            let p = PathBuf::from(dir);
            fs::create_dir_all(&*p).unwrap();
            p.canonicalize().unwrap()
        }
        Err(_) => PathBuf::from(env!("OUT_DIR")),
    };
    let mut output_file = out_dir.clone();
    output_file.push("templates");
    output_file.push(filename);

    compile(&*template_dir, &*input_file, &*output_file, &all_options)
        .map_err(|e| syn::Error::new(Span::call_site(), e))?;

    let input_file_string = input_file.to_string_lossy();
    let output_file_string = output_file.to_string_lossy();

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
            fn render_once(self) -> sailfish::runtime::RenderResult {
                include_bytes!(#input_file_string);

                use sailfish::runtime as sfrt;
                use sfrt::Render as _;

                static SIZE_HINT: sfrt::SizeHint = sfrt::SizeHint::new();
                let _size_hint = SIZE_HINT.get();
                let mut _ctx = sfrt::Context {
                    buf: sfrt::Buffer::with_capacity(_size_hint)
                };

                let #name { #field_names } = self;
                include!(#output_file_string);

                SIZE_HINT.update(_ctx.buf.len());
                _ctx.into_result()
            }
        }
    };

    Ok(tokens)
}

pub fn derive_template(tokens: TokenStream) -> TokenStream {
    derive_template_impl(tokens).unwrap_or_else(|e| e.to_compile_error())
}
