use crate::models::{
    CategoryScore, LocationIntelligence, LocationScore, NearbyService, ServiceType,
};
use std::collections::HashMap;

/// Internal helper to group services by their type
fn group_by_type(services: &[NearbyService]) -> HashMap<ServiceType, Vec<NearbyService>> {
    let mut map: HashMap<ServiceType, Vec<NearbyService>> = HashMap::new();
    for service in services {
        map.entry(service.service_type)
            .or_default()
            .push(service.clone());
    }
    map
}

/// Computes a score (0-100) for a specific category of amenities
pub fn compute_category_score(
    service_type: ServiceType,
    services: &[NearbyService],
    radius_km: f64,
) -> CategoryScore {
    if services.is_empty() {
        return CategoryScore {
            category: format!("{:?}", service_type),
            score: 0.0,
            nearest_distance_km: f64::MAX,
            count_within_radius: 0,
            average_rating: None,
        };
    }

    let nearest_distance_km = services
        .iter()
        .map(|service| service.distance_km)
        .fold(f64::MAX, f64::min);

    let count_within_radius = services.len();

    let ratings: Vec<f64> = services
        .iter()
        .filter_map(|service| service.rating.map(|rating| rating as f64))
        .collect();

    let average_rating = if ratings.is_empty() {
        None
    } else {
        Some(ratings.iter().sum::<f64>() / ratings.len() as f64)
    };

    // --- Scoring Algorithm ---

    // 1. Distance Score (40% weight)
    // Closer is better. 0 km = 100, radius_km = 0.
    let distance_ratio = (nearest_distance_km / radius_km).clamp(0.0, 1.0);
    let distance_score = 100.0 * (1.0 - distance_ratio);

    // 2. Density Score (30% weight)
    // More places is better. 5+ places = 100.
    let density_score = (count_within_radius as f64 * 20.0).clamp(0.0, 100.0);

    // 3. Quality Score (30% weight)
    // Scaled from 1-5 to 0-100. If no ratings exist, neutral 50.
    let quality_score = if let Some(avg) = average_rating {
        (avg / 5.0) * 100.0
    } else {
        50.0 // Neutral fallback
    };

    let total_score = (0.4 * distance_score) + (0.3 * density_score) + (0.3 * quality_score);

    CategoryScore {
        category: format!("{:?}", service_type),
        score: total_score,
        nearest_distance_km,
        count_within_radius,
        average_rating,
    }
}

/// Computes an overall composite score for a location based on intelligence gathered
pub fn compute_location_score(intel: &LocationIntelligence, radius_km: f64) -> LocationScore {
    let grouped = group_by_type(&intel.nearby_services);
    let mut breakdown = Vec::new();

    for (svc_type, services) in grouped {
        let cat_score = compute_category_score(svc_type, &services, radius_km);
        breakdown.push(cat_score);
    }

    // Sort breakdown by score descending
    breakdown.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let overall_score = if breakdown.is_empty() {
        0.0
    } else {
        let sum: f64 = breakdown.iter().map(|c| c.score).sum();
        sum / breakdown.len() as f64
    };

    LocationScore {
        overall_score,
        breakdown,
        location: intel.location.clone(),
    }
}
