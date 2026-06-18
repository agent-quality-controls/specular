# Worklog: dependency forbidden glob plan

## Summary

Updated the builtin Cargo dependency verifier plan to split exact forbidden
packages from forbidden package globs.

## Decisions Made

- Kept `forbidden` exact-only so Specular does not infer package intent from
  glob punctuation.
- Added `forbiddenGlobs` for package-name glob bans because the Cargo file
  engine already models exact package requirements and glob package bans as
  separate requirement fields.
- Updated lint, fixture, documentation, helper, evidence, and dogfood planning
  so the future implementation has one public shape for exact bans and one
  public shape for glob bans.

## Key Files For Context

- `.plans/2026-06-17-134859-builtin-cargo-dependencies.md`
- `.worklogs/2026-06-17-210832-handoff-specular-cargo-dependency-checks.md`
- `.worklogs/2026-06-17-212329-cargo-file-engine-plan-correction.md`

## Next Steps

Implement the Cargo dependency verifier from the updated plan, starting with the
Specular model and lint changes for `files`, exact `forbidden`, and
`forbiddenGlobs`.
