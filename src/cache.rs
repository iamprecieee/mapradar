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
    fn nearby_key(
        lat: f64,
        lng: f64,
        service_type: ServiceType,
        radius_meters: f64,
        max_results: usize,
    ) -> String {
        format!(
            "{:.4},{:.4}:{:?}:{:.0}:{}",
            lat, lng, service_type, radius_meters, max_results
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
        max_results: usize,
    ) -> Option<Vec<NearbyService>> {
        self.nearby
            .get(&Self::nearby_key(
                lat,
                lng,
                service_type,
                radius_meters,
                max_results,
            ))
            .await
    }

    /// Stores nearby search result in cache.
    pub async fn set_nearby(
        &self,
        lat: f64,
        lng: f64,
        service_type: ServiceType,
        radius_meters: f64,
        max_results: usize,
        services: Vec<NearbyService>,
    ) {
        self.nearby
            .insert(
                Self::nearby_key(lat, lng, service_type, radius_meters, max_results),
                services,
            )
            .await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn location(address: &str) -> GeoLocation {
        GeoLocation {
            address: address.to_string(),
            latitude: 6.5244,
            longitude: 3.3792,
            city: Some("Lagos".to_string()),
            state: Some("Lagos".to_string()),
            country: "Nigeria".to_string(),
        }
    }

    fn service(name: &str) -> NearbyService {
        NearbyService {
            name: name.to_string(),
            service_type: ServiceType::Market,
            latitude: 6.5244,
            longitude: 3.3792,
            distance_km: 0.4,
            address: None,
            rating: None,
            place_id: None,
            phone_number: None,
            open_now: None,
        }
    }

    #[tokio::test]
    async fn geocode_lookups_miss_before_anything_is_stored() {
        let cache = GeoCache::new();

        assert!(cache.get_geocode("Yaba, Lagos").await.is_none());
    }

    #[tokio::test]
    async fn geocode_results_are_returned_on_a_repeat_lookup() {
        let cache = GeoCache::new();
        cache
            .set_geocode("Yaba, Lagos", location("Yaba, Lagos"))
            .await;

        let cached = cache.get_geocode("Yaba, Lagos").await;

        assert_eq!(
            cached.map(|hit| hit.address),
            Some("Yaba, Lagos".to_string())
        );
    }

    #[tokio::test]
    async fn geocode_lookups_ignore_case_and_surrounding_whitespace() {
        let cache = GeoCache::new();
        cache
            .set_geocode("Yaba, Lagos", location("Yaba, Lagos"))
            .await;

        // Callers reach the same place typed differently; spending an API call
        // on each spelling would defeat the cache.
        assert!(cache.get_geocode("  YABA, Lagos  ").await.is_some());
        assert!(cache.get_geocode("yaba, lagos").await.is_some());
    }

    #[tokio::test]
    async fn different_addresses_do_not_share_a_geocode_entry() {
        let cache = GeoCache::new();
        cache
            .set_geocode("Yaba, Lagos", location("Yaba, Lagos"))
            .await;

        assert!(cache.get_geocode("Ikeja, Lagos").await.is_none());
    }

    #[tokio::test]
    async fn reverse_geocode_results_are_returned_on_a_repeat_lookup() {
        let cache = GeoCache::new();
        cache
            .set_reverse_geocode(6.5244, 3.3792, location("Yaba, Lagos"))
            .await;

        let cached = cache.get_reverse_geocode(6.5244, 3.3792).await;

        assert_eq!(
            cached.map(|hit| hit.address),
            Some("Yaba, Lagos".to_string())
        );
    }

    #[tokio::test]
    async fn reverse_geocode_treats_sub_centimetre_differences_as_one_place() {
        let cache = GeoCache::new();
        cache
            .set_reverse_geocode(6.5244, 3.3792, location("Yaba, Lagos"))
            .await;

        // Coordinates are keyed to six decimal places, roughly 11 cm. GPS
        // jitter below that still describes the same doorstep.
        assert!(
            cache
                .get_reverse_geocode(6.52440001, 3.37920001)
                .await
                .is_some()
        );
    }

    #[tokio::test]
    async fn reverse_geocode_keeps_distinct_coordinates_apart() {
        let cache = GeoCache::new();
        cache
            .set_reverse_geocode(6.5244, 3.3792, location("Yaba, Lagos"))
            .await;

        assert!(cache.get_reverse_geocode(6.6018, 3.3515).await.is_none());
    }

    #[tokio::test]
    async fn nearby_results_are_returned_on_a_repeat_search() {
        let cache = GeoCache::new();
        cache
            .set_nearby(
                6.5244,
                3.3792,
                ServiceType::Market,
                1000.0,
                20,
                vec![service("Tejuosho Market")],
            )
            .await;

        let cached = cache
            .get_nearby(6.5244, 3.3792, ServiceType::Market, 1000.0, 20)
            .await;

        assert_eq!(
            cached.map(|hits| hits.into_iter().map(|hit| hit.name).collect::<Vec<_>>()),
            Some(vec!["Tejuosho Market".to_string()])
        );
    }

    #[tokio::test]
    async fn nearby_searches_are_separated_by_every_search_parameter() {
        let cache = GeoCache::new();
        cache
            .set_nearby(
                6.5244,
                3.3792,
                ServiceType::Market,
                1000.0,
                20,
                vec![service("Tejuosho Market")],
            )
            .await;

        // Each parameter changes what Google would return, so a differing
        // value must not be served the stored answer.
        assert!(
            cache
                .get_nearby(6.6018, 3.3515, ServiceType::Market, 1000.0, 20)
                .await
                .is_none(),
            "a different location reused the cached search"
        );
        assert!(
            cache
                .get_nearby(6.5244, 3.3792, ServiceType::Hospital, 1000.0, 20)
                .await
                .is_none(),
            "a different service type reused the cached search"
        );
        assert!(
            cache
                .get_nearby(6.5244, 3.3792, ServiceType::Market, 5000.0, 20)
                .await
                .is_none(),
            "a different radius reused the cached search"
        );
        assert!(
            cache
                .get_nearby(6.5244, 3.3792, ServiceType::Market, 1000.0, 5)
                .await
                .is_none(),
            "a different result limit reused the cached search"
        );
    }

    #[tokio::test]
    async fn the_three_caches_do_not_read_each_others_entries() {
        let cache = GeoCache::new();
        cache
            .set_geocode("6.5244,3.3792", location("Yaba, Lagos"))
            .await;

        // The geocode key is the caller's raw string, which can look exactly
        // like a coordinate key.
        assert!(cache.get_reverse_geocode(6.5244, 3.3792).await.is_none());
    }
}
