# Per-category verifiers + front-loaded help

## Summary

Implemented the design from `.plans/2026-06-07-202732-*`. Verifiers are now per
category (builtin or an overriding command in a top-level `verifiers` map);
required+forbidden merge into one row per scope; empty categories are omittable;
all format docs moved into `driftless help`; the skill shrank to a pointer. All gates
green: cargo build/clippy/fmt clean, `fixture3 check --all` matched, the binary
verifies its own build contract (exit 0, 7 requirements).

## Decisions made (implemented)

- `verifiers`: `Vec<VerifierDecl>` (per-requirement `requirementIds`) ->
  `BTreeMap<String, Vec<String>>` (category -> command). `VerifierDecl` and
  `VerifierId` deleted. `VerifierSource::Custom` now carries `Category`.
- Override allowed: listing a builtin category in the map replaces its builtin.
- Merge rule groups by category+scope only (drops polarity) -> one dependencies
  row, one tree row, etc.
- Empty categories omittable: `#[serde(default)]` on every `Requirements` field
  and on each requirement's polarity list fields (so rows omit unused lists).
- Lint: dropped UNKNOWN_CLAIM / OVERLAPPING_CLAIM / BUILTIN_COVERED_CLAIM /
  UNCLAIMED_REQUIREMENT / VERIFIER_COMMAND_MISSING. Added CATEGORY_HAS_NO_VERIFIER
  and UNKNOWN_CATEGORY (clean violation via string-keyed map check, not a serde
  abort).
- Report tally removed (`source_counts` gone); per-item source is the contract.
- Case D (override command file missing) fails at verify (stamp/spawn), exit 2 —
  not a lint concern, since the spec JSON is identical whether the file exists.
- `verify` runs an override as `<command...> <spec.json> <category>`; coverage is
  per category; a line for an id outside the category is a protocol error.
- `driftless help` / `--help` / `-h` prints `include_str!("../HELP.txt")`. HELP.txt is
  the committed help body and a required file in the build contract.
- The build contract is now verified by the BINARY (dogfood): builtin tree+content
  plus the three bash scripts as per-category overrides.

## Key files

- `src/model.rs` (verifiers map, omittable fields, Category::ALL/parse/has_builtin)
- `src/verify.rs` (per-category dispatch, run_override, category_ids)
- `src/lint.rs` (check_verifiers, scope-only check_mergeable)
- `src/evidence.rs` (VerifierSource::Custom(Category), no source_counts)
- `src/main.rs` (help command, no tally), `HELP.txt`
- `.plans/2026-06-07-124603-...md` + `.spec.json` + `.spec.coverage.md`
- `behavior/fixtures/{lint,verify}/*`, `behavior/golden/*`
- skill: `~/.claude/skills/spec-driven-development/SKILL.md` (232 -> 67 lines) + codex copies

## Next steps

- New builtin verifiers (deps-cargo etc.) when usage recurs.
- Consider removing the now-redundant bash orchestrators (scripts/spec-lint.sh,
  spec-verify.sh, verify-tree.sh, verify-content.sh) — the binary supersedes them
  for this repo; the three category verifier scripts stay (used as overrides).
