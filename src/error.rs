use std::collections::HashMap;
use std::fmt;

/// Machine-readable error code returned by the Lettr API.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ErrorCode {
    ValidationError,
    InvalidDomain,
    UnconfiguredDomain,
    SendError,
    TransmissionFailed,
    ResourceAlreadyExists,
    TemplateNotFound,
    NotFound,
    QuotaExceeded,
    DailyQuotaExceeded,
    InsufficientScope,
    ScheduleCancellationFailed,
    /// An unknown error code not yet covered by this enum.
    #[serde(untagged)]
    Unknown(String),
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ValidationError => write!(f, "validation_error"),
            Self::InvalidDomain => write!(f, "invalid_domain"),
            Self::UnconfiguredDomain => write!(f, "unconfigured_domain"),
            Self::SendError => write!(f, "send_error"),
            Self::TransmissionFailed => write!(f, "transmission_failed"),
            Self::ResourceAlreadyExists => write!(f, "resource_already_exists"),
            Self::TemplateNotFound => write!(f, "template_not_found"),
            Self::NotFound => write!(f, "not_found"),
            Self::QuotaExceeded => write!(f, "quota_exceeded"),
            Self::DailyQuotaExceeded => write!(f, "daily_quota_exceeded"),
            Self::InsufficientScope => write!(f, "insufficient_scope"),
            Self::ScheduleCancellationFailed => write!(f, "schedule_cancellation_failed"),
            Self::Unknown(s) => write!(f, "{s}"),
        }
    }
}

/// Error type for operations of a [`Lettr`](crate::Lettr) client.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Errors that may occur during the processing of an HTTP request.
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// API returned an error response.
    #[error("api error: {0}")]
    Api(#[from] ApiError),

    /// Validation error returned by the API.
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),

    /// Failed to parse the API response.
    #[error("failed to parse API response: {0}")]
    Parse(String),
}

/// An error response from the Lettr API.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiError {
    /// Human-readable error message.
    pub message: String,
    /// Machine-readable error code.
    #[serde(default)]
    pub error_code: Option<ErrorCode>,
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref code) = self.error_code {
            write!(f, "[{}] {}", code, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ApiError {}

/// A validation error response from the Lettr API.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ValidationError {
    /// Human-readable error message.
    pub message: String,
    /// Machine-readable error code.
    #[serde(default)]
    pub error_code: Option<ErrorCode>,
    /// Field-level validation errors.
    #[serde(default)]
    pub errors: HashMap<String, Vec<String>>,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;
        for (field, messages) in &self.errors {
            for msg in messages {
                write!(f, "\n  {field}: {msg}")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for ValidationError {}

/// Intermediate struct for detecting error shape from the API.
#[derive(Debug, serde::Deserialize)]
pub(crate) struct RawErrorResponse {
    pub message: String,
    #[serde(default)]
    pub error_code: Option<ErrorCode>,
    #[serde(default)]
    pub errors: Option<HashMap<String, Vec<String>>>,
}

impl RawErrorResponse {
    /// Convert into the appropriate [`Error`] variant.
    pub fn into_error(self) -> Error {
        if let Some(errors) = self.errors {
            Error::Validation(ValidationError {
                message: self.message,
                error_code: self.error_code,
                errors,
            })
        } else {
            Error::Api(ApiError {
                message: self.message,
                error_code: self.error_code,
            })
        }
    }
}
