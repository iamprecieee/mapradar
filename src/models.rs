use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a geographic location.
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoLocation {
    #[pyo3(get, set)]
    pub address: String,
    #[pyo3(get, set)]
    pub latitude: f64,
    #[pyo3(get, set)]
    pub longitude: f64,
    #[pyo3(get, set)]
    pub city: Option<String>,
    #[pyo3(get, set)]
    pub state: Option<String>,
    #[pyo3(get, set)]
    pub country: String,
}

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
#[pyclass(eq, eq_int)]
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
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NearbyService {
    #[pyo3(get, set)]
    pub name: String,
    #[pyo3(get, set)]
    pub service_type: ServiceType,
    #[pyo3(get, set)]
    pub latitude: f64,
    #[pyo3(get, set)]
    pub longitude: f64,
    #[pyo3(get, set)]
    pub distance_km: f64,
    #[pyo3(get, set)]
    pub address: Option<String>,
    #[pyo3(get, set)]
    pub rating: Option<f32>,
    #[pyo3(get, set)]
    pub place_id: Option<String>,
}

/// Comprehensive intelligence about a location.
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationIntelligence {
    #[pyo3(get, set)]
    pub location: GeoLocation,
    #[pyo3(get, set)]
    pub nearby_services: Vec<NearbyService>,
    #[pyo3(get, set)]
    pub total_services_found: usize,
}

#[pymethods]
impl LocationIntelligence {
    #[new]
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
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchQuery {
    Address { address: String },
    Coordinates { latitude: f64, longitude: f64 },
}

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

/// Represents a JSON-RPC 2.0 error object.
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    #[pyo3(get, set)]
    pub code: i32,
    #[pyo3(get, set)]
    pub message: String,
    #[pyo3(get, set)]
    pub data: Option<String>,
}

#[pymethods]
impl JsonRpcError {
    #[new]
    #[pyo3(signature = (code, message, data=None))]
    pub fn new(code: i32, message: String, data: Option<String>) -> Self {
        Self {
            code,
            message,
            data,
        }
    }
}

/// Represents a JSON-RPC 2.0 response wrapper.
#[pyclass]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    #[pyo3(get, set)]
    pub jsonrpc: String,
    #[pyo3(get, set)]
    pub result: Option<String>,
    #[pyo3(get, set)]
    pub error: Option<JsonRpcError>,
    #[pyo3(get, set)]
    pub id: String,
}

#[pymethods]
impl JsonRpcResponse {
    #[new]
    #[pyo3(signature = (id, result=None, error=None))]
    pub fn new(id: String, result: Option<String>, error: Option<JsonRpcError>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result,
            error,
            id,
        }
    }

    /// Converts the response to a JSON string.
    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}
