# Changelog

Format based on [Keep a Changelog](https://keepachangelog.com/).

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
