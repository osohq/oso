use thiserror::Error;

/// oso errors
///
/// TODO: fill in other variants
#[derive(Error, Debug)]
pub enum OsoError {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Polar(#[from] polar_core::error::PolarError),
    #[error("failed to convert type from Polar")]
    FromPolar,
    #[error("policy files must have the .polar extension. {filename} does not.")]
    IncorrectFileType {
        filename: String
    },

    #[error("Invariant error: {source}")]
    InvariantError {
        #[from]
        source: InvariantError,
    },

    /// A TypeError caused by user input.
    #[error(transparent)]
    TypeError(#[from] TypeError),

    #[error("Unsupported operation {operation} for type {type_name}.")]
    UnsupportedOperation {
        operation: String,
        type_name: String,
    },

    #[error("{operation} are unimplemented in oso-rust.")]
    UnimplementedOperation { operation: String },

    #[error("failed to convert type to Polar")]
    ToPolar,

    /// TODO: replace all these with proper variants
    #[error("{message}")]
    Custom { message: String },
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
#[error("Type error: expected `{expected}`")]
pub struct TypeError {
    pub expected: String,
}

impl TypeError {
    /// Convert `self` into `InvariantError`,
    /// indicating an invariant that should never occur.
    pub fn invariant(self) -> InvariantError {
        InvariantError::from(self)
    }

    /// Convert `self` into `OsoError`, indicating a user originating type error.
    /// For example, calling a method with a paramter of an incorrect type from within Polar.
    pub fn user(self) -> OsoError {
        OsoError::from(self)
    }
}

pub type Result<T> = std::result::Result<T, OsoError>;
