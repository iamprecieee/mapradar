# Changelog

Format based on [Keep a Changelog](https://keepachangelog.com/).

## [Unreleased]

## [0.6.0] - 2026-08-14

### Breaking Changes

- `CategoryScore.nearest_distance_km` is now `Option<f64>` in Rust and `Optional[float]` in Python. A category with no matching amenities returns `None`/`null` instead of `f64::MAX`.
- `scoring::compute_location_score` now accepts the requested service types: `compute_location_score(&intelligence, &service_types, radius_km)`. This keeps categories with zero results in the score.
- `LocationIntelligence` now includes `failed_lookups`. Rust callers that construct the struct with a literal must initialize the field; `LocationIntelligence::new` remains compatible.
- Removed the unused `GeoError::ConfigError` variant. Use `GeoError::InvalidInput` for caller-supplied values that fail validation.

### Added

- Canonical service-type support through `ServiceType::ALL`, `slug`, `google_place_type`, `Display`, and `FromStr`.
- Partial-result reporting through `LocationIntelligence.failed_lookups` and CLI warnings.
- Human-readable score output, now the default for `mapradar score`; text output can also be written to `.txt` files.
- Search-center features in intelligence GeoJSON and KML exports, plus the location and complete category breakdown in score exports.
- Cache regression coverage for lookup reuse, normalization, parameter separation, and cache isolation.

### Changed

- Location scoring now searches and scores all 12 supported service categories. Missing categories score zero instead of being omitted from the average.
- The CLI validates service types, travel modes, output formats, and output-file extensions before network requests. Unknown values now return an error instead of silently falling back.
- Exported category names use canonical lowercase slugs that can be passed back to the CLI.
- Requests use a 10-second connection timeout and a 30-second overall timeout.
- When only some nearby categories fail, successful categories are returned with failure details. When all fail, the error reports every failed category.

### Fixed

- Prevented Google Maps API keys from appearing in request errors or CLI `--help` output.
- Preserved JSON-RPC serialization errors instead of returning a misleading successful `null` result.
- Invalid JSON assigned to a Python `JsonRpcResponse.result` now raises `ValueError` instead of being silently discarded.
- Included train stations, taxi stands, and landmarks in location scoring.
- Recomputed distances before sorting points, preventing stale values from a different center from affecting order.
- Skipped malformed address components that do not contain a `types` array while preserving valid geocode data.
- Escaped XML-sensitive content in KML and retained the location, overall score, and category breakdown across score export formats.
- Rounded category average ratings to two decimals to remove floating-point representation noise.
- Guarded scoring against non-positive radii so results remain finite.

## [0.5.0] - 2026-06-17

### Added
- **Distance Matrix / Geofencing**: Fast zero-API-call Point-in-Radius checking via `is_within_radius`, `filter_within_radius`, and `sort_by_distance`.
- **Location Scoring**: Compute automated composite quality scores for locations using `score_location` combining Distance, Density, and Quality.
- **Structured Export**: Export nearby results and location scores via new `export` module (GeoJSON, CSV, KML) to power mapping platforms. Added `--format` flag to `nearby` and `score` CLI subcommands.

---

## [0.4.2] - 2026-06-15

### Fixed
- Enabled `abi3-v39` for `pyo3` to build universal Python wheels for PyPI (compatible with Python 3.9+)

---

## [0.4.1] - 2026-06-15

### Added
- International transit mode support for India and Indonesia (`auto`, `rickshaw`, `tuktuk`, `ojek`, `bajaj`, `becak`, `angkot`, `busway`, `metro`, `local_train`)

### Fixed
- Fixed bug where `ServiceType::Pharmacy` was missing from parsing, which caused `--type pharmacy` queries to fallback to landmarks (`tourist_attraction`)

---

## [0.4.0] - 2026-06-14

### BREAKING CHANGES
- **Rust API (`JsonRpcResponse`)**: The `result` field type has been changed from `Option<String>` to `Option<serde_json::Value>` to fix a double-encoding issue where JSON payloads were improperly stringified inside JSON-RPC responses. Consumers of the Rust library must update their struct bindings. Python users are unaffected as the object serialization remains dynamic.
- **Dependency Bumps (`pyo3`)**: Upgraded `pyo3` and `pyo3-async-runtimes` from `0.27.2` to `0.29.0` to patch out-of-bounds read and missing `Sync` bounds vulnerabilities. Code compiling against the Rust extension feature `python` will require `pyo3 = "0.29.0"`.

### Added
- JSON-RPC 2.0 support natively using `serde_json::Value`
- Routes API (V2) for multiple travel modes (`okada`, `keke`, `danfo`, `brt`) via `--mode`
- Places API (New) integration across backend queries

### Changed
- Standardized error handling and result formatting globally
- Descriptive and explicit variable naming (removing all single-character variables and closures)
- Purged all flagged vulnerability dependencies via ecosystem bump (`cargo update`)

### Fixed
- Cache key fingerprints properly include `max_results` to prevent short-circuit cache truncation
- JSON-RPC result serialization strictly avoids double-encoding string-in-JSON bugs

---

## [0.3.0] - 2026-04-08

### Added
- Travel distance calculation

---

## [0.2.0] - 2026-04-01

### Added
- Initial CLI and Place Details integration

[Unreleased]: https://github.com/iamprecieee/mapradar/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/iamprecieee/mapradar/compare/v0.5.0...v0.6.0
