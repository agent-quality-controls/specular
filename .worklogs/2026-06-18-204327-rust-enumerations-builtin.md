# Rust Enumerations Builtin

## Summary

Implemented `builtin:rust-enumerations` for file-scoped Rust enum checks and
bumped Specular to spec format version 4 / crate version 0.5.0.

## Decisions Made

- Used the published `aqc-rust-syntax = 0.1.0` crate for Rust parsing instead
  of source regexes or g3 migration code.
- Added `files` to enumeration blocks and require it only for
  `builtin:rust-enumerations`, preserving external enumeration verifiers.
- Match bare enum names by enum identifier and qualified names such as
  `wire::Status` by inline module path plus enum name.
- Treat multiple matched enum declarations with different variant sets as an
  ambiguous evidence failure.
- Emit explicit failure evidence for missing variants, unexpected variants, no
  matched files, missing enum declarations, ambiguous matches, and invalid Rust.
- Migrated committed specs and behavior fixtures to version 4 so the new binary
  accepts its own fixtures and dogfood contracts.

## Key Files For Context

- `src/rust_enumerations.rs`
- `src/model.rs`
- `src/lint.rs`
- `src/verify.rs`
- `HELP.txt`
- `README.md`
- `.plans/2026-06-18-201059-rust-enumerations-builtin.md.spec.json`
- `scripts/verify-rust-enumerations-plan.py`
- `behavior/fixtures/verify/verify-R30-rust-enumerations-clean`
- `behavior/fixtures/verify/verify-R31-rust-enumerations-failures`
- `behavior/fixtures/verify/verify-R32-rust-enumerations-invalid-rust`

## Verification

- `cargo test`
- `cargo clippy --all-targets`
- `fixture3 check --suite lint`
- `fixture3 check --suite verify`
- all `.plans/*.spec.json` lint with `target/debug/specular`
- `target/debug/specular verify .plans/2026-06-18-201059-rust-enumerations-builtin.md.spec.json`
- `target/debug/specular verify .plans/2026-06-17-134859-builtin-cargo-dependencies.md.spec.json`
- `target/debug/specular verify .plans/2026-06-12-140909-block-level-verifiers.md.spec.json`

## External State

- `aqc-rust-syntax v0.1.0` was published to crates.io from `aqc-shared`
  commit `c69b562`.
- The `aqc-shared` repo still has pre-existing unstaged rustfmt TOML engine
  changes that were not included in the AQC syntax crate commit.

## Next Steps

Release Specular 0.5.0 and install it locally after this commit is pushed.
