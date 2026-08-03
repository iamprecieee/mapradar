use serde_json::Value;

use crate::error::GeoError;
use crate::models::GeoPoint;

/// Calculate Haversine distance between two points in km.
pub fn calculate_distance(
    origin_latitude: f64,
    origin_longitude: f64,
    destination_latitude: f64,
    destination_longitude: f64,
) -> f64 {
    let earth_radius = 6371.0;

    let lat1_rad = origin_latitude.to_radians();
    let lat2_rad = destination_latitude.to_radians();
    let long1_rad = origin_longitude.to_radians();
    let long2_rad = destination_longitude.to_radians();

    let latitude_difference = lat2_rad - lat1_rad;
    let longitude_difference = long2_rad - long1_rad;

    let half_chord_length_squared = (latitude_difference / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (longitude_difference / 2.0).sin().powi(2);

    let angular_distance = 2.0
        * half_chord_length_squared
            .sqrt()
            .atan2((1.0 - half_chord_length_squared).sqrt());

    earth_radius * angular_distance
}

/// Resolves a user-supplied travel mode into a Google Routes API travel mode.
///
/// Accepts Google's own mode names alongside regional transport names, so
/// `okada`, `keke`, `danfo`, `ojek`, `tuktuk` and friends all map onto the
/// closest supported API mode. Matching is case-insensitive and treats
/// hyphens and underscores interchangeably.
pub fn resolve_travel_mode(mode: &str) -> Result<&'static str, GeoError> {
    let normalized = mode.trim().to_uppercase().replace('-', "_");

    let resolved = match normalized.as_str() {
        // Two- and three-wheelers
        "TWO_WHEELER" | "MOTORCYCLE" | "BIKE" | "OKADA" | "KEKE" | "AUTO" | "RICKSHAW"
        | "TUKTUK" | "OJEK" | "BAJAJ" | "BECAK" => "TWO_WHEELER",
        // Shared and public transit
        "TRANSIT" | "TRAIN" | "BUS" | "DANFO" | "BRT" | "ANGKOT" | "BUSWAY" | "METRO"
        | "LOCAL_TRAIN" => "TRANSIT",
        "WALK" | "WALKING" | "FOOT" => "WALK",
        "BICYCLE" | "BIKING" | "CYCLING" => "BICYCLE",
        "DRIVE" | "DRIVING" | "CAR" => "DRIVE",
        _ => {
            return Err(GeoError::InvalidInput(format!(
                "Unknown travel mode '{}'. Supported modes: drive, walk, bicycle, transit, \
                 two_wheeler (and regional aliases such as okada, keke, danfo, brt, ojek, bajaj, \
                 becak, angkot, busway, metro, local_train, auto, rickshaw, tuktuk)",
                mode.trim()
            )));
        }
    };

    Ok(resolved)
}

/// Parse address components to find city, state, and country.
pub fn parse_address_components(
    address: &Value,
) -> Result<(Option<String>, Option<String>, String), GeoError> {
    let components = address.as_array().ok_or_else(|| {
        GeoError::Unknown("Missing address components in API response".to_string())
    })?;

    let mut city = None;
    let mut state = None;
    let mut country = String::new();

    for component in components {
        // A component without a `types` array carries no information we can
        // use, but the others may still identify the city, state and country,
        // so skip it rather than failing the whole lookup.
        let Some(types) = component["types"].as_array() else {
            continue;
        };

        if types.iter().any(|type_val| {
            type_val == "locality" || type_val == "postal_town" || type_val == "sublocality"
        }) {
            if city.is_none() || types.iter().any(|type_val| type_val == "locality") {
                city = Some(
                    component["long_name"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string(),
                );
            }
        } else if types
            .iter()
            .any(|type_val| type_val == "administrative_area_level_1")
        {
            state = Some(
                component["long_name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            );
        } else if types.iter().any(|type_val| type_val == "country") {
            country = component["long_name"]
                .as_str()
                .unwrap_or_default()
                .to_string();
        }
    }

    Ok((city, state, country))
}

/// Checks if a point is within a specified radius (in km) from a center point.
pub fn is_within_radius(
    point_lat: f64,
    point_lng: f64,
    center_lat: f64,
    center_lng: f64,
    radius_km: f64,
) -> bool {
    calculate_distance(center_lat, center_lng, point_lat, point_lng) <= radius_km
}

/// Filters a list of GeoPoints to only those within the specified radius.
/// It also updates their `distance_km` field.
pub fn filter_within_radius(
    points: &[GeoPoint],
    center_lat: f64,
    center_lng: f64,
    radius_km: f64,
) -> Vec<GeoPoint> {
    points
        .iter()
        .filter_map(|point| {
            let dist = calculate_distance(center_lat, center_lng, point.latitude, point.longitude);
            if dist <= radius_km {
                let mut updated_point = point.clone();
                updated_point.distance_km = Some(dist);
                Some(updated_point)
            } else {
                None
            }
        })
        .collect()
}

/// Sorts a list of GeoPoints by their distance to a center point in place.
/// It also updates their `distance_km` field.
pub fn sort_by_distance(points: &mut [GeoPoint], center_lat: f64, center_lng: f64) {
    for point in points.iter_mut() {
        // Always recompute: an existing value may have been measured from a
        // different center, which would sort the list against stale distances.
        point.distance_km = Some(calculate_distance(
            center_lat,
            center_lng,
            point.latitude,
            point.longitude,
        ));
    }
    points.sort_by(|a, b| {
        a.distance_km
            .unwrap_or(f64::MAX)
            .partial_cmp(&b.distance_km.unwrap_or(f64::MAX))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_address_components() {
        let addr = json!([
            {
                "long_name": "San Francisco",
                "short_name": "SF",
                "types": ["locality", "political"]
            },
            {
                "long_name": "California",
                "short_name": "CA",
                "types": ["administrative_area_level_1", "political"]
            },
            {
                "long_name": "United States",
                "short_name": "US",
                "types": ["country", "political"]
            }
        ]);

        let (city, state, country) = parse_address_components(&addr).unwrap();
        assert_eq!(city, Some("San Francisco".to_string()));
        assert_eq!(state, Some("California".to_string()));
        assert_eq!(country, "United States".to_string());
    }

    #[test]
    fn test_parse_address_components_postal_town() {
        let addr = json!([
            {
                "long_name": "London",
                "short_name": "London",
                "types": ["postal_town"]
            },
            {
                "long_name": "England",
                "short_name": "England",
                "types": ["administrative_area_level_1", "political"]
            },
            {
                "long_name": "United Kingdom",
                "short_name": "GB",
                "types": ["country", "political"]
            }
        ]);

        let (city, state, country) = parse_address_components(&addr).unwrap();
        assert_eq!(city, Some("London".to_string()));
        assert_eq!(state, Some("England".to_string()));
        assert_eq!(country, "United Kingdom".to_string());
    }

    #[test]
    fn test_resolve_travel_mode_accepts_google_modes() {
        for mode in ["DRIVE", "WALK", "BICYCLE", "TRANSIT", "TWO_WHEELER"] {
            assert_eq!(resolve_travel_mode(mode).unwrap(), mode);
        }
    }

    #[test]
    fn test_resolve_travel_mode_maps_regional_aliases() {
        assert_eq!(resolve_travel_mode("okada").unwrap(), "TWO_WHEELER");
        assert_eq!(resolve_travel_mode("keke").unwrap(), "TWO_WHEELER");
        assert_eq!(resolve_travel_mode("tuktuk").unwrap(), "TWO_WHEELER");
        assert_eq!(resolve_travel_mode("danfo").unwrap(), "TRANSIT");
        assert_eq!(resolve_travel_mode("brt").unwrap(), "TRANSIT");
        assert_eq!(resolve_travel_mode("angkot").unwrap(), "TRANSIT");
    }

    #[test]
    fn test_resolve_travel_mode_ignores_case_spacing_and_separators() {
        assert_eq!(resolve_travel_mode("  drive ").unwrap(), "DRIVE");
        assert_eq!(resolve_travel_mode("Walking").unwrap(), "WALK");
        assert_eq!(resolve_travel_mode("two-wheeler").unwrap(), "TWO_WHEELER");
        assert_eq!(resolve_travel_mode("local-train").unwrap(), "TRANSIT");
    }

    #[test]
    fn test_resolve_travel_mode_rejects_unknown_mode() {
        let err = resolve_travel_mode("teleport").unwrap_err();
        assert_eq!(err.json_rpc_code(), -32602);
        let message = err.to_string();
        assert!(message.contains("teleport"), "message was: {}", message);
        assert!(message.contains("drive"), "message was: {}", message);
    }

    #[test]
    fn test_sort_by_distance_recomputes_stale_distances() {
        let mut points = vec![
            GeoPoint {
                latitude: 6.6,
                longitude: 3.4,
                label: Some("far".to_string()),
                // Stale value measured from a different center.
                distance_km: Some(0.0),
            },
            GeoPoint {
                latitude: 6.5,
                longitude: 3.4,
                label: Some("near".to_string()),
                distance_km: None,
            },
        ];

        sort_by_distance(&mut points, 6.5, 3.4);

        assert_eq!(points[0].label.as_deref(), Some("near"));
        assert_eq!(points[1].label.as_deref(), Some("far"));
        assert!(points[1].distance_km.unwrap() > points[0].distance_km.unwrap());
    }

    #[test]
    fn test_parse_address_components_skips_components_without_types() {
        let addr = json!([
            {
                "long_name": "Malformed entry",
                "short_name": "Malformed"
            },
            {
                "long_name": "Lagos",
                "short_name": "Lagos",
                "types": ["locality", "political"]
            },
            {
                "long_name": "Nigeria",
                "short_name": "NG",
                "types": ["country", "political"]
            }
        ]);

        let (city, state, country) = parse_address_components(&addr).unwrap();
        assert_eq!(city, Some("Lagos".to_string()));
        assert_eq!(state, None);
        assert_eq!(country, "Nigeria".to_string());
    }
}
