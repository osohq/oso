use proc_macro::{Ident, Literal, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse::Parser, parse_macro_input, parse_quote::ParseQuote, punctuated::Punctuated, Attribute,
    Data, DataEnum, DataStruct, Expr, Fields, Lit, LitStr, Meta, MetaNameValue, NestedMeta, Path,
    Token, Type,
};

pub fn load_file(input: TokenStream) -> TokenStream {
    // eprintln!("{:#?}", input);
    let filename = parse_macro_input!(input as LitStr);
    let filename = filename.value();
    let name = std::path::Path::new(&filename);
    let root = name.file_stem().unwrap().to_string_lossy();
    let ident = format_ident!("load_{}", root);
    let result = quote! {
        #[ctor]
        fn #ident() {
            let here = std::path::Path::new(file!());
            let mut stem = here.parent().unwrap_or_else(|| std::path::Path::new(""));
            let filename = stem.join(#filename);
            println!("Loading file: {}", filename.to_string_lossy());
            let res = oso::GLOBAL_OSO.lock().unwrap().load_file(filename);
            if let Err(e) = res {
                println!("Loading policy errored:\n{}", e);
            }
        }
    };
    eprintln!("{}", result);
    // eprintln!("{:#?}", result);
    result.into()
}

pub fn is_allowed(input: TokenStream) -> TokenStream {
    let tokens = input.clone();
    let parser = Punctuated::<Expr, Token![,]>::parse_terminated;
    let args = parser.parse(tokens).unwrap();
    assert_eq!(args.len(), 3);
    let actor = args[0].clone();
    let action = args[1].clone();
    let resource = args[2].clone();

    let result = quote! {{
        let _ = tracing_subscriber::fmt::try_init();

        println!("Calling is_allowed({}, {}, {})", #actor, #action, #resource);
        oso::GLOBAL_OSO.lock().unwrap().is_allowed(#actor, #action, #resource).unwrap()
    }};
    result.into()
}
