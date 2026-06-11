# Driftless CLI rebrand

## Summary

Renamed the product, crate, binary, docs, plans, worklogs, fixtures, and report field from the old name to Driftless CLI / `driftless`, while leaving the repository folder path unchanged. Updated local spec-driven-development skill copies so future agents use `driftless`.

## Decisions made

- CLI and package name are `driftless`; prose display name is Driftless CLI.
- Report field is now `driftless_version`.
- Local folder path references remain `/spec3` because the folder rename is out of scope.
- Historical plan and worklog filenames containing the old product name were renamed; generic "spec" terms that refer to JSON spec files were left intact.

## Key files for context

- `Cargo.toml`, `Cargo.lock`
- `src/main.rs`, `src/lib.rs`, `src/evidence.rs`, `src/model.rs`
- `HELP.txt`
- `scripts/behavior/replay.sh`
- `fixture3.yaml`, `behavior/golden/verify/approved.normalized.json`
- `.plans/2026-06-11-103743-driftless-rebrand.md`

## Verification

- `cargo fmt --check`
- `cargo check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `fixture3 check --all`
- `driftless verify .plans/2026-06-10-154943-per-item-spec-model.md.spec.json` -> `conforms: true`
- `driftless verify .plans/2026-06-07-124603-driftless-plan-v2-lock-free.md.spec.json` -> `conforms: true`
- `rg` scan for the old product name only finds explicit folder-path exceptions.

## Next steps

- Rename the repository folder when ready.
- Update remote repository metadata if the hosted repo is also being rebranded.
