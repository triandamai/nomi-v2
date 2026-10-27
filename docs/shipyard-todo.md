# Shipyard to-do: what Nomi's Monitor sheet needs

Nomi's Monitor sheet (`frontend/src/lib/components/project/MonitorPanel.svelte`) answers "how is my
app being used, and is it healthy?". Today it can only show what Shipyard's integration API gives
it, and it never shows a number it can't back. This list is what Shipyard has to add so the
sheet can show the rest. Each item says what Nomi needs, at what resolution, and the call Nomi
would make. Nothing in Nomi needs redesigning when these land: the sheet already has places for
them.

## What Shipyard gives Nomi today

| Call | What comes back | What Monitor does with it |
|---|---|---|
| `GET …/envs/:env/usage` | A live snapshot per container: CPU %, memory used/limit/%, network and disk bytes | Samples it every 5 s while the sheet is open and charts those samples ("Since you opened this") |
| `GET …/envs/:env/services/:service/logs?tail=&since=` | The newest log lines, as plain text | Shows, filters (warnings and errors are guessed from the words in each line) and follows them |
| Webhook events (`service.crashed`, `service.resource_alert`, `publish.*`) | One event each | Lists them as rows |
| `GET …/envs/:env` | Services with replicas, running count, last deployment | "Running · up since …" and "web: 1 of 1 running" |

## What's missing

### 1. Traffic: visitors, requests, errors, response time

Nomi needs, per environment, bucketed over time:

- **Unique visitors** per bucket and for the whole range (unique by a privacy-preserving key,
  e.g. a daily-rotating hash of IP + user agent; Nomi never needs the raw IPs).
- **Requests** per bucket.
- **Errors**: 5xx count and 4xx count, separately, per bucket.
- **Response time**: p50 and p95 per bucket, in milliseconds, measured at Shipyard's proxy.

Ranges and resolution Nomi will ask for:

| Range | Bucket |
|---|---|
| 1 hour | 1 minute |
| 24 hours | 1 hour |
| 7 days | 1 day |

Plus the same totals for the previous range, so the sheet can say "+18% vs yesterday".

### 2. Resource history

CPU % and memory % (and memory bytes with the limit) per container, over time, so the charts
show the last hour, day or week rather than only what was sampled while the sheet was open:

| Range | Resolution |
|---|---|
| 1 hour | 1 minute |
| 24 hours | 5 minutes |
| 7 days | 1 hour |

Keep at least 7 days.

### 3. Structured log lines

Today a log line is a string. Nomi would like `{ "at": "…", "level": "info|warn|error", "message": "…",
"service": "web" }`, with the level taken from the app's own output where it's structured (JSON
logs) and from the stream (stderr as `warn` at least) otherwise. Nomi then stops guessing levels
from words.

## The call Nomi would make

One endpoint for both traffic and resources, so the sheet makes one request per range change:

```
GET /openapi/v1/integration/orgs/{org}/apps/{app}/envs/{env}/metrics?range=24h&step=1h
```

```json
{
  "data": {
    "range": "24h",
    "step": "1h",
    "buckets": [
      {
        "at": "2026-10-26T09:00:00Z",
        "visitors": 34,
        "requests": 812,
        "errors_5xx": 2,
        "errors_4xx": 11,
        "p50_ms": 96,
        "p95_ms": 288,
        "containers": [
          { "service": "web", "container_id": "…", "cpu_percent": 21.4, "memory_percent": 58.1, "memory_bytes": 312475648, "memory_limit_bytes": 536870912 }
        ]
      }
    ],
    "totals": { "visitors": 412, "requests": 9811, "errors_5xx": 38, "errors_4xx": 120, "p50_ms": 120, "p95_ms": 310 },
    "previous_totals": { "visitors": 349, "requests": 8702, "errors_5xx": 41, "errors_4xx": 98, "p50_ms": 131, "p95_ms": 342 }
  },
  "error": null
}
```

Same `{ data, error }` envelope and `manage` key as the rest of the integration API. A field
Shipyard can't measure yet is `null`, not `0`, so Nomi can tell "nothing happened" from "not
measured".

## How it relates to alerts

`service.resource_alert` already tells Nomi when CPU or memory crossed a limit. With history,
Nomi will draw the threshold on the chart and mark when the alert fired. An error-rate alert
(`service.error_rate_alert`, e.g. 5xx above 5% for 5 minutes) would let Nomi tell the project's
admins about a broken app the way it does for crashes today.

## In Nomi, now that the metrics call exists

Items 1 and 2 landed as the one `metrics` call above:

- `nomi-shipyard`: `Shipyard::metrics(org, app, env, range, step)`; the step follows the range
  (1h → 1m, 24h → 5m, 7d → 1h) unless one is asked for.
- `nomi-server`: `GET /api/projects/:id/deploy/metrics?environment=&range=1h|24h|7d` passes it
  through, any project member allowed (like usage and logs).
- Monitor: the 1h / 24h / 7d switch, four tiles (visitors, requests, 5xx, p95) against the range
  before, a small chart per tile, and CPU and memory from history (the sheet's own samples only
  until Shipyard has some). Unmeasured values read "Not measured" and leave gaps in the charts.

Item 3 (structured log lines) is still open.
