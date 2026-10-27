# Relay Phase 3: storage and realtime

**Goal.** Apps keep files in buckets and get live changes to chosen tables, both declared in
`nomi/backend.json` and served by the gateway, with no code of the app's own.

**Spec.** `relay/README.md` (What every project gets, Realtime, Security) and phase 3 of
`docs/superpowers/plans/2026-10-26-relay-service-and-koda.md`. The SDK side already exists
(`relay/sdk/js/src/storage.ts`, `realtime.ts`) and is the contract the gateway answers.

## What Koda writes

```json
{
  "storage": {
    "buckets": [
      { "name": "avatars", "public": true, "write": "owner", "max_size_mb": 5, "types": ["image/*"] },
      { "name": "invoices", "read": "owner", "write": "service" }
    ]
  },
  "realtime": ["message", "task"]
}
```

Bucket rules, checked by the gateway with the caller's claims:

| Rule | Who |
|---|---|
| `anyone` | any caller with the project's key |
| `signed_in` | a signed-in user (default for `read` and `write`) |
| `owner` | a signed-in user whose id is the path's first folder (`<user id>/avatar.png`) |
| `service` | only the service key |

The service key passes every rule. `public: true` also serves files without any key at
`/storage/public/<bucket>/<path>`. `max_size_mb` defaults to 50; `types` (content-type patterns,
`image/*`) is optional.

## Decisions

- **Files live in one S3 bucket Relay is given**, under `<project>/<env>/<bucket>/<path>`. The S3
  settings (endpoint, region, bucket, access key, secret, path-style) are part of Relay's settings
  in Nomi's admin, sealed like the rest; the secret is never shown. Saving checks the bucket
  answers. Without them, storage answers `503 RELAY_NO_STORAGE` and everything else works.
- **The gateway serves every byte.** S3 stays private: uploads, downloads and signed URLs go
  through the gateway. A signed URL is `GET /storage/sign/<bucket>/<path>?token=…`, the token a
  short JWT signed with the environment's signing secret naming project, env, bucket and path.
- **`object_store`** (S3 and in-memory) instead of the AWS SDK: small, and tests run on the
  in-memory store.
- **Realtime triggers** come from `nomi.realtime_notify()` (installed with the `nomi` schema),
  added as `nomi_realtime` triggers on the listed tables with the project id and primary-key
  columns as arguments; a table without a primary key is refused. The payload is
  `{p, s, t, op, k}` (project, schema, table, operation, key), never the row.
- **One `LISTEN nomi_realtime` per shared database** in the gateway, started on the first socket
  and restarted when settings change. For inserts and updates the row is read once per distinct
  identity (role + claims) subscribed to that table, with `SET LOCAL ROLE` and
  `request.jwt.claims`, so row-level security decides; a subscriber whose read finds nothing gets
  nothing. Deletes send only the key.
- **Limits:** 200 sockets per project environment; over 50 changes a second to one socket turn the
  rest of that second into one `resync`.
- **Nomi sends both blocks** whenever `backend.json` is written (dev) and at publish
  (production, after migrations). After migrations apply on dev, `realtime` is sent again, so a
  table listed before its migration gets its trigger once it exists.

## Tasks

1. **Config types and rules** (`relay-core/src/backend.rs`): `BucketConfig`, `Rule`,
   `StorageConfig`, `validate`, `allows(rule, caller, path)`, `StorageSettings` on `Settings`.
2. **Control plane**: state migration `0005_storage_realtime.sql` (`relay.buckets`,
   `relay.realtime`); `PUT …/environments/:env/storage` and `PUT …/environments/:env/realtime`;
   triggers; `nomi.buckets` kept in step; `describe` lists buckets and realtime tables.
3. **Storage settings**: input/view/check in `relay-control/src/settings.rs`; the S3 store on
   `Live`.
4. **Gateway storage**: object PUT/GET, DELETE, list, sign, signed GET, public GET; per-bucket
   size and type; CORS allows `x-nomi-upsert` and `cache-control`.
5. **Gateway realtime**: the listener, the hub, `/realtime` WebSocket (`auth`, `subscribe`,
   `unsubscribe`), role-switched reads, limits.
6. **Nomi**: `nomi-relay-client` `put_storage`/`put_realtime`; `RelayOps::apply_storage` and
   `apply_realtime`; `after_write`, `apply_before_publish`; admin Storage fields
   (`RelayIntegration.svelte`); `guides/relay.md` Storage and Realtime sections.
7. **End to end** with the real SDK: upload, owner rule refusal, signed and public URLs; two
   clients see each other's inserts, a third user doesn't see rows RLS hides.

## Review focus

- A path with `..`, a leading `/`, or an empty segment: refused, never escaping the prefix.
- A token signed for one bucket or path used for another: refused.
- A realtime subscriber whose token expires mid-socket: stops getting rows (re-resolved on `auth`).
- A shared database restart: the listener reconnects and every socket gets `resync`.
- An upload over `max_size_mb` streamed without `content-length`: cut off at the limit.
