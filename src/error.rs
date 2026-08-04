#[cfg(feature = "python")]
use pyo3::prelude::PyErr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeoError {
    /// Errors related to network requests (e.g., timeout, no internet).
    #[error("API request failed: {0}")]
    RequestError(reqwest::Error),

    /// Errors related to JSON parsing (e.g., Google changed their response format).
    #[error("JSON parsing failed: {0}")]
    ParseError(#[from] serde_json::Error),

    /// Errors returned by the Google Maps API itself (e.g., INVALID_REQUEST).
    #[error("Google API error: {status} - {message}")]
    ApiError { status: String, message: String },

    /// Case where no results were found for the query.
    #[error("No results found for the given query")]
    ZeroResults,

    /// Caller-supplied input that could not be understood (e.g. an unknown
    /// travel mode or service type).
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Catch-all for unexpected errors.
    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl GeoError {
    pub fn json_rpc_code(&self) -> i32 {
        match self {
            GeoError::RequestError(_) => -32001, // Custom Server Error
            GeoError::ParseError(_) => -32700,   // Parse error
            GeoError::ApiError { .. } => -32003, // Custom Server Error
            GeoError::ZeroResults => -32602,     // Invalid params (effectively)
            GeoError::InvalidInput(_) => -32602, // Invalid params
            GeoError::Unknown(_) => -32603,      // Internal error
        }
    }
}

/// The geocoding endpoints pass the API key as a `key=` query parameter, and
/// `reqwest::Error` includes the request URL in its `Display` output. Stripping
/// the URL here keeps the key out of error messages, which reach terminals,
/// log files and bug reports.
impl From<reqwest::Error> for GeoError {
    fn from(err: reqwest::Error) -> Self {
        GeoError::RequestError(err.without_url())
    }
}

/// Convention to translate Rust errors into Python-native exceptions.
#[cfg(feature = "python")]
impl From<GeoError> for PyErr {
    fn from(err: GeoError) -> PyErr {
        match err {
            GeoError::ZeroResults => pyo3::exceptions::PyValueError::new_err("No results found"),
            GeoError::InvalidInput(msg) => pyo3::exceptions::PyValueError::new_err(msg),
            GeoError::ApiError { status, message } => {
                pyo3::exceptions::PyRuntimeError::new_err(format!("{}: {}", status, message))
            }
            _ => pyo3::exceptions::PyRuntimeError::new_err(err.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn request_errors_do_not_expose_the_api_key() {
        let api_key = "test-key-that-must-not-leak";
        // Port 1 refuses the connection, so this fails without a live server.
        let url = format!("http://127.0.0.1:1/maps/api/geocode/json?key={}", api_key);

        let reqwest_err = reqwest::Client::new()
            .get(&url)
            .send()
            .await
            .expect_err("a request to a closed port must fail");

        let geo_err: GeoError = reqwest_err.into();
        let message = geo_err.to_string();

        assert!(
            !message.contains(api_key),
            "error message leaked the API key: {}",
            message
        );
    }
}
