extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Fields, Ident, Lit, Meta, MetaNameValue, NestedMeta, Path};

// @TODO: How would I get attributes on methods in an impl block from this derive macro?
// Does that even make sense or do I need another kind of macro.
#[derive(Debug, PartialEq)]
enum OsoAttribute {
    ClassName { name: String },
    Attribute,
}

fn get_single_segment(path: &Path) -> Option<String> {
    if path.segments.len() == 1 {
        Some(path.segments[0].ident.to_string())
    } else {
        None
    }
}

fn get_path(path: &Path) -> Vec<String> {
    path.segments
        .iter()
        .map(|seg| seg.ident.to_string())
        .collect()
}

fn get_nested_attr(nested: NestedMeta, oso_attrs: &mut Vec<OsoAttribute>) {
    match nested {
        NestedMeta::Lit(_) => {
            // Dont have any of these.
        }
        NestedMeta::Meta(meta) => {
            match meta {
                Meta::Path(path) => {
                    match get_single_segment(&path) {
                        Some(ref seg) if seg == "attribute" => {
                            oso_attrs.push(OsoAttribute::Attribute);
                        }
                        _ => (),
                    };
                }
                Meta::List(_) => {
                    // I don't know why this would happen, seems like
                    // polar(foo, bar) would each be two different attributes
                    // nested under polar, not a single nested list.
                    ()
                }
                Meta::NameValue(MetaNameValue { path, lit, .. }) => {
                    if let Some(ref seg) = get_single_segment(&path) {
                        if seg == "class_name" {
                            // @TODO: Type error if it's not a string.
                            if let Lit::Str(class_name) = lit {
                                oso_attrs.push(OsoAttribute::ClassName {
                                    name: class_name.value(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

fn get_oso_attrs(attr: Attribute, oso_attrs: &mut Vec<OsoAttribute>) {
    let style = attr.style;
    let meta = attr.parse_meta().unwrap();
    if let Meta::List(list) = meta {
        match get_single_segment(&list.path) {
            Some(ref seg) if seg == "polar" => {
                for nested in list.nested {
                    get_nested_attr(nested, oso_attrs);
                }
            }
            _ => (),
        }
    }
}

#[proc_macro_derive(PolarClass, attributes(polar))]
pub fn derive_testing_fn(ts: TokenStream) -> TokenStream {
    eprintln!("Derive!");

    let input = syn::parse_macro_input!(ts as syn::ItemStruct);

    eprintln!("{:?}", input);

    let type_name = input.ident;
    let mut class_name = type_name.to_string();

    let attrs = input.attrs;
    eprintln!("Attrs");
    let mut oso_attrs = vec![];
    for attr in attrs {
        get_oso_attrs(attr, &mut oso_attrs);
    }
    for oso_attr in oso_attrs {
        eprintln!("attribute {:?}", oso_attr);
        match oso_attr {
            OsoAttribute::ClassName { name } => {
                class_name = name;
            }
            _ => (), // @TODO: error on attribute on struct,
        }
    }

    let mut getters = vec![];

    match input.fields {
        // Named fields, a normal struct.
        Fields::Named(fields) => {
            for field in fields.named {
                let mut oso_attrs = vec![];
                for attr in field.attrs {
                    get_oso_attrs(attr, &mut oso_attrs);
                }
                if oso_attrs.contains(&OsoAttribute::Attribute) {
                    let attr = field.ident.unwrap();
                    let name = attr.to_string();
                    getters.push(quote! {
                        .add_attribute_getter(#name, |recv: &Foo| recv.#attr.clone())
                    })
                }
            }
        }
        _ => (), // tuple structs and unit structs
    }

    // let attribute_getters = attributes.iter().map(|attr| {
    //     let name = attr.to_string();
    //     quote! {
    //         .add_attribute_getter(#name, |recv: &Foo| recv.#attr.clone())
    //     }
    // });

    // let attribute_getters = quote! {
    //     .add_attribute_getter("a", |recv: &Foo| recv.a.clone())
    // };

    let result = quote! {
        fn register_class(mut oso: &mut oso::Oso) -> Result<(), oso::OsoError> {
            oso::Class::with_constructor(<#type_name>::new)
                .name(#class_name)
                #(#getters)*
                .register(&mut oso)
        }
    };
    result.into()
}
