# Goal

Build `spec3`: a Rust CLI and library for deterministic spec-driven development.

`spec3` verifies that implementation matches a machine-readable JSON spec. It does not know about prose plans, Markdown descriptions, tickets, prompts, or how the spec was produced. The library starts at the JSON spec.

Core flow:

```text
spec.json -> spec.lock.json -> repository facts -> evidence
```

## Problem statement (unchanged)

`spec3` must verify repository shape and file content against a locked JSON contract without:

- duplicating walk, ignore, symlink, encoding, or Git drift semantics in the product
- growing its own crawler, config parser, or AST visitor stack for V1 `tree` / `text` checks
- coupling to Guardrail3 **policy** or the legacy `g3rs validate` family/checker pipeline

V1 built-in checkers (`tree`, `text`) must be **thin**: spec routing + evidence over facts produced by shared platform crates.

## Solution approach (Guardrail3 v2 platform)

Guardrail3 v2 and [`aqc-shared`](https://github.com/agent-quality-controls/aqc-shared) resolve the shared-fact boundary this plan originally left open. The same platform crates serve Spec3 checkers and Guardrail3’s I/O broker; products stay separate at policy, lock, and evidence layers.

| Concern in this plan | Solved by | Plan / doc |
|----------------------|-----------|------------|
| Repository walk, paths, globs, ignore/recovery | `aqc-filetree` | [`packages/aqc-filetree/plan.md`](https://github.com/agent-quality-controls/aqc-shared/blob/main/packages/aqc-filetree/plan.md) |
| Read text/bytes for `text` requirements | `aqc-fs-utils` | [`packages/aqc-fs-utils/plan.md`](https://github.com/agent-quality-controls/aqc-shared/blob/main/packages/aqc-fs-utils/plan.md) |
| Lock/verify Git drift (dirty locked paths) | `aqc-git-helpers` | [`packages/aqc-git-helpers/plan.md`](https://github.com/agent-quality-controls/aqc-shared/blob/main/packages/aqc-git-helpers/plan.md) |
| Typed config reconcile (future spec categories) | `aqc-{domain}-parser` / `aqc-{domain}-engine` in `aqc-shared` | [Guardrail3 v2 architecture](https://github.com/agent-quality-controls/guardrail3/blob/development/.plans/g3v2-architecture/2026-05-21-195830-repo-workspace-plugin-generation-model.md) |

**Decided boundaries (from v2):**

- **No** dependency on Guardrail3 policy crates or connectors.
- **No** shared finding/evidence crate across products — Spec3 keeps its own evidence model (below).
- Legacy `g3-workspace-crawl` and `guardrail3/packages/parsers/` are **rehomed** into `aqc-shared`, not wrapped indefinitely; Spec3 depends on `aqc-*` crates, not on guardrail3 package paths.
- Guardrail3 v2 is a **metalinter** (policies → connectors → broker → linters). That is unrelated to Spec3’s verify lifecycle except where both use the same `aqc-*` platform code.

**Implementation order for this plan:** implement Spec3 spec/lock/lint/status/verify shell first; wire `aqc-filetree`, `aqc-fs-utils`, and `aqc-git-helpers` as those crates land in `aqc-shared`; then add thin `tree` / `text` checkers. Do not block on Guardrail3 v2 product code — only on the neutral `aqc-*` crates.

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

- walk repository via `aqc-filetree`; read scoped files via `aqc-fs-utils` where needed
- run built-in checkers (thin layer over those facts)
- validate evidence coverage
- emit evidence

Exit codes:

- `0`: contract is locked and implementation conforms
- `1`: implementation does not conform
- `2`: spec, lock, parser, drift, or runtime error

# V1 Requirement Categories

V1 `tree` and `text` checkers are implemented as **thin Spec3 logic over `aqc-shared` platform crates** (see [Solution approach](#solution-approach-guardrail3-v2-platform)). They do not call Guardrail3 policies, connectors, or legacy family checkers.

### Ownership boundary

| Layer | Owner |
|-------|--------|
| JSON spec, schema, semantic validation, lock, checker map, evidence coverage, CLI | `spec3` |
| Walk → `FileTree`, path/glob/ignore/symlink semantics | `aqc-filetree` |
| File read semantics for text checks | `aqc-fs-utils` |
| Porcelain worktree state for lock/verify drift | `aqc-git-helpers` |
| Config bytes reconcile (not needed for V1 `tree`/`text`) | `aqc-{domain}-engine` (future categories) |
| Linter/policy enforcement | Guardrail3 v2 (separate product; same `aqc-*` where applicable) |

Spec3 must not reimplement walk, read, or Git rules that already live in those plans.

### Not used for V1 `tree` / `text`

The following remain **Guardrail3-only** (legacy or v2 metalinter). Spec3 does not depend on them for V1:

- `g3rs-*-ingestion`, per-rule family checkers, `G3CheckResult` families
- `g3rs-code-ingestion`, `g3rs-test-ingestion`, `syn` source-check packages

If a later spec category needs Cargo/AST facts, prefer **`aqc-{domain}-parser` / engine** crates in `aqc-shared` (same stack Guardrail3’s broker uses), not guardrail3-internal package paths.

## `tree`

Checks required and forbidden repository paths.

Use cases:

- required files
- required directories
- forbidden files
- forbidden directories
- partial tree contracts where unspecified paths are allowed

Verifier behavior (planned):

- load `FileTree` from `aqc-filetree` with options aligned to this spec’s path/glob section (below)
- repo-root-relative paths and `globset` matching on `rel_path` entries
- required path / forbidden glob checks in Spec3 only — no second walk semantics layer

Path, glob, walk, ignore recovery, and symlink semantics: **`aqc-filetree` plan is authoritative.** Spec3 documents chosen option values; it does not redefine behavior.

## `text`

Checks required or forbidden fixed text in scoped files.

Use cases:

- forbid `#[test]`
- forbid `cargo test`
- require generated marker text
- forbid an old package name

Verifier behavior (planned):

- scope files via `aqc-filetree` entries + spec globs
- read each candidate with `aqc-fs-utils::read_text` and fixed `ReadTextOptions` (documented in spec3; defaults from `aqc-fs-utils` plan unless spec overrides)
- fixed substring only in V1; no regex

Encoding, NUL, size cap, CRLF normalization, and symlink read behavior: **`aqc-fs-utils` plan is authoritative.**

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

V1 must define a **Spec3-owned** evidence model before implementing checkers (not an `aqc-shared` crate; see v2 non-goals).

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

**Spec3 spec rules** (enforced at lint time on the JSON contract):

- all spec paths are repo-root-relative UTF-8 paths using `/`
- reject absolute paths, `..`, and empty path components
- glob patterns must compile (`globset`)

**Runtime walk behavior** is not redefined here. Spec3 passes explicit options into `aqc-filetree` (see its plan: `SymlinkPolicy`, `skip_dir_names`, `.gitignore` / recovery, sorted entries). Record the chosen defaults in Spec3 config or constants when implementing checkers.

Likely dependencies:

- `camino` for UTF-8 paths in the spec layer
- `globset` for spec globs and for matching `FileTree` entries
- `aqc-filetree` (uses `ignore` internally per its plan)

# Git Drift Semantics

**Spec3 policy** (what verify must enforce):

- staged, unstaged, and untracked changes to **locked** paths fail `verify`
- `lock` fails when the worktree is dirty (per command section)
- non-Git directories: unsupported in V1 (`aqc-git-helpers` → `NotARepository`)

**Git invocation and porcelain parsing** are owned by **`aqc-git-helpers`** (`--porcelain=v1 -z`, NUL-separated records). Spec3 compares helper output against locked path sets; it does not fork its own `git status` parser.

Renamed/deleted/conflicted locked-file behavior: follow `ChangeStatus` mapping in the `aqc-git-helpers` plan; Spec3 tests assert the combined contract.

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

Initialize the Rust workspace with G3RS policy files as today (`guardrail3-rs.toml`). Static validation uses Guardrail3; **v2** is the metalinter model in [g3v2 architecture](https://github.com/agent-quality-controls/guardrail3/blob/development/.plans/g3v2-architecture/2026-05-21-195830-repo-workspace-plugin-generation-model.md) (not the legacy `g3rs validate` family stack). Until v2 ships in this repo, keep whatever Guardrail3 entrypoint the workspace already uses.

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
- Guardrail3 validate for this workspace (v2 metalinter entrypoint when available; legacy `g3rs validate` until then)
- `fixture3` behavior fixtures for CLI behavior once there is behavior to check
- `spec3` self-verification once the tool can lock and verify its own spec

# Relationship To Other Tools

`spec3`:

- verifies structural implementation contracts from JSON specs
- prevents spec/verifier/checker-map drift
- owns spec parsing, locking, evidence, and built-in `tree`/`text` checkers
- uses `aqc-filetree`, `aqc-fs-utils`, `aqc-git-helpers` from [`aqc-shared`](https://github.com/agent-quality-controls/aqc-shared)

`fixture3`:

- verifies command output against approved fixtures
- catches behavior drift

`g3rs` / `g3ts` (Guardrail3 v2):

- metalinter/scaffolder: policies, connectors, broker, third-party and first-party **linters**
- separate CLI and config from Spec3; may share `aqc-*` parsers/engines and filetree/fs-utils
- does not replace Spec3 lock/verify; users may run both

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
11. Add `verify` preflight drift checks (`aqc-git-helpers` for dirty locked paths).
12. Add Spec3 evidence model (product-specific; not in `aqc-shared`).
13. Depend on `aqc-filetree`, `aqc-fs-utils`, `aqc-git-helpers` from `aqc-shared` (implement or stub per crate `plan.md`).
14. Add built-in `tree` checker: `FileTree` + path/glob rules only.
15. Add built-in `text` checker: scoped `read_text` + fixed substring rules only.
16. Add fixture coverage for `spec3` itself through `fixture3`.

# V1 Definition Of Done

- `spec3 lint` rejects invalid JSON, JSON Schema violations, invalid typed specs, and semantic validation failures.
- `spec3 lock` writes a lock with spec/verifier/checker-map hashes.
- `spec3 status` reports missing lock and drift states.
- `spec3 verify` refuses to run when spec/verifier files or checker routing drift.
- `spec3 verify` checks built-in `tree` and `text` requirements.
- `tree` and `text` checkers use `aqc-filetree` and `aqc-fs-utils` and do not implement their own walk or read stack.
- Git drift for lock/verify uses `aqc-git-helpers`.
- Non-empty unsupported categories fail.
- `commands` is not a requirement category in V1.
- `cli` is not a requirement category in V1.
- Rust tests are absent.
- The implementation has fixture coverage through `fixture3`.
