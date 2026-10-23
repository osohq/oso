//! # Error types used by the Oso library.
//!
//! This module contains a collection of error types that can be returned by various calls by the
//! Oso library.
use std::fmt;
use thiserror::Error;

/// Errors returned by the Polar library.
pub use polar_core::error as polar;

// TODO stack traces????

/// Oso error type.
///
/// This enum encompasses all things that can go wrong while using the Oso library. It can also be
/// used to wrap a custom error message, using the [`OsoError::Custom`] variant or using the
/// [`lazy_error`](crate::lazy_error) macro.
#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum OsoError {
    /// Input/output error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Polar error, see [`PolarError`](polar::PolarError).
    #[error(transparent)]
    Polar(#[from] polar::PolarError),

    /// Failed to convert type from Polar.
    #[error("failed to convert type from Polar")]
    FromPolar,

    /// Incorrect file name, must have `.polar` extension.
    #[error("policy files must have the .polar extension. {filename} does not.")]
    IncorrectFileType { filename: String },

    /// Invariant error.
    #[error(transparent)]
    InvariantError {
        #[from]
        source: InvariantError,
    },

    /// A TypeError caused by user input.
    #[error(transparent)]
    TypeError(TypeError),

    /// Unsupported operation for the given type.
    #[error("Unsupported operation {operation} for type {type_name}.")]
    UnsupportedOperation {
        operation: String,
        type_name: String,
    },

    /// Unimplemented operation.
    #[error("{operation} are unimplemented in the oso Rust library")]
    UnimplementedOperation { operation: String },

    /// Inline query failed.
    #[error("Inline query failed {location}")]
    InlineQueryFailedError { location: String },

    /// Invalid call error.
    #[error(transparent)]
    InvalidCallError(#[from] InvalidCallError),

    /// Failure converting type to polar.
    #[error("failed to convert type to Polar")]
    ToPolar,

    /// Class already registered.
    #[error("Class {name} already registered")]
    DuplicateClassError { name: String },

    /// Missing class error.
    #[error("No class called {name} has been registered")]
    MissingClassError { name: String },

    /// Missing instance error.
    #[error("Tried to find an instance that doesn't exist -- internal error")]
    MissingInstanceError,

    // TODO: replace all these with proper variants
    /// Custom error.
    #[error("{message}")]
    Custom { message: String },

    /// Error that was returned from application code (method on a class or instance).
    #[error("Error {source} returned from {}.{}",
        type_name.as_deref().unwrap_or("UNKNOWN"),
        attr.as_deref().unwrap_or("UNKNOWN"))]
    ApplicationError {
        source: Box<dyn std::error::Error + 'static + Send + Sync>,
        type_name: Option<String>,
        attr: Option<String>,
    },
    // TODO: Confusing that the above is called application error,
    // while the application_error function actually does something
    // totally different.
}

impl OsoError {
    /// Add `type_name` if `self` is a variant that has one.
    pub fn type_name(&mut self, name: String) {
        if let Self::ApplicationError { type_name, .. } = self {
            type_name.replace(name);
        }
    }

    /// Add `attr` if `self` is a variant that has one.
    pub fn attr(&mut self, name: String) {
        if let Self::ApplicationError { attr, .. } = self {
            attr.replace(name);
        }
    }
}

/// These are conditions that should never occur, and indicate a bug in Oso.
#[derive(Error, Debug)]
pub enum InvariantError {
    #[error("Invalid receiver for method. {0}")]
    InvalidReceiver(#[from] TypeError),

    #[error("invalid receiver - this is a bug")]
    MethodNotFound,
}

/// Type error
///
/// This error results from using the wrong type in a place where a specific type is expected.
#[derive(Error, Debug)]
pub struct TypeError {
    /// Type that was received
    pub got: Option<String>,
    /// Type that was expected
    pub expected: String,
}

impl fmt::Display for TypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref got) = self.got {
            writeln!(f, "Type error: Expected {} got {}", self.expected, got)
        } else {
            writeln!(f, "Type error: Expected {}.", self.expected)
        }
    }
}

impl TypeError {
    /// Create a type error with expected type `expected`.
    pub fn expected<T: Into<String>>(expected: T) -> Self {
        Self {
            got: None,
            expected: expected.into(),
        }
    }

    /// Set `got` on self.
    pub fn got<T: Into<String>>(mut self, got: T) -> Self {
        self.got.replace(got.into());
        self
    }

    /// Convert `self` into `InvariantError`,
    /// indicating an invariant that should never occur.
    pub fn invariant(self) -> InvariantError {
        InvariantError::from(self)
    }

    /// Convert `self` into `OsoError`, indicating a user originating type error.
    /// For example, calling a method with a parameter of an incorrect type from within Polar.
    pub fn user(self) -> OsoError {
        OsoError::TypeError(self)
    }
}

/// Invalid call error.
///
/// Generated when an invalid call is encountered, such as calling a method or attribute on a class
/// that does exist.
#[derive(Error, Debug)]
pub enum InvalidCallError {
    #[error("Class method {method_name} not found on type {type_name}.")]
    ClassMethodNotFound {
        method_name: String,
        type_name: String,
    },
    #[error("Method {method_name} not found on type {type_name}.")]
    MethodNotFound {
        method_name: String,
        type_name: String,
    },
    #[error("Attribute {attribute_name} not found on type {type_name}.")]
    AttributeNotFound {
        attribute_name: String,
        type_name: String,
    },
}

/// Convenience wrapper for Oso results.
///
/// This is the same as the standard library [`Result`], except that it defaults to [`OsoError`] as
/// the error type.
pub type Result<T, E = OsoError> = std::result::Result<T, E>;
