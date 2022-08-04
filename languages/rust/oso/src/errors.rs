use std::fmt;

use thiserror::Error;

pub use polar_core::error as polar;

// TODO stack traces????

/// oso errors
///
/// TODO: fill in other variants
#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum OsoError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Polar(#[from] polar::PolarError),
    #[error("failed to convert type from Polar")]
    FromPolar,
    #[error("policy files must have the .polar extension. {filename} does not.")]
    IncorrectFileType { filename: String },

    #[error("Invariant error: {source}")]
    InvariantError {
        #[from]
        source: InvariantError,
    },

    /// A TypeError caused by user input.
    #[error(transparent)]
    TypeError(TypeError),

    #[error("Unsupported operation {operation} for type {type_name}.")]
    UnsupportedOperation {
        operation: String,
        type_name: String,
    },

    #[error("{operation} are unimplemented in the oso Rust library")]
    UnimplementedOperation { operation: String },

    #[error("Inline query failed {location}")]
    InlineQueryFailedError { location: String },

    #[error(transparent)]
    InvalidCallError(#[from] InvalidCallError),

    #[error("failed to convert type to Polar")]
    ToPolar,

    #[error("Class {name} already registered")]
    DuplicateClassError { name: String },

    #[error("No class called {name} has been registered")]
    MissingClassError { name: String },

    #[error("Tried to find an instance that doesn't exist -- internal error")]
    MissingInstanceError,

    /// TODO: replace all these with proper variants
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

/// These are conditions that should never occur, and indicate a bug in oso.
#[derive(Error, Debug)]
pub enum InvariantError {
    #[error("Invalid receiver for method. {0}")]
    InvalidReceiver(#[from] TypeError),

    #[error("invalid receiver - this is a bug")]
    MethodNotFound,
}

#[derive(Error, Debug)]
pub struct TypeError {
    pub got: Option<String>,
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

pub type Result<T> = std::result::Result<T, OsoError>;
