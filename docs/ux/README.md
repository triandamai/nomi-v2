# Nomi UX system

How user-facing work moves from idea to shipped, and where each agent writes.

## Agents
| Agent | Owns | Writes to | Can change Nomi.pen | Can change app code |
|---|---|---|---|---|
| `ux-lead` | UX truth; gates 1, 2, 3 | `docs/ux/**` | No (read only) | No |
| `pen-designer` | Designs from accepted proposals | `Nomi.pen`, `20-pen-changes.md` | Yes | No |
| `ui-engineer` | Builds verified contracts | app code, `45-build-notes.md` | No (read only) | Yes, only with a verified contract |

## Work item files — `docs/ux/work/<slug>/`
| File | Written by | Meaning |
|---|---|---|
| `00-brief.md` | `/ship` | What we're building and why |
| `10-ux-review.md` | ux-lead | Gate 1 verdict: covered frames, gaps, three-lens review |
| `20-pen-changes.md` | pen-designer | Frames and components created, per gap |
| `30-ux-verify.md` | ux-lead | Gate 2 rounds: what to fix, or pass |
| `40-ux-contract.md` | ux-lead | The verified spec the engineer builds. `status: verified` unlocks UI code |
| `45-build-notes.md` | ui-engineer | Build plan, how to reach each scenario, questions |
| `50-build-verify.md` | ux-lead | Gate 3: build matches pen, or deviations |

Proposals live in `docs/ux/proposals/UXP-<nnn>-<slug>.md` so several work items
can reference the same one.

## Status vocabulary (flow-registry.md)
`gaps` → `designing` → `design-verified` → `building` → `shipped` → `build-verified`

## Commands
- `/ship <request>` — full pipeline.
- `/ux-audit <area>` — audit only; produces a proposal, unlocks nothing.

## Guards
- `.claude/hooks/require-ux-contract.sh` blocks UI edits without a verified contract.
  Edit `UI_PATHS` at the top to match the repo.
- `NOMI_UX_BYPASS=1` for real hotfixes only; log every use in `bypass-log.md`.
- No AI attribution in git: settings + `.githooks/commit-msg` + CI.

## Setup in a fresh clone
```bash
git config core.hooksPath .githooks       # SessionStart also does this
chmod +x .claude/hooks/*.sh .githooks/* scripts/*.sh
mkdir -p .claude/state && echo ".claude/state/" >> .gitignore
```
The agents expect the pen.dev MCP server to be named `pencil` in `.mcp.json`. If
yours has another name, change `mcpServers:` in the three agent files.
