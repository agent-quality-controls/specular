#!/usr/bin/env bash
# verify-custom.sh <spec.json> custom
# Checks this repository's opaque custom entries.
set -uo pipefail

SPEC="${1:?usage: verify-custom.sh <spec.json> custom}"
CATEGORY="${2:?usage: verify-custom.sh <spec.json> custom}"
[ "$CATEGORY" = "custom" ] || exit 2

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

jq -c '.requirements.custom[]' "$SPEC" | while IFS= read -r entry; do
  CHECK=$(echo "$entry" | jq -r '.check')
  FILE=$(echo "$entry" | jq -r '.file // empty')
  TEXT=$(echo "$entry" | jq -r '.text // empty')
  MODE=$(echo "$entry" | jq -r '.mode // "contains"')
  if [ ! -f "$FILE" ]; then
    jq -nc --arg check "$CHECK" --arg path "$FILE" \
      '{check: $check, status: "fail", path: $path, message: "file not found"}'
  elif [ "$MODE" = "contains" ] && grep -Fq -- "$TEXT" "$FILE"; then
    jq -nc --arg check "$CHECK" '{check: $check, status: "pass"}'
  elif [ "$MODE" = "absent" ] && ! grep -Fq -- "$TEXT" "$FILE"; then
    jq -nc --arg check "$CHECK" '{check: $check, status: "pass"}'
  else
    jq -nc --arg check "$CHECK" --arg path "$FILE" --arg text "$TEXT" \
      '{check: $check, status: "fail", path: $path, expected: $text, message: "custom check failed"}'
  fi
done
