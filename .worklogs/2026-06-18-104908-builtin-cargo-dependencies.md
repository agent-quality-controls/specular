# Worklog: builtin Cargo dependency verifier

## Summary

Implemented spec format version 3 and the `builtin:cargo-dependencies`
verifier. Dependency blocks now use `files`, exact `forbidden`, and
`forbiddenGlobs`.

## Decisions Made

- Used crates.io `aqc-cargo-toml-engine` and `aqc-file-engine-core` directly,
  plus `toml_edit` only for discovering dependency-shaped tables.
- Kept exact package bans and package-name glob bans as separate public fields
  so Specular never infers meaning from punctuation inside `forbidden`.
- Added `src/cargo_dependencies.rs` as the Cargo-specific boundary. `verify.rs`
  only dispatches to it.
- Updated legacy dogfood specs to version 3 so current repo contracts still
  verify after the breaking format change.
- Preserved external dependency verifier compatibility by emitting evidence for
  `forbiddenGlobs` as forbidden-polarity items.

## Key Files For Context

- `.plans/2026-06-17-134859-builtin-cargo-dependencies.md`
- `.plans/2026-06-17-134859-builtin-cargo-dependencies.md.spec.json`
- `src/model.rs`
- `src/lint.rs`
- `src/verify.rs`
- `src/cargo_dependencies.rs`
- `HELP.txt`
- `README.md`
- `behavior/fixtures/verify/verify-R27-cargo-dependencies-clean/repo/spec.json`
- `behavior/fixtures/verify/verify-R28-cargo-dependencies-failures/repo/spec.json`
- `behavior/fixtures/verify/verify-R29-cargo-dependencies-invalid/repo/spec.json`

## Verification

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- behavior replay diff for lint and verify fixtures
- all `.plans/*.spec.json` verified with `cargo run --quiet -- verify`
- current Cargo dependency dogfood spec verified
- `slopless README.md`
- `cargo package --no-verify --allow-dirty`
- `scripts/verify-cli-version.py`

## Next Steps

Release `0.4.0` when ready, then install the released CLI locally and continue
with the next ecosystem-specific builtin verifier.
