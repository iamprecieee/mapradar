use clap::{Parser, Subcommand};
use colored::*;
use mapradar::client::MapradarClient;
use mapradar::models::{SearchQuery, ServiceType};
use std::process;

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
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
                process::exit(1);
            }
        },
        Commands::Reverse {
            latitude,
            longitude,
        } => match client.reverse_geocode_async(latitude, longitude).await {
            Ok(address) => println!("{:?}", address),
            Err(e) => {
                eprintln!("{} {}", "Error:".red().bold(), e);
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
        } => {
            let service_types = r#type
                .split(",")
                .map(|s| match s.trim() {
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

            match client
                .fetch_intelligence_async(query, service_types, radius, max_results)
                .await
            {
                Ok(intel) => println!("{}", serde_json::to_string_pretty(&intel).unwrap()),
                Err(e) => {
                    eprintln!("{} {}", "Error:".red().bold(), e);
                    process::exit(1);
                }
            }
        }
    }
}
