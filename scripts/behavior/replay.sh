#!/usr/bin/env bash
# replay.sh <fixture.json>...
# fixture3 replay: copies each fixture's repo/ tree to a temp dir, runs the
# real driftless binary per the fixture's commands, and emits ONE JSON array of
# records {fixture, command, exit_code, output} on stdout.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"
cargo build --quiet >&2 || {
  echo '{"replay_error": "cargo build failed"}'
  exit 1
}
BIN="$ROOT/target/debug/driftless"

RECORDS="[]"
for FIX in "$@"; do
  DIR=$(dirname "$FIX")
  ID=$(jq -r '.id' "$FIX")
  TMP=$(mktemp -d)
  cp -R "$DIR/repo/." "$TMP/"
  while IFS= read -r CMD_JSON; do
    ARGS=()
    while IFS= read -r ARG; do ARGS+=("$ARG"); done < <(echo "$CMD_JSON" | jq -r '.[]')
    OUT=$(cd "$TMP" && "$BIN" "${ARGS[@]}" 2>/dev/null)
    CODE=$?
    OUT_JSON=$(echo "$OUT" | jq -c . 2>/dev/null) || OUT_JSON=$(jq -nc --arg raw "$OUT" '{raw: $raw}')
    REC=$(jq -nc --arg id "$ID" --argjson cmd "$CMD_JSON" --argjson code "$CODE" --argjson out "$OUT_JSON" \
      '{fixture: $id, command: $cmd, exit_code: $code, output: $out}')
    RECORDS=$(echo "$RECORDS" | jq -c --argjson r "$REC" '. + [$r]')
  done < <(jq -c '.commands[]' "$FIX")
  rm -rf "$TMP"
done
echo "$RECORDS" | jq 'sort_by(.fixture)'
