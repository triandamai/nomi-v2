# nomi-baas-cli

The command line for Nomi BaaS. It installs a `nomi-baas` command.

```sh
npx nomi-baas-cli --help
# or
npm install -D nomi-baas-cli && npx nomi-baas --help
```

## Commands

| Command | What it does |
|---|---|
| `nomi-baas init` | Creates `nomi/backend.json`, `nomi/migrations/0001_init.sql` and `nomi/functions/` |
| `nomi-baas migrate new <name>` | Writes the next numbered migration, e.g. `0003_add_orders.sql` |
| `nomi-baas migrate lint` | Checks migrations before they're applied (exits 1 on problems, so it fits CI) |
| `nomi-baas migrate status` | Lists local migrations with their checksums |
| `nomi-baas functions new <name>` | Writes `nomi/functions/<name>.ts` from a starter |

`--dir <path>` points any command at another project folder.

## What `migrate lint` checks

- **Row-level security.** Every `create table` needs `alter table … enable row level security`
  in some migration, unless it's marked `-- nomi: public` on the line above.
- **Drops that lose data.** `drop table` and `alter table … drop column` need
  `-- nomi: destructive` on the line above.
- **Staying inside the project.** No objects in other schemas (such as `nomi.*`), no schema,
  role or database changes, no `set role` or `search_path`.

The BaaS runs the same checks, authoritatively, when it applies migrations; the CLI gives you
the answer before that.

## Coming with the hosted BaaS

`login`, `link`, `migrate apply`, `types`, `functions serve`, `functions deploy` and
`secrets set` are reserved and say so when called.

The app-side client is [`nomi-baas`](https://www.npmjs.com/package/nomi-baas).
