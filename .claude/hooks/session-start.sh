#!/usr/bin/env bash
# Wires the git hooks and tells the agent which UX work item is active.
set -euo pipefail
cd "${CLAUDE_PROJECT_DIR:-.}"

if git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  current="$(git config --get core.hooksPath || true)"
  [ "$current" = ".githooks" ] || git config core.hooksPath .githooks
fi

active=""
[ -f .claude/state/active-work ] && active="$(tr -d '[:space:]' < .claude/state/active-work)"

echo "Git rule: never add AI attribution to commits, trailers, PR titles or PR bodies."
if [ -n "$active" ]; then
  status="$(grep -m1 -E '^status:' "docs/ux/work/$active/40-ux-contract.md" 2>/dev/null | awk '{print $2}' || true)"
  echo "Active UX work item: $active (contract status: ${status:-none})."
else
  echo "No active UX work item. Start user-facing work with /ship."
fi
