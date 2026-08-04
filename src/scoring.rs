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
            category: service_type.slug().to_string(),
            score: 0.0,
            nearest_distance_km: None,
            count_within_radius: 0,
            average_rating: None,
        };
    }

    let nearest_distance_km = services
        .iter()
        .map(|service| service.distance_km)
        .fold(f64::INFINITY, f64::min);

    let count_within_radius = services.len();

    let ratings: Vec<f64> = services
        .iter()
        .filter_map(|service| service.rating.map(|rating| rating as f64))
        .collect();

    let average_rating = if ratings.is_empty() {
        None
    } else {
        // Ratings arrive as f32 with one decimal place, so widening them to f64
        // exposes representation noise (4.1 becomes 4.09999990463…) that the
        // mean then compounds and every export format prints verbatim. Two
        // decimals keep the real precision and drop the artifact.
        let mean = ratings.iter().sum::<f64>() / ratings.len() as f64;
        Some((mean * 100.0).round() / 100.0)
    };

    // --- Scoring Algorithm ---

    // 1. Distance Score (40% weight)
    // Closer is better. 0 km = 100, radius_km = 0.
    // A non-positive radius would make the ratio NaN, which would poison the
    // whole score, so treat it as no distance credit.
    let distance_ratio = if radius_km > 0.0 {
        (nearest_distance_km / radius_km).clamp(0.0, 1.0)
    } else {
        1.0
    };
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
        category: service_type.slug().to_string(),
        score: total_score,
        nearest_distance_km: Some(nearest_distance_km),
        count_within_radius,
        average_rating,
    }
}

/// Computes an overall composite score for a location based on intelligence
/// gathered.
///
/// `requested_types` is the set of categories the search covered. Categories
/// with no results still appear in the breakdown scoring zero, so that a
/// location missing most amenities cannot outscore a well-served one purely by
/// having fewer categories to average over.
pub fn compute_location_score(
    intel: &LocationIntelligence,
    requested_types: &[ServiceType],
    radius_km: f64,
) -> LocationScore {
    let mut grouped = group_by_type(&intel.nearby_services);
    let mut breakdown = Vec::new();

    for &service_type in requested_types {
        let services = grouped.remove(&service_type).unwrap_or_default();
        breakdown.push(compute_category_score(service_type, &services, radius_km));
    }

    // Any category present in the results but absent from `requested_types`
    // still belongs in the breakdown rather than being dropped.
    for (service_type, services) in grouped {
        breakdown.push(compute_category_score(service_type, &services, radius_km));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::GeoLocation;

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

    fn service(service_type: ServiceType, distance_km: f64, rating: Option<f32>) -> NearbyService {
        NearbyService {
            name: format!("{} at {}km", service_type.slug(), distance_km),
            service_type,
            latitude: 6.6,
            longitude: 3.35,
            distance_km,
            address: None,
            rating,
            place_id: None,
            phone_number: None,
            open_now: None,
        }
    }

    #[test]
    fn test_missing_categories_are_scored_zero_and_reported() {
        let intel =
            LocationIntelligence::new(location(), vec![service(ServiceType::Bank, 0.1, Some(4.5))]);
        let requested = [
            ServiceType::Bank,
            ServiceType::Hospital,
            ServiceType::School,
        ];

        let score = compute_location_score(&intel, &requested, 2.0);

        assert_eq!(score.breakdown.len(), 3);
        for service_type in requested {
            let category = score
                .breakdown
                .iter()
                .find(|entry| entry.category == service_type.slug())
                .unwrap_or_else(|| panic!("{} missing from breakdown", service_type));

            if service_type == ServiceType::Bank {
                assert!(category.score > 0.0);
                assert_eq!(category.count_within_radius, 1);
                assert!(category.nearest_distance_km.is_some());
            } else {
                assert_eq!(category.score, 0.0);
                assert_eq!(category.count_within_radius, 0);
                assert_eq!(category.nearest_distance_km, None);
            }
        }
    }

    #[test]
    fn test_missing_categories_lower_the_overall_score() {
        let bank_only =
            LocationIntelligence::new(location(), vec![service(ServiceType::Bank, 0.1, Some(4.5))]);

        let one_of_one = compute_location_score(&bank_only, &[ServiceType::Bank], 2.0);
        let one_of_three = compute_location_score(
            &bank_only,
            &[
                ServiceType::Bank,
                ServiceType::Hospital,
                ServiceType::School,
            ],
            2.0,
        );

        assert!(
            one_of_three.overall_score < one_of_one.overall_score,
            "expected a location missing two categories to score lower: {} vs {}",
            one_of_three.overall_score,
            one_of_one.overall_score
        );
    }

    #[test]
    fn test_well_served_location_outscores_sparse_one() {
        let requested = ServiceType::ALL;

        let sparse = LocationIntelligence::new(
            location(),
            vec![service(ServiceType::Bank, 0.05, Some(5.0))],
        );
        let well_served = LocationIntelligence::new(
            location(),
            requested
                .iter()
                .flat_map(|&service_type| {
                    [
                        service(service_type, 0.3, Some(4.0)),
                        service(service_type, 0.6, Some(4.0)),
                    ]
                })
                .collect(),
        );

        let sparse_score = compute_location_score(&sparse, &requested, 2.0);
        let well_served_score = compute_location_score(&well_served, &requested, 2.0);

        assert!(
            well_served_score.overall_score > sparse_score.overall_score,
            "well served {} should beat sparse {}",
            well_served_score.overall_score,
            sparse_score.overall_score
        );
    }

    #[test]
    fn test_closer_amenities_score_higher() {
        let near =
            LocationIntelligence::new(location(), vec![service(ServiceType::Bank, 0.1, Some(4.0))]);
        let far =
            LocationIntelligence::new(location(), vec![service(ServiceType::Bank, 1.9, Some(4.0))]);

        let near_score = compute_location_score(&near, &[ServiceType::Bank], 2.0);
        let far_score = compute_location_score(&far, &[ServiceType::Bank], 2.0);

        assert!(near_score.overall_score > far_score.overall_score);
    }

    #[test]
    fn test_unrequested_categories_are_still_reported() {
        let intel =
            LocationIntelligence::new(location(), vec![service(ServiceType::Pharmacy, 0.4, None)]);

        let score = compute_location_score(&intel, &[ServiceType::Bank], 2.0);

        assert!(
            score
                .breakdown
                .iter()
                .any(|entry| entry.category == ServiceType::Pharmacy.slug()),
            "pharmacy results should not be discarded"
        );
    }

    #[test]
    fn test_average_rating_is_free_of_float_representation_noise() {
        // 4.1 and 4.2 are not exactly representable as f32, so widening them to
        // f64 and averaging produces a long tail of digits that every export
        // format would print verbatim.
        let intel = LocationIntelligence::new(
            location(),
            vec![
                service(ServiceType::Bank, 0.2, Some(4.1)),
                service(ServiceType::Bank, 0.3, Some(4.2)),
            ],
        );

        let score = compute_location_score(&intel, &[ServiceType::Bank], 2.0);
        let average_rating = score.breakdown[0].average_rating.unwrap();

        assert_eq!(
            average_rating.to_string(),
            "4.15",
            "average rating rendered as {}",
            average_rating
        );
    }

    #[test]
    fn test_scores_stay_within_bounds_for_degenerate_radius() {
        let intel =
            LocationIntelligence::new(location(), vec![service(ServiceType::Bank, 0.0, Some(4.0))]);

        let score = compute_location_score(&intel, &[ServiceType::Bank], 0.0);

        assert!(
            score.overall_score.is_finite() && (0.0..=100.0).contains(&score.overall_score),
            "score was {}",
            score.overall_score
        );
    }
}
