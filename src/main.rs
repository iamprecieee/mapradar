use clap::{Parser, Subcommand};
use colored::*;
use mapradar::client::MapradarClient;
use mapradar::models::{SearchQuery, ServiceType, TravelParameters};
use std::process;

fn validate_extension(file_path: &str, format: &str) -> Result<(), String> {
    let extension = std::path::Path::new(file_path)
        .extension()
        .and_then(std::ffi::OsStr::to_str)
        .unwrap_or("")
        .to_lowercase();

    let format_lower = format.to_lowercase();
    let is_mismatch = match format_lower.as_str() {
        "json" => extension != "json",
        "geojson" => extension != "geojson" && extension != "json",
        "csv" => extension != "csv",
        "kml" => extension != "kml",
        _ => false,
    };

    if is_mismatch {
        Err(format!(
            "Output file extension '.{}' does not match requested format '{}'",
            extension, format_lower
        ))
    } else {
        Ok(())
    }
}

#[derive(Parser)]
#[command(name = "mapradar")]
#[command(about = "CLI for Mapradar Location Intelligence", long_about = None)]
struct Cli {
    #[arg(short, long, env = "MAPRADAR_API_KEY")]
    api_key: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Geocode an address to coordinates
    Geocode { address: String },

    /// Reverse geocode coordinates to an address
    Reverse { latitude: f64, longitude: f64 },

    /// Find nearby amenities
    Nearby {
        #[arg(short, long, alias = "addr")]
        address: Option<String>,

        #[arg(long, alias = "lat")]
        latitude: Option<f64>,

        #[arg(long, alias = "lng", alias = "lon")]
        longitude: Option<f64>,

        /// Radius in meters (default 1000)
        #[arg(short, long, default_value_t = 1000.0)]
        radius: f64,

        /// Type of amenity (bank, hospital, school, etc.)
        #[arg(short, long, default_value = "bank")]
        r#type: String,

        /// Maximum number of results to return per service
        #[arg(short, long, alias = "limit", default_value_t = 10)]
        max_results: usize,

        /// Output format (json, geojson, csv, kml)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file path (optional, prints to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Calculate travel distance between two points
    Distance {
        #[arg(long, help = "Origin address")]
        origin_addr: Option<String>,

        #[arg(long, help = "Origin latitude")]
        origin_lat: Option<f64>,

        #[arg(long, help = "Origin longitude")]
        origin_lng: Option<f64>,

        #[arg(long, help = "Destination address")]
        dest_addr: Option<String>,

        #[arg(long, help = "Destination latitude")]
        dest_lat: Option<f64>,

        #[arg(long, help = "Destination longitude")]
        dest_lng: Option<f64>,

        #[arg(
            long,
            help = "Mode of travel (drive, walk, bicycle, motorcycle, transit)",
            default_value = "drive"
        )]
        mode: String,
    },

    /// Filter and sort points within a radius of a center location
    WithinRadius {
        #[arg(
            long,
            help = "Center latitude (optional if center-address is provided)"
        )]
        lat: Option<f64>,

        #[arg(
            long,
            help = "Center longitude (optional if center-address is provided)"
        )]
        lng: Option<f64>,

        #[arg(
            long,
            short,
            help = "Center address (optional if lat/lng are provided)"
        )]
        address: Option<String>,

        #[arg(
            long,
            help = "Target latitude (optional if target-address is provided)"
        )]
        target_lat: Option<f64>,

        #[arg(
            long,
            help = "Target longitude (optional if target-address is provided)"
        )]
        target_lng: Option<f64>,

        #[arg(
            long,
            help = "Target address (optional if target lat/lng are provided)"
        )]
        target_address: Option<String>,

        #[arg(short, long, help = "Radius in kilometers")]
        radius: f64,
    },

    /// Score a location based on nearby amenities
    Score {
        #[arg(
            long,
            short,
            help = "Address to score (optional if lat/lng are provided)"
        )]
        address: Option<String>,

        #[arg(long, help = "Latitude (optional if address is provided)")]
        lat: Option<f64>,

        #[arg(long, help = "Longitude (optional if address is provided)")]
        lng: Option<f64>,

        #[arg(
            short,
            long,
            help = "Radius in kilometers to search within",
            default_value_t = 2.0
        )]
        radius: f64,

        /// Output format (json, geojson, csv, kml)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file path (optional, prints to stdout if omitted)
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();
    let client = MapradarClient::new(cli.api_key);

    match cli.command {
        Commands::Geocode { address } => match client.geocode_async(&address).await {
            Ok(loc) => println!("{}", serde_json::to_string_pretty(&loc).unwrap()),
            Err(err) => {
                eprintln!("{} {}", "Error:".red().bold(), err);
                process::exit(1);
            }
        },
        Commands::Reverse {
            latitude,
            longitude,
        } => match client.reverse_geocode_async(latitude, longitude).await {
            Ok(address) => println!("{}", serde_json::to_string_pretty(&address).unwrap()),
            Err(err) => {
                eprintln!("{} {}", "Error:".red().bold(), err);
                process::exit(1);
            }
        },
        Commands::Nearby {
            address,
            latitude,
            longitude,
            radius,
            r#type,
            max_results,
            format,
            output,
        } => {
            let service_types = r#type
                .split(",")
                .map(|type_str| match type_str.trim() {
                    "bank" => ServiceType::Bank,
                    "hospital" => ServiceType::Hospital,
                    "school" => ServiceType::School,
                    "restaurant" => ServiceType::Restaurant,
                    "bus-stop" => ServiceType::BusStop,
                    "market" => ServiceType::Market,
                    "mall" => ServiceType::Mall,
                    "fuel-station" => ServiceType::FuelStation,
                    "train-station" => ServiceType::TrainStation,
                    "taxi-stand" => ServiceType::TaxiStand,
                    "landmark" => ServiceType::Landmark,
                    "pharmacy" => ServiceType::Pharmacy,
                    _ => ServiceType::Landmark, // Default fallback
                })
                .collect::<Vec<ServiceType>>();

            let query = if let Some(latitude_val) = latitude {
                if let Some(longitude_val) = longitude {
                    SearchQuery::from_coordinates(latitude_val, longitude_val)
                } else {
                    eprintln!(
                        "{} Longitude is required when latitude is provided",
                        "Error:".red().bold()
                    );
                    process::exit(1);
                }
            } else {
                if let Some(address_val) = address {
                    SearchQuery::from_address(address_val)
                } else {
                    eprintln!(
                        "{} Either address or coordinates must be provided",
                        "Error:".red().bold()
                    );
                    process::exit(1);
                }
            };

            if let Some(file_path) = &output
                && let Err(msg) = validate_extension(file_path, &format)
            {
                eprintln!("{} {}", "Error:".red().bold(), msg);
                process::exit(1);
            }

            match client
                .fetch_intelligence_async(query, service_types, radius / 1000.0, max_results)
                .await
            {
                Ok(intel) => {
                    let out_string = if format.to_lowercase() == "json" {
                        serde_json::to_string_pretty(&intel).unwrap()
                    } else if let Ok(export_fmt) = format.parse::<mapradar::export::ExportFormat>()
                    {
                        mapradar::export::export_intelligence(&intel, export_fmt)
                    } else {
                        eprintln!("{} Invalid format: {}", "Error:".red().bold(), format);
                        process::exit(1);
                    };

                    if let Some(file_path) = output {
                        if let Err(e) = std::fs::write(&file_path, out_string) {
                            eprintln!(
                                "{} Failed to write to file {}: {}",
                                "Error:".red().bold(),
                                file_path,
                                e
                            );
                            process::exit(1);
                        } else {
                            println!(
                                "{} Successfully wrote output to {}",
                                "Success:".green().bold(),
                                file_path
                            );
                        }
                    } else {
                        println!("{}", out_string);
                    }
                }
                Err(err) => {
                    eprintln!("{} {}", "Error:".red().bold(), err);
                    process::exit(1);
                }
            }
        }
        Commands::Distance {
            origin_addr,
            origin_lat,
            origin_lng,
            dest_addr,
            dest_lat,
            dest_lng,
            mode,
        } => {
            let api_mode = match mode.to_lowercase().as_str() {
                "walk" | "walking" => "WALK",
                "bicycle" => "BICYCLE",
                "motorcycle" | "bike" | "okada" | "keke" => "TWO_WHEELER",
                "transit" | "train" | "bus" | "brt" | "danfo" => "TRANSIT",
                _ => "DRIVE",
            };

            let params = TravelParameters {
                origin_latitude: origin_lat,
                origin_longitude: origin_lng,
                origin_address: origin_addr,
                destination_latitude: dest_lat,
                destination_longitude: dest_lng,
                destination_address: dest_addr,
                travel_mode: Some(api_mode.to_string()),
            };

            match client.calculate_travel_distance_async(params).await {
                Ok(dist) => println!("{} {:.2} km", "Distance:".green().bold(), dist),
                Err(err) => {
                    eprintln!("{} {}", "Error:".red().bold(), err);
                    process::exit(1);
                }
            }
        }
        Commands::WithinRadius {
            lat,
            lng,
            address,
            target_lat,
            target_lng,
            target_address,
            radius,
        } => {
            let (center_latitude, center_longitude) =
                if let (Some(latitude_val), Some(longitude_val)) = (lat, lng) {
                    (latitude_val, longitude_val)
                } else if let Some(address_val) = address {
                    let loc = client
                        .geocode_async(&address_val)
                        .await
                        .unwrap_or_else(|e| {
                            eprintln!("{} {}", "Error geocoding center address:".red().bold(), e);
                            process::exit(1);
                        });
                    (loc.latitude, loc.longitude)
                } else {
                    eprintln!(
                        "{} Either center lat/lng or address must be provided",
                        "Error:".red().bold()
                    );
                    process::exit(1);
                };

            let (target_latitude, target_longitude) =
                if let (Some(latitude_val), Some(longitude_val)) = (target_lat, target_lng) {
                    (latitude_val, longitude_val)
                } else if let Some(address_val) = target_address {
                    let loc = client
                        .geocode_async(&address_val)
                        .await
                        .unwrap_or_else(|e| {
                            eprintln!("{} {}", "Error geocoding target address:".red().bold(), e);
                            process::exit(1);
                        });
                    (loc.latitude, loc.longitude)
                } else {
                    eprintln!(
                        "{} Either target lat/lng or address must be provided",
                        "Error:".red().bold()
                    );
                    process::exit(1);
                };

            let within = mapradar::utils::is_within_radius(
                target_latitude,
                target_longitude,
                center_latitude,
                center_longitude,
                radius,
            );
            let dist = mapradar::utils::calculate_distance(
                center_latitude,
                center_longitude,
                target_latitude,
                target_longitude,
            );

            if within {
                println!(
                    "{} Target is {:.2} km away (within {} km radius)",
                    "YES:".green().bold(),
                    dist,
                    radius
                );
            } else {
                println!(
                    "{} Target is {:.2} km away (outside {} km radius)",
                    "NO:".red().bold(),
                    dist,
                    radius
                );
            }
        }
        Commands::Score {
            address,
            lat,
            lng,
            radius,
            format,
            output,
        } => {
            let query = if let (Some(latitude_val), Some(longitude_val)) = (lat, lng) {
                SearchQuery::from_coordinates(latitude_val, longitude_val)
            } else if let Some(address_val) = address {
                SearchQuery::from_address(address_val)
            } else {
                eprintln!(
                    "{} Either lat/lng or address must be provided",
                    "Error:".red().bold()
                );
                process::exit(1);
            };

            if let Some(file_path) = &output
                && let Err(msg) = validate_extension(file_path, &format)
            {
                eprintln!("{} {}", "Error:".red().bold(), msg);
                process::exit(1);
            }

            match client.score_location_async(query, radius).await {
                Ok(score) => {
                    if format.to_lowercase() == "json" && output.is_none() {
                        // Print the default custom UI for json since Score doesn't use raw json in the CLI usually
                        println!("\n{}", "=== Location Score ===".blue().bold());
                        println!("Location: {}", score.location.address);
                        println!(
                            "Coordinates: {}, {}",
                            score.location.latitude, score.location.longitude
                        );
                        println!("Radius: {} km\n", radius);

                        println!(
                            "{} {:.1}/100\n",
                            "OVERALL SCORE:".green().bold(),
                            score.overall_score
                        );

                        println!("{}", "--- Category Breakdown ---".blue());
                        for cat in score.breakdown {
                            let avg_rating = if let Some(r) = cat.average_rating {
                                format!("{:.1}", r)
                            } else {
                                "N/A".to_string()
                            };

                            println!(
                                "{:<15} | Score: {:>5.1} | Count: {:>2} | Nearest: {:>4.1} km | Avg Rating: {}",
                                cat.category,
                                cat.score,
                                cat.count_within_radius,
                                cat.nearest_distance_km,
                                avg_rating
                            );
                        }
                        println!();
                    } else {
                        let out_string = if format.to_lowercase() == "json" {
                            serde_json::to_string_pretty(&score).unwrap()
                        } else if let Ok(export_fmt) =
                            format.parse::<mapradar::export::ExportFormat>()
                        {
                            mapradar::export::export_score(&score, export_fmt)
                        } else {
                            eprintln!("{} Invalid format: {}", "Error:".red().bold(), format);
                            process::exit(1);
                        };

                        if let Some(file_path) = output {
                            if let Err(e) = std::fs::write(&file_path, out_string) {
                                eprintln!(
                                    "{} Failed to write to file {}: {}",
                                    "Error:".red().bold(),
                                    file_path,
                                    e
                                );
                                process::exit(1);
                            } else {
                                println!(
                                    "{} Successfully wrote output to {}",
                                    "Success:".green().bold(),
                                    file_path
                                );
                            }
                        } else {
                            println!("{}", out_string);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("{} {}", "Error:".red().bold(), err);
                    process::exit(1);
                }
            }
        }
    }
}
