extern crate proc_macro;
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Fields, Ident, Lit, Meta, MetaNameValue, NestedMeta, Path};

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
            // @TODO: Probably error.
            unimplemented!("Hit a literal instead of a name.");
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
                    // I don't know why this case would happen, seems like
                    // polar(foo, bar) would each be two different attributes
                    // nested under polar, not a single nested list.
                    // leaving here till we add all the attributes we need in case it pops up.
                    unimplemented!("Hit the list case");
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
    let input = syn::parse_macro_input!(ts as syn::ItemStruct);

    let type_name = input.ident;
    let mut class_name = type_name.to_string();

    let attrs = input.attrs;
    let mut oso_attrs = vec![];
    for attr in attrs {
        get_oso_attrs(attr, &mut oso_attrs);
    }
    for oso_attr in oso_attrs {
        match oso_attr {
            OsoAttribute::ClassName { name } => {
                class_name = name;
            }
            _ => (),
        }
    }

    let mut getters = vec![];

    match input.fields {
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
        _ => (), // don't support tuple structs and unit structs for now
    }

    let result = quote! {
        impl oso::PolarClass for #type_name {
            fn get_polar_class_builder() -> oso::Class<#type_name> {
                oso::Class::with_constructor(<#type_name>::new)
                    .name(#class_name)
                    #(#getters)*
            }

            fn get_polar_class() -> oso::Class<()> {
                let builder = #type_name::get_polar_class_builder();
                builder.build()
            }
        }
    };
    result.into()
}
