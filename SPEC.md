# whisper-sync Specification

## Overview

**whisper-sync** bridges Murmurer's ambient whisper protocol to PLATO's room infrastructure. Fleet agents exchange short, low-priority "whispers" via PLATO rooms instead of file-based bottles.

## What is a Whisper?

A whisper is:
- **Short** — single message, compact payload (<1KB ideal)
- **Ambient** — non-blocking, fire-and-forget delivery
- **Low priority** — no delivery guarantees, best-effort only
- **Ephemeral** — expires via TTL (default 30 min), not permanent storage

Whispers are **status/communication** messages, distinct from PLATO tiles which encode **knowledge**.

## Whisper Taxonomy

```rust
enum WhisperType {
    Status { health: f64, load: f64 },
    Discovery { service: String, endpoint: String },
    Help { question: String, tags: Vec<String> },
    Insight { summary: String, confidence: f64, source_theorem: Option<String> },
    Trust { agent: String, trust_score: f64 },
    Alert { severity: String, message: String },
}
```

## Delivery Modes

| Mode       | Target                          | Use Case                          |
|------------|----------------------------------|-----------------------------------|
| Unicast    | Specific agent (`to` field)      | Direct status/report to one agent |
| Multicast  | Room members                     | Help request, Insight broadcast    |
| Broadcast  | All fleet agents                 | Alert, Discovery announcements    |

## PLATO Room Design

**One room: `fleet_whispers`**

Rationale: Single room simplifies infrastructure. Tag-based filtering handles separation. Overhead of multiple rooms not justified for low-volume ambient chatter.

### Whisper → PLATO Tile Mapping

Whispers are stored as JSON tiles with tags:
```
tag: whisper
tag: type:<status|discovery|help|insight|trust|alert>
to: <agent_id>   (optional, for unicast)
```

TTL is **client-enforced**: whispers older than 30 minutes are ignored on read.

## PLATO Transport API

Base URL: `http://localhost:8847`

| Operation | Method | Endpoint                          |
|-----------|--------|-----------------------------------|
| Submit whisper | POST | `/room/fleet_whispers/submit` |
| Poll inbox | GET | `/room/fleet_whispers/tiles?since=<timestamp>&tag=whisper` |

## Inbox Polling

- Poll interval: configurable (default 10 seconds)
- `since` param uses last-polled timestamp
- Filter by `to` field for unicast messages targeting this agent
- Client-side TTL enforcement on read

## Whisper vs PLATO Tile

| Aspect          | Whisper                          | PLATO Tile                        |
|-----------------|----------------------------------|-----------------------------------|
| Purpose         | Status/communication             | Knowledge/assertion storage       |
| Lifetime        | Ephemeral (TTL ~30 min)          | Persistent                        |
| Priority        | Low                              | Medium-High                       |
| Delivery        | Fire-and-forget                  | Explicit acknowledgment           |
| Volume          | High (ambient chatter)           | Low ( curated knowledge)          |

## Architecture

```
src/
  lib.rs           — public API
  main.rs          — CLI entry point
  whisper.rs       — Whisper struct and WhisperType enum
  plato_transport.rs — HTTP client for PLATO room server
  delivery.rs      — unicast/multicast/broadcast delivery
  inbox.rs         — receive and process whispers from PLATO room
  filter.rs        — filter whispers: unexpired + relevant-to-me
```

## CLI Usage

```bash
# Send a whisper
whisper-sync send --type status --content '{"health": 0.95, "load": 0.3}'

# Send a discovery whisper
whisper-sync send --type discovery --content '{"service": "plato-kernel", "endpoint": "http://localhost:8848"}'

# Start inbox listener
whisper-sync listen --agent-id <AGENT_ID> --interval 10
```

## Dependencies

- `reqwest` — HTTP client
- `tokio` — async runtime
- `serde` / `serde_json` — serialization
- `chrono` — timestamps
- `clap` — CLI args
- `tracing` — logging

## Test Plan

1. **Unit tests**: Whisper serialization, TTL filtering, tag extraction
2. **Integration tests**: Two agents exchanging whispers through PLATO room

## Open Questions

- [ ] Should whispers be signed (agent identity verification)?
- [ ] Rate limiting per agent to prevent spam?
- [ ] Whisper deduplication (idempotency key)?
