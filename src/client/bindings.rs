use crate::models::{SearchQuery, ServiceType};

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymethods]
impl super::MapradarClient {
    #[new]
    pub fn new(api_key: String) -> Self {
        Self::_new(api_key)
    }

    /// Converts an address string into a geographic location.
    pub fn geocode<'py>(&self, py: Python<'py>, address: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let location = client.geocode_async(&address).await?;
            Ok(location)
        })
    }

    /// Converts geographic coordinates into a human-readable address.
    pub fn reverse_geocode<'py>(
        &self,
        py: Python<'py>,
        latitude: f64,
        longitude: f64,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let location = client.reverse_geocode_async(latitude, longitude).await?;
            Ok(location)
        })
    }

    /// Searches for nearby amenities of a specific type.
    pub fn search_nearby<'py>(
        &self,
        py: Python<'py>,
        lat: f64,
        lng: f64,
        service_type: ServiceType,
        radius_meters: f64,
        max_results: usize,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let services = client
                .search_nearby_async(lat, lng, service_type, radius_meters, max_results)
                .await?;
            Ok(services)
        })
    }

    /// Fetches comprehensive location intelligence, including multiple types of amenities in parallel.
    #[pyo3(signature = (query, service_types, radius_km=5.0, max_results_per_type=5))]
    pub fn fetch_intelligence<'py>(
        &self,
        py: Python<'py>,
        query: SearchQuery,
        service_types: Vec<ServiceType>,
        radius_km: f64,
        max_results_per_type: usize,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let intel = client
                .fetch_intelligence_async(query, service_types, radius_km, max_results_per_type)
                .await?;
            Ok(intel)
        })
    }

    /// Fetches geocode information in JSON-RPC 2.0 format.
    #[pyo3(signature = (address, id="1".to_string()))]
    pub fn geocode_rpc<'py>(
        &self,
        py: Python<'py>,
        address: String,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.geocode_async(&address).await;
            Ok(client.rpc_response(id, result))
        })
    }

    /// Fetches reverse geocode information in JSON-RPC 2.0 format.
    #[pyo3(signature = (latitude, longitude, id="1".to_string()))]
    pub fn reverse_geocode_rpc<'py>(
        &self,
        py: Python<'py>,
        latitude: f64,
        longitude: f64,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client.reverse_geocode_async(latitude, longitude).await;
            Ok(client.rpc_response(id, result))
        })
    }

    /// Fetches nearby amenities in JSON-RPC 2.0 format.
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (lat, lng, service_type, radius_meters, max_results, id="1".to_string()))]
    pub fn search_nearby_rpc<'py>(
        &self,
        py: Python<'py>,
        lat: f64,
        lng: f64,
        service_type: ServiceType,
        radius_meters: f64,
        max_results: usize,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client
                .search_nearby_async(lat, lng, service_type, radius_meters, max_results)
                .await;
            Ok(client.rpc_response(id, result))
        })
    }

    /// Fetches comprehensive location intelligence in JSON-RPC 2.0 format.
    #[pyo3(signature = (query, service_types, radius_km=5.0, max_results_per_type=5, id="1".to_string()))]
    pub fn fetch_intelligence_rpc<'py>(
        &self,
        py: Python<'py>,
        query: SearchQuery,
        service_types: Vec<ServiceType>,
        radius_km: f64,
        max_results_per_type: usize,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let result = client
                .fetch_intelligence_async(query, service_types, radius_km, max_results_per_type)
                .await;
            Ok(client.rpc_response(id, result))
        })
    }
}
