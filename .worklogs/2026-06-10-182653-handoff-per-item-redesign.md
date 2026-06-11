# Handoff: per-item spec model redesign (not yet implemented)

## What speculus is

Rust CLI + library. Verifies a repository against a JSON spec: `speculus lint
<spec>`, `speculus verify <spec>`. Exit codes: 0 valid/conforms, 1 nonconform,
2 error. Used by the spec-driven-development skill
(`~/.claude/skills/spec-driven-development/SKILL.md` + codex copies): prose plan
-> extracted spec -> agent builds until verify exits 0.

## Committed state (HEAD = d178221, all gates green)

- Working ID-based model: requirements carry SCREAMING_SNAKE ids; verifiers are
  a per-category map (builtin tree/content; scripts for dependencies/exports/
  enumerations); scripts judge and emit `{"id", "status"}` lines.
- Gates: `cargo check/clippy/fmt` clean; `fixture3 check --all` matched (two
  suites: lint, verify); the binary verifies its own build contract
  (`.plans/2026-06-07-124603-speculus-plan-v2-lock-free.md.spec.json`) exit 0.
- `speculus --help` prints `HELP.txt` (committed, embedded via include_str!).

## What we are doing: full format redesign

Plan: `.plans/2026-06-10-154943-per-item-spec-model.md` — read it first; it is
the single source for the new model. Summary:

- No IDs. The atom is the item (path, substring, package name); one report
  entry per item. Reason: items self-name; IDs were the main source of 3-pass
  extraction divergence.
- Quantifiers, same keys everywhere: `required` (every matched place), `exists`
  (at least one), `forbidden` (none). Strict-by-default; silent passes are the
  failure mode to avoid.
- `tree` is a single object; other typed categories are arrays of blocks with
  descriptive targets (`files`, `manifests`, `package`, `name`).
- `forbidden` accepts globs in tree/dependencies (`guardrail*`);
  `forbiddenPrefixes` and `schemas` deleted.
- `custom` category: free-form dict entries speculus never interprets; its
  verifier (declared in the `verifiers` map like every other category) does its
  own checks.
- One wire protocol for all scripts: evidence JSON lines, `status` mandatory;
  typed lines must carry `item`; custom lines are free beyond `status`. Script
  exit code = health only; 60s timeout per invocation.
- All output JSON only; `--json` flag deleted.

## Implementation status

NOT started. Everything committed still runs the ID model. Work order is in the
plan: library (model/lint/verify/evidence/main) -> HELP.txt -> own build
contract + scripts -> fixtures -> skill + plan-doc sync. Commit per unit with
worklogs.

## Open / flagged

- Default set without explicit user sign-off: a custom verifier emitting zero
  lines and exiting 0 = runtime error (silent verifier is broken). User has not
  objected; confirm if it surfaces.
- Timeout fixed at 60s (user approved "for now") — constant, not configurable.
- Timeout is not fixture-tested (too slow); note that in fixture docs.

## Session lessons (the user corrected these repeatedly)

- Short statements, lists, code blocks. No walls of text. Examples over
  abstractions; no jargon ("scope", "per-concern evidence" had to be re-explained).
- Format docs must contain complete, valid, copy-pasteable JSON — the embedded
  HELP example is extracted and linted in CI of the work itself.
- Do not touch files or pin design decisions without explicit go; propose in
  conversation first, with the analysis (options, edge cases, failure modes),
  not one-line suggestions.
- Consistency matters: one verifier model, one wire protocol, uniform keys —
  every special case I introduced got rejected.

## Cold-start reading list

1. `.plans/2026-06-10-154943-per-item-spec-model.md` (the work)
2. `.plans/2026-06-07-124603-speculus-plan-v2-lock-free.md` (product plan; some
   sections superseded by the above — sync is step 11 of the new plan)
3. `src/` (5 files, small), `HELP.txt`
4. `fixture3.yaml` + `behavior/` (fixture suites; `fixture3 check --all`)
5. The skill: `~/.claude/skills/spec-driven-development/SKILL.md`

## Next step

Implement the plan, step 1: rewrite `src/model.rs` per the new format, then
lint/verify/evidence/main, keeping cargo gates green before moving to HELP.txt.
