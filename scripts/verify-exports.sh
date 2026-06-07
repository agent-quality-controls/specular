#!/usr/bin/env bash
# verify-exports.sh <spec.json>
# Checks requirements.exports: each listed type and function is declared public
# somewhere under src/. Approximation: textual scan for `pub struct|enum NAME`,
# `pub use ...NAME`, `pub fn name` — does not prove reachability from the crate
# root; upgrade to cargo public-api when the workspace builds.
# Emits one JSON evidence line per requirement.
set -uo pipefail

SPEC="${1:?usage: verify-exports.sh <spec.json>}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

jq -c '.requirements.exports[]' "$SPEC" | while IFS= read -r REQ; do
  ID=$(echo "$REQ" | jq -r '.id')
  PROBLEMS=""
  if [ ! -d src ]; then
    PROBLEMS="src/ does not exist;"
  else
    while IFS= read -r T; do
      [ -z "$T" ] && continue
      grep -rqE "pub (struct|enum) $T\b|pub use [^;]*\b$T\b|pub type $T\b" src/ \
        || PROBLEMS="$PROBLEMS public type '$T' not found;"
    done < <(echo "$REQ" | jq -r '.types[]?')
    while IFS= read -r F; do
      [ -z "$F" ] && continue
      grep -rqE "pub fn $F\b|pub use [^;]*\b$F\b" src/ \
        || PROBLEMS="$PROBLEMS public function '$F' not found;"
    done < <(echo "$REQ" | jq -r '.functions[]?')
  fi
  if [ -z "$PROBLEMS" ]; then
    jq -nc --arg id "$ID" '{id: $id, status: "pass"}'
  else
    jq -nc --arg id "$ID" --arg m "$PROBLEMS" '{id: $id, status: "fail", message: $m}'
  fi
done
