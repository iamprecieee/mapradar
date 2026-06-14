# Mapradar

[![Crates.io](https://img.shields.io/crates/v/mapradar.svg)](https://crates.io/crates/mapradar)
[![PyPI](https://img.shields.io/pypi/v/mapradar.svg)](https://pypi.org/project/mapradar/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Mapradar is a location intelligence library that converts addresses to coordinates, finds nearby services (banks, hospitals), and calculates travel distances via Google Maps APIs.

---

## Installation

<details open>
<summary><strong>Primary Method</strong></summary>

**Python**
```bash
uv add mapradar
```

**Rust**
```toml
[dependencies]
mapradar = { version = "0.3.0", default-features = false }
tokio = { version = "1", features = ["full"] }
```
> **Note:** Use `default-features = false` for pure Rust (no Python bindings).

**CLI Tool**
```bash
cargo install mapradar
```

</details>

<details>
<summary><strong>From Source</strong></summary>

```bash
git clone https://github.com/iamprecieee/mapradar
cd mapradar

# Python
uv add maturin
maturin develop

# Rust
cargo build
```

</details>

---

## Usage

**CLI Commands**

```bash
# Geocode an address
mapradar geocode "1600 Amphitheatre Parkway, Mountain View, CA"

# Find nearby amenities
mapradar nearby --lat 6.6018 --lng 3.3515 --radius 500 --type bank,school

# Distance calculation
mapradar distance --origin-addr "Shibuya, Tokyo" --dest-addr "Shinjuku, Tokyo" --mode drive
```

<details>
<summary><strong>Library Usage</strong></summary>

**Python Example**
```python
import asyncio
from mapradar import MapradarClient, SearchQuery, ServiceType, TravelParameters

async def main():
    client = MapradarClient("YOUR_GOOGLE_MAPS_API_KEY")
    
    # Calculate travel distance between two points
    params = TravelParameters(
        origin_address="Shibuya, Tokyo",
        destination_address="Shinjuku, Tokyo"
    )
    distance = await client.calculate_travel_distance(params)
    print(f"Distance: {distance:.2f} km")

    # Find banks and hospitals near an address
    query = SearchQuery.from_address("Shibuya, Tokyo")
    intel = await client.fetch_intelligence(
        query,
        service_types=[ServiceType.Bank, ServiceType.Hospital],
        radius_km=3.0
    )
    print(f"Location: {intel.location.address}")

asyncio.run(main())
```

**Rust Example**
```rust
use mapradar::client::MapradarClient;
use mapradar::models::{SearchQuery, ServiceType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = MapradarClient::new("YOUR_API_KEY".to_string());
    let location = client.geocode_async("Times Square, NYC").await?;
    println!("{}, {} ({})", location.latitude, location.longitude, location.country);
    Ok(())
}
```

</details>

---

## Features

| Feature | Description |
|---------|-------------|
| **Geocoding** | Convert addresses to coordinates |
| **Reverse Geocoding** | Convert coordinates to addresses |
| **Nearby Search** | Find banks, hospitals, schools, etc. |
| **Distance Fetching** | Fast travel/Haversine distance with address fallthrough |
| **Parallel Fetching** | Search multiple service types at once |
| **Caching** | Automatic in-memory cache reduces API calls |
| **JSON-RPC 2.0** | Built-in format for microservice APIs |

---

## Configuration

| Option | Default | Description |
|--------|---------|-------------|
| `GOOGLE_MAPS_API_KEY` | None | Your Google Maps API key. Enable Geocoding API, Places API (New), and Routes API. |
| `max_results` | 20 | Maximum number of places returned per category. |
| `radius` | 5.0 | Search radius in kilometers. |
| `--mode` | `drive` | Transit mode for distance calculation (`drive`, `okada`, `keke`, `danfo`, `brt`). |

---

## FAQ

<details>
<summary>What APIs do I need enabled?</summary>

Enable these in Google Cloud Console:
- Geocoding API
- Places API (New)
- Routes API (V2)

</details>

<details>
<summary>Is there rate limiting?</summary>

Mapradar does not rate limit locally. Your Google Maps API quota applies. Use the built-in Moka cache to reduce outbound calls.
</details>

<details>
<summary>Does caching persist across restarts?</summary>

No. Cache is in-memory only. It persists for the lifetime of your `MapradarClient` instance to ensure maximum security.
</details>

---

## License

[MIT](LICENSE)

---

[Contributing](docs/CONTRIBUTING.md) | [Security](docs/SECURITY.md) | [Code of Conduct](docs/CODE_OF_CONDUCT.md)
