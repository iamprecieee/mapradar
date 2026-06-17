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
        ExportFormat::GeoJson => export_nearby_geojson(&intel.nearby_services),
        ExportFormat::Csv => export_nearby_csv(&intel.nearby_services),
        ExportFormat::Kml => export_nearby_kml(&intel.nearby_services),
    }
}

pub fn export_score(score: &LocationScore, format: ExportFormat) -> String {
    match format {
        ExportFormat::GeoJson => {
            // A point for the score center, with properties for each category
            let mut props = serde_json::Map::new();
            props.insert(
                "overall_score".to_string(),
                serde_json::json!(score.overall_score),
            );
            for cat in &score.breakdown {
                props.insert(cat.category.clone(), serde_json::json!(cat.score));
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
                csv.push_str(&format!(
                    "{},{:.2},{},{:.2},{}\n",
                    cat.category,
                    cat.score,
                    cat.count_within_radius,
                    cat.nearest_distance_km,
                    rating
                ));
            }
            csv
        }
        ExportFormat::Kml => {
            let mut kml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
            kml.push_str("<kml xmlns=\"http://www.opengis.net/kml/2.2\">\n  <Document>\n");
            kml.push_str(&format!(
                "    <Placemark>\n      <name>Score: {:.1}</name>\n",
                score.overall_score
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

fn export_nearby_geojson(services: &[NearbyService]) -> String {
    let mut features = Vec::new();

    for service in services {
        let feature = serde_json::json!({
            "type": "Feature",
            "geometry": {
                "type": "Point",
                "coordinates": [service.longitude, service.latitude]
            },
            "properties": {
                "name": service.name,
                "type": format!("{:?}", service.service_type),
                "distance_km": service.distance_km,
                "address": service.address.clone().unwrap_or_default(),
                "rating": service.rating,
                "phone": service.phone_number.clone().unwrap_or_default(),
                "open_now": service.open_now,
            }
        });
        features.push(feature);
    }

    let collection = serde_json::json!({
        "type": "FeatureCollection",
        "features": features
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
            "\"{}\",{:?},{},{},{:.2},{},\"{}\",\"{}\",{}\n",
            name,
            service.service_type,
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

fn export_nearby_kml(services: &[NearbyService]) -> String {
    let mut kml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    kml.push_str("<kml xmlns=\"http://www.opengis.net/kml/2.2\">\n  <Document>\n");

    for service in services {
        kml.push_str("    <Placemark>\n");
        // Escape XML entities
        let name = service
            .name
            .replace("&", "&amp;")
            .replace("<", "&lt;")
            .replace(">", "&gt;");
        kml.push_str(&format!("      <name>{}</name>\n", name));

        let desc = format!(
            "Type: {:?}\nDistance: {:.2} km",
            service.service_type, service.distance_km
        );
        kml.push_str(&format!("      <description>{}</description>\n", desc));

        kml.push_str("      <Point>\n");
        kml.push_str(&format!(
            "        <coordinates>{},{},0</coordinates>\n",
            service.longitude, service.latitude
        ));
        kml.push_str("      </Point>\n");
        kml.push_str("    </Placemark>\n");
    }

    kml.push_str("  </Document>\n</kml>");
    kml
}
