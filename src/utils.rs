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
        let types = component["types"].as_array().ok_or_else(|| {
            GeoError::Unknown("Missing component types in API response".to_string())
        })?;

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
pub fn filter_within_radius(points: &[GeoPoint], center_lat: f64, center_lng: f64, radius_km: f64) -> Vec<GeoPoint> {
    points
        .iter()
        .filter_map(|p| {
            let dist = calculate_distance(center_lat, center_lng, p.latitude, p.longitude);
            if dist <= radius_km {
                let mut updated_p = p.clone();
                updated_p.distance_km = Some(dist);
                Some(updated_p)
            } else {
                None
            }
        })
        .collect()
}

/// Sorts a list of GeoPoints by their distance to a center point in place.
/// It also updates their `distance_km` field.
pub fn sort_by_distance(points: &mut [GeoPoint], center_lat: f64, center_lng: f64) {
    for p in points.iter_mut() {
        if p.distance_km.is_none() {
            p.distance_km = Some(calculate_distance(center_lat, center_lng, p.latitude, p.longitude));
        }
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
}
