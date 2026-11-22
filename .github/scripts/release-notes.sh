#!/usr/bin/env bash
# The GitHub release of a deploy tag, and the release notes the frontend-new image carries.
#
#   release-notes.sh <tag> <staging|production>
#
# 1. Makes sure the tag has a GitHub release with notes: a release published by hand keeps its
#    notes; one without notes (or a tag pushed with no release) gets GitHub's generated notes since
#    the previous tag of the same environment, so staging compares with staging and production
#    with production. A staging release is never marked Latest: that stays production's.
# 2. Writes `notes=<base64 JSON>` to $GITHUB_OUTPUT: this environment's latest releases, from the
#    tag's own back (at most MAX_RELEASES), for frontend-new's RELEASE_NOTES build arg
#    (frontend-new/release-notes.js). Kept under MAX_BYTES, so the build's environment can hold it.
#
# Needs GH_TOKEN (contents: write) and GITHUB_REPOSITORY. Never fails the deploy: without notes
# the app simply shows none.
set -uo pipefail

TAG="$1"
ENVIRONMENT="$2"
REPO="${GITHUB_REPOSITORY:?}"
OUTPUT="${GITHUB_OUTPUT:-/dev/stdout}"
MAX_RELEASES=10
# One environment variable may hold at most 128 KiB on Linux; leave room to spare.
MAX_BYTES=100000

case "$ENVIRONMENT" in
  staging) PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+-staging[0-9]*$' ;;
  production) PATTERN='^v[0-9]+\.[0-9]+\.[0-9]+$' ;;
  *) echo "::error::environment must be staging or production, not '$ENVIRONMENT'"; exit 1 ;;
esac

# --- 1. The tag's release ---------------------------------------------------------------------
previous=$(gh api "repos/$REPO/tags" --paginate --jq '.[].name' 2>/dev/null \
  | grep -E "$PATTERN" | grep -vxF "$TAG" \
  | { cat; echo "$TAG"; } | sort -V | awk -v tag="$TAG" '$0 == tag { print previous; exit } { previous = $0 }')
echo "previous $ENVIRONMENT tag: ${previous:-none}"

generate_args=(-f "tag_name=$TAG")
[ -n "$previous" ] && generate_args+=(-f "previous_tag_name=$previous")
notes_file=$(mktemp)
if ! gh api "repos/$REPO/releases/generate-notes" "${generate_args[@]}" --jq .body > "$notes_file"; then
  echo "::warning::couldn't generate release notes for $TAG"
  : > "$notes_file"
fi

existing_body=$(gh release view "$TAG" --repo "$REPO" --json body --jq .body 2>/dev/null)
if [ $? -ne 0 ]; then
  latest=true
  [ "$ENVIRONMENT" = staging ] && latest=false
  gh release create "$TAG" --repo "$REPO" --verify-tag --title "$TAG" --notes-file "$notes_file" --latest="$latest" \
    && echo "created the release for $TAG" \
    || echo "::warning::couldn't create the release for $TAG"
elif [ -z "${existing_body//[[:space:]]/}" ] && [ -s "$notes_file" ]; then
  gh release edit "$TAG" --repo "$REPO" --notes-file "$notes_file" \
    && echo "added generated notes to the release for $TAG" \
    || echo "::warning::couldn't update the release for $TAG"
else
  echo "the release for $TAG already has notes; keeping them"
fi

# --- 2. The notes the image carries ------------------------------------------------------------
releases=$(gh api "repos/$REPO/releases?per_page=100" 2>/dev/null) || releases='[]'
count=$MAX_RELEASES
encoded=''
while [ "$count" -gt 0 ]; do
  encoded=$(jq -c --arg tag "$TAG" --arg env "$ENVIRONMENT" --arg pattern "$PATTERN" --argjson count "$count" '
    [ .[] | select(.draft | not) | select(.tag_name | test($pattern)) ]
    | sort_by(.published_at) | reverse
    | ((map(.tag_name) | index($tag)) // 0) as $at
    | .[$at:] | .[:$count]
    | { environment: $env, releases: map({ version: .tag_name, name: (.name // .tag_name), publishedAt: .published_at, url: .html_url, body: (.body // "") }) }
  ' <<< "$releases" | base64 -w0) || encoded=''
  [ "${#encoded}" -le "$MAX_BYTES" ] && break
  count=$((count - 1))
  encoded=''
done

echo "notes=$encoded" >> "$OUTPUT"
echo "release notes for the image: $count release(s), ${#encoded} bytes"
