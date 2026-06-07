#!/usr/bin/env bash
# verify-tree.sh <spec.json>
# Checks requirements.tree: requiredPaths exist; no repository path matches forbiddenGlobs.
# Emits one JSON evidence line per requirement on stdout. Nonzero exit = runtime error.
set -uo pipefail

SPEC="${1:?usage: verify-tree.sh <spec.json>}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

glob_to_re() {
  printf '%s' "$1" | sed \
    -e 's/[][()+?^${|}\\]/\\&/g' -e 's/\./\\./g' \
    -e 's|\*\*/|<DSL>|g' -e 's|\*\*|<DS>|g' -e 's|\*|[^/]*|g' \
    -e 's|<DSL>|(.*/)?|g' -e 's|<DS>|.*|g'
}

# Tracked + untracked-but-not-ignored paths (files; git does not record empty dirs).
PATHS=$(git ls-files --cached --others --exclude-standard)

jq -c '.requirements.tree[]' "$SPEC" | while IFS= read -r REQ; do
  ID=$(echo "$REQ" | jq -r '.id')
  PROBLEMS=""
  while IFS= read -r P; do
    [ -z "$P" ] && continue
    [ -e "$P" ] || PROBLEMS="$PROBLEMS missing required path: $P;"
  done < <(echo "$REQ" | jq -r '.requiredPaths[]?')
  while IFS= read -r G; do
    [ -z "$G" ] && continue
    RE="^$(glob_to_re "$G")$"
    HITS=$(echo "$PATHS" | grep -E "$RE" | head -3 | tr '\n' ' ')
    [ -z "${HITS// /}" ] || PROBLEMS="$PROBLEMS forbidden glob '$G' matched: $HITS;"
  done < <(echo "$REQ" | jq -r '.forbiddenGlobs[]?')
  if [ -z "$PROBLEMS" ]; then
    jq -nc --arg id "$ID" '{id: $id, status: "pass"}'
  else
    jq -nc --arg id "$ID" --arg m "$PROBLEMS" '{id: $id, status: "fail", message: $m}'
  fi
done
