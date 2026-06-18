#!/usr/bin/env bash
# verify-dependencies.sh <spec.json> dependencies <blockIndex>
# Emits one JSON evidence line per package item in the selected block.
set -uo pipefail

SPEC="${1:?usage: verify-dependencies.sh <spec.json> dependencies <blockIndex>}"
CATEGORY="${2:?usage: verify-dependencies.sh <spec.json> dependencies <blockIndex>}"
INDEX="${3:?usage: verify-dependencies.sh <spec.json> dependencies <blockIndex>}"
[ "$CATEGORY" = "dependencies" ] || exit 2

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

BLOCK=$(jq -c --argjson index "$INDEX" '.requirements.dependencies[$index]' "$SPEC")

DECLARED=""
SOURCE="cargo-metadata"
if [ -f Cargo.toml ] && DECLARED=$(cargo metadata --no-deps --format-version 1 2>/dev/null |
  jq -r '.packages[].dependencies[].name' | sort -u); then
  :
else
  SOURCE="manifest-text-scan"
  DECLARED=$(find . -maxdepth 3 -name Cargo.toml -print0 2>/dev/null |
    xargs -0 grep -hE '^[[:space:]]*"?[A-Za-z0-9_-]+"?[[:space:]]*=' 2>/dev/null |
    sed -E 's/^[[:space:]]*"?([A-Za-z0-9_-]+)"?[[:space:]]*=.*/\1/' |
    sort -u)
fi

has_declared() {
  echo "$DECLARED" | grep -qx "$1"
}

matches_declared() {
  local pattern="$1"
  while IFS= read -r name; do
    [ -z "$name" ] && continue
    case "$name" in
      $pattern) echo "$name" ;;
    esac
  done <<< "$DECLARED"
}

echo "$BLOCK" | jq -r '.required[]?' | while IFS= read -r item; do
  if has_declared "$item"; then
    jq -nc --arg item "$item" '{item: $item, status: "pass"}'
  else
    jq -nc --arg item "$item" --arg source "$SOURCE" \
      '{item: $item, status: "fail", message: ("package not declared; source: " + $source)}'
  fi
done

echo "$BLOCK" | jq -r '.exists[]?' | while IFS= read -r item; do
  if has_declared "$item"; then
    jq -nc --arg item "$item" '{item: $item, status: "pass"}'
  else
    jq -nc --arg item "$item" --arg source "$SOURCE" \
      '{item: $item, status: "fail", message: ("package not declared; source: " + $source)}'
  fi
done

echo "$BLOCK" | jq -r '.forbidden[]?' | while IFS= read -r item; do
  HITS=$(matches_declared "$item" | tr '\n' ' ')
  if [ -z "${HITS// /}" ]; then
    jq -nc --arg item "$item" '{item: $item, status: "pass"}'
  else
    jq -nc --arg item "$item" --arg observed "$HITS" --arg source "$SOURCE" \
      '{item: $item, status: "fail", message: ("forbidden package declared; source: " + $source), observed: $observed}'
  fi
done

echo "$BLOCK" | jq -r '.forbiddenGlobs[]?' | while IFS= read -r item; do
  HITS=$(matches_declared "$item" | tr '\n' ' ')
  if [ -z "${HITS// /}" ]; then
    jq -nc --arg item "$item" '{item: $item, status: "pass"}'
  else
    jq -nc --arg item "$item" --arg observed "$HITS" --arg source "$SOURCE" \
      '{item: $item, status: "fail", message: ("forbidden package declared; source: " + $source), observed: $observed}'
  fi
done
