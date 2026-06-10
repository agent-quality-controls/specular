# Per-item spec model redesign

## Summary

Implemented `.plans/2026-06-10-154943-per-item-spec-model.md`: spec3 now uses item-based evidence instead of requirement IDs, uniform `required` / `exists` / `forbidden` typed blocks, an opaque `custom` category, JSON-only CLI output, and one verifier protocol for builtin and script-produced evidence. Updated HELP, build-contract specs, fixture suites, scripts, skill copies, and the older product plan to match.

## Decisions made

- Replaced the old ID model with item evidence. Tree is now one object; content/dependencies/exports/enumerations are target-keyed blocks; custom entries are only checked for object shape.
- Script verifiers now run per typed block as `<command...> <spec.json> <category> <blockIndex>` and must emit one line per item. Custom verifiers run once as `<command...> <spec.json> custom`; zero lines with exit 0 is a runtime error.
- Added `VERIFIER_COMMAND_EMPTY` as a lint error. Without it, an empty verifier command would become an indexing panic or late runtime failure.
- Typed script evidence now rejects unsupported extra fields and records inferred `polarity` in reports. Custom evidence keeps extra fields verbatim.
- Kept timeout at the plan-approved fixed 60 seconds and did not fixture-test timeout because that would slow the behavior suite.
- Replaced both plan specs with new-format contracts. The per-item redesign spec and the older product-plan spec both lint and verify true.
- Updated `~/.codex/skills/spec-driven-development/SKILL.md` and `~/.claude/skills/spec-driven-development/SKILL.md` from requirement-ID wording to item-target wording.

## Key files for context

- `.plans/2026-06-10-154943-per-item-spec-model.md`
- `.plans/2026-06-10-154943-per-item-spec-model.md.spec.json`
- `.plans/2026-06-10-154943-per-item-spec-model.md.spec.coverage.md`
- `HELP.txt`
- `src/model.rs`, `src/lint.rs`, `src/verify.rs`, `src/evidence.rs`, `src/main.rs`
- `scripts/verify-dependencies.sh`, `scripts/verify-exports.sh`, `scripts/verify-enumerations.sh`, `scripts/verify-custom.sh`
- `behavior/fixtures/` and `behavior/golden/`

## Verification

- `cargo fmt --check`
- `cargo check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `fixture3 check --all`
- `spec3 verify .plans/2026-06-10-154943-per-item-spec-model.md.spec.json` -> `conforms: true`
- `spec3 verify .plans/2026-06-07-124603-spec3-plan-v2-lock-free.md.spec.json` -> `conforms: true`

## Next steps

- Consider adding a fast unit-level protocol fixture for typed extra-field rejection if fixture3 coverage grows.
- Configurable timeout remains out of scope.
