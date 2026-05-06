# whisper-sync

Bridge Murmurer's ambient whisper protocol to PLATO's room infrastructure.

Fleet agents exchange short, low-priority "whispers" via PLATO rooms instead of file-based bottles.

## What is a Whisper?

- **Short** — single message, compact payload
- **Ambient** — non-blocking, fire-and-forget
- **Low priority** — best-effort delivery only
- **Ephemeral** — expires via TTL (~30 min)

## Whisper Types

| Type | Description |
|------|-------------|
| `status` | Health/load heartbeat |
| `discovery` | New service announcement |
| `help` | Help request to fleet |
| `insight` | Flash insight with confidence |
| `trust` | Trust score update |
| `alert` | Attention-worthy alert |

## Installation

```bash
cargo install --path .
```

## Usage

```bash
# Send a status whisper
whisper-sync send --from agent1 --whisper-type status --content '{"health": 0.95, "load": 0.3}'

# Send a discovery whisper
whisper-sync send --from agent1 --whisper-type discovery --content '{"service": "plato-kernel", "endpoint": "http://localhost:8848"}'

# Listen for whispers
whisper-sync listen --agent-id my-agent --interval 10
```

## Architecture

```
src/
  lib.rs             — public API
  main.rs            — CLI entry
  whisper.rs         — Whisper struct and WhisperType
  plato_transport.rs — HTTP client for PLATO
  delivery.rs        — unicast/multicast/broadcast
  inbox.rs           — poll and process whispers
  filter.rs          — TTL + relevance filtering
```

## PLATO Integration

Whispers are submitted to the `fleet_whispers` PLATO room as JSON tiles tagged with `whisper` and `type:<variant>`. Client-side TTL filtering enforces 30-minute expiration.

## License

MIT
