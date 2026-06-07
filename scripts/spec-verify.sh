#!/usr/bin/env bash
# spec-verify.sh <spec.json>
# Checks the implementation against the spec (spec-driven-development skill, section 5).
# Runs lint, then every category verifier; enforces evidence coverage; prints the report.
# Exit: 0 = conforms; 1 = does not conform; 2 = spec, verifier, or runtime error.
set -uo pipefail

SPEC="${1:?usage: spec-verify.sh <spec.json>}"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Lint first; any lint failure is exit 2.
scripts/spec-lint.sh "$SPEC" >/dev/null || { echo "exit 2 (lint failed; run scripts/spec-lint.sh $SPEC)" >&2; exit 2; }

EVIDENCE="$(mktemp)"
trap 'rm -f "$EVIDENCE"' EXIT

# Report header: input-closure stamps.
echo "spec sha256: $(shasum -a 256 "$SPEC" | cut -d' ' -f1)"
STAMPS=""
for CAT in tree content dependencies exports enumerations schemas; do
  N=$(jq -r --arg c "$CAT" '.requirements[$c] | length' "$SPEC")
  [ "$N" -gt 0 ] && STAMPS="$STAMPS verify-$CAT.sh $(shasum -a 256 "scripts/verify-$CAT.sh" | cut -d' ' -f1)"
done
echo "verifiers sha256:${STAMPS:- (none)}"
echo "note: all verifiers are agent-authored (manual workflow; no builtin checkers exist yet)"

# Run each non-empty category's verifier; collect JSON-line evidence.
for CAT in tree content dependencies exports enumerations schemas; do
  N=$(jq -r --arg c "$CAT" '.requirements[$c] | length' "$SPEC")
  [ "$N" -eq 0 ] && continue
  OUT=$("scripts/verify-$CAT.sh" "$SPEC") || { echo "exit 2 (verify-$CAT.sh runtime error)" >&2; exit 2; }
  while IFS= read -r LINE; do
    [ -z "$LINE" ] && continue
    echo "$LINE" | jq -e '.id and (.status == "pass" or .status == "fail")' >/dev/null 2>&1 \
      || { echo "exit 2 (verify-$CAT.sh protocol violation: $LINE)" >&2; exit 2; }
    echo "$LINE" | jq -c --arg cat "$CAT" '. + {category: $cat}' >> "$EVIDENCE"
  done <<< "$OUT"
done

# Evidence coverage: every requirement ID reports exactly once; unknown IDs fatal.
SPEC_IDS=$(jq -r '.requirements[] | .[]? | .id' "$SPEC" | sort)
REPORTED_IDS=$(jq -r '.id' "$EVIDENCE" 2>/dev/null | sort)
UNKNOWN=$(comm -13 <(echo "$SPEC_IDS") <(echo "$REPORTED_IDS" | uniq) | tr '\n' ' ')
[ -z "${UNKNOWN// /}" ] || { echo "exit 2 (evidence cites unknown ids: $UNKNOWN)" >&2; exit 2; }
DUP=$(echo "$REPORTED_IDS" | uniq -d | tr '\n' ' ')
[ -z "${DUP// /}" ] || { echo "exit 2 (ids reported more than once: $DUP)" >&2; exit 2; }
MISSING=$(comm -23 <(echo "$SPEC_IDS") <(echo "$REPORTED_IDS") | tr '\n' ' ')
[ -z "${MISSING// /}" ] || { echo "exit 2 (requirements with no evidence: $MISSING)" >&2; exit 2; }

# Per-ID report lines.
FAILS=0
while IFS= read -r LINE; do
  ID=$(echo "$LINE" | jq -r '.id')
  CAT=$(echo "$LINE" | jq -r '.category')
  STATUS=$(echo "$LINE" | jq -r '.status')
  if [ "$STATUS" = "pass" ]; then
    echo "$ID ($CAT): pass"
  else
    FAILS=$((FAILS + 1))
    MSG=$(echo "$LINE" | jq -r '[.message // empty, (if .observed != null then "observed: \(.observed)" else empty end), (if .expected != null then "expected: \(.expected)" else empty end), (.path // empty)] | join(" | ")')
    echo "$ID ($CAT): FAIL${MSG:+ — $MSG}"
  fi
done < "$EVIDENCE"

TOTAL=$(echo "$SPEC_IDS" | grep -c . || true)
echo "coverage: $TOTAL/$TOTAL requirements reported"

[ "$FAILS" -eq 0 ] || exit 1
exit 0
