# Coverage Map: Block-level verifiers

Extraction note: this spec was not produced by isolated sub-agents because sub-agent use was not explicitly requested. I performed the extraction locally after reading the current model, lint, verify, evidence, help text, and recent worklogs.

## Goal

- Coverage: content checks for docs and core implementation files
- Requirement targets:
  - `src/model.rs`
  - `src/lint.rs`
  - `src/verify.rs`
  - `src/evidence.rs`
  - `HELP.txt`
  - `README.md`

## New Spec Shape

- Coverage: content checks, custom verifier
- Requirement targets:
  - model checks for `VerifierCommand`, block-level `verifier`, version 2, and no top-level `Spec.verifiers`
  - docs checks for version 2 examples and explicit block-level verifier commands
  - custom check `protocol-docs`

## Requirement Categories

- Coverage: content checks, custom verifier
- Requirement targets:
  - model checks for typed block verifier fields and custom typed object
  - docs checks for `builtin:tree`, `builtin:content`, and script verifier examples
  - custom check `valid-fixture-specs-v2`

## Verifier Protocol

- Coverage: content checks, custom verifier
- Requirement targets:
  - `src/verify.rs` checks for block index dispatch and custom handling
  - `scripts/verify-repo-quality.py` checks for custom block-index protocol
  - custom check `new-lint-fixtures`

## Lint Rules

- Coverage: content checks, custom verifier
- Requirement targets:
  - `src/lint.rs` checks for new lint error names and removal of category-level verifier errors
  - custom check `new-lint-fixtures`

## Verify Dispatch

- Coverage: content checks, custom verifier
- Requirement targets:
  - `src/verify.rs` checks for builtin registry names, script dispatch, and removal of category-level lookup
  - `src/evidence.rs` checks for `verifier` and script source
  - custom check `golden-reports`

## Implementation Work

- Coverage: tree and content checks
- Requirement targets:
  - all listed core files exist
  - required future code markers appear in the right files
  - obsolete markers are forbidden where deterministic
  - `scripts/verify-block-level-verifiers.py` dogfoods fixture, report, docs, and repo-plan-spec migration state

## Key Decisions

- Coverage: content checks and hand review
- Requirement targets:
  - docs, model, and custom checks enforce `verifier`, no top-level `verifiers`, built-in names, and one argv command shape

## Files To Modify

- Coverage: tree checks for file and directory existence
- Requirement targets:
  - plan files
  - core source files
  - docs
  - existing script verifier
  - behavior fixtures and golden outputs
  - custom migration verifier script

## Out Of Scope

- Coverage: hand review
- Requirement targets:
  - no spec item requires Cargo/npm/Rust/TypeScript ecosystem builtin behavior in this migration
  - no top-level aliases or shell strings are required
