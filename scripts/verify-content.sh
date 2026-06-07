#!/usr/bin/env bash
# verify-content.sh <spec.json>
# Checks requirements.content: forbiddenSubstrings absent from all scoped files;
# each requiredSubstrings string present in at least one scoped file.
# Fixed substrings only (grep -F). Emits one JSON evidence line per requirement.
set -uo pipefail

SPEC="${1:?usage: verify-content.sh <spec.json>}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

glob_to_re() {
  printf '%s' "$1" | sed \
    -e 's/[][().+?^${|}\\]/\\&/g' -e 's/\./\\./g' \
    -e 's|\*\*/|<DSL>|g' -e 's|\*\*|<DS>|g' -e 's|\*|[^/]*|g' \
    -e 's|<DSL>|(.*/)?|g' -e 's|<DS>|.*|g'
}

PATHS=$(git ls-files --cached --others --exclude-standard)

jq -c '.requirements.content[]' "$SPEC" | while IFS= read -r REQ; do
  ID=$(echo "$REQ" | jq -r '.id')
  PROBLEMS=""
  SCOPED=""
  while IFS= read -r G; do
    [ -z "$G" ] && continue
    RE="^$(glob_to_re "$G")$"
    SCOPED="$SCOPED$(echo "$PATHS" | grep -E "$RE")"$'\n'
  done < <(echo "$REQ" | jq -r '.files[]?')
  SCOPED=$(echo "$SCOPED" | grep -v '^$' | sort -u)

  while IFS= read -r T; do
    [ -z "$T" ] && continue
    while IFS= read -r F; do
      [ -z "$F" ] && continue
      if grep -Fq -- "$T" "$F" 2>/dev/null; then
        PROBLEMS="$PROBLEMS forbidden text '$T' found in $F;"
      fi
    done <<< "$SCOPED"
  done < <(echo "$REQ" | jq -r '.forbiddenSubstrings[]?')

  while IFS= read -r T; do
    [ -z "$T" ] && continue
    FOUND=0
    while IFS= read -r F; do
      [ -z "$F" ] && continue
      grep -Fq -- "$T" "$F" 2>/dev/null && { FOUND=1; break; }
    done <<< "$SCOPED"
    [ "$FOUND" -eq 1 ] || PROBLEMS="$PROBLEMS required text '$T' found in no scoped file;"
  done < <(echo "$REQ" | jq -r '.requiredSubstrings[]?')

  if [ -z "$PROBLEMS" ]; then
    jq -nc --arg id "$ID" '{id: $id, status: "pass"}'
  else
    jq -nc --arg id "$ID" --arg m "$PROBLEMS" '{id: $id, status: "fail", message: $m}'
  fi
done
