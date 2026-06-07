#!/usr/bin/env bash
# verify-dependencies.sh <spec.json>
# Checks requirements.dependencies: requiredCrates declared somewhere in the workspace;
# forbiddenCrates declared nowhere.
# Fact source: cargo metadata (the ecosystem's machine interface) when a workspace exists;
# falls back to a manifest text scan when cargo metadata is unavailable, and reports
# required crates as failing when no manifests exist at all.
# Emits one JSON evidence line per requirement.
set -uo pipefail

SPEC="${1:?usage: verify-dependencies.sh <spec.json>}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

glob_to_re() {
  printf '%s' "$1" | sed \
    -e 's/[][().+?^${|}\\]/\\&/g' -e 's/\./\\./g' \
    -e 's|\*\*/|<DSL>|g' -e 's|\*\*|<DS>|g' -e 's|\*|[^/]*|g' \
    -e 's|<DSL>|(.*/)?|g' -e 's|<DS>|.*|g'
}

PATHS=$(git ls-files --cached --others --exclude-standard)

# Declared dependency names, workspace-wide.
SOURCE="cargo-metadata"
DECLARED=""
if [ -f Cargo.toml ] && DECLARED=$(cargo metadata --no-deps --format-version 1 2>/dev/null \
    | jq -r '.packages[].dependencies[].name' | sort -u); then
  :
else
  SOURCE="manifest-text-scan"
  DECLARED=""
fi

jq -c '.requirements.dependencies[]' "$SPEC" | while IFS= read -r REQ; do
  ID=$(echo "$REQ" | jq -r '.id')
  PROBLEMS=""

  MANIFESTS=""
  while IFS= read -r G; do
    [ -z "$G" ] && continue
    RE="^$(glob_to_re "$G")$"
    MANIFESTS="$MANIFESTS$(echo "$PATHS" | grep -E "$RE")"$'\n'
  done < <(echo "$REQ" | jq -r '.manifestGlobs[]?')
  MANIFESTS=$(echo "$MANIFESTS" | grep -v '^$' | sort -u)

  declared_has() {
    local CRATE="$1"
    if [ "$SOURCE" = "cargo-metadata" ]; then
      echo "$DECLARED" | grep -qx "$CRATE"
    else
      [ -n "$MANIFESTS" ] || return 1
      while IFS= read -r M; do
        [ -z "$M" ] && continue
        grep -Eq "^[[:space:]]*\"?$CRATE\"?[[:space:]]*=" "$M" && return 0
      done <<< "$MANIFESTS"
      return 1
    fi
  }

  while IFS= read -r C; do
    [ -z "$C" ] && continue
    if [ -z "$MANIFESTS" ]; then
      PROBLEMS="$PROBLEMS required crate '$C' absent (no Cargo.toml manifests exist);"
    elif ! declared_has "$C"; then
      PROBLEMS="$PROBLEMS required crate '$C' not declared (source: $SOURCE);"
    fi
  done < <(echo "$REQ" | jq -r '.requiredCrates[]?')

  while IFS= read -r C; do
    [ -z "$C" ] && continue
    if [ -n "$MANIFESTS" ] && declared_has "$C"; then
      PROBLEMS="$PROBLEMS forbidden crate '$C' declared (source: $SOURCE);"
    fi
  done < <(echo "$REQ" | jq -r '.forbiddenCrates[]?')

  while IFS= read -r P; do
    [ -z "$P" ] && continue
    [ -z "$MANIFESTS" ] && continue
    if [ "$SOURCE" = "cargo-metadata" ]; then
      HITS=$(echo "$DECLARED" | grep "^$P" | tr '\n' ' ')
    else
      HITS=""
      while IFS= read -r M; do
        [ -z "$M" ] && continue
        HITS="$HITS$(grep -hoE "^[[:space:]]*\"?$P[a-z0-9_-]*\"?[[:space:]]*=" "$M" 2>/dev/null | tr -d ' ="' )"$'\n'
      done <<< "$MANIFESTS"
      HITS=$(echo "$HITS" | grep -v '^$' | sort -u | tr '\n' ' ')
    fi
    [ -z "${HITS// /}" ] || PROBLEMS="$PROBLEMS crates with forbidden prefix '$P' declared: $HITS(source: $SOURCE);"
  done < <(echo "$REQ" | jq -r '.forbiddenCratePrefixes[]?')

  if [ -z "$PROBLEMS" ]; then
    jq -nc --arg id "$ID" '{id: $id, status: "pass"}'
  else
    jq -nc --arg id "$ID" --arg m "$PROBLEMS" '{id: $id, status: "fail", message: $m}'
  fi
done
