use crate::{
    error::GeoError,
    models::{
        GeoLocation, JsonRpcError, JsonRpcResponse, LocationIntelligence, NearbyService,
        SearchQuery, ServiceType, TravelParameters,
    },
    utils::{calculate_distance, parse_address_components},
};

use serde_json::Value;

impl super::MapradarClient {
    #[cfg(not(feature = "python"))]
    pub fn new(api_key: String) -> Self {
        Self::_new(api_key)
    }

    pub fn rpc_response<T: serde::Serialize>(
        &self,
        id: String,
        result: Result<T, GeoError>,
    ) -> JsonRpcResponse {
        match result {
            Ok(data) => match serde_json::to_value(&data) {
                Ok(result_val) => JsonRpcResponse::new(id, Some(result_val), None),
                Err(err) => {
                    let geo_err = GeoError::ParseError(err);
                    let rpc_err =
                        JsonRpcError::new(geo_err.json_rpc_code(), geo_err.to_string(), None);
                    JsonRpcResponse::new(id, None, Some(rpc_err))
                }
            },
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
            .get_nearby(lat, lng, service_type, radius_meters, max_results)
            .await
        {
            return Ok(cached.into_iter().take(max_results).collect());
        }

        let url = "https://places.googleapis.com/v1/places:searchNearby";
        let google_type = service_type.google_place_type();

        let body = serde_json::json!({
            "includedTypes": [google_type],
            "maxResultCount": max_results,
            "locationRestriction": {
                "circle": {
                    "center": {
                        "latitude": lat,
                        "longitude": lng
                    },
                    "radius": radius_meters
                }
            }
        });

        let response = self
            .http_client
            .post(url)
            .header("X-Goog-Api-Key", &self.api_key)
            .header("X-Goog-FieldMask", "places.displayName.text,places.location.latitude,places.location.longitude,places.formattedAddress,places.rating,places.id,places.nationalPhoneNumber,places.regularOpeningHours.openNow")
            .json(&body)
            .send()
            .await?;

        let data: Value = response.json().await?;

        if let Some(error) = data.get("error") {
            return Err(GeoError::ApiError {
                status: error["status"].as_str().unwrap_or("UNKNOWN").to_string(),
                message: error["message"]
                    .as_str()
                    .unwrap_or("Places API search failed")
                    .to_string(),
            });
        }

        let mut services = Vec::new();
        if let Some(places) = data
            .get("places")
            .and_then(|place_list| place_list.as_array())
        {
            for place in places.iter().take(max_results) {
                let loc = &place["location"];
                let p_lat = loc["latitude"].as_f64().unwrap_or_default();
                let p_lng = loc["longitude"].as_f64().unwrap_or_default();

                services.push(NearbyService {
                    name: place
                        .get("displayName")
                        .and_then(|display_name| display_name.get("text"))
                        .and_then(|text_val| text_val.as_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    service_type,
                    latitude: p_lat,
                    longitude: p_lng,
                    distance_km: calculate_distance(lat, lng, p_lat, p_lng),
                    address: place
                        .get("formattedAddress")
                        .and_then(|addr_val| addr_val.as_str())
                        .map(|addr_str| addr_str.to_string()),
                    rating: place
                        .get("rating")
                        .and_then(|rating_val| rating_val.as_f64())
                        .map(|float_val| float_val as f32),
                    place_id: place
                        .get("id")
                        .and_then(|id_val| id_val.as_str())
                        .map(|id_str| id_str.to_string()),
                    phone_number: place
                        .get("nationalPhoneNumber")
                        .and_then(|phone_val| phone_val.as_str())
                        .map(|phone_str| phone_str.to_string()),
                    open_now: place
                        .get("regularOpeningHours")
                        .and_then(|hours_val| hours_val.get("openNow"))
                        .and_then(|open_val| open_val.as_bool()),
                });
            }
        }

        self.cache
            .set_nearby(
                lat,
                lng,
                service_type,
                radius_meters,
                max_results,
                services.clone(),
            )
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
        let mut failed_lookups = Vec::new();

        for (&service_type, result) in service_types.iter().zip(results) {
            match result {
                Ok(services) => all_services.extend(services),
                Err(err) => failed_lookups.push(format!("{}: {}", service_type.slug(), err)),
            }
        }

        // Nothing usable came back, so the failure is the result. Otherwise the
        // partial result is returned with the failures recorded on it, so
        // callers can tell the picture is incomplete.
        if all_services.is_empty() && !failed_lookups.is_empty() {
            return Err(GeoError::ApiError {
                status: "NEARBY_SEARCH_FAILED".to_string(),
                message: failed_lookups.join("; "),
            });
        }

        all_services.sort_by(|service_a, service_b| {
            service_a
                .distance_km
                .partial_cmp(&service_b.distance_km)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(LocationIntelligence::new(location, all_services).with_failed_lookups(failed_lookups))
    }

    pub async fn calculate_travel_distance_async(
        &self,
        travel_distance_params: TravelParameters,
    ) -> Result<f64, GeoError> {
        let (origin_latitude, origin_longitude) = match (
            travel_distance_params.origin_latitude,
            travel_distance_params.origin_longitude,
        ) {
            (Some(lat), Some(lng)) => (lat, lng),
            (None, None) => {
                if let Some(origin_addr) = travel_distance_params.origin_address {
                    let location = self.geocode_async(&origin_addr).await?;
                    (location.latitude, location.longitude)
                } else {
                    return Err(GeoError::ApiError {
                        status: "INVALID_ORIGIN_PARAMETERS".to_string(),
                        message: "Origin address or coordinates are required".to_string(),
                    });
                }
            }
            _ => {
                return Err(GeoError::ApiError {
                    status: "INVALID_ORIGIN_PARAMETERS".to_string(),
                    message:
                        "Both origin latitude and longitude are required when using coordinates"
                            .to_string(),
                });
            }
        };

        let (destination_latitude, destination_longitude) = match (
            travel_distance_params.destination_latitude,
            travel_distance_params.destination_longitude,
        ) {
            (Some(lat), Some(lng)) => (lat, lng),
            (None, None) => {
                if let Some(destination_addr) = travel_distance_params.destination_address {
                    let location = self.geocode_async(&destination_addr).await?;
                    (location.latitude, location.longitude)
                } else {
                    return Err(GeoError::ApiError {
                        status: "INVALID_DESTINATION_PARAMETERS".to_string(),
                        message: "Destination address or coordinates are required".to_string(),
                    });
                }
            }
            _ => {
                return Err(GeoError::ApiError {
                    status: "INVALID_DESTINATION_PARAMETERS".to_string(),
                    message: "Both destination latitude and longitude are required when using coordinates".to_string(),
                });
            }
        };

        let url = "https://routes.googleapis.com/directions/v2:computeRoutes";

        let travel_mode_api = match travel_distance_params.travel_mode.as_deref() {
            Some(mode) => crate::utils::resolve_travel_mode(mode)?,
            None => "DRIVE",
        };

        let body = serde_json::json!({
            "origin": {
                "location": {
                    "latLng": {
                        "latitude": origin_latitude,
                        "longitude": origin_longitude
                    }
                }
            },
            "destination": {
                "location": {
                    "latLng": {
                        "latitude": destination_latitude,
                        "longitude": destination_longitude
                    }
                }
            },
            "travelMode": travel_mode_api
        });

        let response = self
            .http_client
            .post(url)
            .header("X-Goog-Api-Key", &self.api_key)
            .header("X-Goog-FieldMask", "routes.distanceMeters")
            .json(&body)
            .send()
            .await?;

        let data: Value = response.json().await?;

        if let Some(error) = data.get("error") {
            return Err(GeoError::ApiError {
                status: error["status"].as_str().unwrap_or("UNKNOWN").to_string(),
                message: error["message"]
                    .as_str()
                    .unwrap_or("Routes API failed")
                    .to_string(),
            });
        }

        if let Some(routes) = data
            .get("routes")
            .and_then(|route_list| route_list.as_array())
            && let Some(route) = routes.first()
            && let Some(distance) = route.get("distanceMeters").and_then(|dist_val| {
                dist_val
                    .as_f64()
                    .or_else(|| dist_val.as_u64().map(|unsigned_val| unsigned_val as f64))
            })
        {
            return Ok(distance / 1000.0);
        }

        Err(GeoError::Unknown(
            "Could not compute travel distance between these locations".to_string(),
        ))
    }

    pub async fn score_location_async(
        &self,
        query: SearchQuery,
        radius_km: f64,
    ) -> Result<crate::models::LocationScore, GeoError> {
        let all_types = vec![
            ServiceType::Bank,
            ServiceType::Hospital,
            ServiceType::School,
            ServiceType::Restaurant,
            ServiceType::Market,
            ServiceType::Mall,
            ServiceType::Pharmacy,
            ServiceType::BusStop,
            ServiceType::FuelStation,
        ];

        let intel = self
            .fetch_intelligence_async(query, all_types, radius_km, 20)
            .await?;

        Ok(crate::scoring::compute_location_score(&intel, radius_km))
    }
}
