use crate::cache::GeoCache;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Maximum time to wait for a full request/response cycle against a Google API.
const REQUEST_TIMEOUT_SECS: u64 = 30;

/// Maximum time to wait for the TCP/TLS connection to be established.
const CONNECT_TIMEOUT_SECS: u64 = 10;

/// Client for interacting with Google Maps APIs with built-in caching.
#[cfg_attr(feature = "python", pyclass(from_py_object))]
#[derive(Clone)]
pub struct MapradarClient {
    api_key: String,
    http_client: reqwest::Client,
    cache: GeoCache,
}

impl MapradarClient {
    /// Shared constructor behind the Rust and Python entry points.
    pub(crate) fn build(api_key: String) -> Self {
        // Without an explicit timeout a stalled connection hangs the caller
        // indefinitely, which strands CLI invocations and blocks the Python
        // event loop.
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .connect_timeout(std::time::Duration::from_secs(CONNECT_TIMEOUT_SECS))
            .build()
            .expect("HTTP client construction requires a working TLS backend");

        Self {
            api_key,
            http_client,
            cache: GeoCache::new(),
        }
    }
}

#[cfg(feature = "python")]
pub mod bindings;
pub mod core;
