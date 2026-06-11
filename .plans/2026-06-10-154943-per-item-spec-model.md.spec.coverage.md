# Coverage Map: per-item spec model

Extraction note: this spec was not produced by three isolated sub-agent passes because the available agent tool is restricted unless the user explicitly asks for sub-agents. The accepted contract is still checked mechanically by speculus after the implementation.

## Plan: per-item spec model

- Coverage: not-applicable
- Reason: title only.

## Goal

- Coverage: content checks, exports, enumerations, custom verifier, fixtures
- Requirement targets:
  - `src/model.rs`: no IDs, custom category, tree object.
  - `src/evidence.rs`: per-item report types.
  - `src/main.rs`: JSON-only output.
  - `Category`, `Status`, `VerifierSource`, `Polarity` enumerations.
- Behavior details are covered by fixture3 verify and lint suites.

## The format

- Coverage: content checks, HELP example, lint fixtures
- Requirement targets:
  - `HELP.txt`: new copy-paste JSON example and field docs.
  - `src/model.rs`: new category shape.
  - Lint fixtures: schema and shape cases.

## Semantics (typed categories)

- Coverage: content checks, builtin verifier behavior, fixture3
- Requirement targets:
  - `src/verify.rs`: item evidence, zero-match failure behavior.
  - `src/lint.rs`: `EXISTS_SINGLE_PLACE`, `ITEM_FORMAT`.
  - Verify fixtures: pass and fail cases for required, exists, forbidden.

## Verifier model

- Coverage: content checks, script checks, custom verifier, fixture3
- Requirement targets:
  - `src/verify.rs`: per-block script invocation and custom invocation.
  - `scripts/verify-*.sh`: script protocol.
  - Verify fixtures: typed protocol and custom cases.

## Wire protocol (one, for all scripts)

- Coverage: content checks, scripts, fixture3
- Requirement targets:
  - `src/evidence.rs`: `WireEvidence` and report fields.
  - `src/verify.rs`: item bookkeeping, zero-line custom error, timeout.
  - `HELP.txt`: typed and custom script examples.
  - Verify fixtures: missing, unknown, duplicate, nonzero, silent custom.

## Lint rules

- Coverage: content checks, lint fixtures
- Requirement targets:
  - `src/lint.rs`: listed violation codes.
  - Lint fixtures: JSON schema, vacuous, duplicate target, duplicate item, contradiction, redundant, item format, exists single place, dead verifier, custom shape.

## Report (JSON only)

- Coverage: content checks, main, fixture3
- Requirement targets:
  - `src/main.rs`: no flags and JSON-only printing.
  - `src/evidence.rs`: report structure with `conforms`.
  - `HELP.txt`: report example.

## Code changes

- Coverage: tree, content, exports, enumerations
- Requirement targets:
  - `src/model.rs`
  - `src/lint.rs`
  - `src/verify.rs`
  - `src/evidence.rs`
  - `src/main.rs`
  - `HELP.txt`

## Repo artifacts

- Coverage: tree, custom, fixture3
- Requirement targets:
  - `.plans/2026-06-10-154943-per-item-spec-model.md.spec.json`
  - `.plans/2026-06-10-154943-per-item-spec-model.md.spec.coverage.md`
  - `scripts/verify-dependencies.sh`
  - `scripts/verify-exports.sh`
  - `scripts/verify-enumerations.sh`
  - `scripts/verify-custom.sh`
  - `behavior/fixtures/**`
  - `behavior/golden/**`
  - Skill and old plan sync are checked by hand because they live partly outside the binary's contract surface.

## Order of work

- Coverage: not-applicable
- Reason: process order, not a repository invariant.

## Out of scope

- Coverage: custom checks and hand review
- Requirement targets:
  - `Cargo.toml`: no added dependency for configurable timeout or new builtins.
  - `HELP.txt`: fixed 60 second timeout.
