//! # oso policy engine for authorization
//!
//! TODO: API documentation

#[macro_use]
pub mod errors;

pub(crate) mod builtins;
mod host;
mod oso;
mod query;

pub use crate::oso::Oso;
pub use errors::OsoError;
pub use host::{Class, FromPolar, HostClass, ToPolar};
pub use polar_core::polar::Polar;
pub use query::{Query, ResultSet};

pub type Result<T> = std::result::Result<T, OsoError>;
