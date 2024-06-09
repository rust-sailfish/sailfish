#![forbid(unsafe_code)]

extern crate proc_macro;

use proc_macro::TokenStream;

#[proc_macro_derive(TemplateOnce, attributes(template))]
pub fn derive_template_once(tokens: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(tokens);
    let output = sailfish_compiler::procmacro::derive_template_once(input);
    TokenStream::from(output)
}

#[proc_macro_derive(TemplateMut, attributes(template))]
pub fn derive_template_mut(tokens: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(tokens);
    let output = sailfish_compiler::procmacro::derive_template_mut(input);
    TokenStream::from(output)
}

#[proc_macro_derive(Template, attributes(template))]
pub fn derive_template(tokens: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(tokens);
    let output = sailfish_compiler::procmacro::derive_template(input);
    TokenStream::from(output)
}

#[proc_macro_derive(TemplateSimple, attributes(template))]
pub fn derive_template_simple(tokens: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(tokens);
    let output = sailfish_compiler::procmacro::derive_template_simple(input);
    TokenStream::from(output)
}
