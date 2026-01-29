use serde_json::Value;

use crate::error::GeoError;

/// Calculate Haversine distance between two points in km.
pub fn calculate_distance(
    latitude_1: f64,
    longitude_1: f64,
    latitude_2: f64,
    longitude_2: f64,
) -> f64 {
    let earth_radius = 6371.0;

    let lat1_rad = latitude_1.to_radians();
    let lat2_rad = latitude_2.to_radians();
    let long1_rad = longitude_1.to_radians();
    let long2_rad = longitude_2.to_radians();

    let latitude_difference = lat2_rad - lat1_rad;
    let longitude_difference = long2_rad - long1_rad;

    let a = (latitude_difference / 2.0).sin().powi(2)
        + lat1_rad.cos() * lat2_rad.cos() * (longitude_difference / 2.0).sin().powi(2);

    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());

    earth_radius * c
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

        if types.iter().any(|t| t == "locality") {
            city = Some(
                component["long_name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            );
        } else if types.iter().any(|t| t == "administrative_area_level_1") {
            state = Some(
                component["long_name"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
            );
        } else if types.iter().any(|t| t == "country") {
            country = component["short_name"]
                .as_str()
                .unwrap_or_default()
                .to_string();
        }
    }

    Ok((city, state, country))
}
