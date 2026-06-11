# Speculus rebrand

## Goal

Rename the current product surface to Speculus while leaving the active project folder path alone.

## Approach

- Change the Cargo package and binary name to `speculus`.
- Change Rust crate imports, module comments, CLI usage text, and help text to use Speculus / `speculus`.
- Rename the report version field to `speculus_version` because report JSON is part of the public interface.
- Update fixture commands, behavior replay, and approved verify goldens to expect the new binary and report field.
- Update plan/worklog filenames and prose references so repository-visible mentions use Speculus.
- Update local spec-driven-development skill copies so future sessions call `speculus`.

## Key Decisions

- The workspace folder is not renamed in this change because the current session is running inside it.
- Generic "spec" terms stay unchanged because they describe JSON spec files, not the product name.
- Existing unrelated `resume` changes stay untouched.

## Files To Modify

- `Cargo.toml`, `Cargo.lock`
- `src/main.rs`, `src/lib.rs`, `src/evidence.rs`, `src/model.rs`
- `HELP.txt`
- `scripts/behavior/replay.sh`, `fixture3.yaml`
- `behavior/golden/verify/approved.normalized.json`, `behavior/golden/verify/approved.meta.json`
- Existing `.plans/` and `.worklogs/` files that mention the previous name
- `/Users/tartakovsky/.codex/skills/spec-driven-development/SKILL.md`
- `/Users/tartakovsky/.claude/skills/spec-driven-development/SKILL.md`
