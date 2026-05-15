# Goal

Build `spec3`: a Rust CLI and library for deterministic spec-driven development.

`spec3` verifies that implementation matches a machine-readable JSON spec. It does not know about prose plans, Markdown descriptions, tickets, prompts, or how the spec was produced. The library starts at the JSON spec.

Core flow:

```text
spec.json -> spec.lock.json -> repository facts -> evidence
```

# Repository

Local path:

```text
/Users/tartakovsky/Projects/agent-quality-controls/spec3
```

Remote:

```text
https://github.com/agent-quality-controls/spec3
```

Package and binary name:

```text
spec3
```

# Non-Goals

Do not build these in V1:

- prose-plan parsing
- Markdown parsing
- JSONC comments
- CUE/Pkl/HCL/Dhall authoring layers
- OPA/Rego policy engine integration
- TLA+ or Alloy runtime integration
- generic command runner
- CLI surface checking as a requirement category
- dependency/import checking
- export/API checking
- enumeration checking
- broad schema checking
- external verifier fact protocol
- Rust unit, integration, or doc tests

# V1 Source Format

V1 source format is strict JSON.

Comments are not supported in V1. Use explicit fields such as `reason` for human explanations.

Rust types are the source of truth for spec shape.

Validation stack:

```text
spec.json
-> parse as strict JSON
-> validate against JSON Schema generated from Rust types with schemars
-> deserialize with Serde
-> run Rust semantic validation
-> produce canonical internal contract
```

Dependencies already checked as viable:

- `schemars`: generate JSON Schema from Rust types
- `jsonschema`: validate JSON source against generated schema
- `serde` and `serde_json`: deserialize into Rust types
- `garde`: validate deserialized Rust structs and keep parity with guardrail3 input-boundary validation

Semantic validation remains Rust-owned because JSON Schema does not check repository state or all cross-field rules.

Use `garde` for struct-level validation. Do not add `validator` unless a concrete blocker appears in `garde`.

# JSON Shape Rules

Avoid dynamic object maps where duplicate keys can hide data.

Prefer arrays with explicit identifiers or paths where uniqueness matters.

Bad shape:

```json
{
  "dirs": {
    "src": { "files": ["main.rs"] }
  }
}
```

Preferred shape:

```json
{
  "dirs": [
    { "path": "src", "files": ["main.rs"] }
  ]
}
```

Rust semantic validation must enforce uniqueness for IDs, paths, checker mappings, and any other fields where duplicates would change meaning.

# Spec Model

The spec file is the source contract.

Draft top-level shape:

```json
{
  "version": 1,
  "requirements": {
    "tree": [],
    "text": [],
    "dependencies": [],
    "exports": [],
    "enumerations": [],
    "schemas": [],
    "fixtures": []
  },
  "verifiers": []
}
```

Every requirement entry has:

- `id`
- optional `reason`
- category-specific fields

Requirement IDs must be unique across all categories.

Supported non-empty categories in V1:

- `tree`
- `text`

Unsupported categories in V1:

- `dependencies`
- `exports`
- `enumerations`
- `schemas`
- `fixtures`

Non-empty unsupported categories fail `lint`. They must not be silently ignored.

# Requirement Coverage

Every active requirement must have a checker.

Every checker result must include the requirement ID it checked.

Fatal verification errors:

- requirement has no checker
- checker returns no result for a requirement
- checker returns a result for an unknown requirement ID
- checker routing changes after lock

The lock records which checker owns each requirement ID.

Example checker map:

```json
[
  { "requirementId": "NO_RUST_TESTS", "category": "text", "checker": "builtin:text" },
  { "requirementId": "ROOT_README", "category": "tree", "checker": "builtin:tree" }
]
```

# Verification Phases

Use these implementation phases:

- input validity
- lock validity
- requirement conformance
- evidence validity

Input validity examples:

- JSON parses
- JSON Schema validates
- Serde deserializes
- requirement IDs are unique
- paths are valid
- glob patterns compile

Lock validity examples:

- canonical spec hash matches lock
- verifier file hashes match lock
- checker map hash matches lock
- locked spec or verifier files are not dirty

Requirement conformance examples:

- required file exists
- forbidden path is absent
- forbidden text is absent

Evidence validity examples:

- every evidence item references a known requirement ID
- every active requirement has evidence
- no checker reports orphan evidence

Do not expose precondition, postcondition, or invariant terminology in the user-facing spec.

# Lock File

The lock file is a generated receipt. It is not a second spec.

The lock freezes:

- canonical spec hash
- checker map hash
- optional expanded checker map for diagnostics
- verifier file hashes
- `spec3` version
- created time metadata

Draft shape:

```json
{
  "version": 1,
  "specPath": ".spec3/spec.json",
  "hashes": {
    "canonicalSpec": "...",
    "checkerMap": "..."
  },
  "checkers": [
    {
      "requirementId": "NO_RUST_TESTS",
      "category": "text",
      "checker": "builtin:text"
    }
  ],
  "metadata": {
    "spec3Version": "0.1.0",
    "createdAt": "..."
  }
}
```

Open lock decisions:

- exact canonical JSON algorithm and key ordering
- whether to store raw source-file hash for diagnostics only
- whether to include Git commit metadata

# Commands

Initial command surface:

```bash
spec3 lint <spec>
spec3 lock <spec>
spec3 status <spec-or-lock>
spec3 verify <spec-or-lock>
```

## `lint`

Validates the source spec without writing a lock or checking implementation.

Checks:

- JSON parses
- JSON Schema validates
- Serde deserializes
- schema version is supported
- unknown fields are rejected
- requirement IDs are unique
- non-empty unsupported categories fail
- supported category fields are valid
- paths are normalized
- globs compile
- every active requirement has a checker

## `lock`

Creates or updates a lock for the current JSON spec.

Before writing:

- run `lint`
- fail if the Git worktree is dirty
- build checker map
- canonicalize internally and compute canonical spec hash
- compute checker map hash
- hash declared verifier files

The lock does not prove implementation. It freezes the contract and checker routing.

## `status`

Reports whether the lock can currently be trusted.

Reports:

- missing lock
- spec drift
- verifier drift
- checker map drift
- dirty locked inputs

## `verify`

Checks implementation against the locked contract.

Before repository checks:

- run `status`
- fail if spec hash differs
- fail if verifier file hash differs
- fail if checker map hash differs
- fail if locked spec or verifier files are dirty

Then:

- extract or load repository facts through shared AQC fact crates
- run built-in checkers
- validate evidence coverage
- emit evidence

Exit codes:

- `0`: contract is locked and implementation conforms
- `1`: implementation does not conform
- `2`: spec, lock, parser, drift, or runtime error

# V1 Requirement Categories

Do not implement V1 checkers until the shared fact boundary with guardrail3 is settled.

Known reusable guardrail3 code:

- `g3-workspace-crawl`: repository crawl, `.gitignore` handling, ignored-file recovery, path model, readability, sorted entries.
- `g3rs-code-ingestion`: Rust source-file selection, owned config selection, typed config parsing, code-family source input assembly.
- `g3rs-test-ingestion`: test-family root discovery, Cargo manifest parsing, sidecar/harness classification, Rust test-source analysis.
- guardrail3 parser crates: typed parsers for Cargo, clippy, deny, rustfmt, rust-toolchain, nextest, mutants, package JSON, tsconfig, and related config files.
- guardrail3 source-check packages: existing `syn` visitors and AST-derived facts for lint attributes, `cfg_attr`, `garde(skip)`, direct filesystem use, test functions, ignore reasons, proof-bearing assertions, public surface, and related source facts.

Ownership boundary:

- `spec3` owns JSON spec shape, JSON Schema generation, semantic spec validation, lock files, checker routing, evidence coverage, and verification lifecycle.
- shared AQC fact crates own repository walking, path semantics, ignore semantics, file readability, config parsing, and language-specific source facts.
- guardrail3 owns guardrail policy checks over those facts.
- `spec3` built-in V1 checkers must be thin checks over shared facts. They must not grow their own crawler, parser, AST visitor stack, or config parser stack.

Open extraction decision:

- depend directly on existing guardrail3 shared crates when their names and APIs are neutral enough
- or extract them into AQC-neutral crates before spec3 checker implementation

Implementation blocker:

- no `tree` or `text` checker implementation until this decision is recorded and the chosen shared crate boundary is wired into the plan

## `tree`

Checks required and forbidden repository paths.

Use cases:

- required files
- required directories
- forbidden files
- forbidden directories
- partial tree contracts where unspecified paths are allowed

Verifier behavior:

- built in after shared-fact boundary is settled
- repository facts come from `g3-workspace-crawl` or its extracted neutral successor
- normalized paths
- required path checks
- forbidden glob checks

Exact path, glob, walk, and symlink semantics must follow the chosen shared crawl crate. Do not define a second semantics layer in `spec3`.

## `text`

Checks required or forbidden fixed text in scoped files.

Use cases:

- forbid `#[test]`
- forbid `cargo test`
- require generated marker text
- forbid an old package name

Verifier behavior:

- built in after shared-fact boundary is settled
- fixed string only in V1
- explicit scoped path globs over shared crawl entries
- no regex in V1

Exact binary-file, encoding, line-ending, and symlink behavior must follow the chosen shared crawl/read layer. Do not define a second file-reading semantics layer in `spec3`.

# Deferred Categories

These categories may exist in the typed spec model, but non-empty values fail in V1:

- `dependencies`
- `exports`
- `enumerations`
- `schemas`
- `fixtures`

These categories do not exist in V1:

- `commands`
- `cli`

Reason:

- `commands` risks turning `spec3` into a generic task runner.
- `cli` requires command execution and belongs either in a later explicit model or in behavior fixtures.

# Repository Fact And Evidence Model

V1 must define a shared evidence model before implementing checkers.

Every evidence item includes at least:

- requirement ID
- checker ID
- status
- message
- observed value when useful
- expected value when useful
- path when applicable

Open decisions:

- line/byte ranges for text evidence
- JSON output shape
- whether pass evidence is emitted by default or only in JSON mode
- whether all checker evidence is stored in the lock or only emitted during verify

# Path, Glob, And Walk Semantics

Open decisions before checker implementation:

- all spec paths must be repo-root-relative UTF-8 paths using `/`
- absolute paths should be rejected
- `..` should be rejected
- empty path components should be rejected
- decide symlink behavior
- decide whether walking respects `.gitignore`
- decide whether hidden files are included
- decide whether generated directories such as `target` are excluded by default or only by spec
- choose glob grammar

Likely dependencies:

- `camino` for UTF-8 paths
- `globset` for glob matching
- `ignore` for `.gitignore`-aware walking if that behavior is chosen

# Git Drift Semantics

Open decisions before `lock`, `status`, and `verify` implementation:

- use `git status --porcelain=v1 -z` or `--porcelain=v2 -z`
- staged changes to locked inputs should fail `verify`
- unstaged changes to locked inputs should fail `verify`
- untracked locked inputs should fail `verify`
- decide renamed/deleted/conflicted locked-file behavior
- decide whether non-Git directories are unsupported in V1

Prefer Git porcelain over Gitoxide in V1 unless subprocess use proves insufficient.

# Dependency Health Gate

Any dependency proposed for implementation must pass this gate before use:

- GitHub repository has at least 100 stars
- latest repository commit is no older than one year
- license is acceptable
- crate/subdirectory has recent activity when repository is a monorepo
- transitive dependency tree is reviewed after `Cargo.toml` exists

Dependencies already checked as viable:

- `jsonschema`: 777 stars, active 2026-05-13, MIT, Rust 1.83
- `schemars`: 1350 stars, active 2026-02-03, MIT, Rust 1.74
- `camino`: 557 stars, active 2026-03-31, MIT/Apache-2.0
- `globset`: under `BurntSushi/ripgrep`, 63787 stars, path active 2025-10-22, MIT/Unlicense
- `ignore`: under `BurntSushi/ripgrep`, 63787 stars, path active 2026-02-13, MIT/Unlicense

Potential semantic-validation dependencies:

- `validator`: 2470 stars, active 2026-04-22, MIT

Chosen semantic-validation dependency:

- `garde`: 850 stars, active 2025-11-30, MIT/Apache-2.0

Reason:

- guardrail3 already standardizes on `garde`
- `garde` supports nested validation and validation context
- using both `garde` and `validator` would split validation conventions across AQC Rust tools without a current benefit

Do not use without explicit exception:

- `jsonc-parser`: 58 stars, fails star gate
- `serde_json_canonicalizer`: 19 stars, fails star gate
- `canon-json`: 2 stars, fails star gate

# G3RS And Test Policy

Initialize the Rust workspace with current G3RS adoption:

```text
guardrail3-rs.toml
```

Use G3RS for static validation.

Test policy for this repository:

- Do not add Rust unit tests
- Do not add Rust integration tests
- Do not add doc tests
- Do not add `#[test]`
- Do not add `#[cfg(test)]`
- Do not add a `tests/` directory
- Do not use `cargo test` as project verification

Verification model:

- `cargo check`
- `cargo clippy`
- `cargo fmt --check`
- `g3rs validate workspace --path <path>`
- `fixture3` behavior fixtures for CLI behavior once there is behavior to check
- `spec3` self-verification once the tool can lock and verify its own spec

# Relationship To Other Tools

`spec3`:

- verifies structural implementation contracts from JSON specs
- prevents spec/verifier/checker-map drift
- owns spec parsing, locking, and universal checks

`fixture3`:

- verifies command output against approved fixtures
- catches behavior drift

`g3rs` and `g3ts`:

- enforce architecture and style guardrails
- can be called separately by users or future external verifier scripts

# First Implementation Plan

1. Initialize Rust workspace and G3RS policy.
2. Add Rust spec types with Serde and Schemars derives.
3. Add strict JSON source loading.
4. Add generated JSON Schema validation with `jsonschema`.
5. Add Rust semantic validation for IDs, unsupported categories, paths, globs, and checker coverage.
6. Add internal canonical contract generation for hashing.
7. Add lock file writing and reading.
8. Add `lint`.
9. Add `lock`.
10. Add `status`.
11. Add `verify` preflight drift checks.
12. Add shared evidence model.
13. Resolve guardrail3/shared AQC fact-crate boundary.
14. Wire shared repository crawl facts.
15. Add built-in `tree` checker as a thin checker over shared crawl facts.
16. Add built-in `text` checker as a thin checker over shared crawl/read facts.
17. Add fixture coverage for `spec3` itself through `fixture3`.

# V1 Definition Of Done

- `spec3 lint` rejects invalid JSON, JSON Schema violations, invalid typed specs, and semantic validation failures.
- `spec3 lock` writes a lock with spec/verifier/checker-map hashes.
- `spec3 status` reports missing lock and drift states.
- `spec3 verify` refuses to run when spec/verifier files or checker routing drift.
- `spec3 verify` checks built-in `tree` and `text` requirements.
- `tree` and `text` checkers reuse shared AQC repository facts and do not implement their own crawler or parser stack.
- Non-empty unsupported categories fail.
- `commands` is not a requirement category in V1.
- `cli` is not a requirement category in V1.
- Rust tests are absent.
- The implementation has fixture coverage through `fixture3`.
