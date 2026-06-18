# Worklog: Cargo dependency verifier dogfood spec

## Summary

Added the dogfood Specular contract for the builtin Cargo dependency verifier
plan, plus a Python custom verifier and coverage map.

## Decisions Made

- Wrote the spec as version 3 because the plan removes `manifests` and adds
  `files` plus `forbiddenGlobs`, which is a breaking spec-format change.
- Used a Python custom verifier for implementation-specific checks that the
  current builtins cannot prove.
- Recorded that extraction was local rather than independently agent-isolated
  because sub-agents are only permitted when explicitly requested.

## Key Files For Context

- `.plans/2026-06-17-134859-builtin-cargo-dependencies.md`
- `.plans/2026-06-17-134859-builtin-cargo-dependencies.md.spec.json`
- `.plans/2026-06-17-134859-builtin-cargo-dependencies.md.spec.coverage.md`
- `scripts/verify-cargo-dependencies-plan.py`

## Next Steps

Implement the version 3 dependency block format and the
`builtin:cargo-dependencies` verifier until this dogfood spec passes.
