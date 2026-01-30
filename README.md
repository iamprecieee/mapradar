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

## API Reference

### MapradarClient

<details open>
<summary><strong>Python</strong></summary>

```python
client = MapradarClient("YOUR_API_KEY")
```

#### Core Methods

| Method | Parameters | Returns |
|--------|-----------|---------|
| `geocode(address)` | `address: str` | `GeoLocation` |
| `reverse_geocode(lat, lng)` | `latitude: float`, `longitude: float` | `GeoLocation` |
| `search_nearby(...)` | `lat`, `lng`, `service_type`, `radius_meters`, `max_results` | `list[NearbyService]` |
| `fetch_intelligence(...)` | `query`, `service_types`, `radius_km=5.0`, `max_results_per_type=5` | `LocationIntelligence` |

#### JSON-RPC Methods

| Method | Extra Parameter |
|--------|-----------------|
| `geocode_rpc(address, id?)` | `id: str = "1"` |
| `reverse_geocode_rpc(lat, lng, id?)` | `id: str = "1"` |
| `search_nearby_rpc(..., id?)` | `id: str = "1"` |
| `fetch_intelligence_rpc(..., id?)` | `id: str = "1"` |

</details>

<details>
<summary><strong>Rust</strong></summary>

```rust
let client = MapradarClient::new("YOUR_API_KEY".to_string());
```

#### Core Methods (async)

| Method | Parameters | Returns |
|--------|-----------|---------|
| `geocode_async(address)` | `address: &str` | `Result<GeoLocation, GeoError>` |
| `reverse_geocode_async(lat, lng)` | `lat: f64`, `lng: f64` | `Result<GeoLocation, GeoError>` |
| `search_nearby_async(...)` | `lat`, `lng`, `service_type`, `radius_meters`, `max_results` | `Result<Vec<NearbyService>, GeoError>` |
| `fetch_intelligence_async(...)` | `query`, `service_types`, `radius_km`, `max_results_per_type` | `Result<LocationIntelligence, GeoError>` |

#### RPC Helper

| Method | Returns |
|--------|---------|
| `rpc_response(id, result)` | `JsonRpcResponse` |

</details>

---

### SearchQuery

<details open>
<summary><strong>Python</strong></summary>

| Constructor | Description |
|-------------|-------------|
| `SearchQuery.from_address(address)` | Create query from address string |
| `SearchQuery.from_coordinates(lat, lng)` | Create query from coordinates |

</details>

<details>
<summary><strong>Rust</strong></summary>

| Constructor | Description |
|-------------|-------------|
| `SearchQuery::from_address(address: String)` | Create query from address string |
| `SearchQuery::from_coordinates(lat: f64, lng: f64)` | Create query from coordinates |

</details>

---

### Response Types

<details open>
<summary><strong>Python</strong></summary>

#### GeoLocation

| Field | Type |
|-------|------|
| `address` | `str` |
| `latitude` | `float` |
| `longitude` | `float` |
| `city` | `str \| None` |
| `state` | `str \| None` |
| `country` | `str` |

#### NearbyService

| Field | Type |
|-------|------|
| `name` | `str` |
| `service_type` | `ServiceType` |
| `latitude` | `float` |
| `longitude` | `float` |
| `distance_km` | `float` |
| `address` | `str \| None` |
| `rating` | `float \| None` |
| `place_id` | `str \| None` |

#### LocationIntelligence

| Field | Type |
|-------|------|
| `location` | `GeoLocation` |
| `nearby_services` | `list[NearbyService]` |
| `total_services_found` | `int` |

#### JsonRpcResponse

| Field | Type |
|-------|------|
| `jsonrpc` | `str` |
| `result` | `str \| None` |
| `error` | `JsonRpcError \| None` |
| `id` | `str` |

</details>

<details>
<summary><strong>Rust</strong></summary>

#### GeoLocation

| Field | Type |
|-------|------|
| `address` | `String` |
| `latitude` | `f64` |
| `longitude` | `f64` |
| `city` | `Option<String>` |
| `state` | `Option<String>` |
| `country` | `String` |

#### NearbyService

| Field | Type |
|-------|------|
| `name` | `String` |
| `service_type` | `ServiceType` |
| `latitude` | `f64` |
| `longitude` | `f64` |
| `distance_km` | `f64` |
| `address` | `Option<String>` |
| `rating` | `Option<f32>` |
| `place_id` | `Option<String>` |

#### LocationIntelligence

| Field | Type |
|-------|------|
| `location` | `GeoLocation` |
| `nearby_services` | `Vec<NearbyService>` |
| `total_services_found` | `usize` |

#### JsonRpcResponse

| Field | Type |
|-------|------|
| `jsonrpc` | `String` |
| `result` | `Option<String>` |
| `error` | `Option<JsonRpcError>` |
| `id` | `String` |

</details>

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
