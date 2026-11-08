# Sandbox runtime (boat.dev)

An optional server-side runtime for projects: instead of WebContainer in the user's browser, a
project's files, commands and dev server live in a boat.dev sandbox driven by the Nomi server
(`nomi-sandbox` crate, `nomi-server/src/sandbox_*.rs`). It is off unless an admin turns it on in
Admin -> Configuration -> Sandbox and sets a boat API key.

## Security model

### What the sandbox is trusted with

A sandbox is treated as untrusted, project-owned compute. It runs the project's own code and
Koda's commands, so anything inside it can read anything else inside it. It is given only:

- the project's files, copied from Nomi's storage;
- `NOMI_PREVIEW` and `NODE_NO_WARNINGS`;
- for a project on Relay, the project's own **dev** backend address and anon key, and its dev
  service key (the owner's role is admin, so the owner's browser already holds the same key in
  the WebContainer runtime). These are project-scoped app keys, never Nomi's.

It is never given Nomi's own secrets: the JWT secret, the database URL, the settings encryption
key, LLM keys, Shipyard keys or the boat API key. The environment is built from a fixed list
(`sandbox_devserver::preview_env`), never from the server's process environment; a unit test pins
the list. Variables reach the sandbox as `env NAME='value'` with every value shell-quoted. Note
that the dev service key does travel to boat.dev as part of the start command, so boat is trusted
with it as it is with the project's code.

### boat API key

The key is stored encrypted as a `Secret` settings field and is never returned by the admin API.
`BoatConfig` and `SandboxConfig` redact it in `Debug`. It is only ever sent as the bearer header
to the configured API base, never in a request body, a command, the sandbox, an MQTT event or the
frontend. Every error built from provider text passes through `BoatProvider::scrub`, which removes
the key and URL query strings, user info and fragments (`nomi_sandbox::redact`); network errors
also drop their URL. The API base is admin-configurable; admins are trusted not to point it (and
the key) at a host they do not control.

### Preview URL

A private port's preview URL carries its access token (`?_token=...`), so the URL itself is the
secret. It is stored in `project_sandboxes.preview_url` and returned by
`GET /api/projects/:id/sandbox` only to people who can open the project, and only while the
sandbox is running. `SandboxChanged` MQTT events carry the phase only. `PreviewUrl`'s `Debug` is
redacted and the URL is never logged. Anything saved as `last_error` goes through `redact_urls`
in the store, so a provider or step error that echoes a URL cannot persist its token there. The
provider must return an `https` URL (checked server-side, and again in the frontend before it is
used), and the frontend only ever uses it as the preview iframe's `src`.

Residual risk: boat keeps the same token for a (sandbox, port) pair across stops and resumes, so a
person removed from a project who copied the URL keeps access to that preview until the sandbox
is recreated. Deleting the project deletes the sandbox.

### Commands and paths

Koda's commands are queued as program plus arguments. Before running one the server re-checks it
(`sandbox_runs::validate`): it must survive `parse_command` unchanged, so the program is on the
`npm | npx | node` allow-list, no shell operators are present and dev-server starts are refused;
each argument is then single-quoted. The program name is the only unquoted word and comes from
the allow-list. Other command lines (install, dev server start, readiness check, port cleanup,
archive extraction) are built from constants plus shell-quoted values.

Project paths come from the database and are checked with `validate_path` (relative, no `..`, no
empty segment, no NUL or backslash, not under `node_modules`) before they are written, archived or
deleted; the per-sandbox memory of synced paths is re-checked too, since a remembered path is
deleted recursively. `build_archive` refuses absolute and `..` paths and writes regular files
only, and extraction uses `tar --no-same-owner --no-same-permissions` (GNU tar also strips a
leading `/` and refuses `..` members). Code inside the sandbox can create symlinks in the project
directory, and a later extraction would follow them, but that only reaches other places in the
same sandbox, which that code could already write to. Sandbox and process ids are checked to be
plain names before they go into a boat request path.

### Server-side requests (SSRF)

The server makes outgoing requests only to the configured API base, plus one `PUT` of the project
archive to the signed upload URL boat returns. That URL is checked first: it must be on the API's
own origin, or be `https` on a public host name (no IP literal, no `localhost`, `.internal`,
`.local` and similar names, with or without a trailing dot, no user info). The PUT is sent on a
client that never follows redirects, and a redirect answer fails the upload, so a checked URL
cannot hop to an unchecked host. The preview URL is never fetched by the server; the readiness
check runs `curl` inside the sandbox against `localhost`.

Residual risk: the check looks at the host name only, not at what it resolves to. A public name
that resolves to a private address (for example `127.0.0.1.nip.io`), or DNS rebinding between
the check and the request, is not blocked; that would need a resolver that checks the addresses
it connects to.

### Authorization

Every `/api/projects/:id/sandbox*` route loads the project with `can_open_project` and answers
404 to anyone else (tested for all four routes). Starting also requires the admin switch, the
rollout and the owner's monthly allowance; stopping is open to members. The sandbox settings
section is behind the admin configuration permission. `sandbox_deletions` and the reaper are
worker-only and have no routes. The reaper only touches sandboxes labelled `nomi-project`, so a
boat account should be dedicated to one Nomi deployment.

### Outbound network from the sandbox

Decision: egress is left open (boat's default), because `npm install`, `npx` and the project's
own Relay backend need the public internet. The sandbox runs on boat's infrastructure, not inside
Nomi's network, so it cannot reach Nomi's database or internal services directly. If boat offers
firewall rules, the recommended tightening is to deny link-local and cloud metadata ranges
(`169.254.0.0/16`, `fd00:ec2::254`) and private ranges (`10.0.0.0/8`, `172.16.0.0/12`,
`192.168.0.0/16`, `100.64.0.0/10`) while keeping public egress open. This is not configured by
Nomi today.
