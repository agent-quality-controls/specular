Summary
- Added a plan for Rust enum syntax facts in AQC and a future Specular `builtin:rust-enumerations` verifier.
- The plan keeps Rust syntax parsing separate from crate walking and public API resolution.

Decisions made
- Planned `aqc-rust-syntax` as a file-local fact crate, not a Guardrail3 rule package and not a Specular adapter.
- Scoped the first shared API to enum declarations and variant names only.
- Required one Guardrail3 enum-related consumer to migrate before treating the crate as a shared boundary.
- Planned Specular's Rust enum verifier as file-scoped and syntax-only, with `files` added to enumeration blocks.

Key files for context
- `.plans/2026-06-18-201059-rust-enumerations-builtin.md`
- `/Users/tartakovsky/Projects/agent-quality-controls/guardrail3/packages/rs/apparch/g3rs-apparch-ingestion/crates/runtime/src/run/source.rs`
- `/Users/tartakovsky/Projects/agent-quality-controls/guardrail3/packages/rs/apparch/g3rs-apparch-ingestion/crates/runtime/src/run/source_support.rs`
- `/Users/tartakovsky/Projects/agent-quality-controls/guardrail3/packages/rs/code/g3rs-code-source-checks/crates/runtime/src/parse/visitors/core.rs`
- `/Users/tartakovsky/Projects/agent-quality-controls/guardrail3/packages/rs/garde/g3rs-garde-ingestion/crates/runtime/src/source_analysis/parse/analysis.rs`

Next steps
- Implement and publish `aqc-rust-syntax` in `aqc-shared`.
- Migrate one Guardrail3 enum consumer to prove the boundary.
- Then dogfood and implement Specular `builtin:rust-enumerations`.
