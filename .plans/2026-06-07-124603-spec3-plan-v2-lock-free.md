# Goal

Build `spec3`: a Rust CLI and library for deterministic spec-driven development.

`spec3` verifies that implementation matches a machine-readable JSON spec. It does not know about prose plans, Markdown descriptions, tickets, prompts, or how the spec was produced. The library starts at the JSON spec.

Core flow:

```text
spec.json -> repository facts -> evidence
```

Supersedes `.plans/2026-05-14-203402-spec3-preliminary-plan.md`. Major changes from that plan:

- The lock file is removed from V1 and redesigned as a thin, additive later layer.
- Custom verifiers move from non-goal to V1 core, with a narrow execution and evidence protocol.
- Evidence carries verifier source (`builtin` vs `custom`) and input-closure identity stamps.
- Deferred categories are specified as per-ecosystem builtins over toolchain machine interfaces.
- Explicit mechanism-not-policy boundary: the library has no roles, identity, approval, or agent concepts.

Superseded in part by `.plans/2026-06-10-154943-per-item-spec-model.md`: the committed V2 format has no requirement IDs, no `schemas` category, no `--json` flag, and one evidence object per item. The sections below named Spec Model, Verifiers, Requirement Coverage, Commands, Public Library Surface, Evidence, Verification Phases, and Deferred Categories are synchronized to that redesign.

# Problem statement

`spec3` must verify repository shape and file content against a JSON contract without:

- duplicating walk, ignore, symlink, encoding, or Git semantics in the product
- growing its own crawler, config parser, or AST visitor stack for V1 `tree` / `content` checks
- coupling to Guardrail3 policy or the legacy `g3rs validate` family/checker pipeline

V1 builtin verifiers (`tree`, `content`) must be thin: spec routing + evidence over facts produced by shared platform crates from [`aqc-shared`](https://github.com/agent-quality-controls/aqc-shared).

# Design principles

## Mechanism, not policy

The library cannot know its caller: human, CI, orchestrator, or agent. It therefore contains no roles, no identity, no approval flows, no agent-awareness. It provides mechanisms that any outer policy can attach to:

- Every command's semantics are visible in its syntax, so external permission systems can discriminate by command pattern.
- All commands in V1 are read-only with respect to the repository. `spec3` writes nothing into the repository it verifies.
- No bypass flags. There is no `--force`, `--skip`, or environment variable that softens a refusal. Integrity refusals are non-negotiable in-library.
- Every decision point emits structured JSON output with stable exit codes, so outer layers consume facts as data, never parse prose.

## Source recorded, never judged

- `builtin` evidence: produced by builtin verifiers shipped and fixture-tested with `spec3`, independent of whoever authored the implementation under verification.
- `custom` evidence: produced by verifier commands declared in the spec — authored by the same parties writing the implementation.

Every evidence item carries its verifier source (`builtin` or `custom`). The library records source as a fact and attaches no judgment to it — what builtin vs custom implies about trustworthiness is the consumer's inference. There is no trust type, grade, score, or rolled-up builtin/custom tally anywhere in the model; the per-item source is the whole contract, and a report must never present the two sources indistinguishably.

## Closed-world core, open-world escape hatch

Typed builtin categories are the product: convergent to extract, mechanically trustworthy, finite in coverage. The custom verifier lane covers everything not yet built, with its source loudly labeled. Coverage gaps fail `lint` explicitly; they are never silently skipped and never improvised.

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

- lock file, status command, drift refusal (deferred layer; see "Deferred: lock layer")
- prose-plan parsing
- Markdown parsing
- JSONC comments
- CUE/Pkl/HCL/Dhall authoring layers
- OPA/Rego policy engine integration
- TLA+ or Alloy runtime integration
- roles, identity, approval, or any caller-awareness
- CLI surface checking as a requirement category
- builtin dependency/import checking
- builtin export/API checking
- builtin enumeration checking
- builtin schema checking
- Rust unit, integration, or doc tests

A custom verifier is not a generic task runner: it runs only commands declared in the spec, its only output channel is the evidence protocol, and its results are labeled `custom`.

# V1 Source Format

V1 source format is strict JSON.

Comments are not supported. Use explicit fields such as `reason` for human explanations.

Rust types are the source of truth for spec shape.

Validation stack:

```text
spec.json
-> parse as strict JSON
-> validate against JSON Schema generated from Rust types with schemars
-> deserialize with Serde
-> run Rust semantic validation
-> produce internal contract
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

Rust semantic validation must enforce uniqueness for IDs, paths, verifier requirement IDs, and any other fields where duplicates would change meaning.

# Spec Model

The spec file is the source contract. The atom is an item: a path, substring, package name, export name, enum value, or custom evidence object. There are no requirement IDs.

Top-level shape:

```json
{
  "version": 1,
  "verifiers": {
    "dependencies": ["scripts/verify-deps.sh"],
    "exports": ["scripts/verify-exports.sh"],
    "enumerations": ["scripts/verify-enums.sh"],
    "custom": ["scripts/verify-custom.sh"]
  },
  "requirements": {
    "tree": {
      "required": ["src/lib.rs"],
      "forbidden": ["tests/**"]
    },
    "content": [],
    "dependencies": [],
    "exports": [],
    "enumerations": [],
    "custom": []
  }
}
```

Typed categories use uniform quantifier keys where they apply:

- `required`: each item is present in every matched place
- `exists`: each item is present in at least one matched place
- `forbidden`: each item is present in no matched place

Category shape:

- `tree`: one object for the repository; `required`, `exists`, `forbidden`; `exists` is rejected by lint.
- `content`: array of blocks with `files`, `required`, `exists`, `forbidden`.
- `dependencies`: array of blocks with `manifests`, `required`, `exists`, `forbidden`; forbidden items may be globs.
- `exports`: array of blocks with `package`, `required`, `exists`, `forbidden`; `exists` is rejected by lint.
- `enumerations`: array of blocks with `name` and exact `values`.
- `custom`: array of opaque objects. spec3 validates only that each entry is an object.

`reason` is optional on every typed block. It may be a string or an array of strings. The extraction workflow fills it, but the schema does not require it.

Granularity is derived by target: `content.files`, `dependencies.manifests`, `exports.package`, and `enumerations.name`. Two blocks in one category with the same target fail `lint` as `DUPLICATE_TARGET`.

Builtin-supported categories in V1:

- `tree`
- `content`

Categories with no builtin in V1:

- `dependencies`
- `exports`
- `enumerations`
- `custom`

Categories may be omitted from `requirements` when unused; an absent category is empty. A non-empty category with no builtin and no verifier override fails `lint` (`CATEGORY_HAS_NO_VERIFIER`).

# Verifiers (per category)

A verifier is per category, not per requirement. The builtin runs for `tree` and `content`; any category can be overridden by a command in the top-level `verifiers` map:

```json
{
  "verifiers": { "dependencies": ["scripts/verify-deps.sh"] }
}
```

Rules:

- A category absent from the map uses its builtin (tree, content) or, for the others, fails `lint` (`CATEGORY_HAS_NO_VERIFIER`).
- A map key that is not one of the six categories fails `lint` (`UNKNOWN_CATEGORY`).
- Listing a builtin category overrides it; the command then owns that category. Override is allowed — the script takes responsibility for the whole category.
- There is no per-requirement ownership and no overlap to resolve. Granularity is target-based (see Spec Model); `DUPLICATE_TARGET` enforces it.

Execution protocol (`verify`):

- For typed categories, `spec3` runs the override once per block as `<command...> <spec.json> <category> <blockIndex>` from the repository root. The script reads that block and emits one JSON line per item in the block: `{"item": "...", "status": "pass" | "fail", "message": "...", "observed": ..., "expected": ..., "path": ...}`.
- For `custom`, `spec3` runs the override once as `<command...> <spec.json> custom`. The script reads the whole `requirements.custom` array. Only `status` is mandatory; all other fields are script-owned and copied into the report.
- A nonzero exit, timeout, missing item, duplicate item, unknown item, invalid JSON line, or custom verifier that emits zero lines is a runtime error (exit class 2), not a requirement failure.
- A missing command file is not a lint concern (the spec is identical whether or not the file exists); it fails at `verify` when the command cannot be stamped or spawned.

All script evidence is labeled `"source": "custom"`; builtin evidence is labeled `"source": "builtin"`. Verifier command files are part of the input closure: their raw-byte hashes are stamped into the report (see Evidence Model).

# Requirement Coverage

Each category routes to exactly one verifier (its builtin or its override), which judges every item in that category.

Fatal verification errors:

- a typed item produces no evidence
- a typed item appears more than once
- a typed verifier reports an item outside its block
- a custom verifier exits 0 but emits no evidence lines

# Commands

```bash
spec3 lint <spec>
spec3 verify <spec>
```

All command output is JSON. No `--json` flag exists. All commands are read-only with respect to the repository.

## `lint`

Validates the source spec without checking implementation.

Checks:

- JSON parses
- JSON Schema validates
- Serde deserializes
- schema version is supported
- unknown fields are rejected
- paths are normalized, globs compile
- DUPLICATE_TARGET: no two blocks in one category share the same target
- DUPLICATE_ITEM: no item appears twice in one block
- CONTRADICTION: an item is not both required and forbidden
- REDUNDANT: an item is not both required and exists
- ITEM_FORMAT: required and exists items are non-empty, trimmed, and non-glob
- EXISTS_SINGLE_PLACE: `exists` is not used on tree or exports
- VACUOUS_SPEC: the spec contains at least one positive assertion; a spec of pure prohibitions passes on an empty repository
- CATEGORY_HAS_NO_VERIFIER: a non-empty category has no builtin and no override in the `verifiers` map
- UNKNOWN_CATEGORY: a `verifiers` map key is not one of the six categories
- DEAD_VERIFIER: a verifier is not declared for an empty category
- CUSTOM_SHAPE: every custom entry is an object
- a missing override command file is NOT a lint error (the spec is identical with or without the file on disk); it fails at `verify`

## `verify`

Checks implementation against the spec.

Steps:

- run `lint` internally; any lint failure is exit class 2
- resolve the input closure: spec file + every declared custom verifier file + `spec3` version
- walk repository via `aqc-filetree`; read scoped files via `aqc-fs-utils` where needed
- run builtin verifiers (thin layer over those facts)
- execute custom verifiers per the protocol
- enforce evidence coverage
- emit the report

Exit codes:

- `0`: implementation conforms
- `1`: implementation does not conform
- `2`: spec, parser, protocol, or runtime error

# Spec Change Review

There is no `diff` command. Spec changes are reviewed by reading the version-control diff of `spec.json`: the typed-row shape rules exist precisely so that diffs read row-per-requirement. A mechanical tighten/loosen classifier was considered and rejected twice over:

- The verdict feeds an adjudication step performed by an agent or human, whose judgment caps the trustworthiness of the outcome; mechanical precision upstream of judgment adds nothing the reader of the raw diff lacks.
- The verdict is unreliable where it would matter: glob-relation and changed-scalar direction are undecidable or approximate, forcing conservative "loosening" answers that fire on routine fixes.

If the lock layer ever returns with command-pattern gating, direction classification returns with it as an internal dependency, scoped to the exact-only cases (set additions/removals), never as a standalone command.

# Public Library Surface

Decided 2026-06-07. `src/main.rs` is a thin CLI caller of the library; everything below is exposed from `src/lib.rs`.

Functions:

```rust
lint(path) -> Result<Spec, LintError>
verify(&Spec, root) -> Result<Report, VerifyError>
```

- `lint` is the only constructor of a valid `Spec`; `verify` takes what only `lint` produces, so the pipeline order is enforced by the type system, not convention.
- `LintError::InvalidSpec(Vec<SpecViolation>)` carries the full violation list as one variant, alongside I/O and parse variants. Symmetric naming with `VerifyError`; an invalid spec stops every caller, so it is an error, not a report.

Types:

- `Spec` + five typed requirement structs (`TreeRequirement`, `ContentRequirement`, `DependencyRequirement`, `ExportRequirement`, `EnumerationRequirement`) plus opaque `custom` JSON objects
- `Category` — the closed set of six
- `Report`, `Evidence`
- `Status { Pass, Fail }` — same name as its wire field `status`
- `VerifierSource { Builtin, Custom }` — serializes to `source`
- `Polarity { Required, Exists, Forbidden }`
- `LintError` (+ `SpecViolation`), `VerifyError`

Naming decisions and their reasons:

- One noun: **verifier**. Builtin and custom are adjectives (a fact recorded in `VerifierSource`), not kinds — the only difference is dispatch (in-process vs subprocess), and dispatch is an implementation detail. The word "checker" is not used.
- No trust/provenance/grade type. `VerifierSource` is the fact; trust is the consumer's inference. Nothing derivable gets a type.
- No exit-code type. The CLI maps `Ok`+conform / `Ok`+nonconform / `Err` to 0/1/2; the code set is a binary behavior, verified in fixture3.
- One concept, one name across representations: the type is `Status`, the wire field is `status` (`"pass"` | `"fail"`). The overload concern that argued for a different type name died with the removal of the `status` command. Script exit codes never express check results: nonzero exit = runtime error, never a failed requirement.

Spec3-owned (not an `aqc-shared` crate).

Every evidence item includes at least:

- category
- source (`"builtin"` or `"custom"`; typed as `VerifierSource`)
- status (`pass` | `fail`; typed as `Status`)
- item and polarity for typed categories
- message
- observed value when useful
- expected value when useful
- path when applicable

The report header stamps the input closure:

- raw-byte hash of the spec file
- raw-byte hash of every custom verifier file
- `spec3` version
- Git state of the closure files as diagnostics (tracked/dirty/untracked, via `aqc-git-helpers`); recorded, never enforced

External tool versions observed by verifiers are recorded in evidence as diagnostics, never enforced. Platform-dependent binary identity must not gate verification.

The stamps are what make outer integrity policy possible without a lock: any outer layer can compare the stamped hashes against a baseline it keeps. The library records; it does not gate.

Open decisions:

- line/byte ranges for content evidence

# Verification Phases

- input validity (JSON parses, schema validates, Serde deserializes, paths valid, globs compile, targets unique, items non-contradictory)
- requirement conformance (builtin checks and verifier script results)
- evidence validity (every typed script result cites a known item; every typed item reported exactly once; no orphans)

Do not expose precondition, postcondition, or invariant terminology in the user-facing spec.

# V1 Requirement Categories

V1 `tree` and `content` builtin verifiers are thin Spec3 logic over `aqc-shared` platform crates. They do not call Guardrail3 policies, connectors, or legacy family checkers (Guardrail3 vocabulary).

## Ownership boundary

| Layer | Owner |
|-------|--------|
| JSON spec, schema, semantic validation, routing, evidence, CLI | `spec3` |
| Walk -> `FileTree`, path/glob/ignore/symlink semantics | `aqc-filetree` |
| File read semantics for text checks | `aqc-fs-utils` |
| Porcelain worktree state for closure diagnostics | `aqc-git-helpers` |
| Linter/policy enforcement | Guardrail3 v2 (separate product; same `aqc-*` where applicable) |

Spec3 must not reimplement walk, read, or Git rules that already live in those crates' plans.

## Backend attachment (decided 2026-06-07, revised same day)

Originally: facade traits with no implementation ("backend not attached", exit class 2) because the `aqc-*` crates did not exist. They landed in `aqc-shared` on 2026-06-07 (`cargo check` clean, interfaces per their `plan.md` files), so V1 attaches them directly:

- builtin `tree` and `content` verifiers are implemented over `aqc-filetree` (`build_file_tree` -> `FileTree`) and `aqc-fs-utils` (`read_text`) in V1
- closure Git diagnostics use `aqc-git-helpers` (`worktree_changes` + `changes_affecting_paths`; `NotARepository` is the recorded diagnostic for non-Git directories)
- `aqc-filetree`, `aqc-fs-utils`, `aqc-git-helpers` are required dependencies
- no placeholder walk or read implementation may be added anywhere in the workspace — unchanged
- spec glob matching uses spec3's own `GlobSet` (literal separator on) over `FileTree.entries`, not `FileTree::glob` (whose default lets `*` cross `/`), so spec glob semantics stay under spec3's own lint contract

## `tree`

Checks required and forbidden repository paths.

Use cases: required files, required directories, forbidden files, forbidden directories, partial tree contracts where unspecified paths are allowed.

Verifier behavior:

- load `FileTree` from `aqc-filetree` with documented option values
- repo-root-relative paths and `globset` matching on `rel_path` entries
- required path / forbidden glob checks in Spec3 only — no second walk semantics layer

Path, glob, walk, ignore recovery, and symlink semantics: `aqc-filetree` plan is authoritative.

## `content`

Checks required or forbidden fixed substrings in scoped files.

Use cases: forbid `#[test]`, require generated marker text, forbid an old package name.

Verifier behavior:

- scope files via `aqc-filetree` entries + spec globs
- read each candidate with `aqc-fs-utils::read_text` and fixed `ReadTextOptions`
- fixed substring only in V1; no regex

Encoding, NUL, size cap, CRLF normalization, and symlink read behavior: `aqc-fs-utils` plan is authoritative.

# Deferred Categories: Per-Ecosystem Builtins

`dependencies`, `exports`, and `enumerations` exist in the typed model from V1 but have no builtin verifiers in V1. The custom lane carries them and opaque checks until builtins land.

When built, each backend is a per-ecosystem builtin inside `spec3` for its category and ecosystem, for example cargo dependencies, Go dependencies, or npm dependencies:

- `dependencies`: consume the ecosystem's stable machine interface — `cargo metadata`, `go list -deps -json`, package-manager graphs. No linter layer in between; a linter between `spec3` and the toolchain interface adds a vocabulary translation and a managed binary while contributing nothing.
- `exports`, `enumerations`: bespoke static-analysis work (source parsing, DDL parsing). Each pair is built when usage shows recurrence, not upfront.

Coverage rule: a non-empty category with no builtin and no verifier entry fails `lint` with `CATEGORY_HAS_NO_VERIFIER`.

Where Guardrail3 needs the identical fact (for example the cargo metadata graph), the parsing lives once in an `aqc-*` fact crate consumed by both products. No shared linter layer, no shared findings schema.

`commands` and `cli` do not exist as categories: `commands` risks turning `spec3` into a generic task runner beyond the constrained verifier protocol; `cli` belongs in behavior fixtures (`fixture3`).

# Deferred: Lock Layer

The lock is removed from V1. Rationale: its irreplicable in-library contribution is mandatory refusal on drifted inputs; everything else it provided reconstructs from the report's input-closure stamps plus an outer baseline comparison. Its demand is hypothetical until an outer process exists that wants to gate it. Findings from the design review that removed it:

- The checker-map hash was redundant: routing is derived from spec + version, both already covered.
- Canonical JSON hashing existed only for the lock; raw-byte stamps need no canonicalization.
- Baseline storage and baseline comparison were conflated; `diff` provides comparison statelessly, storage belongs to the caller.

If usage proves the need, the lock returns as a thin additive layer with no changes to verifiers, categories, the verifier protocol, or the evidence model:

- `spec3 lock <spec>`: serialize the input closure hashes to `spec.lock.json`.
- `spec3 status <lock>`: compare recorded closure to current files.
- `spec3 verify <lock>`: compare, refuse on mismatch, then verify.
- Command surface splits along the trust boundary: a loosening re-lock requires an explicit `--allow-loosening --reason <text>` invocation (direction computed internally, exact cases only; see Spec Change Review), so any external permission system can gate it as a command pattern.
- The lock records parent-lock hash, diff classification, and declared reason (self-describing lineage; a record, not a proof).

V1 must keep this addition cheap: the input closure stays a first-class internal concept, stamps stay in every report, and the exit-code class for integrity errors stays reserved.

# Path, Glob, And Walk Semantics

Spec rules (enforced at lint time on the JSON contract):

- all spec paths are repo-root-relative UTF-8 paths using `/`
- reject absolute paths, `..`, and empty path components
- glob patterns must compile (`globset`)

Runtime walk behavior is not redefined here. Spec3 passes explicit options into `aqc-filetree` (see its plan: `SymlinkPolicy`, `skip_dir_names`, `.gitignore` / recovery, sorted entries). Record the chosen defaults in Spec3 constants when implementing builtin verifiers.

Likely dependencies:

- `camino` for UTF-8 paths in the spec layer
- `globset` for spec globs and for matching `FileTree` entries
- `aqc-filetree` (uses `ignore` internally per its plan)

# Git State Diagnostics

There is no Git enforcement in V1. `verify` records the worktree state of the input-closure files (spec, custom verifier scripts) in the report header as diagnostics: tracked/dirty/untracked per file.

Git invocation and porcelain parsing are owned by `aqc-git-helpers` (`--porcelain=v1 -z`, NUL-separated records). Spec3 consumes helper output; it does not fork its own `git status` parser. Non-Git directories: the diagnostic field reports `NotARepository`; verification proceeds.

# Boundary With Guardrail3 v2

The discriminator for where a rule lives:

- Standing workspace invariant, true every commit -> a Guardrail3 policy (persistent config, linters wired into hooks/CI).
- Plan-scoped contract, done when verified -> a `spec3` requirement (spec exists for the implementation window).
- The same rule may migrate between them; the underlying fact extraction lives once in an `aqc-*` crate either way.

Capability mirror:

- Guardrail3 v2 authors configuration and never executes processes or parses source (its R1/R2 capability boundary).
- `spec3` executes (custom verifiers, toolchain interfaces) and parses anything, and authors nothing persistent in the repository it verifies.

Decided boundaries:

- No dependency on Guardrail3 policy crates, adapters, engines, or connectors. Concretely (decided 2026-06-07): no crate whose name starts with `g3` or `guardrail`, and none of the guardrails file-engine machinery in aqc-shared — `aqc-file-engine-core`, `aqc-cargo-toml-engine`, `aqc-clippy-toml-engine`, or any future `aqc-*-engine` crate. The allowed aqc-shared crates are exactly the three fact crates: `aqc-filetree`, `aqc-fs-utils`, `aqc-git-helpers`.
- No shared finding/evidence crate across products; Spec3 owns its evidence model.
- No shared linter layer and no shared findings schema between the products.
- Spec3 depends on neutral `aqc-*` crates, never on guardrail3 package paths.

# Relationship To Other Tools

`spec3`:

- verifies structural implementation contracts from JSON specs
- owns spec parsing, routing, evidence, builtin `tree`/`content` verifiers, and the custom verifier protocol
- uses `aqc-filetree`, `aqc-fs-utils`, `aqc-git-helpers` from `aqc-shared`

The `spec-driven-development` skill:

- is the prose-to-spec front end, permanently: plan prose -> independent extraction passes -> adjudication -> `spec.json`. The library starts at the JSON spec and never parses prose.
- owns the plan-relative strength check the library structurally cannot do: the coverage map (every plan section -> requirement IDs / fixtures / custom verifier / not-applicable / UNCOVERED, with UNCOVERED adjudicated). `lint` catches mergeable rows and vacuous specs; only the coverage map catches a spec that is too weak for its plan.
- its hand-rolled lint/lock/status/verify script harness is replaced wholesale by the library when it ships.

`fixture3`:

- verifies command output against approved fixtures; catches behavior drift

`g3rs` / `g3ts` (Guardrail3 v2):

- metalinter/scaffolder: policies, adapters, broker, engines, wired linters
- separate CLI and config from Spec3; may share `aqc-*` crates
- does not replace Spec3 verification; users may run both

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
- `garde`: 850 stars, active 2025-11-30, MIT/Apache-2.0 (chosen; guardrail3 already standardizes on it)

Do not use without explicit exception:

- `jsonc-parser`: 58 stars, fails star gate
- `serde_json_canonicalizer`, `canon-json`: fail star gate; also no longer needed (no canonical hashing in V1)

# G3RS And Test Policy

Initialize the Rust workspace with G3RS policy files as today (`guardrail3-rs.toml`). Until v2 ships in this repo, keep whatever Guardrail3 entrypoint the workspace already uses.

Test policy for this repository:

- Do not add Rust unit tests, integration tests, or doc tests
- Do not add `#[test]` or `#[cfg(test)]`
- Do not add a `tests/` directory
- Do not use `cargo test` as project verification

Verification model:

- `cargo check`, `cargo clippy`, `cargo fmt --check`
- Guardrail3 validate for this workspace
- `fixture3` behavior fixtures for CLI behavior

# Implementation Order

1. Initialize Rust workspace and G3RS policy.
2. Add Rust spec types with Serde and Schemars derives, including the `verifiers` array and all six category models (typed from day one; only `tree`/`content` get builtins).
3. Add strict JSON source loading and generated JSON Schema validation with `jsonschema`.
4. Add Rust semantic validation: ID uniqueness, paths, globs, ownership totality, custom verifier rules.
5. Add `lint`.
6. Add the input-closure concept and report stamps (spec hash, verifier file hashes, version, Git diagnostics via `aqc-git-helpers`).
7. Add the Spec3 evidence model with trust-grade labels.
8. Wire `aqc-filetree`, `aqc-fs-utils`, `aqc-git-helpers` as dependencies (see Backend attachment).
9. Add builtin `tree` verifier: `FileTree` + path/glob rules only.
10. Add builtin `content` verifier: scoped `read_text` + fixed substring rules only.
11. Add custom verifier execution: protocol, JSON-lines parsing, coverage enforcement.
12. Add `verify` end to end with the report.
13. Add fixture coverage for `spec3` itself through `fixture3`.

Do not block on Guardrail3 v2 product code — only on the neutral `aqc-*` crates.

# V1 Definition Of Done

- `spec3 lint` rejects invalid JSON, JSON Schema violations, invalid typed specs, semantic validation failures, duplicate targets/items, contradictions, redundant items, vacuous specs, categories with no verifier, dead verifiers, bad custom shapes, and unknown verifier-map keys.
- `spec3 verify` checks builtin `tree` and `content` requirements over `aqc-filetree`/`aqc-fs-utils` with no own walk or read stack.
- `spec3 verify` runs each category's verifier (builtin or script), enforces the evidence protocol and per-item coverage, and fails loudly on protocol violations.
- Every report labels evidence `builtin`/`custom` and stamps the input closure (spec hash, verifier file hashes, `spec3` version, Git diagnostics).
- Exit codes: 0 conform, 1 nonconform, 2 input/protocol/runtime error. No bypass flags exist.
- The library contains no roles, identity, approval, or caller-awareness; all commands are repository-read-only.
- Rust tests are absent.
- The implementation has fixture coverage through `fixture3`.
