#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a geographic location.
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    pub address: String,
    pub latitude: f64,
    pub longitude: f64,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: String,
}

#[cfg(feature = "python")]
#[pymethods]
impl GeoLocation {
    /// Returns a string representation for debugging in Python.
    fn __repr__(&self) -> String {
        format!(
            "Location(address='{}', lat={}, lon={})",
            self.address, self.latitude, self.longitude
        )
    }
}

/// Supported amenity types for nearby search.
#[cfg_attr(feature = "python", pyclass(eq, eq_int))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceType {
    BusStop,
    Market,
    School,
    Mall,
    Hospital,
    Bank,
    Restaurant,
    FuelStation,
    TrainStation,
    TaxiStand,
    Landmark,
}

/// Represents a specific amenity found near a location.
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearbyService {
    pub name: String,
    pub service_type: ServiceType,
    pub latitude: f64,
    pub longitude: f64,
    pub distance_km: f64,
    pub address: Option<String>,
    pub rating: Option<f32>,
    pub place_id: Option<String>,
}

/// Comprehensive intelligence about a location.
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationIntelligence {
    pub location: GeoLocation,
    pub nearby_services: Vec<NearbyService>,
    pub total_services_found: usize,
}

#[cfg(feature = "python")]
#[pymethods]
impl LocationIntelligence {
    #[new]
    pub fn py_new(location: GeoLocation, nearby_services: Vec<NearbyService>) -> Self {
        Self::new(location, nearby_services)
    }
}

impl LocationIntelligence {
    pub fn new(location: GeoLocation, nearby_services: Vec<NearbyService>) -> Self {
        let total = nearby_services.len();
        Self {
            location,
            nearby_services,
            total_services_found: total,
        }
    }
}

/// Represents a search query, either by address or coordinates.
#[cfg_attr(feature = "python", pyclass)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchQuery {
    Address { address: String },
    Coordinates { latitude: f64, longitude: f64 },
}

#[cfg(feature = "python")]
#[pymethods]
impl SearchQuery {
    #[staticmethod]
    pub fn from_address(address: String) -> Self {
        Self::Address { address }
    }

    #[staticmethod]
    pub fn from_coordinates(latitude: f64, longitude: f64) -> Self {
        Self::Coordinates {
            latitude,
            longitude,
        }
    }
}

#[cfg(not(feature = "python"))]
impl SearchQuery {
    pub fn from_address(address: String) -> Self {
        Self::Address { address }
    }

    pub fn from_coordinates(latitude: f64, longitude: f64) -> Self {
        Self::Coordinates {
            latitude,
            longitude,
        }
    }
}

/// Represents a JSON-RPC 2.0 error object.
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl JsonRpcError {
    #[new]
    #[pyo3(signature = (code, message, data=None))]
    pub fn py_new(code: i32, message: String, data: Option<String>) -> Self {
        Self::new(code, message, data)
    }
}

impl JsonRpcError {
    pub fn new(code: i32, message: String, data: Option<String>) -> Self {
        Self {
            code,
            message,
            data,
        }
    }
}

/// Represents a JSON-RPC 2.0 response wrapper.
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<String>,
    pub error: Option<JsonRpcError>,
    pub id: String,
}

#[cfg(feature = "python")]
#[pymethods]
impl JsonRpcResponse {
    #[new]
    #[pyo3(signature = (id, result=None, error=None))]
    pub fn py_new(id: String, result: Option<String>, error: Option<JsonRpcError>) -> Self {
        Self::new(id, result, error)
    }

    /// Converts the response to a JSON string.
    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

impl JsonRpcResponse {
    pub fn new(id: String, result: Option<String>, error: Option<JsonRpcError>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result,
            error,
            id,
        }
    }

    #[cfg(not(feature = "python"))]
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}
