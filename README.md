# Mapradar

[![Crates.io](https://img.shields.io/crates/v/mapradar.svg)](https://crates.io/crates/mapradar)
[![PyPI](https://img.shields.io/pypi/v/mapradar.svg)](https://pypi.org/project/mapradar/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Mapradar is a Rust and Python location-intelligence library and CLI for geocoding, reverse geocoding, nearby-service discovery, route distance, geofencing, location scoring, and structured map exports using Google Maps APIs.

## Installation

Python:

```bash
uv add mapradar
# or: pip install mapradar
```

Rust library:

```toml
[dependencies]
mapradar = "0.6"
tokio = { version = "1", features = ["full"] }
```

CLI:

```bash
cargo install mapradar
```

To build from source:

```bash
git clone https://github.com/iamprecieee/mapradar
cd mapradar

# Rust library and CLI
cargo build

# Python extension
uv add --dev maturin
uv run maturin develop
```

## Configuration

Enable the following APIs in Google Cloud:

- Geocoding API
- Places API (New)
- Routes API (V2)

Set the API key for the CLI, or pass it with `--api-key`:

```bash
export MAPRADAR_API_KEY="your-api-key"
```

The CLI also reads a local `.env` file. Keep that file out of version control; this repository's `.gitignore` already excludes it. Library users pass the key to `MapradarClient` directly.

## CLI usage

```bash
# Geocode and reverse geocode
mapradar geocode "1600 Amphitheatre Parkway, Mountain View, CA"
mapradar reverse 6.6018 3.3515

# Nearby radius is in meters; types are comma-separated
mapradar nearby --lat 6.6018 --lng 3.3515 \
  --radius 1000 --type bank,school,pharmacy

# Nearby formats: json, geojson, csv, kml
mapradar nearby --address "Ikeja, Lagos" --type hospital,pharmacy \
  --format geojson --output nearby.geojson

# Route distance
mapradar distance --origin-addr "Shibuya, Tokyo" \
  --dest-addr "Shinjuku, Tokyo" --mode transit

# Geofence radius is in kilometers
mapradar within-radius --lat 6.6018 --lng 3.3515 \
  --target-address "Yaba, Lagos" --radius 5

# Score radius is in kilometers; text is the default format
mapradar score --address "Yaba, Lagos" --radius 3
mapradar score --address "Yaba, Lagos" --radius 3 \
  --format csv --output score.csv
```

Supported nearby service types are `bank`, `hospital`, `school`, `restaurant`, `bus-stop`, `market`, `mall`, `fuel-station`, `train-station`, `taxi-stand`, `landmark`, and `pharmacy`. Matching is case-insensitive, and hyphens, underscores, and spaces are interchangeable.

Supported route modes are `drive`, `walk`, `bicycle`, `transit`, and `two_wheeler`. Aliases include `car`, `walking`, `cycling`, `motorcycle`, `bike`, `okada`, `keke`, `auto`, `rickshaw`, `tuktuk`, `ojek`, `bajaj`, `becak`, `train`, `bus`, `danfo`, `brt`, `angkot`, `busway`, `metro`, and `local_train`. Unknown service types, modes, and output formats are rejected before an API request is made.

## Python usage

```python
import asyncio
import os

import mapradar
from mapradar import MapradarClient, SearchQuery, ServiceType, TravelParameters


async def main():
    client = MapradarClient(os.environ["MAPRADAR_API_KEY"])

    params = TravelParameters(
        origin_address="Shibuya, Tokyo",
        destination_address="Shinjuku, Tokyo",
        travel_mode="transit",
    )
    distance_km = await client.calculate_travel_distance(params)
    print(f"Distance: {distance_km:.2f} km")

    query = SearchQuery.from_address("Ikeja, Lagos")
    intelligence = await client.fetch_intelligence(
        query,
        service_types=[ServiceType.Bank, ServiceType.Hospital],
        radius_km=3.0,
        max_results_per_type=10,
    )

    for warning in intelligence.failed_lookups:
        print(f"Partial result: {warning}")

    geojson = mapradar.export_intelligence(intelligence, "geojson")
    print(geojson)

    score = await client.score_location(query, radius_km=5.0)
    print(f"Overall score: {score.overall_score:.2f}")
    for category in score.breakdown:
        print(category.category, category.nearest_distance_km)


asyncio.run(main())
```

`fetch_intelligence` returns successful categories even if another category fails. Check `failed_lookups` to detect a partial result. If every requested category fails, the call returns an error. Requests use a 10-second connection timeout and a 30-second overall timeout.

`score_location` searches all 12 supported service categories. Categories with no result remain in the breakdown with a score of `0` and `nearest_distance_km = None`, so missing amenities lower the overall score instead of disappearing from the average.

## Rust usage

```rust
use mapradar::client::MapradarClient;
use mapradar::models::{SearchQuery, ServiceType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("MAPRADAR_API_KEY")?;
    let client = MapradarClient::new(api_key);

    let query = SearchQuery::from_address("Ikeja, Lagos".to_string());
    let intelligence = client
        .fetch_intelligence_async(
            query,
            vec![ServiceType::Bank, ServiceType::Hospital],
            3.0,
            10,
        )
        .await?;

    for service in intelligence.nearby_services {
        println!("{}: {:.2} km", service.name, service.distance_km);
    }

    for warning in intelligence.failed_lookups {
        eprintln!("partial result: {warning}");
    }

    Ok(())
}
```

## Exports and scoring

- Nearby and intelligence exports support GeoJSON, CSV, and KML.
- Score exports support GeoJSON, CSV, and KML; the CLI additionally supports JSON and human-readable text.
- Intelligence GeoJSON and KML include the search center as well as nearby amenities.
- Score exports include the location, overall score, and full per-category breakdown.
- Exported service labels use the same canonical slugs accepted by `--type`.

## Features

| Feature | Description |
|---------|-------------|
| Geocoding | Convert addresses to coordinates |
| Reverse geocoding | Convert coordinates to structured locations |
| Nearby search | Search all 12 supported amenity categories in parallel |
| Route distance | Calculate Google Routes distance with global and regional travel modes |
| Geofencing | Filter and sort points by Haversine distance without an API call |
| Location scoring | Combine distance, density, and rating quality across every category |
| Structured export | Generate GeoJSON, CSV, and KML for nearby results and scores |
| Partial-result reporting | Preserve successful categories and report failed lookups |
| Caching | Reuse geocode, reverse-geocode, and nearby results in memory |
| JSON-RPC 2.0 | Return typed RPC results and errors for service integrations |

## Notes

- The cache is in memory and lasts for the lifetime of a `MapradarClient`; it does not persist across restarts.
- Mapradar does not add local rate limiting. Google Maps quotas and billing still apply.
- API keys are stripped from request error messages and hidden from CLI help output.
- See [CHANGELOG.md](CHANGELOG.md) for release and migration details.

## License

[MIT](LICENSE)

[Contributing](docs/CONTRIBUTING.md) | [Security](docs/SECURITY.md) | [Code of Conduct](docs/CODE_OF_CONDUCT.md)
