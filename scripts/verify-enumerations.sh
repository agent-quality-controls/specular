#!/usr/bin/env bash
# verify-enumerations.sh <spec.json>
# Checks requirements.enumerations: the named pub enum exists under src/ and its
# variant identifiers match the spec exactly (drift in either direction fails).
# Emits one JSON evidence line per requirement.
set -uo pipefail

SPEC="${1:?usage: verify-enumerations.sh <spec.json>}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

jq -c '.requirements.enumerations[]' "$SPEC" | while IFS= read -r REQ; do
  ID=$(echo "$REQ" | jq -r '.id')
  TYPE=$(echo "$REQ" | jq -r '.type')
  EXPECTED=$(echo "$REQ" | jq -r '.variants[]' | sort)
  PROBLEMS=""
  FILE=$(grep -rlE "pub enum $TYPE\b" src/ 2>/dev/null | head -1)
  if [ -z "$FILE" ]; then
    PROBLEMS="pub enum $TYPE not found under src/;"
  else
    # Take the enum block: from the declaration to the first line that is just '}'.
    BLOCK=$(awk "/pub enum $TYPE\b/{f=1} f{print} f && /^}/{exit}" "$FILE")
    # Variant identifiers: lines starting with an uppercase identifier,
    # optionally followed by payload/discriminant. Attributes and comments excluded.
    ACTUAL=$(echo "$BLOCK" | sed -nE 's/^[[:space:]]+([A-Z][A-Za-z0-9_]*)[[:space:]]*([,({=].*)?$/\1/p' | sort)
    if [ "$ACTUAL" != "$EXPECTED" ]; then
      PROBLEMS="variants of $TYPE differ; expected: $(echo $EXPECTED | tr '\n' ' '); observed: $(echo $ACTUAL | tr '\n' ' ') (in $FILE);"
    fi
  fi
  if [ -z "$PROBLEMS" ]; then
    jq -nc --arg id "$ID" '{id: $id, status: "pass"}'
  else
    jq -nc --arg id "$ID" --arg m "$PROBLEMS" '{id: $id, status: "fail", message: $m}'
  fi
done
