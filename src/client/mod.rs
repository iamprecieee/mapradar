use crate::cache::GeoCache;

#[cfg(feature = "python")]
use pyo3::prelude::*;

/// Client for interacting with Google Maps APIs with built-in caching.
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone)]
pub struct MapradarClient {
    api_key: String,
    http_client: reqwest::Client,
    cache: GeoCache,
}

impl MapradarClient {
    pub fn _new(api_key: String) -> Self {
        Self {
            api_key,
            http_client: reqwest::Client::new(),
            cache: GeoCache::new(),
        }
    }
}

#[cfg(feature = "python")]
pub mod bindings;
pub mod core;
