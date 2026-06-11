# Speculus rebrand

## Summary

Renamed the product surface from the prior name to Speculus / `speculus` across the Rust crate, CLI binary, help text, public report JSON, fixtures, plans, worklogs, and local spec-driven-development skill copies. Installed the new local `speculus` binary and removed the stale installed old binary.

## Decisions made

- Cargo package and binary are now `speculus`; prose display name is Speculus CLI where a display name is needed.
- The report field is now `speculus_version`, because report JSON is part of the public library/CLI interface.
- Tracked plan and worklog filenames containing the prior package name were renamed to `speculus`.
- The active project folder path was left unchanged because this session is running inside it.
- `.git` internals and remote metadata were not edited. The unrelated `resume` modification remains unstaged and untouched.

## Key files for context

- `Cargo.toml`, `Cargo.lock`
- `src/main.rs`, `src/lib.rs`, `src/evidence.rs`, `src/model.rs`
- `HELP.txt`
- `fixture3.yaml`
- `scripts/behavior/replay.sh`
- `behavior/golden/verify/approved.normalized.json`
- `.plans/2026-06-11-133803-speculus-rebrand.md`
- `/Users/tartakovsky/.codex/skills/spec-driven-development/SKILL.md`
- `/Users/tartakovsky/.claude/skills/spec-driven-development/SKILL.md`

## Verification

- `cargo fmt --check`
- `cargo check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `fixture3 check --all`
- `./target/debug/speculus verify .plans/2026-06-10-154943-per-item-spec-model.md.spec.json` -> `conforms: true`
- `./target/debug/speculus verify .plans/2026-06-07-124603-speculus-plan-v2-lock-free.md.spec.json` -> `conforms: true`
- `speculus --help` prints the Speculus help text from the installed binary.
- `rg` found no tracked old product-name mentions outside `.git` and excluded `resume`.

## Next steps

- Rename the project folder when ready.
- Update hosted repository metadata or Git remotes if the remote repository is also renamed.
