# nomi-relay-cli

The command line for Nomi Relay. It installs a `nomi-relay` command.

```sh
npx nomi-relay-cli --help
# or
npm install -D nomi-relay-cli && npx nomi-relay --help
```

## Commands

| Command | What it does |
|---|---|
| `nomi-relay init` | Creates `nomi/backend.json`, `nomi/migrations/0001_init.sql` and `nomi/functions/` |
| `nomi-relay migrate new <name>` | Writes the next numbered migration, e.g. `0003_add_orders.sql` |
| `nomi-relay migrate lint` | Checks migrations before they're applied (exits 1 on problems, so it fits CI) |
| `nomi-relay migrate status` | Lists local migrations with their checksums |
| `nomi-relay functions new <name>` | Writes `nomi/functions/<name>.ts` from a starter |

`--dir <path>` points any command at another project folder.

## What `migrate lint` checks

- **Row-level security.** Every `create table` needs `alter table … enable row level security`
  in some migration, unless it's marked `-- nomi: public` on the line above.
- **Drops that lose data.** `drop table` and `alter table … drop column` need
  `-- nomi: destructive` on the line above.
- **Staying inside the project.** No objects in other schemas (such as `nomi.*`), no schema,
  role or database changes, no `set role` or `search_path`.

Relay runs the same checks, authoritatively, when it applies migrations; the CLI gives you
the answer before that.

## Coming with the hosted Relay

`login`, `link`, `migrate apply`, `types`, `functions serve`, `functions deploy` and
`secrets set` are reserved and say so when called.

The app-side client is [`nomi-relay`](https://www.npmjs.com/package/nomi-relay).
