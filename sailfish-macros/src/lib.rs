#![forbid(unsafe_code)]

extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(TemplateOnce, attributes(template))]
pub fn derive_template_once(tokens: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(tokens);
    let output = sailfish_compiler::procmacro::derive_template(input);
    TokenStream::from(output)
}

/// WIP
#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(tokens: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(tokens);
    let output = sailfish_compiler::procmacro::derive_template(input);
    TokenStream::from(output)
}
