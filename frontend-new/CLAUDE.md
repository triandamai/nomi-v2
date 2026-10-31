# frontend-new

The rules for this folder are in AGENTS.md, shared by every agent. Follow them in full:

@AGENTS.md

## For Claude Code

- The pen file is reached only through the Pencil MCP tools (`mcp__pencil__get_app_state`,
  `mcp__pencil__execute`, `mcp__pencil__read_skill`). Never open a `.pen` file with Read, Grep or
  the shell; never change the design file unless asked to.
- Before building or changing a component, read its pen frame with `mcp__pencil__execute`
  (`Get(id, {depth: 3, resolveInstances: true})` and `ctx.bounds` in a visitor), then follow
  AGENTS.md §3 to translate it. The snippets that export `pen/bounds.json` and `pen/export/` are
  in `pen/README.md`.
- Finish with `npm run pen:report` against the dev server and the full gate chain (AGENTS.md §7);
  report the numbers you actually saw.
