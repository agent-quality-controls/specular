# Block-level verifier migration

## Summary

Migrated Specular to spec format version 2 with one explicit verifier command on
each non-empty requirement block. Removed top-level `verifiers`, added named
built-ins (`builtin:tree`, `builtin:content`), and made script evidence report
the verifier that produced it.

## Decisions made

- Kept `verifier` singular because each block is judged by exactly one verifier.
- Kept verifier commands as argv arrays so Specular can run scripts without a
  shell and without quoting ambiguity.
- Treated `builtin:<name>` as the dispatch prefix. Unknown built-ins and
  category mismatches are lint errors.
- Kept custom checks as one object per `requirements.custom[]` entry. Each
  entry owns its verifier and can carry script-owned fields.
- Stamped repo-relative script files found in verifier argv parts. Built-ins are
  not file-stamped.
- Updated fixture specs, golden outputs, README, and help text to the v2 shape.

## Key files for context

- `src/model.rs`: v2 model, `VerifierCommand`, and custom entry shape.
- `src/lint.rs`: v2 semantic lint rules for block-level verifiers.
- `src/verify.rs`: block dispatch, builtin execution, script protocol, and
  verifier file stamping.
- `src/evidence.rs`: report evidence now includes `verifier`; script evidence
  uses `source: "script"`.
- `HELP.txt` and `README.md`: user-facing v2 spec format and verifier protocol.
- `.plans/2026-06-12-140909-block-level-verifiers.md`: implementation plan.
- `.plans/2026-06-12-140909-block-level-verifiers.md.spec.json`: dogfood spec.
- `scripts/verify-block-level-verifiers.py`: custom dogfood verifier.

## Verification

- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `fixture3 check --all`
- `cargo run --quiet -- lint .plans/2026-06-12-140909-block-level-verifiers.md.spec.json`
- `cargo run --quiet -- verify .plans/2026-06-12-140909-block-level-verifiers.md.spec.json`
- `cargo run --quiet -- verify .plans/2026-06-11-171235-repo-quality-hardening.md.spec.json`
- `python3 -m py_compile scripts/verify-block-level-verifiers.py scripts/verify-repo-quality.py scripts/verify-cli-version.py`
- `python3 scripts/verify-cli-version.py`
- `slopless README.md`
- `cargo package --no-verify --allow-dirty`

## Next steps

- Bump the crate version before release because v2 is a breaking spec format
  change.
- Publish only after the version bump commit and a clean package run.
