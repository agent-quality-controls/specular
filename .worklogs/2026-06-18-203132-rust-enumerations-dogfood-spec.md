# Rust Enumerations Dogfood Spec

## Summary

Created the Specular dogfood contract for the Rust enumeration builtin plan and
removed the stale g3 migration work-order gap from the plan.

## Decisions Made

- Kept the dogfood spec on version 3 for this checkpoint so the current
  installed Specular can lint it and produce the expected failing verify report.
- Used built-in tree/content checks where v3 can express the requirement.
- Added a Python custom verifier for cross-repo AQC crate checks and new
  Specular behavior that cannot be checked by current built-ins before the
  Rust enumeration verifier exists.
- Rejected any g3 migration requirement because the updated plan treats g3 as a
  dead library and removes that proof step.

## Key Files For Context

- `.plans/2026-06-18-201059-rust-enumerations-builtin.md`
- `.plans/2026-06-18-201059-rust-enumerations-builtin.md.spec.json`
- `.plans/2026-06-18-201059-rust-enumerations-builtin.md.spec.coverage.md`
- `scripts/verify-rust-enumerations-plan.py`

## Verification

- `specular lint .plans/2026-06-18-201059-rust-enumerations-builtin.md.spec.json`
  passed.
- `specular verify .plans/2026-06-18-201059-rust-enumerations-builtin.md.spec.json`
  failed as expected because implementation has not started.

## Next Steps

Implement `aqc-rust-syntax` in `aqc-shared`, then implement
`builtin:rust-enumerations` in Specular and update the dogfood spec to the new
spec format version.
