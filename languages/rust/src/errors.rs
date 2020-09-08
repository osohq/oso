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
    #[error("policy files must end in .polar")]
    IncorrectFileType,
    #[error("invalid receiver - this is a bug")]
    InvalidReceiver,
    #[error("invalid receiver - this is a bug")]
    MethodNotFound,
    #[error("failed to convert type to Polar")]
    ToPolar,

    /// TODO: replace all these with proper variants
    #[error("`{message}`")]
    Custom { message: String },
}
