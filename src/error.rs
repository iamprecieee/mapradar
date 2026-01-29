#[cfg(feature = "python")]
use pyo3::prelude::PyErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoError {
    /// Errors related to network requests (e.g., timeout, no internet).
    #[error("API request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    /// Errors related to JSON parsing (e.g., Google changed their response format).
    #[error("JSON parsing failed: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Configuration errors (e.g., missing API key).
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Errors returned by the Google Maps API itself (e.g., INVALID_REQUEST).
    #[error("Google API error: {status} - {message}")]
    ApiError { status: String, message: String },

    /// Case where no results were found for the query.
    #[error("No results found for the given query")]
    ZeroResults,

    /// Catch-all for unexpected errors.
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl GeoError {
    pub fn json_rpc_code(&self) -> i32 {
        match self {
            GeoError::RequestError(_) => -32001, // Custom Server Error
            GeoError::ParseError(_) => -32700,   // Parse error
            GeoError::ConfigError(_) => -32002,  // Custom Server Error
            GeoError::ApiError { .. } => -32003, // Custom Server Error
            GeoError::ZeroResults => -32602,     // Invalid params (effectively)
            GeoError::Unknown(_) => -32603,      // Internal error
        }
    }
}

/// Convention to translate Rust errors into Python-native exceptions.
#[cfg(feature = "python")]
impl From<GeoError> for PyErr {
    fn from(err: GeoError) -> PyErr {
        match err {
            GeoError::ConfigError(msg) => pyo3::exceptions::PyValueError::new_err(msg),
            GeoError::ZeroResults => pyo3::exceptions::PyValueError::new_err("No results found"),
            GeoError::ApiError { status, message } => {
                pyo3::exceptions::PyRuntimeError::new_err(format!("{}: {}", status, message))
            }
            _ => pyo3::exceptions::PyRuntimeError::new_err(err.to_string()),
        }
    }
}
