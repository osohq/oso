//! # oso policy engine for authorization
//!
//! TODO: API documentation

#[macro_use]
pub mod macros;

pub(crate) mod builtins;
mod errors;
mod host;
mod oso;
mod query;

pub use crate::oso::Oso;
pub use errors::OsoError;
pub use host::{Class, FromPolar, HostClass, ToPolar};
pub use polar_core::{polar::Polar, terms::Value};
pub use query::{Query, ResultSet};

pub type Result<T> = std::result::Result<T, OsoError>;

pub trait PolarClass {
    fn get_polar_class() -> Class<()>;
    fn get_polar_class_builder() -> Class<Self>
    where
        Self: Sized;
}
