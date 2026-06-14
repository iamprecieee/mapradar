# Security Policy

## Threat Model

Mapradar protects API keys and ensures secure RPC communication for location intelligence.

### In Scope

| Threat | Protection |
|--------|------------|
| API Key Leakage | No keys are hardcoded; environment variables required |
| Network Sniffing | HTTPS is used for all Google Maps API requests |

### Out of Scope

- Root/administrator access
- Physical access attacks
- Social engineering

---

## Implementation

| Component | Choice | Rationale |
|-----------|--------|-----------|
| Transport | HTTPS via reqwest | Secures API requests to external providers |
| Storage | In-memory Cache (Moka) | Fast and prevents persistent data leakage |

---

## Known Limitations

1. API keys are sent over HTTPS to Google Maps but reside in memory during execution.

---

## Vulnerability Disclosure

**Email:** emmypresh777@gmail.com

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
| reqwest | HTTP client |

Advisories tracked via `cargo-audit`.
