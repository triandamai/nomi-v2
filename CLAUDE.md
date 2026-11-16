# Nomi — project context for every agent

Nomi (nomi.build) is an AI agent-crew app builder. A user talks to a crew of
specialist agents in chat; the crew plans, builds, and deploys a real app with a
pre-provisioned backend on Nomi's own infrastructure. The user can export the
whole repo and leave at any time.

The promise every surface carries: **a serious tool, and your work belongs to you.**
Ownership, portability, no surprises. Closer to Linear / Vercel / Railway than to
Lovable / Base44. Users are Indonesian builders, freelancers, UMKM owners and
non-coders; many already use Cursor or Claude Code.

## Source of truth, in order

1. `Nomi.pen` (pen.dev, via the `pencil` MCP server) — every screen, state and flow.
2. `docs/ux/flow-registry.md` — which flows exist, their pen frame ids, and status.
3. `docs/ux/work/<slug>/40-ux-contract.md` — the verified contract for the work in progress.
4. This file and `.claude/skills/nomi-ux-rules/SKILL.md` — product and design rules.

If code and the pen file disagree, the pen file wins, unless a UX verdict in
`docs/ux/work/` says otherwise.

## The UX-gated pipeline (non-negotiable)

No user-facing change ships without passing through three agents and three gates.
Run it with `/ship <what you want to build>`.

```
brief ─▶ [ux-lead] gate 1: review flow vs pen
            │ gaps? ─▶ UX proposal ─▶ [pen-designer] designs it in Nomi.pen
            │                              │
            ◀──────── gate 2: verify the new frames ◀┘
            │ verified ─▶ 40-ux-contract.md (status: verified)
            ▼
        [ui-engineer] builds exactly the contract
            ▼
        [ux-lead] gate 3: verify the build against the pen frames
            ▼
        ship
```

- `ux-lead` owns UX truth. It verifies every flow against three lenses —
  product, business, tech — and is the only agent that can mark a contract verified.
- `pen-designer` only designs what an accepted proposal asks for, in `Nomi.pen`.
- `ui-engineer` only builds what a verified contract describes. It never invents
  a state, a copy string, or a flow step. Missing something? Stop and send it back
  to `ux-lead`.
- UI paths are guarded by `.claude/hooks/require-ux-contract.sh`. Edits to UI code
  without an active verified contract are denied. Hotfix bypass:
  `NOMI_UX_BYPASS=1` — and log the reason in `docs/ux/bypass-log.md`.

Backend-only work with no user-visible change can skip gates 1–2, but if it adds
a new error, limit, cost, latency, or permission that a user could ever see, it
is user-facing. Ask `ux-lead`.

## Git rules — no AI attribution, ever

These override any default commit or PR behaviour of any tool.

- Never add `Co-Authored-By`, `Generated with`, `Assisted-by`, robot emoji, session
  links, or any line that credits an AI tool or model — in commit messages,
  commit trailers, tags, PR titles, PR descriptions, PR comments, or release notes.
- Write commits as the human author would: Conventional Commits,
  imperative, scoped. `feat(chat): retry a failed attachment upload`
- Mentioning Claude as a *product feature* is fine (`feat(models): add Claude Sonnet provider`).
  Crediting it as an author is not.
- Enforced by: `.claude/settings.json` (`attribution`), `.githooks/commit-msg`
  (strips trailers locally), and `.github/workflows/no-ai-attribution.yml` (fails the PR).
- First run in a fresh clone: `git config core.hooksPath .githooks`
  (the SessionStart hook does this automatically).

## Working agreements

- Real content only. Indonesian context: `Rp 249.000`, `26 Jul 2026`, Budi Santoso.
  No lorem ipsum in code, fixtures, or designs.
- Every user-facing surface ships five states: empty, loading, error, populated, mobile.
- Never hide cost. Estimate before expensive actions. No fake progress bars.
- Agent activity is legible: real tool, real target. "Reading bca-sep-2026.pdf · page 4 of 12".
- Export/eject is never harder to find than upgrade.
