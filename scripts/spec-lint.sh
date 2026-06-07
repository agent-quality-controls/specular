#!/usr/bin/env bash
# spec-lint.sh <spec.json> [--skip-coverage]
# Validates a spec file alone (spec-driven-development skill, section 5).
# --skip-coverage: omit the verifier-script coverage check (extraction-candidate stage only).
# Exit: 0 = spec valid; 2 = spec invalid or runtime error.
set -uo pipefail

SPEC="${1:?usage: spec-lint.sh <spec.json> [--skip-coverage]}"
SKIP_COVERAGE="${2:-}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FAIL=0
err() { echo "LINT FAIL: $*" >&2; FAIL=1; }

command -v jq >/dev/null || { echo "LINT FAIL: jq not installed" >&2; exit 2; }
[ -f "$SPEC" ] || { echo "LINT FAIL: no such file: $SPEC" >&2; exit 2; }

# JSON parses
jq empty "$SPEC" 2>/dev/null || { echo "LINT FAIL: not valid JSON" >&2; exit 2; }

# version
[ "$(jq -r '.version' "$SPEC")" = "1" ] || err "version must be 1"

# top-level keys
EXTRA=$(jq -r '[keys[] | select(. != "version" and . != "requirements")] | join(", ")' "$SPEC")
[ -z "$EXTRA" ] || err "unknown top-level keys: $EXTRA"
jq -e '.requirements | type == "object"' "$SPEC" >/dev/null || { err "requirements must be an object"; echo "LINT FAIL" >&2; exit 2; }

# categories: exactly these six, each an array
CATS='["tree","content","dependencies","exports","enumerations","schemas"]'
EXTRA=$(jq -r --argjson c "$CATS" '[.requirements | keys[] | select(. as $k | $c | index($k) | not)] | join(", ")' "$SPEC")
[ -z "$EXTRA" ] || err "unknown categories: $EXTRA"
MISSING=$(jq -r --argjson c "$CATS" '($c - (.requirements | keys)) | join(", ")' "$SPEC")
[ -z "$MISSING" ] || err "missing categories (use empty arrays): $MISSING"
jq -e '[.requirements[]] | all(type == "array")' "$SPEC" >/dev/null || err "every category must be an array"

# ids: present, SCREAMING_SNAKE_CASE, unique across all categories
IDS=$(jq -r '.requirements[] | .[]? | .id // "<<MISSING>>"' "$SPEC")
if [ -n "$IDS" ]; then
  echo "$IDS" | grep -q '^<<MISSING>>$' && err "requirement without id"
  BAD=$(echo "$IDS" | grep -v '^<<MISSING>>$' | grep -vE '^[A-Z][A-Z0-9_]*$' | tr '\n' ' ')
  [ -z "${BAD// /}" ] || err "ids not SCREAMING_SNAKE_CASE: $BAD"
  DUPS=$(echo "$IDS" | sort | uniq -d | tr '\n' ' ')
  [ -z "${DUPS// /}" ] || err "duplicate ids: $DUPS"
fi

# paths and globs: repo-root-relative, no '..', no absolute paths, no empty components
PATHS=$(jq -r '[.. | objects | (.requiredPaths? // [], .forbiddenGlobs? // [], .files? // [])] | flatten | .[]' "$SPEC")
while IFS= read -r P; do
  [ -z "$P" ] && continue
  case "$P" in
    /*) err "absolute path: $P" ;;
  esac
  case "$P" in
    *..*) err "path contains '..': $P" ;;
    *//*) err "path has empty component: $P" ;;
  esac
done <<< "$PATHS"

# MERGEABLE_REQUIREMENTS: granularity is derived, not chosen.
# Two rows in a category with identical scope fields and polarity are one rule by definition.
MERGEABLE=$(jq -r '
  def pairs(cat; scope; pol): [.requirements[cat][]? | select((.[pol] // []) | length > 0)]
    | group_by(scope) | map(select(length > 1) | map(.id) | join("+")) | .[];
  [ pairs("tree"; ""; "requiredPaths"),
    pairs("tree"; ""; "forbiddenGlobs"),
    pairs("content"; (.files // [] | sort); "forbiddenSubstrings"),
    pairs("content"; (.files // [] | sort); "requiredSubstrings"),
    pairs("dependencies"; (.manifestGlobs // [] | sort); "requiredCrates"),
    pairs("dependencies"; (.manifestGlobs // [] | sort); "forbiddenCrates") ]
  | join("; ")' "$SPEC")
[ -z "$MERGEABLE" ] || err "MERGEABLE_REQUIREMENTS (same scope and polarity; merge into one row): $MERGEABLE"

# VACUOUS_SPEC: at least one positive assertion must exist.
jq -e '[.requirements | (.tree[]?.requiredPaths // []), (.content[]?.requiredSubstrings // []), (.dependencies[]?.requiredCrates // [])] | flatten | length > 0' "$SPEC" >/dev/null \
  || err "VACUOUS_SPEC: no positive assertion; this spec passes on an empty repository"

# verifier coverage: each non-empty category requires an executable scripts/verify-<category>.sh
if [ "$SKIP_COVERAGE" != "--skip-coverage" ]; then
  for CAT in tree content dependencies exports enumerations schemas; do
    N=$(jq -r --arg c "$CAT" '.requirements[$c] | length' "$SPEC")
    if [ "$N" -gt 0 ] && [ ! -x "$ROOT/scripts/verify-$CAT.sh" ]; then
      err "category '$CAT' non-empty but scripts/verify-$CAT.sh missing or not executable"
    fi
  done
fi

[ "$FAIL" -eq 0 ] || exit 2
echo "LINT PASS: $SPEC"
