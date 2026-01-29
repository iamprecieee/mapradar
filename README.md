# Mapradar

[![Crates.io](https://img.shields.io/crates/v/mapradar.svg)](https://crates.io/crates/mapradar)
[![PyPI](https://img.shields.io/pypi/v/mapradar.svg)](https://pypi.org/project/mapradar/)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Turn addresses into coordinates and find nearby banks, hospitals, and other amenities.

---

## What It Does

Mapradar is a location intelligence library. Give it an address like "Shibuya, Tokyo" and it returns:

1. **Coordinates** - Latitude and longitude
2. **Nearby Services** - Banks, hospitals, schools, fuel stations within a radius
3. **Distance** - How far each service is from your location

Built in Rust. Works in both Python and Rust.

---

## Installation

<details open>
<summary><strong>Python</strong></summary>

```bash
uv add mapradar
```

</details>

<details>
<summary><strong>Rust</strong></summary>

```toml
[dependencies]
mapradar = { version = "0.1", default-features = false }
tokio = { version = "1", features = ["full"] }
```

> **Note:** Use `default-features = false` for pure Rust (no Python bindings).

</details>

<details>
<summary><strong>From Source</strong></summary>

**Python:**
```bash
git clone https://github.com/iamprecieee/mapradar
cd mapradar
uv add maturin
maturin develop
```

**Rust:**
```toml
[dependencies]
mapradar = { git = "https://github.com/iamprecieee/mapradar" }
```

</details>

---

## Usage

### Python

```python
import asyncio
from mapradar import MapradarClient, SearchQuery, ServiceType

async def main():
    client = MapradarClient("YOUR_GOOGLE_MAPS_API_KEY")
    
    # Find banks and hospitals near an address
    query = SearchQuery.from_address("Shibuya, Tokyo")
    intel = await client.fetch_intelligence(
        query,
        service_types=[ServiceType.Bank, ServiceType.Hospital],
        radius_km=3.0
    )
    
    print(f"Location: {intel.location.address}")
    print(f"Country: {intel.location.country}")
    for service in intel.nearby_services:
        print(f"  {service.name} - {service.distance_km:.2f} km")

asyncio.run(main())
```

<details>
<summary><strong>More Examples</strong></summary>

**Geocoding only:**
```python
location = await client.geocode("1 Marina, Lagos")
print(location.latitude, location.longitude, location.country)
```

**Reverse geocoding:**
```python
location = await client.reverse_geocode(6.4541, 3.3947)
print(location.address, location.country)
```

**JSON-RPC format (for microservices):**
```python
response = await client.geocode_rpc("Lekki, Lagos", id="req-123")
print(response.to_json())
```

</details>

---

### Rust

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

---

## Features

| Feature | Description |
|---------|-------------|
| **Geocoding** | Convert addresses to coordinates |
| **Reverse Geocoding** | Convert coordinates to addresses |
| **Nearby Search** | Find banks, hospitals, schools, etc. |
| **Parallel Fetching** | Search multiple service types at once |
| **Caching** | Automatic in-memory cache reduces API calls |
| **JSON-RPC 2.0** | Built-in format for microservice APIs |

---

## Service Types

| Type | Google Maps Category |
|------|---------------------|
| `Bank` | bank |
| `Hospital` | hospital |
| `School` | school |
| `Market` | supermarket |
| `Mall` | shopping_mall |
| `Restaurant` | restaurant |
| `FuelStation` | gas_station |
| `BusStop` | bus_station |
| `TrainStation` | train_station |
| `TaxiStand` | taxi_stand |
| `Landmark` | tourist_attraction |

---

## Configuration

| Variable | Description |
|----------|-------------|
| `GOOGLE_MAPS_API_KEY` | Your Google Maps API key. Enable Geocoding API and Places API. |

---

## FAQ

<details>
<summary>What APIs do I need enabled?</summary>

Enable these in Google Cloud Console:
- Geocoding API
- Places API (New)

</details>

<details>
<summary>Is there rate limiting?</summary>

Mapradar does not rate limit. Your Google Maps API quota applies. Use the built-in cache to reduce calls.

</details>

<details>
<summary>Does caching persist across restarts?</summary>

No. Cache is in-memory only. It persists for the lifetime of your `MapradarClient` instance.

</details>

---

## License

[MIT](LICENSE)

---

[Contributing](docs/CONTRIBUTING.md) | [Security](docs/SECURITY.md)
