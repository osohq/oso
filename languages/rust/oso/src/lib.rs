//! Oso policy engine for authorization
//!
//! # Overview
//!
//! Oso is a policy engine for authorization that's embedded in your application.
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
//! The Oso Rust library is still in early development relative to the other
//! Oso libraries.
//!
//! # Example
//!
//! To get started, create a new [`Oso`] instance, and load Polar policies from either a
//! string or a file:
//!
//! ```
//! # use oso::Oso;
//! # fn main() -> anyhow::Result<()> {
//! let mut oso = Oso::new();
//! oso.load_str(r#"allow(actor, _action, _resource) if actor.username = "alice";"#)?;
//! # Ok(())
//! # }
//! ```
//!
//! You can register classes with oso, which makes it possible to use them for type checking, as
//! well as accessing attributes in policies.
//! The [`PolarClass`](oso_derive::PolarClass) derive macro can handle some of this.
//!
//! ```
//! # fn main() -> anyhow::Result<()> {
//! use oso::{Oso, PolarClass};
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
//! let mut oso = Oso::new();
//!
//! oso.register_class(
//!    User::get_polar_class_builder()
//!         .add_class_method("superusers", User::superuser)
//!         .build()
//! )?;
//!
//! oso.load_str(r#"allow(actor: User, _action, _resource) if
//!                     actor.username.ends_with("example.com");"#)?;
//!
//! let user = User {
//!     username: "alice@example.com".to_owned(),
//! };
//!
//! assert!(oso.is_allowed(user, "foo", "bar")?);
//! # Ok(())
//! # }
//! ```
//! For more examples, see the [Oso documentation](https://docs.osohq.com).
//!

#[macro_use]
mod macros;

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

use polar_core::polar::Polar;

/// Classes that can be used as types in Polar policies.
///
/// Implementing this trait and [`Clone`] automatically makes the type [`FromPolar`] and
/// [`ToPolar`], so it can be used with [`Oso::is_allowed()`] calls.
///
/// The default implementation creates a class definition with no attributes or methods registered.
/// Either use [`get_polar_class_builder()`](PolarClass::get_polar_class_builder) or the
/// [`PolarClass`](oso_derive::PolarClass) proc macro to register attributes and methods.
///
/// **Note**: the returned [`Class`] still must be registered on an [`Oso`] instance using
/// [`Oso::register_class()`].
///
/// # Examples
///
/// Register polar class:
///
/// ```
/// use oso::{Oso, PolarClass};
///
/// #[derive(PolarClass)]
/// struct MyClass {
///     #[polar(attribute)]
///     name: String,
/// }
///
/// let mut oso = Oso::new();
/// oso.register_class(MyClass::get_polar_class());
/// ```
///
/// Register polar class with customisations:
///
/// ```
/// use oso::{Oso, PolarClass};
///
/// #[derive(PolarClass, Default, PartialEq)]
/// struct MyClass {
///     name: String,
/// }
///
/// let class = MyClass::get_polar_class_builder()
///     .add_attribute_getter("name", |value| value.name.clone())
///     .with_equality_check()
///     .build();
///
/// let mut oso = Oso::new();
/// oso.register_class(class);
/// ```
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
