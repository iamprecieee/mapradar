use crate::models::{LocationIntelligence, LocationScore, NearbyService};
use serde::{Deserialize, Serialize};

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg_attr(feature = "python", pyclass(eq, eq_int, from_py_object))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    GeoJson,
    Csv,
    Kml,
}

impl std::str::FromStr for ExportFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "geojson" => Ok(ExportFormat::GeoJson),
            "csv" => Ok(ExportFormat::Csv),
            "kml" => Ok(ExportFormat::Kml),
            _ => Err(format!("Unknown export format: {}", s)),
        }
    }
}

pub fn export_nearby(services: &[NearbyService], format: ExportFormat) -> String {
    match format {
        ExportFormat::GeoJson => export_nearby_geojson(services),
        ExportFormat::Csv => export_nearby_csv(services),
        ExportFormat::Kml => export_nearby_kml(services),
    }
}

pub fn export_intelligence(intel: &LocationIntelligence, format: ExportFormat) -> String {
    match format {
        // GeoJSON and KML are map formats, so the queried location is included
        // as a point alongside the amenities found around it. CSV is a flat
        // table of amenities, where a center row would not belong.
        ExportFormat::GeoJson => export_intelligence_geojson(intel),
        ExportFormat::Csv => export_nearby_csv(&intel.nearby_services),
        ExportFormat::Kml => export_intelligence_kml(intel),
    }
}

fn export_intelligence_geojson(intel: &LocationIntelligence) -> String {
    let center = serde_json::json!({
        "type": "Feature",
        "geometry": {
            "type": "Point",
            "coordinates": [intel.location.longitude, intel.location.latitude]
        },
        "properties": {
            "role": "search-center",
            "address": intel.location.address,
            "city": intel.location.city,
            "state": intel.location.state,
            "country": intel.location.country,
            "total_services_found": intel.total_services_found,
        }
    });

    let mut features = vec![center];
    features.extend(nearby_geojson_features(&intel.nearby_services));

    let collection = serde_json::json!({
        "type": "FeatureCollection",
        "features": features
    });

    serde_json::to_string_pretty(&collection).unwrap_or_default()
}

fn export_intelligence_kml(intel: &LocationIntelligence) -> String {
    let mut kml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    kml.push_str("<kml xmlns=\"http://www.opengis.net/kml/2.2\">\n  <Document>\n");

    kml.push_str("    <Placemark>\n");
    kml.push_str(&format!(
        "      <name>{}</name>\n",
        escape_xml(&intel.location.address)
    ));
    kml.push_str("      <description>Search center</description>\n");
    kml.push_str(&format!(
        "      <Point>\n        <coordinates>{},{},0</coordinates>\n      </Point>\n",
        intel.location.longitude, intel.location.latitude
    ));
    kml.push_str("    </Placemark>\n");

    kml.push_str(&nearby_kml_placemarks(&intel.nearby_services));
    kml.push_str("  </Document>\n</kml>");
    kml
}

pub fn export_score(score: &LocationScore, format: ExportFormat) -> String {
    match format {
        ExportFormat::GeoJson => {
            // A point for the score center, with the full breakdown attached.
            let mut props = serde_json::Map::new();
            props.insert(
                "overall_score".to_string(),
                serde_json::json!(score.overall_score),
            );
            props.insert(
                "address".to_string(),
                serde_json::json!(score.location.address),
            );
            for cat in &score.breakdown {
                props.insert(
                    cat.category.clone(),
                    serde_json::json!({
                        "score": cat.score,
                        "count_within_radius": cat.count_within_radius,
                        "nearest_distance_km": cat.nearest_distance_km,
                        "average_rating": cat.average_rating,
                    }),
                );
            }

            let feature = serde_json::json!({
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [score.location.longitude, score.location.latitude]
                },
                "properties": props
            });
            serde_json::to_string_pretty(&feature).unwrap_or_default()
        }
        ExportFormat::Csv => {
            let mut csv = String::from("category,score,count,nearest_km,avg_rating\n");
            for cat in &score.breakdown {
                let rating = cat
                    .average_rating
                    .map(|r| r.to_string())
                    .unwrap_or_default();
                let nearest = cat
                    .nearest_distance_km
                    .map(|distance| format!("{:.2}", distance))
                    .unwrap_or_default();
                csv.push_str(&format!(
                    "{},{:.2},{},{},{}\n",
                    cat.category, cat.score, cat.count_within_radius, nearest, rating
                ));
            }
            csv
        }
        ExportFormat::Kml => {
            let mut description = format!("Overall score: {:.1}/100", score.overall_score);
            for cat in &score.breakdown {
                let nearest = match cat.nearest_distance_km {
                    Some(distance) => format!("{:.2} km", distance),
                    None => "none found".to_string(),
                };
                description.push_str(&format!(
                    "\n{}: {:.1} ({} found, nearest {})",
                    cat.category, cat.score, cat.count_within_radius, nearest
                ));
            }

            let mut kml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
            kml.push_str("<kml xmlns=\"http://www.opengis.net/kml/2.2\">\n  <Document>\n");
            kml.push_str("    <Placemark>\n");
            kml.push_str(&format!(
                "      <name>{} (score {:.1})</name>\n",
                escape_xml(&score.location.address),
                score.overall_score
            ));
            kml.push_str(&format!(
                "      <description>{}</description>\n",
                escape_xml(&description)
            ));
            kml.push_str(&format!(
                "      <Point>\n        <coordinates>{},{},0</coordinates>\n      </Point>\n",
                score.location.longitude, score.location.latitude
            ));
            kml.push_str("    </Placemark>\n");
            kml.push_str("  </Document>\n</kml>");
            kml
        }
    }
}

fn nearby_geojson_features(services: &[NearbyService]) -> Vec<serde_json::Value> {
    services
        .iter()
        .map(|service| {
            serde_json::json!({
                "type": "Feature",
                "geometry": {
                    "type": "Point",
                    "coordinates": [service.longitude, service.latitude]
                },
                "properties": {
                    "name": service.name,
                    "type": service.service_type.slug(),
                    "distance_km": service.distance_km,
                    "address": service.address.clone().unwrap_or_default(),
                    "rating": service.rating,
                    "phone": service.phone_number.clone().unwrap_or_default(),
                    "open_now": service.open_now,
                }
            })
        })
        .collect()
}

fn export_nearby_geojson(services: &[NearbyService]) -> String {
    let collection = serde_json::json!({
        "type": "FeatureCollection",
        "features": nearby_geojson_features(services)
    });

    serde_json::to_string_pretty(&collection).unwrap_or_default()
}

fn export_nearby_csv(services: &[NearbyService]) -> String {
    let mut csv =
        String::from("name,type,latitude,longitude,distance_km,rating,address,phone,open_now\n");

    for service in services {
        // Escape quotes and commas in name and address
        let name = service.name.replace("\"", "\"\"");
        let address = service
            .address
            .clone()
            .unwrap_or_default()
            .replace("\"", "\"\"");

        csv.push_str(&format!(
            "\"{}\",{},{},{},{:.2},{},\"{}\",\"{}\",{}\n",
            name,
            service.service_type.slug(),
            service.latitude,
            service.longitude,
            service.distance_km,
            service.rating.map(|r| r.to_string()).unwrap_or_default(),
            address,
            service.phone_number.clone().unwrap_or_default(),
            service.open_now.map(|o| o.to_string()).unwrap_or_default()
        ));
    }

    csv
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn nearby_kml_placemarks(services: &[NearbyService]) -> String {
    let mut placemarks = String::new();

    for service in services {
        placemarks.push_str("    <Placemark>\n");
        placemarks.push_str(&format!(
            "      <name>{}</name>\n",
            escape_xml(&service.name)
        ));

        let desc = format!(
            "Type: {}\nDistance: {:.2} km",
            service.service_type.slug(),
            service.distance_km
        );
        placemarks.push_str(&format!(
            "      <description>{}</description>\n",
            escape_xml(&desc)
        ));

        placemarks.push_str("      <Point>\n");
        placemarks.push_str(&format!(
            "        <coordinates>{},{},0</coordinates>\n",
            service.longitude, service.latitude
        ));
        placemarks.push_str("      </Point>\n");
        placemarks.push_str("    </Placemark>\n");
    }

    placemarks
}

fn export_nearby_kml(services: &[NearbyService]) -> String {
    let mut kml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    kml.push_str("<kml xmlns=\"http://www.opengis.net/kml/2.2\">\n  <Document>\n");
    kml.push_str(&nearby_kml_placemarks(services));
    kml.push_str("  </Document>\n</kml>");
    kml
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{GeoLocation, ServiceType};

    fn location() -> GeoLocation {
        GeoLocation {
            address: "Ikeja, Lagos".to_string(),
            latitude: 6.6,
            longitude: 3.35,
            city: Some("Lagos".to_string()),
            state: Some("Lagos".to_string()),
            country: "Nigeria".to_string(),
        }
    }

    fn service(name: &str, service_type: ServiceType) -> NearbyService {
        NearbyService {
            name: name.to_string(),
            service_type,
            latitude: 6.61,
            longitude: 3.36,
            distance_km: 1.25,
            address: Some("12 Allen Avenue".to_string()),
            rating: Some(4.5),
            place_id: Some("place-1".to_string()),
            phone_number: Some("+2348000000000".to_string()),
            open_now: Some(true),
        }
    }

    fn intelligence() -> LocationIntelligence {
        LocationIntelligence::new(
            location(),
            vec![
                service("First Bank", ServiceType::Bank),
                service("Reddington", ServiceType::Hospital),
            ],
        )
    }

    #[test]
    fn test_export_format_parses_known_names_case_insensitively() {
        assert_eq!(
            "GeoJSON".parse::<ExportFormat>().unwrap(),
            ExportFormat::GeoJson
        );
        assert_eq!("csv".parse::<ExportFormat>().unwrap(), ExportFormat::Csv);
        assert_eq!("KML".parse::<ExportFormat>().unwrap(), ExportFormat::Kml);
        assert!("xlsx".parse::<ExportFormat>().is_err());
    }

    #[test]
    fn test_geojson_export_is_valid_and_includes_every_service() {
        let output = export_intelligence(&intelligence(), ExportFormat::GeoJson);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        assert_eq!(parsed["type"], "FeatureCollection");
        let features = parsed["features"].as_array().unwrap();

        // Two amenities plus the search center.
        assert_eq!(features.len(), 3);
        assert_eq!(features[0]["properties"]["role"], "search-center");
        assert_eq!(
            features[0]["geometry"]["coordinates"],
            serde_json::json!([3.35, 6.6])
        );

        let bank = &features[1];
        assert_eq!(bank["properties"]["name"], "First Bank");
        assert_eq!(bank["properties"]["type"], "bank");
        // GeoJSON coordinates are [longitude, latitude], in that order.
        assert_eq!(
            bank["geometry"]["coordinates"],
            serde_json::json!([3.36, 6.61])
        );
    }

    #[test]
    fn test_geojson_service_types_parse_back_into_service_types() {
        let output = export_intelligence(&intelligence(), ExportFormat::GeoJson);
        let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

        for feature in parsed["features"].as_array().unwrap() {
            let Some(type_str) = feature["properties"]["type"].as_str() else {
                continue;
            };
            assert!(
                type_str.parse::<ServiceType>().is_ok(),
                "exported type '{}' did not parse back",
                type_str
            );
        }
    }

    #[test]
    fn test_csv_export_has_a_header_and_one_row_per_service() {
        let output = export_intelligence(&intelligence(), ExportFormat::Csv);
        let lines: Vec<&str> = output.lines().collect();

        assert_eq!(lines.len(), 3);
        assert!(lines[0].starts_with("name,type,latitude,longitude"));
        assert!(lines[1].contains("\"First Bank\""));
        assert!(lines[1].contains("bank"));
        assert!(lines[2].contains("\"Reddington\""));
    }

    #[test]
    fn test_csv_export_escapes_embedded_quotes_and_commas() {
        let services = vec![service("Ade\"s Bank, Ikeja", ServiceType::Bank)];
        let output = export_nearby(&services, ExportFormat::Csv);
        let row = output.lines().nth(1).unwrap();

        // The embedded quote is doubled and the whole field stays quoted, so
        // the comma inside the name does not start a new column.
        assert!(
            row.starts_with("\"Ade\"\"s Bank, Ikeja\","),
            "row was: {}",
            row
        );
    }

    #[test]
    fn test_kml_export_is_well_formed_and_escapes_markup() {
        let services = vec![service("Bank & <Trust>", ServiceType::Bank)];
        let output = export_nearby(&services, ExportFormat::Kml);

        assert!(output.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(output.trim_end().ends_with("</kml>"));
        assert!(output.contains("<name>Bank &amp; &lt;Trust&gt;</name>"));
        assert!(!output.contains("<Trust>"));
        assert_eq!(output.matches("<Placemark>").count(), 1);
        assert_eq!(output.matches("</Placemark>").count(), 1);
    }

    #[test]
    fn test_kml_export_of_intelligence_includes_the_search_center() {
        let output = export_intelligence(&intelligence(), ExportFormat::Kml);

        // Two amenities plus the search center.
        assert_eq!(output.matches("<Placemark>").count(), 3);
        assert!(output.contains("<name>Ikeja, Lagos</name>"));
        assert!(output.contains("<coordinates>3.35,6.6,0</coordinates>"));
    }

    #[test]
    fn test_empty_results_still_produce_valid_documents() {
        let empty = LocationIntelligence::new(location(), Vec::new());

        let geojson = export_intelligence(&empty, ExportFormat::GeoJson);
        let parsed: serde_json::Value = serde_json::from_str(&geojson).unwrap();
        assert_eq!(parsed["features"].as_array().unwrap().len(), 1);

        let csv = export_intelligence(&empty, ExportFormat::Csv);
        assert_eq!(csv.lines().count(), 1);

        let kml = export_intelligence(&empty, ExportFormat::Kml);
        assert!(kml.trim_end().ends_with("</kml>"));
    }

    #[test]
    fn test_score_export_reports_categories_with_no_results() {
        let score = crate::scoring::compute_location_score(
            &intelligence(),
            &[ServiceType::Bank, ServiceType::School],
            2.0,
        );

        let geojson = export_score(&score, ExportFormat::GeoJson);
        let parsed: serde_json::Value = serde_json::from_str(&geojson).unwrap();
        let props = &parsed["properties"];
        assert_eq!(props["school"]["count_within_radius"], 0);
        assert_eq!(
            props["school"]["nearest_distance_km"],
            serde_json::json!(null)
        );
        assert_eq!(props["bank"]["count_within_radius"], 1);

        let csv = export_score(&score, ExportFormat::Csv);
        assert!(csv.lines().any(|line| line.starts_with("school,")));

        let kml = export_score(&score, ExportFormat::Kml);
        assert!(kml.contains("school"));
        assert!(kml.trim_end().ends_with("</kml>"));
    }
}
