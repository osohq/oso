#[macro_use]
extern crate quote;
extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;

// stub

#[proc_macro_derive(PolarClass)]
pub fn derive_testing_fn(_item: TokenStream) -> TokenStream {
    r#"fn register_class(mut oso: &mut oso::Oso) -> Result<(), oso::OsoError> { 
        oso::Class::with_constructor(Foo::new)
            .name("Foo")
            .register(&mut oso)
    }"#
    .parse()
    .unwrap()
}
