use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    Attribute, Data, DataEnum, DataStruct, Fields, Lit, Meta, MetaNameValue, NestedMeta, Path,
};

mod compile;

#[proc_macro]
pub fn load_file(input: TokenStream) -> TokenStream {
    compile::load_file(input)
}

#[proc_macro]
pub fn is_allowed(input: TokenStream) -> TokenStream {
    compile::is_allowed(input)
}
