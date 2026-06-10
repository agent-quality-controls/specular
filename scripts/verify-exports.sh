#!/usr/bin/env bash
# verify-exports.sh <spec.json> exports <blockIndex>
# Emits one JSON evidence line per export item in the selected block.
set -uo pipefail

SPEC="${1:?usage: verify-exports.sh <spec.json> exports <blockIndex>}"
CATEGORY="${2:?usage: verify-exports.sh <spec.json> exports <blockIndex>}"
INDEX="${3:?usage: verify-exports.sh <spec.json> exports <blockIndex>}"
[ "$CATEGORY" = "exports" ] || exit 2

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BLOCK=$(jq -c --argjson index "$INDEX" '.requirements.exports[$index]' "$SPEC")

has_export() {
  local item="$1"
  [ -d src ] || return 1
  grep -rqE "pub (struct|enum|type|trait) $item\b|pub fn $item\b|pub use [^;]*\b$item\b" src/
}

echo "$BLOCK" | jq -r '.required[]?' | while IFS= read -r item; do
  if has_export "$item"; then
    jq -nc --arg item "$item" '{item: $item, status: "pass"}'
  else
    jq -nc --arg item "$item" '{item: $item, status: "fail", message: "public item not found"}'
  fi
done

echo "$BLOCK" | jq -r '.exists[]?' | while IFS= read -r item; do
  if has_export "$item"; then
    jq -nc --arg item "$item" '{item: $item, status: "pass"}'
  else
    jq -nc --arg item "$item" '{item: $item, status: "fail", message: "public item not found"}'
  fi
done

echo "$BLOCK" | jq -r '.forbidden[]?' | while IFS= read -r item; do
  if has_export "$item"; then
    jq -nc --arg item "$item" '{item: $item, status: "fail", message: "forbidden public item found"}'
  else
    jq -nc --arg item "$item" '{item: $item, status: "pass"}'
  fi
done
