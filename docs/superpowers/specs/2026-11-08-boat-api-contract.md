# boat.dev Sandbox REST API contract (from docs, 2026-11-08)

Sources: docs.boat.dev/api/v1.md, /api/reference/{sandboxes,agent}/*.md, /limits.md. No live calls made. "UNSPECIFIED" = docs silent.

## 1. Global

| Item | Value |
|---|---|
| Base URL | `https://boat.dev/api/v1` |
| Auth | `Authorization: Bearer boat_<key>` (legacy `box_` prefix still works). Per-sandbox keys only work on their own sandbox. Scopes exist (e.g. `exec`, `file.write`) |
| Billing wallet (optional) | header `X-Boat-Org` or query `org` (`team_...` id, org name, or `personal`); query wins. 409 `ambiguous_org` if two orgs share a name |
| Sandbox id | `^bx_[23456789abcdefghjkmnpqrstuvwxyz]{8}$` |
| Content type | JSON in/out, except stream endpoints (`application/x-ndjson`) |
| Success envelope | `{ok:true, type:"<kind>", ...}` |
| Rate limit (IP) | 10,000 req / 60 s / IP. Headers `RateLimit-Policy`, `RateLimit` on every response |
| `Retry-After` header | ONLY on `IP_RATE_LIMIT_EXCEEDED`; other 429s put the wait in the body |

### Error envelope (all endpoints)
```
{ok:false, type:"sandbox.error", status:int, code:str, message:str,
 error:{code, message, status, details?:{...}}, requestId:"req_..."}
```
`error.details` may hold `provider`, `setupUrl`, `retryable`. Client rule: key off top-level `code` + HTTP status.

| HTTP | Codes (global list) |
|---|---|
| 400 | invalid_json, invalid_name, invalid_path, invalid_timeout, invalid_env, unknown_environment, no_changes, machine_not_running, trial_auto_stop_required, ... |
| 401 | unauthorized |
| 402 | billing_required, team_member_cap_reached |
| 403 | trial_machine_class_not_allowed, org_suspended, environment_forbidden, machine_class_plan_required, forbidden |
| 404 | not_found, snapshot_not_found |
| 409 | account_not_ready, sandbox_resuming, sandbox_stopping, sandbox_securing, sandbox_starting, sandbox_not_ready, sandbox_running_stop_first, resume_failed, snapshots_disabled, snapshot_data_expired, type_too_small, idempotency_in_progress, idempotency_key_reused, agent_outdated, monitoring_off, ambiguous_org |
| 429 | rate_limited, limit_reached, member_limit_reached, trial_compute_limit_reached, IP_RATE_LIMIT_EXCEEDED |
| 500 | http_500, `*_failed` |
| 502 | sandbox_direct_failed (exec docs spell it `boat_direct_failed`; treat both as same) |
| 503 | no_ready_machine, out_of_capacity, browser_not_ready, ssh_host_key_unavailable |

### Retry hints (exact fields)
| Where | Field |
|---|---|
| `IP_RATE_LIMIT_EXCEEDED` (429) | body `retryAfter` (seconds) + `Retry-After` header |
| `rate_limited` (429) | body names exhausted `window` (`minute`/`hour`/`day`) and `limit`; `starts` object = remaining per window; `startLimitsWithWallet` (null when extra starts unavailable). Exact nesting (top-level vs `error.details`): UNSPECIFIED. No wait-seconds field documented |
| 503 | body `retryAfterSeconds` (documented in overview; which 503 codes carry it: UNSPECIFIED) |
| `nextStartAt` | NOT documented anywhere. Treat as UNSPECIFIED; do not rely on it |
| `sandbox_direct_failed` | `error.details.retryable` (bool) |
| exec stream `error` frame | `retryable` (true = command never ran) |
| Proactive check | `GET /limits` returns `canStart`, `starts.minute.remaining` (shape beyond that UNSPECIFIED) |
| 429 `limit_reached` / `member_limit_reached` | concurrency caps; no retry hint documented |

### Idempotency-Key
- Header `Idempotency-Key` (string, max 255; use UUID) on `POST /sandboxes` and `POST /sandboxes/{id}/fork`. Not on resume/stop/exec.
- Scope: account + key + request body. Retention 24 h.
- Same key+body: returns the original sandbox (response `status` may be `ready` instead of `provisioning`).
- First still in flight: `409 idempotency_in_progress` (retry shortly, same key).
- Same key, different body: `409 idempotency_key_reused`.
- Creation failed before sandbox existed: key released after ~2 min.
- Timeouts/5xx: safe to retry with same key. Without header, create is not idempotent.

### Sandbox object (shared by list/get/create/stop/resume)
Required: `id, name, state, desktopAvailable, snapshotAvailable`. Others nullable/optional.

| Field | Type / values |
|---|---|
| `state` | `init` `provisioning` `provisioned` `cloning` `ready` `idle` `running` `archiving` `archived` `error` `cancelled` |
| `error` | string\|null |
| `health` | `ok` \| `degraded`; `healthReason`, `degradedSince` |
| `type` | `small` \| `default` \| `large` \| `xlarge`; `vcpu`, `memoryGB` int; `billingMultiplier` num |
| `labels` | object string->string |
| `url`, `ip`, `sshEndpoint` (host:port), `subdomain` | string\|null |
| `desktopUrl` | string\|null (secret-bearing; redact) |
| `ttlSeconds` | int\|null; `archiveAfter`, `stoppedAt`, `dataExpiredAt` date-time\|null |
| `createdAt`, `updatedAt` | date-time |
| `hydrated` bool; `hydration` | null or `{phase, entriesDone, entriesTotal, percent}`; phase in `dvol sysdelta images hydrating reconciling degraded stuck` (GET only) |
| `snapshots`, `monitoring` | bool; `snapshotCompletedAt`, `snapshotVerifiedAt`, `lastSnapshotAttemptAt` |
| `lastSnapshotStatus` | `queued` `in_progress` `completed` `failed` `cancelled` \| null |
| `access` | `owner` `use` `view`; `visibility` `shared` `private` null; `team` null or `{id,name}` |
| `setupStatus` | `pending` `running` `done` `failed` \| null; `setupError` |
| `environment`, `environmentVersion` | string / int |
| `lastReboot` | null or `{at, reason: crash\|clean\|unknown}` |
| `stop` | null or StopOperation (below) |
| `cost` | `{since, seconds, dollars}` only with `include=cost` on list |

State notes: `cancelled` is terminal; first GET returns only `{id,state,error}`, later GETs 404. Stopped = `archiving` -> `archived`. "Usable for exec" states: `ready`/`idle`/`running` (exec docs say sandbox in `error` or `stopped` -> 409 `sandbox_not_ready`; the word `stopped` is not in the state enum, UNSPECIFIED mapping).

StopOperation: `{id:"stop_<hex>", status: pending|failing|completed|superseded, requestedAt, lastAttemptAt, error, endedAt, forced?:bool}`.

## 2. Sandboxes

### POST /sandboxes  (create)
Headers: `Idempotency-Key`, `X-Boat-Org` (optional). Body optional; all fields optional.

| Field | Type | Constraints |
|---|---|---|
| `type` | str | small/default/large/xlarge, default `default` |
| `ttlSeconds` | int\|null | 1..2592000, default 3600, null = no auto-stop (trial max 2h) |
| `labels` | obj str->str | max 20; key `[A-Za-z0-9][A-Za-z0-9._/-]{0,62}`; value <=256 bytes. **This is the tagging mechanism**; filter via `GET /sandboxes?label=k=v` |
| `name` | str | max 120, display name (not unique; default "Sandbox YYYY-MM-DD HH:MM") |
| `env` | obj str->str | max 100 vars, 64 KB total; key `[A-Za-z_][A-Za-z0-9_]{0,127}`; reserved names -> 400 invalid_env |
| `environment` | str | default `base`; unknown -> `unknown_environment` |
| `noEnv`, `snapshots`(default true), `monitoring`(default true), `failFast` | bool | snapshots=false => cannot resume/fork |
| `count` | int | 1..1000, requires `from`; excl. failFast/hostedPorts |
| `setupScript` | str | <=65536 bytes, runs as user `user` after ready |
| `visibility` | str | private/shared (org sandboxes only; else `not_org_sandbox`) |
| `hostedPorts` | [{port:int 1..65535 req, public?:bool, localhost?:bool, title?:str}] | max 50 |
| `files` | [{path req, content req, encoding?: utf8\|base64}] | max 32, 256 KB total, written before `ready` |
| `from` | str | snapshot name or `bsh_` share code |
| `org`/`teamId` | str | wallet; `org` wins |

Response **202**: `{ok, type:"sandbox.created", status:"provisioning"|"ready", ttlSeconds, sandbox:{...}, sandboxes?:[...] (count>1), hostedPorts?:[{port, url, isProtected, access, localhost} | {port, error, message}], giftLimitNotice?}`.
Id path: `response.sandbox.id`. Poll `GET /sandboxes/{id}` until `state` is `ready`/`idle`.
Errors: 400 invalid_setup_script/invalid_files/invalid_hosted_ports/invalid_env/unknown_environment/count_*; 401; 402; 403 environment_forbidden/machine_class_plan_required; 404 snapshot_not_found; 409 idempotency_*, ambiguous_org, monitoring_off_for_account; 429 rate_limited (`limit`, `startLimitsWithWallet`)/limit_reached/member_limit_reached; 503 no_ready_machine (failFast; nothing billed), out_of_capacity.

### GET /sandboxes  (list)
Query: `limit` 1..200 (default 100), `cursor` (opaque), `sort` asc|desc (default desc, by creation presumably; sort key UNSPECIFIED), `state` (comma list e.g. `ready,idle,running`), `namePrefix`, `stoppedBefore` (ISO), `dataExpired` bool, `label` (repeatable `key=value`), `include` (`cost`,`deletedCost`), `org`.
200: `{ok, type:"sandbox.list", sandboxes:[Sandbox], pageInfo:{nextCursor:str|null, hasMore:bool, limit:int}, deletedCost?:{since,seconds,dollars}}`. Loop until `hasMore` false, passing `pageInfo.nextCursor` as `cursor`.
Errors: 401, 409 ambiguous_org.

### GET /sandboxes/{id}  (get)
200: `{ok, type:"sandbox.info", giftLimitNotice?, sandbox:{...}}`. Poll after create/stop/resume/fork. Errors: 401, 404 (also for `cancelled` after first read).

### POST /sandboxes/{id}/stop  (stop and archive)
Body optional: `{force?:bool=false}` (force: stop even if disk snapshot fails; unsnapshotted writes lost; only after a refused stop). Stop always snapshots first; on failure sandbox keeps running.
202: `{ok, type:"sandbox.stopping", id, status:"archiving", sandbox:{...}|null}`. Stop id: `sandbox.stop.id` on later GETs (not in 202 top level; 202 body stop id location UNSPECIFIED). Repeat stop while pending returns same stop id; force escalates.
Poll `GET /sandboxes/{id}` until `stop.status == "completed"` (state `archived`). `failing` auto-retries. Errors: 400 invalid_json, 401, 404.

### GET /sandboxes/{id}/stops/{stopId}
200: `{ok, type:"stop.status", id, stop:{StopOperation}}`. Errors: 401, 404 (unknown/other sandbox).

### POST /sandboxes/{id}/resume
Body optional: `{failFast?:bool, type?:small|default|large|xlarge, env?:obj, environment?:str, noEnv?:bool (one-way), ttlSeconds?:int 1..2592000|null}`.
202: `{ok, type:"sandbox.resuming", id, status:"resuming", sandbox:{...}|null, giftLimitNotice?}`. Poll GET until `ready`/`idle`.
Errors: 401; 402; 403 environment_forbidden/machine_class_plan_required; 404; 409 snapshot_data_expired (once `dataExpiredAt` set), unknown_environment, type_too_small, sandbox_stopping, resume_failed; 429 rate_limited/limit_reached/member_limit_reached; 503 no_ready_machine (failFast), out_of_capacity. No `nextStartAt`.

### DELETE /sandboxes/{id}  (permanently delete data)
Required header `X-Ascii-Confirm-Delete: <sandboxId>` (missing/mismatch -> 409, nothing deleted). No body.
202: `{ok, type:"sandbox.deleting", operation:{id:"bdop_<32hex>", kind:sandbox|snapshot, targetId, reason:explicit|zdr|account, status:pending|processing|blocked|completed, stage:removing|waiting_for_uploads|retrying|completed, expectedBy, attemptCount, requestedAt, completedAt}}`. Irreversible. Operation poll endpoint: UNSPECIFIED (docs say "poll the operation" without path). Errors: 401, 403 `forbidden`, 404, 409.

### GET /sandboxes/{id}/usage
Query `since`, `until` (ISO or epoch secs; default creation / now).
200: `{ok, type:"sandbox.usage", sandboxId, sandboxType, billingMultiplier, since, until, seconds:int, dollars:num, secondsPerDollar:int, running:bool}`. Errors 400, 401, 404.

## 3. Agent endpoints

### POST /sandboxes/{id}/commands  (execute)
Body: `command` str (req), `cwd` str (relative to work dir `/home/user`), `timeoutSeconds` int 1..600 default 30 (else 400 `invalid_timeout`), `detached` bool=false, `stream` bool=false (ignored if detached). Waits up to 60 s if sandbox still starting after resume.

| Mode | 200 body |
|---|---|
| waiting | `{ok, type:"command.finished", success, exitCode:int\|null, signal?, oomKilled?, stdout, stderr, stdoutTruncated?, stderrTruncated?, timedOut, commandId?, cwd?, startedAt?, finishedAt?}` (flat; 8 MiB/stream cap) |
| detached | `{ok, type:"command.started", success, processId:int, pid:int, command, startedAt, commandId?, cwd?, logPath?, errLogPath?}` |
| stream | NDJSON frames: `{type:"started",commandId}`, `{type:"stdout"\|"stderr",data}`, final `{type:"exit",exitCode,success,timedOut}` or `{type:"error",error,message,retryable}` |

Non-zero exit is still HTTP 200 with `success:false`. Errors: 400 invalid_json/invalid_timeout; 401; 404; 409 `boat_starting` (still not ready after 60 s, retryable), 409 `sandbox_not_ready` (sandbox in error/stopped; response includes `state`; not retryable until resumed); 502 `boat_direct_failed`/`sandbox_direct_failed` (NOT auto-retried; command may be running; check `error.details.retryable`). Message `sandbox_restoring` = command did not run.

### GET /sandboxes/{id}/commands/{processId}  (status)
Query: `tailBytes` 1..8388608 (default 8 MiB, ignored with offsets), `stdoutOffset`, `stderrOffset` (>=0).
200 `{ok, type:"command.status", success, processId, pid, status:"running"|"exited"|"lost", known, running, exitCode:int|null, signal, oomKilled, command, cwd, startedAt, finishedAt, stdout, stderr, stdoutTruncated, stderrTruncated, logPath, errLogPath, stdoutOffset, stderrOffset, stdoutNextOffset, stderrNextOffset, more}`. `lost` = agent restarted; exitCode best-effort. Errors 400, 401, 404, 409 `agent_outdated`.

### GET /sandboxes/{id}/commands/{processId}/stream  (attach, needs `exec` scope)
Query `stdoutOffset`, `stderrOffset` (default 0). NDJSON frames: `attached{processId,stdoutOffset,stderrOffset}`, `stdout|stderr{data,nextOffset}`, `heartbeat` (after 15 s idle), `exit{status,exitCode,signal,stdoutOffset,stderrOffset}`, `reconnect` (after 10 min; reopen with last offsets), `error{error,message,stdoutOffset,stderrOffset}`. Errors 400/401/404/409.

### DELETE /sandboxes/{id}/commands/{processId}  (stop command)
No body. SIGTERM, SIGKILL after 5 s. Detached processes only. 200 `{ok, type:"command.stopped", processId, running:false, signaled:bool, command:CommandRecord|null}`. CommandRecord: `commandId, sandboxId, command, mode(sync|stream|detached|process), status(running|exited|timed_out|failed|killed|lost), stdoutBytes, stderrBytes, outputTruncated, startedAt, cwd?, processId?, exitCode?, signal?, finishedAt?`. Errors 400, 401, 404, 409.

### PUT /sandboxes/{id}/files  (write)
Body `{path (req, abs or relative to /home/user, must resolve under /home/user or /tmp), content (req), encoding?: "utf8"|"base64" (default utf8)}`. Max 5 MiB decoded (larger: upload-url). Waits up to 60 s for sandbox ready.
200 `{ok, type:"file.written", success, path, encoding, size}`. Errors 400 invalid_path/invalid_json; others UNSPECIFIED (expect 401/404/409).

### GET /sandboxes/{id}/files  (read)
Query `path` (req), `encoding` (utf8|base64). 5 MiB max (larger: `POST .../files/download-url`, body presumably same as upload-url; UNSPECIFIED).
200 `{ok, type:"file.read", success, path, encoding, size, content}`. Errors 400 invalid_path; missing-file code UNSPECIFIED.

### DELETE /sandboxes/{id}/files  (delete)
Query `path` (req), `recursive` bool=false (required for directories). Needs `file.write`. Symlink deleted itself; `/home/user` and `/tmp` roots not deletable.
200 `{ok, type:"file.deleted", success, path}`. Errors 400 only documented.

### POST /sandboxes/{id}/files/upload-url
Body `{path (req), ttlSeconds? 1..3600 default 900}`. Needs `file.write`.
200 `{ok, type:"file.upload_url", url, method:"PUT", path, expiresAt}`.
Usage: `PUT <url>` with the raw file bytes as body; NO Authorization header (URL self-authenticates, treat as secret); valid only for bound `path` until `expiresAt`; creates parent dirs; replaces existing file after upload completes; size limit UNSPECIFIED; upload response body UNSPECIFIED. Errors 400/401/404/409.

### POST /sandboxes/{id}/host  (expose a port)
Body `{port:int 1..65535 (req), public?:bool=false, localhost?:bool=false (true if bound to 127.0.0.1/::1 only), title?:str}`.
200 `{ok, type:"port.hosted", success, port, url, isProtected:bool, access:"private"|"public", localhost?}`. No separate token field: private ports carry the token as `?_token=...` inside `url` (redact from logs); `public:true` returns URL without `_token`. Idempotent per (sandbox, port): same URL/token. URL form `https://<subdomain>-<port>.on.boat.dev`. Errors 400, 401, 402, 404.

## 4. Limits (docs/limits)
- Waiting command timeout default 30 s, max 600 s; output 8 MiB/stream. Auto-stop default 1 h, max 30 d (trial 2 h). Stopped disk kept >= 14 days.
- Concurrent sandboxes: trial 2, $20 100, $100 300, $500 1000, $2000 2000. Starts/min/hr/day: trial 5/25/75, $20 12/60/200, $100 30/210/840 (higher tiers more); extra starts billed up to 6x then fail.
- IP: 10,000 req/60 s. Port limits: not documented. Snapshots: 10 included per $20/seat.

## 5. Mock fixtures
Derived from doc examples where present; fields marked (illustrative) are constructed from the schema.

create 202 (doc example):
```json
{"ok":true,"type":"sandbox.created","status":"provisioning","ttlSeconds":1800,
 "sandbox":{"id":"bx_7k2m9p4q","name":"Sandbox 2026-06-01 09:30","state":"provisioning","type":"default",
 "vcpu":2,"memoryGB":4,"billingMultiplier":1,"health":"ok","hydrated":true,"access":"owner",
 "visibility":"private","labels":{"session_id":"sess_8841"},"environment":"base","environmentVersion":1,
 "setupStatus":"pending","setupError":null,"desktopAvailable":false,"desktopUrl":null,
 "snapshotAvailable":false,"snapshotCompletedAt":null,"createdAt":"2026-06-01T09:30:00Z",
 "updatedAt":"2026-06-01T09:30:00Z","archiveAfter":"2026-06-01T10:00:00Z","stop":null}}
```
get 200 (illustrative, shaped from doc):
```json
{"ok":true,"type":"sandbox.info","sandbox":{"id":"bx_7k2m9p4q","name":"Sandbox 2026-06-01 09:30",
 "state":"idle","url":"https://bx-7k2m9p4q.example","ip":"10.0.0.7","labels":{"session_id":"sess_8841"},
 "desktopAvailable":false,"snapshotAvailable":false,"snapshotCompletedAt":null,
 "archiveAfter":"2026-06-01T10:00:00Z","ttlSeconds":1800,"stop":null}}
```
exec waiting 200 (illustrative):
```json
{"ok":true,"type":"command.finished","success":true,"exitCode":0,"signal":null,"stdout":"hello\n","stderr":"",
 "stdoutTruncated":false,"stderrTruncated":false,"timedOut":false,"commandId":"cmd_01","cwd":"/home/user",
 "startedAt":"2026-06-01T09:31:00Z","finishedAt":"2026-06-01T09:31:01Z"}
```
exec detached 200 (illustrative):
```json
{"ok":true,"type":"command.started","success":true,"processId":42,"pid":1337,"command":"npm run dev",
 "startedAt":"2026-06-01T09:31:00Z","commandId":"cmd_02","cwd":"/home/user",
 "logPath":"~/.ascii/processes/1337.log","errLogPath":"~/.ascii/processes/1337.err.log"}
```
command status 200 (illustrative; docs give no example):
```json
{"ok":true,"type":"command.status","success":true,"processId":42,"pid":1337,"status":"running","known":true,
 "running":true,"exitCode":null,"signal":null,"oomKilled":false,"command":"npm run dev","cwd":"/home/user",
 "startedAt":"2026-06-01T09:31:00Z","finishedAt":null,"stdout":"ready\n","stderr":"","stdoutTruncated":false,
 "stderrTruncated":false,"stdoutOffset":0,"stderrOffset":0,"stdoutNextOffset":6,"stderrNextOffset":0,"more":false}
```
expose port 200 (doc example):
```json
{"ok":true,"type":"port.hosted","success":true,"port":3000,
 "url":"https://swift-otter-9021-3000.on.boat.dev?_token=redacted","isProtected":true,"access":"private"}
```
429 rate_limited (illustrative; docs give field names, not a body; nesting of window/limit/starts UNSPECIFIED):
```json
{"ok":false,"type":"sandbox.error","status":429,"code":"rate_limited","message":"Start limit reached for the minute window.",
 "window":"minute","limit":12,"starts":{"minute":{"remaining":0}},"startLimitsWithWallet":null,
 "error":{"code":"rate_limited","message":"Start limit reached for the minute window.","status":429,"details":{}},
 "requestId":"req_01HX..."}
```
