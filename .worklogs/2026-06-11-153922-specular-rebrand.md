# Specular rebrand

## Summary

Renamed the product surface to Specular / `specular` across the Rust crate, CLI binary, help text, public report JSON, fixtures, plans, worklogs, and local spec-driven-development skill copies. Installed the new local `specular` binary and removed the stale previously installed binary.

## Decisions made

- Cargo package and binary are now `specular`; prose display name is Specular CLI where a display name is needed.
- The report field is now `specular_version`, because report JSON is part of the public library/CLI interface.
- Tracked plan and worklog filenames containing the prior package name were renamed to `specular`.
- The active project folder path was left unchanged because this session is running inside it.
- `.git` internals and remote metadata were not edited. The unrelated `resume` modification remains unstaged and untouched.

## Key files for context

- `Cargo.toml`, `Cargo.lock`
- `src/main.rs`, `src/lib.rs`, `src/evidence.rs`, `src/model.rs`
- `HELP.txt`
- `fixture3.yaml`
- `scripts/behavior/replay.sh`
- `behavior/golden/verify/approved.normalized.json`
- `.plans/2026-06-11-153744-specular-rebrand.md`
- `/Users/tartakovsky/.codex/skills/spec-driven-development/SKILL.md`
- `/Users/tartakovsky/.claude/skills/spec-driven-development/SKILL.md`

## Verification

- `cargo fmt --check`
- `cargo check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `fixture3 check --all`
- `./target/debug/specular verify .plans/2026-06-10-154943-per-item-spec-model.md.spec.json` -> `conforms: true`
- `./target/debug/specular verify .plans/2026-06-07-124603-specular-plan-v2-lock-free.md.spec.json` -> `conforms: true`
- `specular --help` prints the Specular help text from the installed binary.
- `rg` found no tracked old product-name mentions outside `.git` and excluded `resume`.

## Next steps

- Rename the project folder when ready.
- Update hosted repository metadata or Git remotes if the remote repository is also renamed.
