//! # oso policy engine for authorization
//!
//! TODO: API documentation

#[macro_use]
pub mod errors;

mod host;
mod oso;
mod query;

pub use errors::OsoError;
pub use host::{Class, FromPolar, HostClass, ToPolar};
pub use oso::Oso;
pub use polar_core::polar::Polar;

pub type Result<T> = std::result::Result<T, OsoError>;
