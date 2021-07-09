//! oso policy engine for authorization
//!
//! # Overview
//!
//! oso is a policy engine for authorization that's embedded in your application.
//! It provides a declarative policy language for expressing authorization logic.
//! You define this logic separately from the rest of your application code,
//! but it executes inside the application and can call directly into it.
//!
//! For more information, guides on using oso, writing policies and adding to your
//! application, go to the [oso documentation](https://docs.osohq.com).
//!
//! For specific information on using with Rust, see the [Rust documentation](https://docs.osohq.com/using/libraries/rust/index.html).
//!
//! ## Note
//!
//! The oso Rust library is still in early development relative to the other
//! oso libraries.
//!
//! # Example
//!
//! To get started, create a new `Oso` instance, and load Polar policies from either a
//! string or a file:
//!
//! ```
//! # use oso::Oso;
//! # fn main() -> anyhow::Result<()> {
//! let mut oso = Oso::new();
//! oso.load_str(r#"allow(actor, action, resource) if actor.username = "alice";"#)?;
//! # Ok(())
//! # }
//! ```
//!
//! You can register classes with oso, which makes it possible to use them for type checking,
//! as well as accessing attributes in policies.
//! The `PolarClass` derive macro can handle some of this
//! ```
//! # fn main() -> anyhow::Result<()> {
//! use oso::{Oso, PolarClass};
//!
//! let mut oso = Oso::new();
//!
//! #[derive(Clone, PolarClass)]
//! struct User {
//!     #[polar(attribute)]
//!     pub username: String,
//! }
//!
//! impl User {
//!     fn superuser() -> Vec<String> {
//!         return vec!["alice".to_string(), "charlie".to_string()]
//!     }
//! }
//!
//! oso.register_class(
//!    User::get_polar_class_builder()
//!         .add_class_method("superusers", User::superuser)
//!         .build()
//! )?;
//!
//! oso.load_str(r#"allow(actor: User, action, resource) if
//!                     actor.username.ends_with("example.com");"#)?;
//!
//! let user = User {
//!     username: "alice@example.com".to_owned(),
//! };
//! assert!(oso.is_allowed(user, "foo", "bar")?);
//! Ok(())
//! # }
//! ```
//! For more examples, see the [oso documentation](https://docs.osohq.com).
//!

#[macro_use]
pub mod macros;

pub(crate) mod builtins;
pub mod errors;
mod extras;
mod host;
mod oso;
mod query;

pub use crate::oso::{Action, Oso};
pub use errors::{OsoError, Result};
pub use host::{Class, ClassBuilder, FromPolar, FromPolarList, PolarValue, ToPolar, ToPolarList};
pub use query::{Query, ResultSet};

use polar_core::{
    polar::Polar,
    terms::{
        InstanceLiteral, Numeric, Operation, Operator, Pattern, Symbol, Term, ToPolarString, Value,
    },
    visitor::Visitor,
};

/// Classes that can be used as types in Polar policies.
///
/// Implementing this trait and `Clone` automatically makes the
/// type `FromPolar` and `ToPolar`, so it can be used with
/// `Oso::is_allowed` calls.
///
/// The default implementation creates a class definition with
/// no attributes or methods registered. Either use `get_polar_class_builder`
/// or the `#[derive(PolarClass)]` proc macro to register attributes and methods.
///
/// **Note** that the returned `Class` still must be registered on an `Oso`
/// instance using `Oso::register_class`.
pub trait PolarClass: Sized + 'static {
    /// Returns the `Class` ready for registration
    fn get_polar_class() -> Class {
        Self::get_polar_class_builder().build()
    }

    /// Returns the partially defined `Class` for this type.
    ///
    /// Can still have methods added to it with `add_method`, and attributes
    /// with `add_attribute_getter`.
    /// Use `Class::build` to finish defining the type.
    fn get_polar_class_builder() -> ClassBuilder<Self> {
        Class::builder()
    }
}

#[cfg(feature = "derive")]
#[allow(unused_imports)]
#[macro_use]
extern crate oso_derive;
#[cfg(feature = "derive")]
#[doc(hidden)]
pub use oso_derive::*;

use lazy_static::lazy_static;
use std::{
    borrow::Borrow,
    collections::HashMap,
    sync::{Arc, Mutex},
};
lazy_static! {
    pub static ref GLOBAL_OSO: Arc<Mutex<Oso>> = Default::default();
    pub static ref ALLOW_CACHE: Arc<Mutex<HashMap<(PolarValue, PolarValue), PolarValue>>> =
        Default::default();
}
pub fn magic_is_allowed<Actor, Action, Resource>(
    actor: Actor,
    action: Action,
    resource: Resource,
) -> crate::Result<bool>
where
    Actor: ToPolar,
    Action: ToPolar,
    Resource: ToPolar,
{
    let oso = GLOBAL_OSO.lock().unwrap();
    let mut cache = ALLOW_CACHE.lock().unwrap();

    let actor = actor.to_polar();
    let action = action.to_polar();
    let resource = resource.to_polar();

    let mut cached = None;

    if let Some(partial_res) = cache.get(&(actor.clone(), action.clone())) {
        println!(
            "Using precomputed partial: {}",
            match partial_res {
                PolarValue::Expression(o) => {
                    o.to_polar()
                }
                v => format!("{:#?}", v),
            }
        );
        cached = Some(partial_res.clone());
    } else {
        let resource_var = PolarValue::Variable("resource".to_string());
        let mut query = oso.query_rule("allow", (actor.clone(), action.clone(), resource_var))?;
        if let Some(Ok(res)) = query.next_result() {
            let partial_res = res.get("resource").unwrap();
            cache.insert((actor.clone(), action.clone()), partial_res.clone());
            cached = Some(partial_res)
        }
    }

    match cached {
        None => Ok(false),
        Some(p @ PolarValue::Expression(_)) => oso.query_partial(p, resource),
        Some(p) => Ok(p == resource.to_polar()),
    }
}

// pub fn type_constraint<T>() -> PolarValue {
//     let term = Operation {
//         operator: Operator::And,
//         args: vec![Term::new_temporary(Value::Expression(Operation {
//             operator: Operator::Isa,
//             args: vec![Term::new_temporary(Value::Pattern(Pattern {
//                 tag: std::any::type_name::<T>().to_string(),
//             }))],
//         }))],
//     };
//     PolarValue::Expression(term)
// }

#[derive(Default, Debug)]
pub struct CodegenVisitor {
    pub tokens: proc_macro2::TokenStream,
}

use quote::{format_ident, quote, ToTokens, TokenStreamExt};

impl CodegenVisitor {
    fn visit_any<T: ToTokens>(&mut self, t: T) {
        let tokens = self.tokens.clone();
        self.tokens = quote! {
            #tokens
            #t
        }
    }
}
fn term_to_tokens(term: &Term) -> proc_macro2::TokenStream {
    let mut cv = CodegenVisitor::default();
    cv.visit_term(&term);
    cv.tokens
}

impl Visitor for CodegenVisitor {
    fn visit_number(&mut self, n: &polar_core::terms::Numeric) {
        match n {
            Numeric::Float(f) => self.visit_any(f),
            Numeric::Integer(i) => self.visit_any(i),
        }
    }

    fn visit_string(&mut self, s: &str) {
        self.visit_any(s)
    }

    fn visit_boolean(&mut self, b: &bool) {
        self.visit_any(b)
    }

    fn visit_instance_id(&mut self, _i: &u64) {
        todo!()
    }

    fn visit_symbol(&mut self, s: &polar_core::terms::Symbol) {
        let name = format_ident!("{}", s.0);
        self.tokens.append(name);
    }

    fn visit_variable(&mut self, v: &polar_core::terms::Symbol) {
        let name = format_ident!("{}", v.0);
        self.tokens.append(name);
    }

    fn visit_rest_variable(&mut self, _r: &polar_core::terms::Symbol) {
        todo!()
    }

    fn visit_operator(&mut self, _o: &polar_core::terms::Operator) {
        todo!()
    }

    fn visit_rule(&mut self, r: &polar_core::rules::Rule) {
        panic!("should not need to visit a rule")
    }

    fn visit_term(&mut self, t: &polar_core::terms::Term) {
        polar_core::visitor::walk_term(self, t)
    }

    fn visit_field(&mut self, k: &polar_core::terms::Symbol, v: &polar_core::terms::Term) {
        polar_core::visitor::walk_field(self, k, v)
    }

    fn visit_external_instance(&mut self, e: &polar_core::terms::ExternalInstance) {
        polar_core::visitor::walk_external_instance(self, e)
    }

    fn visit_instance_literal(&mut self, i: &polar_core::terms::InstanceLiteral) {
        polar_core::visitor::walk_instance_literal(self, i)
    }

    fn visit_dictionary(&mut self, d: &polar_core::terms::Dictionary) {
        polar_core::visitor::walk_dictionary(self, d)
    }

    fn visit_pattern(&mut self, p: &polar_core::terms::Pattern) {
        polar_core::visitor::walk_pattern(self, p)
    }

    fn visit_call(&mut self, c: &polar_core::terms::Call) {
        polar_core::visitor::walk_call(self, c)
    }

    fn visit_list(&mut self, l: &polar_core::terms::TermList) {
        polar_core::visitor::walk_list(self, l)
    }

    fn visit_operation(&mut self, o: &polar_core::terms::Operation) {
        match o.operator {
            Operator::Debug => todo!(),
            Operator::Print => todo!(),
            Operator::Cut => todo!(),
            Operator::In => todo!(),
            Operator::Isa => {
                let lhs = term_to_tokens(&o.args[0]);
                let name = match &o.args[1].value() {
                    Value::Pattern(Pattern::Instance(InstanceLiteral { tag, .. })) => tag.0.clone(),
                    v => todo!("can only match instance literals, this is {:#?}", v),
                };
                let rhs = format_ident!("{}", name);
                self.tokens.extend(quote! {
                    std::any::Any::is::<#rhs>(&#lhs)
                });
            }
            Operator::New => todo!(),
            Operator::Dot => {
                let lhs = term_to_tokens(&o.args[0]);
                let attr = match o.args[1].value() {
                    Value::String(s) => format_ident!("{}", s),
                    _ => todo!("only support attribute lookups"),
                };
                self.tokens.extend(quote! {
                    #lhs.#attr
                })
            }
            Operator::Not => todo!(),
            Operator::Mul => todo!(),
            Operator::Div => todo!(),
            Operator::Mod => todo!(),
            Operator::Rem => todo!(),
            Operator::Add => todo!(),
            Operator::Sub => todo!(),
            Operator::Eq => todo!(),
            Operator::Geq => todo!(),
            Operator::Leq => todo!(),
            Operator::Neq => todo!(),
            Operator::Gt => todo!(),
            Operator::Lt => todo!(),
            Operator::Unify => {
                let lhs = term_to_tokens(&o.args[0]);
                let rhs = term_to_tokens(&o.args[1]);

                self.tokens.extend(quote! {
                    #lhs == #rhs
                })
            }
            Operator::Or => todo!(),
            Operator::And => {
                let children = o.args.iter().map(|t| {
                    let mut cv = CodegenVisitor::default();
                    cv.visit_term(&t);
                    cv.tokens
                });
                self.tokens.extend(quote! {
                    true
                    #(
                        && (
                            #children
                        )
                    )*
                });
            }
            Operator::ForAll => todo!(),
            Operator::Assign => todo!(),
        }
    }

    fn visit_param(&mut self, p: &polar_core::rules::Parameter) {
        polar_core::visitor::walk_param(self, p)
    }
}
