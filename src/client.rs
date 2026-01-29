use crate::{
    cache::GeoCache,
    error::GeoError,
    models::{
        GeoLocation, JsonRpcError, JsonRpcResponse, LocationIntelligence, NearbyService,
        SearchQuery, ServiceType,
    },
    utils::{calculate_distance, parse_address_components},
};

#[cfg(feature = "python")]
use pyo3::prelude::*;
use serde_json::Value;

/// Client for interacting with Google Maps APIs with built-in caching.
#[cfg_attr(feature = "python", pyclass)]
#[derive(Clone)]
pub struct MapradarClient {
    api_key: String,
    http_client: reqwest::Client,
    cache: GeoCache,
}

#[cfg(feature = "python")]
#[pymethods]
impl MapradarClient {
    #[new]
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http_client: reqwest::Client::new(),
            cache: GeoCache::new(),
        }
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
            Ok(client._to_rpc_response(id, result))
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
            Ok(client._to_rpc_response(id, result))
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
            Ok(client._to_rpc_response(id, result))
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
            Ok(client._to_rpc_response(id, result))
        })
    }
}

impl MapradarClient {
    #[cfg(not(feature = "python"))]
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http_client: reqwest::Client::new(),
            cache: GeoCache::new(),
        }
    }

    fn _to_rpc_response<T: serde::Serialize>(
        &self,
        id: String,
        result: Result<T, GeoError>,
    ) -> JsonRpcResponse {
        match result {
            Ok(data) => {
                let result_json = serde_json::to_string(&data).unwrap_or_default();
                JsonRpcResponse::new(id, Some(result_json), None)
            }
            Err(err) => {
                let rpc_err = JsonRpcError::new(err.json_rpc_code(), err.to_string(), None);
                JsonRpcResponse::new(id, None, Some(rpc_err))
            }
        }
    }

    pub async fn geocode_async(&self, address: &str) -> Result<GeoLocation, GeoError> {
        if let Some(cached) = self.cache.get_geocode(address).await {
            return Ok(cached);
        }

        let url = "https://maps.googleapis.com/maps/api/geocode/json";
        let response = self
            .http_client
            .get(url)
            .query(&[("address", address), ("key", &self.api_key)])
            .send()
            .await?;

        let data: Value = response.json().await?;
        let status = data["status"].as_str().unwrap_or("UNKNOWN");

        if status != "OK" {
            if status == "ZERO_RESULTS" {
                return Err(GeoError::ZeroResults);
            }
            return Err(GeoError::ApiError {
                status: status.to_string(),
                message: data["error_message"]
                    .as_str()
                    .unwrap_or("Geocoding failed")
                    .to_string(),
            });
        }

        let result = &data["results"][0];
        let geometry = &result["geometry"]["location"];
        let (city, state, country) = parse_address_components(&result["address_components"])?;

        let location = GeoLocation {
            address: result["formatted_address"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            latitude: geometry["lat"].as_f64().unwrap_or_default(),
            longitude: geometry["lng"].as_f64().unwrap_or_default(),
            city,
            state,
            country,
        };

        self.cache.set_geocode(address, location.clone()).await;
        Ok(location)
    }

    pub async fn reverse_geocode_async(&self, lat: f64, lng: f64) -> Result<GeoLocation, GeoError> {
        if let Some(cached) = self.cache.get_reverse_geocode(lat, lng).await {
            return Ok(cached);
        }

        let url = "https://maps.googleapis.com/maps/api/geocode/json";
        let response = self
            .http_client
            .get(url)
            .query(&[
                ("latlng", format!("{},{}", lat, lng)),
                ("key", self.api_key.clone()),
            ])
            .send()
            .await?;

        let data: Value = response.json().await?;
        let status = data["status"].as_str().unwrap_or("UNKNOWN");

        if status != "OK" {
            if status == "ZERO_RESULTS" {
                return Err(GeoError::ZeroResults);
            }
            return Err(GeoError::ApiError {
                status: status.to_string(),
                message: data["error_message"]
                    .as_str()
                    .unwrap_or("Reverse geocoding failed")
                    .to_string(),
            });
        }

        let result = &data["results"][0];
        let geometry = &result["geometry"]["location"];
        let (city, state, country) = parse_address_components(&result["address_components"])?;

        let location = GeoLocation {
            address: result["formatted_address"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            latitude: geometry["lat"].as_f64().unwrap_or_default(),
            longitude: geometry["lng"].as_f64().unwrap_or_default(),
            city,
            state,
            country,
        };

        self.cache
            .set_reverse_geocode(lat, lng, location.clone())
            .await;
        Ok(location)
    }

    pub async fn search_nearby_async(
        &self,
        lat: f64,
        lng: f64,
        service_type: ServiceType,
        radius_meters: f64,
        max_results: usize,
    ) -> Result<Vec<NearbyService>, GeoError> {
        if let Some(cached) = self
            .cache
            .get_nearby(lat, lng, service_type, radius_meters)
            .await
        {
            return Ok(cached.into_iter().take(max_results).collect());
        }

        let url = "https://maps.googleapis.com/maps/api/place/nearbysearch/json";
        let google_type = match service_type {
            ServiceType::BusStop => "bus_station",
            ServiceType::Market => "supermarket",
            ServiceType::School => "school",
            ServiceType::Mall => "shopping_mall",
            ServiceType::Hospital => "hospital",
            ServiceType::Bank => "bank",
            ServiceType::Restaurant => "restaurant",
            ServiceType::FuelStation => "gas_station",
            ServiceType::TrainStation => "train_station",
            ServiceType::TaxiStand => "taxi_stand",
            ServiceType::Landmark => "tourist_attraction",
        };

        let response = self
            .http_client
            .get(url)
            .query(&[
                ("location", format!("{},{}", lat, lng)),
                ("radius", radius_meters.to_string()),
                ("type", google_type.to_string()),
                ("key", self.api_key.clone()),
            ])
            .send()
            .await?;

        let data: Value = response.json().await?;
        let status = data["status"].as_str().unwrap_or("UNKNOWN");

        if status != "OK" && status != "ZERO_RESULTS" {
            return Err(GeoError::ApiError {
                status: status.to_string(),
                message: data["error_message"]
                    .as_str()
                    .unwrap_or("Places API search failed")
                    .to_string(),
            });
        }

        let mut services = Vec::new();
        if let Some(results) = data["results"].as_array() {
            for place in results.iter().take(max_results) {
                let loc = &place["geometry"]["location"];
                let p_lat = loc["lat"].as_f64().unwrap_or_default();
                let p_lng = loc["lng"].as_f64().unwrap_or_default();

                services.push(NearbyService {
                    name: place["name"].as_str().unwrap_or("Unknown").to_string(),
                    service_type,
                    latitude: p_lat,
                    longitude: p_lng,
                    distance_km: calculate_distance(lat, lng, p_lat, p_lng),
                    address: place
                        .get("vicinity")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    rating: place
                        .get("rating")
                        .and_then(|r| r.as_f64())
                        .map(|f| f as f32),
                    place_id: place
                        .get("place_id")
                        .and_then(|p| p.as_str())
                        .map(|s| s.to_string()),
                });
            }
        }

        self.cache
            .set_nearby(lat, lng, service_type, radius_meters, services.clone())
            .await;
        Ok(services)
    }

    pub async fn fetch_intelligence_async(
        &self,
        query: SearchQuery,
        service_types: Vec<ServiceType>,
        radius_km: f64,
        max_results_per_type: usize,
    ) -> Result<LocationIntelligence, GeoError> {
        let location = match query {
            SearchQuery::Address { address } => self.geocode_async(&address).await?,
            SearchQuery::Coordinates {
                latitude,
                longitude,
            } => self.reverse_geocode_async(latitude, longitude).await?,
        };

        let radius_meters = radius_km * 1000.0;
        let mut futures = Vec::new();

        for &service_type in &service_types {
            futures.push(self.search_nearby_async(
                location.latitude,
                location.longitude,
                service_type,
                radius_meters,
                max_results_per_type,
            ));
        }

        let results = futures::future::join_all(futures).await;

        let mut all_services = Vec::new();
        for services in results.into_iter().flatten() {
            all_services.extend(services);
        }

        all_services.sort_by(|a, b| {
            a.distance_km
                .partial_cmp(&b.distance_km)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(LocationIntelligence::new(location, all_services))
    }
}
