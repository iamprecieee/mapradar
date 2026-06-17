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
pub mod export;
pub mod models;
pub mod scoring;
pub mod utils;

#[cfg(feature = "python")]
#[pyfunction]
fn is_within_radius(
    point_lat: f64,
    point_lng: f64,
    center_lat: f64,
    center_lng: f64,
    radius_km: f64,
) -> bool {
    utils::is_within_radius(point_lat, point_lng, center_lat, center_lng, radius_km)
}

#[cfg(feature = "python")]
#[pyfunction]
fn filter_within_radius(
    points: Vec<models::GeoPoint>,
    center_lat: f64,
    center_lng: f64,
    radius_km: f64,
) -> Vec<models::GeoPoint> {
    utils::filter_within_radius(&points, center_lat, center_lng, radius_km)
}

#[cfg(feature = "python")]
#[pyfunction]
fn sort_by_distance(
    mut points: Vec<models::GeoPoint>,
    center_lat: f64,
    center_lng: f64,
) -> Vec<models::GeoPoint> {
    utils::sort_by_distance(&mut points, center_lat, center_lng);
    points
}

#[cfg(feature = "python")]
#[pyfunction]
fn export_nearby(services: Vec<models::NearbyService>, format_str: String) -> PyResult<String> {
    let format: export::ExportFormat = format_str
        .parse()
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid format: {}", e)))?;
    Ok(export::export_nearby(&services, format))
}

#[cfg(feature = "python")]
#[pyfunction]
fn export_intelligence(
    intel: models::LocationIntelligence,
    format_str: String,
) -> PyResult<String> {
    let format: export::ExportFormat = format_str
        .parse()
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid format: {}", e)))?;
    Ok(export::export_intelligence(&intel, format))
}

#[cfg(feature = "python")]
#[pyfunction]
fn export_score(score: models::LocationScore, format_str: String) -> PyResult<String> {
    let format: export::ExportFormat = format_str
        .parse()
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Invalid format: {}", e)))?;
    Ok(export::export_score(&score, format))
}

#[cfg(feature = "python")]
#[pymodule]
fn mapradar(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<models::GeoLocation>()?;
    m.add_class::<models::TravelParameters>()?;
    m.add_class::<models::ServiceType>()?;
    m.add_class::<models::NearbyService>()?;
    m.add_class::<models::LocationIntelligence>()?;
    m.add_class::<models::SearchQuery>()?;
    m.add_class::<models::JsonRpcError>()?;
    m.add_class::<models::JsonRpcResponse>()?;
    m.add_class::<models::GeoPoint>()?;
    m.add_class::<models::CategoryScore>()?;
    m.add_class::<models::LocationScore>()?;
    m.add_class::<export::ExportFormat>()?;
    m.add_class::<client::MapradarClient>()?;

    m.add_function(wrap_pyfunction!(is_within_radius, m)?)?;
    m.add_function(wrap_pyfunction!(filter_within_radius, m)?)?;
    m.add_function(wrap_pyfunction!(sort_by_distance, m)?)?;
    m.add_function(wrap_pyfunction!(export_nearby, m)?)?;
    m.add_function(wrap_pyfunction!(export_intelligence, m)?)?;
    m.add_function(wrap_pyfunction!(export_score, m)?)?;

    Ok(())
}
