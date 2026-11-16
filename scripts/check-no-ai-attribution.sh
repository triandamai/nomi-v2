#!/usr/bin/env bash
# Usage:
#   check-no-ai-attribution.sh --strip <file>   remove attribution lines in place
#   check-no-ai-attribution.sh --check <file>   exit 1 if attribution is present
#   check-no-ai-attribution.sh --check -        read stdin
# Only attribution is matched. "feat(models): add Claude provider" is fine.
set -euo pipefail

PATTERN='^[[:space:]]*(co-authored-by|assisted-by|generated-by|ai-assisted|signed-off-by)[[:space:]]*:.*(claude|anthropic|openai|chatgpt|gpt-|copilot|cursor|gemini|codex|devin|aider|windsurf|codeium|ai assistant)|generated (with|by) .*(claude|chatgpt|copilot|cursor|gemini|codex|aider|windsurf|an? ai)|claude\.(ai|com)/(code|claude-code)|🤖'

mode="${1:-}"; target="${2:-}"
case "$mode" in
  --strip)
    tmp="$(mktemp)"
    grep -Eiv "$PATTERN" "$target" > "$tmp" || true
    # drop trailing blank lines left behind
    awk 'NF{for(i=0;i<blank;i++)print "";blank=0;print;next}{blank++}' "$tmp" > "$target"
    rm -f "$tmp"
    ;;
  --check)
    if [ "$target" = "-" ]; then content="$(cat)"; else content="$(cat "$target")"; fi
    if printf '%s\n' "$content" | grep -Eiq "$PATTERN"; then
      echo "AI attribution found:" >&2
      printf '%s\n' "$content" | grep -Ein "$PATTERN" >&2
      exit 1
    fi
    ;;
  *) echo "usage: $0 --strip|--check <file|->" >&2; exit 2 ;;
esac
