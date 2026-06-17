#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// Represents a geographic location.
#[cfg_attr(feature = "python", pyclass(get_all, set_all, from_py_object))]
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

/// Represents travel parameters for distance calculation.
#[cfg_attr(feature = "python", pyclass(get_all, set_all, from_py_object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TravelParameters {
    pub origin_latitude: Option<f64>,
    pub origin_longitude: Option<f64>,
    pub origin_address: Option<String>,
    pub destination_latitude: Option<f64>,
    pub destination_longitude: Option<f64>,
    pub destination_address: Option<String>,
    pub travel_mode: Option<String>,
}

#[cfg(feature = "python")]
#[pymethods]
impl TravelParameters {
    #[new]
    #[pyo3(signature = (
        origin_latitude=None,
        origin_longitude=None,
        origin_address=None,
        destination_latitude=None,
        destination_longitude=None,
        destination_address=None,
        travel_mode=None,
    ))]
    pub fn py_new(
        origin_latitude: Option<f64>,
        origin_longitude: Option<f64>,
        origin_address: Option<String>,
        destination_latitude: Option<f64>,
        destination_longitude: Option<f64>,
        destination_address: Option<String>,
        travel_mode: Option<String>,
    ) -> Self {
        Self {
            origin_latitude,
            origin_longitude,
            origin_address,
            destination_latitude,
            destination_longitude,
            destination_address,
            travel_mode,
        }
    }
}

/// Supported amenity types for nearby search.
#[cfg_attr(feature = "python", pyclass(eq, eq_int, from_py_object))]
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
    Pharmacy,
}

/// Represents a specific amenity found near a location.
#[cfg_attr(feature = "python", pyclass(get_all, set_all, from_py_object))]
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
    pub phone_number: Option<String>,
    pub open_now: Option<bool>,
}

/// Comprehensive intelligence about a location.
#[cfg_attr(feature = "python", pyclass(get_all, set_all, from_py_object))]
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
#[cfg_attr(feature = "python", pyclass(from_py_object))]
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
#[cfg_attr(feature = "python", pyclass(get_all, set_all, from_py_object))]
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
#[cfg_attr(feature = "python", pyclass(from_py_object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
    pub id: String,
}

#[cfg(feature = "python")]
#[pymethods]
impl JsonRpcResponse {
    #[new]
    #[pyo3(signature = (id, result=None, error=None))]
    pub fn py_new(id: String, result: Option<String>, error: Option<JsonRpcError>) -> Self {
        let result_val = result.and_then(|r| serde_json::from_str(&r).ok());
        Self::new(id, result_val, error)
    }

    #[getter]
    pub fn get_jsonrpc(&self) -> String {
        self.jsonrpc.clone()
    }

    #[setter]
    pub fn set_jsonrpc(&mut self, value: String) {
        self.jsonrpc = value;
    }

    #[getter]
    pub fn get_id(&self) -> String {
        self.id.clone()
    }

    #[setter]
    pub fn set_id(&mut self, value: String) {
        self.id = value;
    }

    #[getter]
    pub fn get_error(&self) -> Option<JsonRpcError> {
        self.error.clone()
    }

    #[setter]
    pub fn set_error(&mut self, value: Option<JsonRpcError>) {
        self.error = value;
    }

    #[getter]
    pub fn get_result(&self) -> Option<String> {
        self.result
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap_or_default())
    }

    #[setter]
    pub fn set_result(&mut self, value: Option<String>) {
        self.result = value.and_then(|v| serde_json::from_str(&v).ok());
    }

    /// Converts the response to a JSON string.
    pub fn to_json(&self) -> PyResult<String> {
        serde_json::to_string(self)
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))
    }
}

impl JsonRpcResponse {
    pub fn new(id: String, result: Option<serde_json::Value>, error: Option<JsonRpcError>) -> Self {
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

/// Represents a geographic point with an optional label and distance, useful for geofencing.
#[cfg_attr(feature = "python", pyclass(get_all, set_all, from_py_object))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPoint {
    pub latitude: f64,
    pub longitude: f64,
    pub label: Option<String>,
    pub distance_km: Option<f64>,
}

#[cfg(feature = "python")]
#[pymethods]
impl GeoPoint {
    #[new]
    #[pyo3(signature = (latitude, longitude, label=None, distance_km=None))]
    pub fn py_new(latitude: f64, longitude: f64, label: Option<String>, distance_km: Option<f64>) -> Self {
        Self {
            latitude,
            longitude,
            label,
            distance_km,
        }
    }

    fn __repr__(&self) -> String {
        let label_str = self.label.as_deref().unwrap_or("Unnamed");
        if let Some(dist) = self.distance_km {
            format!("GeoPoint(label='{}', lat={}, lon={}, distance_km={:.2})", label_str, self.latitude, self.longitude, dist)
        } else {
            format!("GeoPoint(label='{}', lat={}, lon={})", label_str, self.latitude, self.longitude)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jsonrpc_serialization() {
        let loc = GeoLocation {
            address: "123 Main St".to_string(),
            latitude: 10.0,
            longitude: 20.0,
            city: None,
            state: None,
            country: "US".to_string(),
        };
        let result_val = serde_json::to_value(&loc).unwrap();
        let rpc = JsonRpcResponse::new("test-123".to_string(), Some(result_val), None);
        let json_str = serde_json::to_string(&rpc).unwrap();

        // Assert that the result contains actual JSON and not stringified JSON (no double encoding)
        assert!(json_str.contains(r#""result":{"address":"123 Main St""#));
        // Ensure there's no double quoting: "result":"{\"address\":...}"
        assert!(!json_str.contains(r#""result":"{"#));
    }
}
