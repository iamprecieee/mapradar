//! # Mapradar
//!
//! Fast geocoding and nearby places search.
//!
//! ## Features
//!
//! - **Geocoding** - Convert addresses to coordinates
//! - **Reverse Geocoding** - Convert coordinates to addresses
//! - **Nearby Search** - Find banks, hospitals, schools, and more
//! - **Caching** - Automatic in-memory cache for repeated queries
//! - **JSON-RPC 2.0** - Built-in response format for microservices
//!
//! ## Example
//!
//! ```rust,ignore
//! use mapradar::client::MapradarClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = MapradarClient::new("YOUR_API_KEY".to_string());
//!     let location = client.geocode_async("Times Square, NYC").await?;
//!     println!("{}, {}", location.latitude, location.longitude);
//!     Ok(())
//! }
//! ```

#[cfg(feature = "python")]
use pyo3::prelude::*;

pub mod cache;
pub mod client;
pub mod error;
pub mod models;
pub mod utils;

#[cfg(feature = "python")]
#[pymodule]
fn mapradar(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<models::GeoLocation>()?;
    m.add_class::<models::ServiceType>()?;
    m.add_class::<models::NearbyService>()?;
    m.add_class::<models::LocationIntelligence>()?;
    m.add_class::<models::SearchQuery>()?;
    m.add_class::<models::JsonRpcError>()?;
    m.add_class::<models::JsonRpcResponse>()?;
    m.add_class::<client::MapradarClient>()?;
    Ok(())
}
