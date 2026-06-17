# Worklog: builtin Cargo dependency verifier plan

## Summary

Planned the Specular migration for `builtin:cargo-dependencies`, including the
dependency block target rename from `manifests` to `files`, exact package checks,
forbidden package globs, AQC integration, documentation updates, fixtures,
dogfood coverage, and release blockers.

## Decisions Made

- Planned direct use of `aqc-cargo-toml-engine` and `aqc-file-engine-core`
  instead of depending on Guardrail3.
- Treated `required` and `exists` as exact Cargo package names.
- Treated `forbidden` as exact package names or package-name globs.
- Scoped the initial builtin to dependency-shaped Cargo tables and excluded
  `[patch.<registry>]` until Specular has an explicit patch target.
- Marked the `manifests` to `files` rename as a spec-format-breaking change.

## Key Files For Context

- `.plans/2026-06-17-134859-builtin-cargo-dependencies.md`
- `src/model.rs`
- `src/lint.rs`
- `src/verify.rs`
- `HELP.txt`
- `README.md`
- `/Users/tartakovsky/Projects/agent-quality-controls/aqc-shared/packages/file-types/toml/aqc-cargo-toml-engine/src/requirement/cargo_toml.rs`
- `/Users/tartakovsky/Projects/agent-quality-controls/aqc-shared/packages/file-types/toml/aqc-cargo-toml-engine/src/reconcile/dependencies.rs`

## Next Steps

- Accept or revise the plan.
- Push and publish the needed AQC engine crates, or use local path dependencies
  only during implementation.
- Extract a Specular dogfood spec from the accepted plan before implementation.
