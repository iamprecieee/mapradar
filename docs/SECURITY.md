# Security Policy

## Threat Model

Mapradar is a location intelligence library that interacts with external APIs. It protects **API credentials** and ensures **data integrity** for developers building location-based applications.

### In Scope

| Threat | Protection |
|--------|------------|
| API key exposure | Keys stored in environment variables, never logged |
| Invalid API responses | Structured error handling prevents crashes |
| Data injection | All inputs validated before API calls |

### Out of Scope

- Google Maps API security (managed by Google)
- Network-level attacks (MITM, DNS spoofing)
- Physical access to development machines

---

## Implementation

| Component | Choice | Rationale |
|-----------|--------|-----------|
| HTTP Client | `reqwest` with TLS | Industry-standard secure HTTP |
| Error Handling | `thiserror` | Type-safe error propagation |
| Async Runtime | `tokio` | Non-blocking I/O for performance |

---

## Known Limitations

1. API keys must be stored securely by the user (use `envcipher` or similar)
2. Rate limiting depends on Google Maps API quotas

---

## Vulnerability Disclosure

**Email:** security@example.com

Do not file public issues for security vulnerabilities.

| Stage | Timeline |
|-------|----------|
| Acknowledgment | 24 hours |
| Assessment | 72 hours |
| Fix | Severity-dependent |

---

## Dependencies

| Library | Purpose |
|---------|---------|
| `reqwest` | HTTP client with TLS |
| `serde` / `serde_json` | JSON serialization |
| `thiserror` | Error type definitions |
| `pyo3` | Python bindings |

Advisories tracked via `cargo audit`.
