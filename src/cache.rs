use moka::future::Cache;
use std::time::Duration;

use crate::models::{GeoLocation, NearbyService, ServiceType};

const GEOCODE_TTL_SECS: u64 = 3600;
const PLACES_TTL_SECS: u64 = 900;
const MAX_GEOCODE_ENTRIES: u64 = 10_000;
const MAX_PLACES_ENTRIES: u64 = 50_000;

#[derive(Clone)]
pub struct GeoCache {
    geocode: Cache<String, GeoLocation>,
    reverse_geocode: Cache<String, GeoLocation>,
    nearby: Cache<String, Vec<NearbyService>>,
}

impl Default for GeoCache {
    fn default() -> Self {
        Self::new()
    }
}

impl GeoCache {
    pub fn new() -> Self {
        Self {
            geocode: Cache::builder()
                .max_capacity(MAX_GEOCODE_ENTRIES)
                .time_to_live(Duration::from_secs(GEOCODE_TTL_SECS))
                .build(),
            reverse_geocode: Cache::builder()
                .max_capacity(MAX_GEOCODE_ENTRIES)
                .time_to_live(Duration::from_secs(GEOCODE_TTL_SECS))
                .build(),
            nearby: Cache::builder()
                .max_capacity(MAX_PLACES_ENTRIES)
                .time_to_live(Duration::from_secs(PLACES_TTL_SECS))
                .build(),
        }
    }

    /// Generates cache key for geocoding requests.
    fn geocode_key(address: &str) -> String {
        address.to_lowercase().trim().to_string()
    }

    /// Generates cache key for reverse geocoding requests.
    fn reverse_geocode_key(lat: f64, lng: f64) -> String {
        format!("{:.6},{:.6}", lat, lng)
    }

    /// Generates cache key for nearby search requests.
    fn nearby_key(lat: f64, lng: f64, service_type: ServiceType, radius_meters: f64) -> String {
        format!(
            "{:.4},{:.4}:{:?}:{:.0}",
            lat, lng, service_type, radius_meters
        )
    }

    /// Gets cached geocode result.
    pub async fn get_geocode(&self, address: &str) -> Option<GeoLocation> {
        self.geocode.get(&Self::geocode_key(address)).await
    }

    /// Stores geocode result in cache.
    pub async fn set_geocode(&self, address: &str, location: GeoLocation) {
        self.geocode
            .insert(Self::geocode_key(address), location)
            .await;
    }

    /// Gets cached reverse geocode result.
    pub async fn get_reverse_geocode(&self, lat: f64, lng: f64) -> Option<GeoLocation> {
        self.reverse_geocode
            .get(&Self::reverse_geocode_key(lat, lng))
            .await
    }

    /// Stores reverse geocode result in cache.
    pub async fn set_reverse_geocode(&self, lat: f64, lng: f64, location: GeoLocation) {
        self.reverse_geocode
            .insert(Self::reverse_geocode_key(lat, lng), location)
            .await;
    }

    /// Gets cached nearby search result.
    pub async fn get_nearby(
        &self,
        lat: f64,
        lng: f64,
        service_type: ServiceType,
        radius_meters: f64,
    ) -> Option<Vec<NearbyService>> {
        self.nearby
            .get(&Self::nearby_key(lat, lng, service_type, radius_meters))
            .await
    }

    /// Stores nearby search result in cache.
    pub async fn set_nearby(
        &self,
        lat: f64,
        lng: f64,
        service_type: ServiceType,
        radius_meters: f64,
        services: Vec<NearbyService>,
    ) {
        self.nearby
            .insert(
                Self::nearby_key(lat, lng, service_type, radius_meters),
                services,
            )
            .await;
    }
}
