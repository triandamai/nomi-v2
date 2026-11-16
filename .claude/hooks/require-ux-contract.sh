#!/usr/bin/env bash
# Denies edits to UI code unless the active work item has a verified UX contract.
# Adjust UI_PATHS to match the repo layout.
set -euo pipefail

# The web app (frontend-new: routes, the design system, the chat view, the admin), the old web app
# (frontend: routes and components) and the native app (Compose features and design system, and
# each platform's app). Docs, gallery sheets, server code and pure logic aren't gated.
UI_PATHS='(^|/)(frontend-new/src/routes|frontend-new/src/lib/(ds|chat|admin)|frontend/src/routes|frontend/src/lib/components|native/shared/(feature|designsystem)|native/(androidApp|iosApp|desktopApp))/'
ROOT="${CLAUDE_PROJECT_DIR:-.}"

input="$(cat)"
file="$(printf '%s' "$input" | sed -n 's/.*"file_path"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' | head -n1)"

[ -z "$file" ] && exit 0
printf '%s' "$file" | grep -Eq "$UI_PATHS" || exit 0
[ "${NOMI_UX_BYPASS:-0}" = "1" ] && exit 0

deny() {
  printf '{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"deny","permissionDecisionReason":"%s"}}\n' "$1"
  exit 0
}

active=""
[ -f "$ROOT/.claude/state/active-work" ] && active="$(tr -d '[:space:]' < "$ROOT/.claude/state/active-work")"
[ -z "$active" ] && deny "UI edit blocked: no active UX work item. Run /ship so ux-lead verifies the flow first. Hotfix: NOMI_UX_BYPASS=1 and log it in docs/ux/bypass-log.md."

contract="$ROOT/docs/ux/work/$active/40-ux-contract.md"
[ -f "$contract" ] || deny "UI edit blocked: $active has no 40-ux-contract.md yet. ux-lead must verify the pen frames first."
grep -Eq '^status:[[:space:]]*verified' "$contract" || deny "UI edit blocked: the contract for $active is not verified by ux-lead."

exit 0
