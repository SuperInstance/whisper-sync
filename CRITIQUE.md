# Critical Evaluation: whisper-sync

## Is "Whisper via PLATO" Better Than File-Based Bottles?

**Better when:**
- Fleet agents already use PLATO rooms — single infrastructure
- HTTP-based access (no filesystem coordination needed)
- Real-time polling vs polling for file changes
- Easier to trace/debug (HTTP logs vs file diffs)

**Worse when:**
- PLATO is down — file-based bottles are independent services
- High-volume whispers — HTTP overhead per whisper vs batch file writes
- Network partition — local files work offline, PLATO does not

**Verdict:** Better for HTTP-native fleets. Worse for filesystem-native or offline-first scenarios.

## Failure Modes

| Failure | Impact | Mitigation |
|---------|--------|------------|
| PLATO down | Whispers undelivered, agents isolated | Murmurer falls back to files |
| Whisper lost in transit | No retry, fire-and-forget | Acceptable for ambient/low-priority |
| Duplicate delivery | Same whisper processed twice | Client deduplication by whisper ID |
| Clock skew | TTL filtering inaccurate | Use server timestamps from PLATO |
| Polling race | Miss whispers between polls | Shorter interval, accept small gaps |

## Is 30-Minute TTL Appropriate?

**Too long for:** Status heartbeats (should expire in 2-5 min to detect agent death faster)
**Too short for:** Insight flashes (might want to retain for hours)
**Appropriate for:** General ambient chatter, discovery, help requests

**Recommendation:** Make TTL configurable per whisper type, not global. Status: 5 min, Insight: 2 hours, Discovery: 30 min.

## What Would Make whisper-sync Exceptional?

1. **WebSocket upgrade path** — PLATO could push whispers instead of polling. Real-time delivery, no polling overhead.

2. **Adaptive TTL** — Whisper type dictates TTL. Status whispers auto-expire faster. Insight whispers persist longer.

3. **Whisper routing hints** — "Send to nearest PLATO replica" for geo-distributed fleets.

4. **Delivery receipts** — Optional ACK when recipient receives unicast. Turns ambient into reliable-ish.

5. **Whisper signatures** — Agent identity in whisper header, verifiable by recipients.

6. **Rate limiting** — Prevent spam. Max N whispers/minute per agent.

7. **Whisper threading** — Reply-to support for conversations, not just broadcasts.

## Comparison: whisper-sync vs Murmurer Original

| Aspect | Murmurer (original) | whisper-sync |
|--------|---------------------|--------------|
| Transport | File-based bottles | HTTP/PLATO rooms |
| Persistence | Filesystem | PLATO tiles |
| Delivery | File polling | Room polling |
| Real-time | Latency via polling | Latency via polling |
| Offline | Works (local files) | Fails (needs PLATO) |
| Debugging | File inspection | HTTP logs |
| Scalability | File system limits | PLATO capacity |

**Key insight:** whisper-sync trades independence for integration. It's not strictly better — it's *different*. For fleets already running PLATO, it's simpler. For fleets that need offline resilience, Murmurer's file approach wins.

## Conclusion

whisper-sync is *adequate* for ambient inter-agent communication over PLATO. It's *exceptional* if PLATO is already your fleet's backbone and you want unified infrastructure. It's *inadequate* if your fleet needs offline resilience or sub-second delivery.

**Next steps if exceptional:**
- Add WebSocket support to PLATO for push-based delivery
- Configurable per-type TTLs
- Agent signing/verification
- Rate limiting
