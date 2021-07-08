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
pub mod proc_madness;
mod query;

pub use crate::oso::{Action, Oso};
pub use errors::{OsoError, Result};
pub use host::{Class, ClassBuilder, FromPolar, FromPolarList, PolarValue, ToPolar, ToPolarList};
pub use query::{Query, ResultSet};

use polar_core::{polar::Polar, terms::Numeric};

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
use std::sync::{Arc, Mutex};
lazy_static! {
    pub static ref GLOBAL_OSO: Arc<Mutex<Oso>> = Default::default();
}

struct CodegenVisitor {
    tokens: proc_macro2::TokenStream,
}

use quote::{quote, ToTokens};

impl CodegenVisitor {
    fn visit_any<T: ToTokens>(&mut self, t: T) {
        let tokens = self.tokens.clone();
        self.tokens = quote! {
            #tokens
            #t
        }
    }
}

impl polar_core::visitor::Visitor for CodegenVisitor {
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

    fn visit_symbol(&mut self, _s: &polar_core::terms::Symbol) {
        todo!()
    }

    fn visit_variable(&mut self, _v: &polar_core::terms::Symbol) {
        todo!()
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
        polar_core::visitor::walk_operation(self, o)
    }

    fn visit_param(&mut self, p: &polar_core::rules::Parameter) {
        polar_core::visitor::walk_param(self, p)
    }
}
