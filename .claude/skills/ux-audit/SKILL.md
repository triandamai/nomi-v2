---
name: ux-audit
description: Ask ux-lead for a standalone completeness audit of one area of Nomi.pen (chat, attachments, build, projects, settings) without starting a build. Produces a UX proposal for pen-designer.
argument-hint: "[area, e.g. chat attachments]"
disable-model-invocation: true
---

# /ux-audit

Area: $ARGUMENTS

1. Create `docs/ux/work/audit-<area-slug>/00-brief.md` with: "Audit <area> for
   missing UX across product, business and tech. No build requested."
2. Delegate to `ux-lead`: "Gate 1 for audit-<area-slug>. Audit every flow in the
   registry for this area, desktop + mobile, dark + light."
3. Show the user the gap list grouped by priority, and offer to send the proposal
   to `pen-designer`. Do not set `.claude/state/active-work` — audits don't unlock
   UI code.
