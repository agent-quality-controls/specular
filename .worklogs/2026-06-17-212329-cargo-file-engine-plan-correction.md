# Worklog: Cargo file engine plan correction

## Summary

Updated the builtin Cargo dependency verifier plan to use the Cargo file engine
directly instead of the Guardrail3 cargo adapter. The plan now uses the current
AQC forbidden-glob API names and spells out the Specular-side code structure for
matching files, discovering dependency tables, running the Cargo engine, and
aggregating evidence.

## Decisions Made

- Kept Specular on the `aqc-cargo-toml-engine` and `aqc-file-engine-core`
  boundary because Specular needs Cargo.toml file-engine behavior, not
  Guardrail policy machinery.
- Replaced the old package-pattern terms with `DependencyPackageGlob` and
  `ForbiddenGlobRequirements`.
- Treated unreadable or invalid matched Cargo files as failing every item in
  the dependency block because presence and absence cannot be proven.
- Kept `[patch.<registry>]` out of the first Specular dependency builtin until
  the spec format has an explicit patch target.

## Key Files For Context

- `.plans/2026-06-17-134859-builtin-cargo-dependencies.md`
- `.worklogs/2026-06-17-135054-builtin-cargo-dependencies-plan.md`
- `/Users/tartakovsky/Projects/agent-quality-controls/aqc-shared/packages/file-types/toml/aqc-cargo-toml-engine/src/requirement/cargo_toml.rs`
- `/Users/tartakovsky/Projects/agent-quality-controls/aqc-shared/packages/file-types/toml/aqc-cargo-toml-engine/src/reconcile/dependencies.rs`

## Next Steps

- Extract the dogfood Specular spec from the corrected plan.
- Implement `src/cargo_dependencies.rs` against the Cargo file engine once the
  AQC crates are available as normal dependencies.
