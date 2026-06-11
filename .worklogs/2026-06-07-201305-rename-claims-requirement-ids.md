# Rename: claims -> requirementIds

## Summary

Renamed the custom verifier declaration field `claims` to `requirementIds` (Rust: `requirement_ids`) — the field is just the list of requirement IDs the verifier owns; "claims" added a second noun for an existing concept. All gates green after the rename.

## Decisions made

- Field only: violation codes (UNKNOWN_CLAIM etc.) and prose keep "claim" as the verb for the act; the data is named for what it contains.
- Fixture goldens re-approved: drift was exactly the spec sha256 stamps of fixture spec.json files (field rename changes bytes), verified line-by-line before approval.

## Key files for context

- src/model.rs (VerifierDecl), src/lint.rs, src/verify.rs
- .plans/2026-06-07-124603-driftless-plan-v2-lock-free.md (Custom Verifiers section)
- behavior/fixtures/*/repo/spec.json (7 files), behavior/golden/verify/

## Next steps

- None pending; future builtin verifiers (deps-cargo first) when usage recurs.
