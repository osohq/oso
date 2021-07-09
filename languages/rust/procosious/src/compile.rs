use proc_macro::{Ident, Literal, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse::Parser,
    parse_macro_input,
    parse_quote::{self, ParseQuote},
    punctuated::Punctuated,
    Attribute, Data, DataEnum, DataStruct, Expr, Fields, Lit, LitStr, Meta, MetaNameValue,
    NestedMeta, Path, Token, Type,
};

use oso::{CodegenVisitor, PolarValue};
use polar_core::formatting::ToPolarString;
use polar_core::visitor::Visitor;

pub fn load_file(input: TokenStream) -> TokenStream {
    // eprintln!("{:#?}", input);
    let filename = parse_macro_input!(input as LitStr);
    let filename = filename.value();
    let name = std::path::Path::new(&filename);
    let root = name.file_stem().unwrap().to_string_lossy();
    let ident = format_ident!("load_{}", root);
    eprintln!("{:#?}", std::env::current_dir().unwrap());
    eprintln!("{:#?}", filename);
    let oso = oso::GLOBAL_OSO.lock().unwrap();
    oso.load_file(format!("languages/rust/oso/examples/{}", filename.clone()))
        .unwrap();

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
    // let actor = args[0].clone();
    // let action = args[1].clone();
    let resource = args[2].clone();
    // let (resource_ident, resource_type) = match resource {
    //     Expr::Cast(cast) => (*cast.expr, Some(cast.ty)),
    //     _ => (resource, None),
    // };
    // eprintln!("{:#?}. {:#?}", resource_ident, resource_type);

    let actor_var = PolarValue::Variable("actor".to_string());
    let action_var = PolarValue::Variable("action".to_string());
    let resource_var = PolarValue::Variable("resource".to_string());

    let mut oso = oso::GLOBAL_OSO.lock().unwrap();
    let mut query = oso
        .query_rule("allow", (actor_var, action_var, resource_var))
        .unwrap();

    let mut tokens = quote! {};
    if let Some(Ok(res)) = query.next_result() {
        let actor = res.get("actor").unwrap();
        let action = res.get("action").unwrap();
        let resource_result = res.get("resource").unwrap();

        match resource_result {
            PolarValue::Expression(o) => {
                eprintln!("Compile: {}", o.to_polar());
                let mut codegen = CodegenVisitor::default();
                codegen.visit_operation(&o);
                tokens = codegen.tokens;
                tokens = quote! {
                    let resource = #resource;
                    let res = {
                        #tokens
                    };
                    res
                }
            }
            _ => todo!(),
        }
    } else {
        panic!("This rule will _never_ succeed!");
    }

    let result = quote! {{
        let _ = tracing_subscriber::fmt::try_init();
        #tokens
    }};
    eprintln!("{}", result);
    result.into()
}
