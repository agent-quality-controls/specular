#!/usr/bin/env bash
# verify-enumerations.sh <spec.json> enumerations <blockIndex>
# Emits one JSON evidence line per expected value in the selected block.
set -uo pipefail

SPEC="${1:?usage: verify-enumerations.sh <spec.json> enumerations <blockIndex>}"
CATEGORY="${2:?usage: verify-enumerations.sh <spec.json> enumerations <blockIndex>}"
INDEX="${3:?usage: verify-enumerations.sh <spec.json> enumerations <blockIndex>}"
[ "$CATEGORY" = "enumerations" ] || exit 2

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BLOCK=$(jq -c --argjson index "$INDEX" '.requirements.enumerations[$index]' "$SPEC")
NAME=$(echo "$BLOCK" | jq -r '.name')
EXPECTED=$(echo "$BLOCK" | jq -r '.values[]?' | sort)
FILE=$(grep -rlE "pub enum $NAME\b" src/ 2>/dev/null | head -1)
ACTUAL=""
if [ -n "$FILE" ]; then
  ENUM_BLOCK=$(awk "/pub enum $NAME[ {]/{f=1} f{print} f && /^}/{exit}" "$FILE")
  ACTUAL=$(echo "$ENUM_BLOCK" |
    sed -nE 's/^[[:space:]]+([A-Z][A-Za-z0-9_]*)[[:space:]]*([,({=].*)?$/\1/p' |
    sort)
fi

EXTRA=$(comm -13 <(echo "$EXPECTED") <(echo "$ACTUAL") | tr '\n' ' ')
echo "$BLOCK" | jq -r '.values[]?' | while IFS= read -r item; do
  if [ -z "$FILE" ]; then
    jq -nc --arg item "$item" --arg name "$NAME" \
      '{item: $item, status: "fail", message: ("enum " + $name + " not found")}'
  elif echo "$ACTUAL" | grep -qx "$item" && [ -z "${EXTRA// /}" ]; then
    jq -nc --arg item "$item" '{item: $item, status: "pass"}'
  elif echo "$ACTUAL" | grep -qx "$item"; then
    jq -nc --arg item "$item" --arg observed "$EXTRA" \
      '{item: $item, status: "fail", message: "enum has extra values", observed: $observed}'
  else
    jq -nc --arg item "$item" \
      '{item: $item, status: "fail", message: "enum value not found"}'
  fi
done
