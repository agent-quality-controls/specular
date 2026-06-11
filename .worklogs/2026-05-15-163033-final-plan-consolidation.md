Summary
- Consolidated the planning material into one active final plan.
- Deleted obsolete research and scratch plan files to remove ambiguity about the source of truth.

Decisions made
- Kept `.plans/2026-05-14-203402-driftless-preliminary-plan.md` as the only active plan.
- Removed the validation-model research memo and source-format scratch file from `.plans`.
- Rewrote the active plan around the current V1 decision: strict JSON, generated JSON Schema, Serde, Rust semantic validation, lock receipt, and fixture-based verification.

Key files for context
- `.plans/2026-05-14-203402-driftless-preliminary-plan.md`

Verification
- Listed `.plans` before consolidation.
- Replaced the active plan with a self-contained version.
- Removed obsolete plan files.

Next steps
- Implement from the single active plan.
- Resolve open decisions for canonicalization, path/glob/walk semantics, Git drift semantics, and evidence JSON shape before writing checker code.
