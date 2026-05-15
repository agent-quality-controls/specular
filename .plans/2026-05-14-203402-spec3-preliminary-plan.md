# Goal

Build `spec3`: a Rust CLI and library for deterministic spec-driven development.

`spec3` verifies that implementation matches a reviewed machine-readable spec. It does not replace behavior fixtures. Behavior fixtures detect output drift after implementation. `spec3` checks whether the planned structure and contracts were built.

Core flow:

```text
prose plan -> reviewed spec -> locked spec -> implementation -> deterministic conformance check
```

# Problem

Current manifest-driven development is too easy to weaken.

The failure mode is:

```text
agent writes plan
agent writes manifest
agent writes verifier script
agent edits code
agent edits manifest or verifier when blocked
verifier passes
agent claims done
```

That proves only that the current verifier agrees with current code. It does not prove that the code matches the reviewed plan.

`spec3` must make contract changes explicit. Once a spec is locked, implementation verification must fail if the plan, spec, or verifier files changed.

# Repository

`spec3` lives in its own repository:

```text
/Users/tartakovsky/Projects/agent-quality-controls/spec3
```

Package and binary name:

```text
spec3
```

Remote:

```text
https://github.com/agent-quality-controls/spec3
```

# Agent Starting Context

This plan is the handoff source of truth for a new implementation agent.

The agent should assume no prior conversation context except:

- `spec3` is a spec-driven development tool.
- `fixture3` is a separate behavior-fixture tool.
- `g3rs` is the Rust guardrail validator used in these repositories.
- Implementation must strengthen the contract boundary between plan, spec, verifier, and code.

Do not rebuild `fixture3` behavior in `spec3`.

Do not implement language-specific Rust or TypeScript export parsing in the first pass.

Do not add command execution as a requirement category in V1.

# Plan Review Result

The V1 plan is sound because it keeps `spec3` limited to contract verification:

- lock reviewed plan, spec, and verifier inputs
- refuse drift before checking implementation
- implement universal checks first
- keep behavior output comparison in `fixture3`
- keep architecture and style validation in `g3rs` and `g3ts`

The implementation agent must preserve these boundaries:

- `spec3` checks whether planned structural contracts exist.
- `fixture3` checks whether command output changed.
- `g3rs` and `g3ts` check repository guardrails.

Corrections from review:

- repository path is now the moved `agent-quality-controls/spec3` path
- `schemas` must not run broad JSON Schema validation in V1
- `dependencies`, `exports`, `enumerations`, `schemas`, `commands`, and `cli` must not become active implementation checks before their verifier model is designed

Research memo:

- See `.plans/2026-05-15-132431-spec3-validation-model-research.md` before implementing V1.

# Blocking Open Questions

Do not start implementation beyond repository scaffolding until these questions are answered or explicitly narrowed.

## Spec-Driven Development Model

The current plan defines file checks, but not a complete spec validation model.

Decisions:

- Requirements do not need source references back to prose-plan lines.
- Requirements do not need a separate `verificationMethod` field in V1.
- Every non-empty supported requirement category must have a checker.
- Non-empty unsupported requirement categories fail. They are not silently ignored.
- Every checker result must include the requirement ID it checked.
- Requirements that produce no checker result are fatal.
- Checker results for unknown requirement IDs are fatal.
- The lock records which checker owns each requirement ID.
- Requirement IDs must be stable across edits. Replacement or deletion policy is still open.

Plain examples:

- Orphan requirement: the spec contains `ROOT_README`, but `verify` returns no pass/fail result for `ROOT_README`.
- Orphan checker output: a checker returns a result for `UNKNOWN_REQUIREMENT`, but the locked spec has no such requirement.
- Requirement-to-checker map: the lock records that `NO_RUST_TESTS` is checked by `text` and `ROOT_README` is checked by `tree`.

References to evaluate:

- ISO/IEC/IEEE 29148 requirements engineering and traceability model.
- Requirements traceability matrix practice: source -> requirement -> design or implementation artifact -> verification evidence.
- NASA SWE-059 bidirectional traceability practice.

Reason:

- `spec3` is meant to prevent plan/code drift. Without traceability, it only becomes a file linter with hashes.

## Contract Semantics

The current plan does not define whether a spec entry is a declarative requirement, a policy, a postcondition, or a fixture reference.

Open questions:

- Should each requirement have explicit preconditions, postconditions, and invariants?
- Should `tree` and `text` entries be treated as postconditions over repository state?
- Should lock drift checks be treated as preconditions for running implementation verification?
- Should global properties such as "no Rust tests exist" be modeled as invariants?
- Should `spec3` expose these terms in the JSON model, or keep them internal?

References to evaluate:

- Design by Contract: preconditions, postconditions, invariants.
- Policy-as-code systems such as Open Policy Agent.
- CUE's constraint model for data validation.

Reason:

- If the contract model is informal, each category will invent its own semantics and the tool will become inconsistent.

## Formal Model Boundary

The current plan does not say which parts need formal modeling before code.

Open questions:

- Does the lock/status/verify state machine need a small Alloy or TLA+ model before implementation?
- What are the allowed states for source spec, lock, plan, verifier files, worktree cleanliness, and implementation conformance?
- Which transitions are allowed: lint, normalize, lock, status, verify?
- What must be impossible: verifying against drifted inputs, accepting orphan verifier output, treating unlocked specs as trusted?
- Can this be represented as a compact finite-state model instead of prose?

References to evaluate:

- TLA+ for checking high-level system state transitions.
- Alloy for lightweight relational modeling and invariant checking.
- AWS formal methods practice: prose design first, then precise model for critical design pieces.

Reason:

- The highest-risk behavior is not parsing JSON. The highest-risk behavior is trusting the wrong contract at the wrong time.

## Policy Engine Versus Built-In Verifiers

The current plan assumes `spec3` owns built-in category logic.

Open questions:

- Should `spec3` remain a small fixed verifier with typed categories?
- Should it embed or call a policy engine for category logic?
- Should OPA/Rego be considered for arbitrary invariants over extracted repository facts?
- Should CUE be considered for spec shape and constraint validation?
- Should the first implementation avoid both and stay fixed until the data model stabilizes?

References to evaluate:

- Open Policy Agent: policy decision separated from policy enforcement.
- CUE: data validation and constraints as a first-class language.
- JSON Schema: standard validation vocabulary for JSON shape.

Reason:

- A hand-rolled rule system will degrade if `spec3` grows into arbitrary policy checks without a policy model.

## Canonicalization And Locking

The current plan says "canonical JSON" but does not define the algorithm.

Open questions:

- Should canonicalization follow RFC 8785 JSON Canonicalization Scheme?
- Should the spec format forbid JSON numbers entirely except integer-like values where needed?
- Should duplicate object keys be rejected before canonicalization?
- Should strings be accepted only when they satisfy I-JSON constraints?
- Should `sourceHash` be recorded only for diagnostics, separate from the canonical contract hash?

References to evaluate:

- RFC 8785 JSON Canonicalization Scheme.
- I-JSON constraints referenced by RFC 8785.
- Existing Rust JCS crates, only if they pass dependency health review.

Reason:

- Lock hashes are security-critical for this tool's purpose. A loose serializer is not good enough.

## Repository Fact Model

The current plan names `tree` and `text`, but not the fact model they consume.

Open questions:

- Should `spec3` first extract a repository fact graph, then run requirements against that graph?
- What are the first V1 facts: file exists, directory exists, file type, path, file content, text match, Git tracked state?
- Should every verifier consume facts and produce evidence, rather than directly printing failures?
- Should evidence include requirement ID, path, byte range or line range, observed value, expected value, and verifier ID?
- Should external verifiers return the same fact/evidence model later?

References to evaluate:

- Policy engines that evaluate structured input data.
- Requirements traceability evidence models.

Reason:

- Without a common fact/evidence model, built-in and external verifiers cannot converge cleanly.

## Path, Glob, And Walk Semantics

The current plan says path normalization and forbidden globs, but does not define semantics.

Open questions:

- Are all spec paths required to be repo-root-relative UTF-8 paths using `/`?
- Are absolute paths, `..`, empty components, and non-UTF-8 paths rejected?
- Are symlinks followed, rejected, or treated as their own file type?
- Does walking respect `.gitignore`, or does it scan all files except explicit tool exclusions?
- Are hidden files included?
- Are generated directories such as `target` excluded by default or only by spec?
- Which glob grammar is normative?

References to evaluate:

- `camino` for UTF-8 paths.
- `globset` for ripgrep-style glob matching.
- `ignore` for `.gitignore`-aware walking.

Reason:

- Most verifier bugs in `tree` and `text` will be path edge cases.

## Git Drift Semantics

The current plan says dirty worktree checks, but not the exact Git model.

Open questions:

- Should dirty checks use `git status --porcelain=v1 -z` or `--porcelain=v2 -z`?
- Do staged changes to locked inputs fail `verify`?
- Do unstaged changes to locked inputs fail `verify`?
- Do untracked locked inputs fail `verify`?
- How are renamed locked files handled?
- Should non-Git directories be unsupported in V1?

References to evaluate:

- Git `status --porcelain` documentation. Porcelain v1 is documented as stable and repository-root-relative.
- `gix-status`, only if avoiding Git subprocesses is worth the dependency surface.

Reason:

- The tool's core promise fails if it misreads repository drift.

## Fixture Strategy For A No-Test Rust Repo

The current plan forbids Rust tests but does not define first fixture coverage.

Open questions:

- What is the minimal fixture3 suite before coding starts?
- Should fixtures cover every CLI command from the first implementation slice?
- Should there be fixtures for invalid JSONC, duplicate keys, canonicalization stability, plan drift, spec drift, verifier drift, tree pass/fail, and text pass/fail?
- Should fixtures be written before or alongside the first CLI behavior?
- How does `fixture3` handle expected failure exit codes and stderr/stdout separation?

References to evaluate:

- Current `fixture3` capabilities and manifest format.

Reason:

- Without tests and without fixture coverage, implementation changes have no mechanical regression signal.

# Dependency Health Gate

Any dependency proposed for implementation must pass this gate before use:

- GitHub repository has at least 100 stars.
- Latest repository commit is no older than one year.
- License is acceptable for this repository.
- The crate or subdirectory being used has recent activity when the repository is a monorepo.
- Transitive dependency tree is reviewed after `Cargo.toml` exists.

Current checks on 2026-05-15:

- `jsonc-parser`: `dprint/jsonc-parser`, 58 stars, latest commit 2026-05-10. Fails star threshold.
- `serde_json_canonicalizer`: `evik42/serde-json-canonicalizer`, 19 stars, latest commit 2026-02-20. Fails star threshold.
- `canon-json`: `bootc-dev/canon-json-rs`, 2 stars, latest commit 2026-05-11. Fails star threshold.
- `camino`: `camino-rs/camino`, 557 stars, latest commit 2026-03-31. Passes initial gate.
- `globset`: `BurntSushi/ripgrep`, 63787 stars, latest repository commit 2026-02-27, latest `crates/globset` path commit 2025-10-22. Passes initial gate.
- `ignore`: `BurntSushi/ripgrep`, 63787 stars, latest repository commit 2026-02-27, latest `crates/ignore` path commit 2026-02-13. Passes initial gate.
- `gix-status`: `GitoxideLabs/gitoxide`, 11421 stars, latest repository commit 2026-05-12, latest `gix-status` path commit 2026-05-12. Passes initial gate, but should not be used in V1 unless Git subprocesses are insufficient.

Candidate systems for approach research:

- `cue-lang/cue`: 6091 stars, latest commit 2026-05-15.
- `open-policy-agent/opa`: 11725 stars, latest commit 2026-05-13.
- `AlloyTools/org.alloytools.alloy`: 836 stars, latest commit 2026-03-18.
- `tlaplus/tlaplus`: 2907 stars, latest commit 2026-05-12.

# Research Task List

These tasks must be completed before V1 implementation starts.

## R-001 Requirements Traceability

Goal:

- Decide the minimum requirement metadata that `spec3` must enforce.

Inputs:

- ISO/IEC/IEEE 29148 summary material.
- NASA SWE-059 bidirectional traceability guidance.
- Existing plan in this file.

Output:

- A proposed spec shape for requirement source, verification method, and evidence links.
- A decision on whether orphan requirements and orphan verifier outputs are fatal.

Done when:

- The plan states exact required metadata for every requirement.

## R-002 Contract Model

Goal:

- Decide whether `spec3` uses precondition, postcondition, and invariant terms.

Inputs:

- Design by Contract references.
- Current command lifecycle: lint, normalize, lock, status, verify.

Output:

- A precise model for lock drift checks, implementation checks, and global invariants.

Done when:

- The plan says which checks run before verification and which checks are implementation conformance checks.

## R-003 Lock State Machine

Goal:

- Model the lock/status/verify lifecycle.

Inputs:

- Current command list.
- Drift scenarios for plan, spec, verifier files, and implementation files.
- Alloy or TLA+ documentation.

Output:

- A small model or explicit state table for trusted and untrusted states.

Done when:

- The plan says which states are allowed, which transitions are allowed, and which states must fail.

## R-004 Policy Engine Decision

Goal:

- Decide whether V1 stays fixed-category or borrows a policy engine model.

Inputs:

- OPA/Rego.
- CUE.
- JSON Schema.
- Current V1 scope.

Output:

- A decision to use, integrate, or reject these systems for V1.
- If rejected for V1, define what would cause reconsideration.

Done when:

- The plan names the policy model for V1 and explains why it will not become ad hoc rules.

## R-005 Canonicalization Decision

Goal:

- Decide the canonical JSON algorithm.

Inputs:

- RFC 8785.
- Rust JCS crate candidates.
- JSONC parser behavior around duplicate keys and invalid input.

Output:

- A normative canonicalization algorithm.
- A dependency decision or a decision to constrain the JSON subset enough to avoid unsafe hand-rolled behavior.

Done when:

- The plan gives exact behavior for duplicate keys, numbers, strings, object key ordering, and source comments.

## R-006 Repository Fact And Evidence Model

Goal:

- Define the common data model consumed and produced by verifiers.

Inputs:

- `tree` and `text` requirements.
- Future external verifier boundary.

Output:

- A V1 fact model.
- A V1 evidence model.

Done when:

- Built-in and future external verifiers can be described using the same requirement ID and evidence structure.

## R-007 Path, Glob, And Walk Semantics

Goal:

- Define file discovery and matching behavior.

Inputs:

- `camino`.
- `globset`.
- `ignore`.
- Git tracked/untracked behavior.

Output:

- Exact path normalization rules.
- Exact glob grammar.
- Exact walk inclusion and exclusion rules.
- Symlink behavior.

Done when:

- `tree` and `text` can be implemented without inventing path semantics.

## R-008 Git Drift Semantics

Goal:

- Define how `spec3` detects drift in plan, spec, and verifier files.

Inputs:

- Git porcelain documentation.
- `gix-status` only as a possible alternative.

Output:

- A decision between Git subprocess porcelain and Gitoxide.
- Exact handling for staged, unstaged, untracked, renamed, deleted, and conflicted files.

Done when:

- `lock`, `status`, and `verify` have exact dirty-state behavior.

## R-009 Fixture3 Bootstrap

Goal:

- Define the first behavior fixture suite for `spec3`.

Inputs:

- `fixture3` repository and behavior format.
- The no-Rust-tests policy.

Output:

- Required fixture scenarios.
- Expected stdout, stderr, and exit code handling.

Done when:

- The plan says what fixtures must exist before V1 is considered mechanically verified.

# Source Format

Support two source formats:

- `.spec3.jsonc`
- `.spec3.json`

JSONC is the human authoring format. Plain JSON is accepted for generated specs or users who do not want comments.

JSONC handling:

- Use a real JSONC parser.
- Do not strip comments with regex.
- Comments must not affect the canonical contract hash.
- Requirement explanations that matter must be in explicit fields such as `reason`, not only comments.

Implementation dependency decision:

- Do not use `jsonc-parser` without an explicit exception or replacement decision.
- Local verification on 2026-05-15 found crates.io version `0.32.4`, MIT license, repository `https://github.com/dprint/jsonc-parser`, optional `serde` / `serde_json` features, 58 GitHub stars, and latest commit 2026-05-10.
- It fails the current dependency health gate because it has fewer than 100 GitHub stars.
- R-005 must decide whether to grant an exception, find a healthier JSONC parser, use CUE or another existing tool, or narrow V1 to strict JSON first.

# Canonical Contract

The source spec is parsed into a typed Rust model, then emitted as canonical strict JSON.

Pipeline:

```text
.spec3.jsonc or .spec3.json
-> parse source
-> deserialize into typed spec3 model
-> validate typed model
-> emit canonical strict JSON
-> hash canonical JSON
```

The lock file stores the canonical contract hash, not the raw JSONC source hash.

Open decision:

- Store optional `sourceHash` for comment-only change reporting.
- Do not use `sourceHash` to decide conformance validity.

# Files

Recommended naming:

```text
.plans/my-plan.md
.plans/my-plan.spec3.jsonc
.plans/my-plan.spec3.lock.json
```

The spec source references the prose plan:

```json
{
  "version": 1,
  "plan": ".plans/my-plan.md"
}
```

# Commands

Initial command surface:

```bash
spec3 lint <spec>
spec3 normalize <spec>
spec3 lock <spec>
spec3 verify <spec-or-lock>
spec3 status <spec-or-lock>
```

## `lint`

Validates the source spec without writing a lock or checking implementation.

Checks:

- source parses as JSONC or JSON
- typed model deserializes
- schema version is supported
- requirement IDs are unique
- requirement blocks are known
- paths are normalized
- lock-relevant verifier files are declared

## `normalize`

Prints canonical strict JSON for a source spec.

Use cases:

- inspect what JSONC becomes after parsing
- debug lock diffs
- feed generated tooling

## `lock`

Creates or updates a lock after the spec has been reviewed.

Lock includes:

- spec3 version
- plan path
- plan hash
- source spec path
- canonical spec hash
- verifier file hashes
- created time

The lock does not prove implementation. It freezes the contract.

## `verify`

Checks implementation against the locked contract.

Before running any implementation checks:

- plan hash must match lock
- canonical spec hash must match lock
- verifier file hashes must match lock

Then it runs the relevant built-in and external verifiers.

Exit codes:

- `0`: contract is locked and implementation conforms
- `1`: implementation does not conform
- `2`: spec, lock, parser, drift, or runtime error

Dirty worktree policy:

- `spec3 lock` should fail by default if the Git worktree is dirty.
- `spec3 verify` should fail by default if any locked plan, spec, or verifier file is dirty.
- V1 may allow dirty implementation files during `verify`, because implementation is what is being checked.
- If an override is added later, it must be explicit and visible in output.

Reason:

- A lock should represent reviewed, auditable contract inputs.
- Agents must not lock uncommitted spec or verifier changes and then claim the implementation matched a stable contract.

## `status`

Reports whether the spec can currently be trusted.

Reports:

- missing lock
- plan drift
- spec drift
- verifier drift
- last verification state when available

# Requirement Shape

Use a typed top-level `requirements` object.

Draft:

```jsonc
{
  "version": 1,
  "plan": ".plans/my-plan.md",
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

Do not repeat `kind` inside grouped arrays unless we later support a flat mixed array. The group name already gives the kind.

# Requirement Categories

## `tree`

File tree contracts.

Use cases:

- required files
- required directories
- forbidden files
- forbidden directories
- partial tree contracts where unspecified paths are allowed

Example:

```jsonc
{
  "id": "TREE_CORE",
  "reason": "The CLI crate must keep a stable module boundary.",
  "required": {
    "crates": {
      "spec3": {
        "files": ["Cargo.toml"],
        "dirs": {
          "src": {
            "files": ["main.rs", "args.rs", "spec.rs", "lock.rs"]
          }
        }
      }
    }
  },
  "forbidden": ["tests", "**/*_tests.rs"]
}
```

Verifier:

- built in
- filesystem crawl
- normalized paths
- required subtree check
- forbidden glob check

## `text`

Required or forbidden text contracts.

Use cases:

- forbid `#[test]`
- forbid `cargo test`
- require generated marker text
- forbid an old package name

Initial matching:

- fixed string only
- explicit scoped path globs

Do not use regex in V1.

Example:

```jsonc
{
  "id": "NO_TESTS",
  "reason": "This repository verifies behavior through fixtures, not Rust tests.",
  "scope": ["crates/**", "scripts/**"],
  "forbidden": ["#[test]", "#[cfg(test)]", "cargo test"]
}
```

Verifier:

- built in
- path glob expansion
- fixed-string search

## `dependencies`

Dependency and import contracts.

Use cases:

- required Cargo dependency with path/features
- forbidden Cargo dependency
- forbidden TypeScript import
- allowed module dependency edge
- forbidden module dependency edge

This category is partly language-specific.

V1 approach:

- spec3 owns the JSON shape
- external verifiers extract language-specific facts
- spec3 compares extracted facts when the external output format is stable
- do not implement dependency verification before the external verifier fact format is designed

Example:

```jsonc
{
  "id": "NO_DIRECT_FS",
  "reason": "Filesystem access must go through the adapter boundary.",
  "forbiddenImports": [
    {
      "scope": "crates/core/**",
      "import": "std::fs"
    }
  ]
}
```

## `exports`

Exported type, function, constant, and command-surface contracts.

This replaces the earlier `publicSurface` name.

Use cases:

- required exported type
- required exported function
- forbidden exported type
- closed export inventory

V1 approach:

- no built-in Rust or TypeScript parser yet
- use external verifier commands to produce export facts
- compare facts to spec when the fact format is stable
- do not implement exports verification before the external verifier fact format is designed

Example:

```jsonc
{
  "id": "SPEC_MODEL_EXPORTS",
  "crate": "spec3",
  "required": {
    "types": ["Spec", "RequirementSet", "LockFile"],
    "functions": []
  },
  "forbidden": {
    "types": ["UncheckedSpec"],
    "functions": []
  }
}
```

## `enumerations`

Finite value-set contracts.

Do not call this `closedSets`. That name is precise but not clear enough.

Do not call this only `enums`. Some entries are language enum variants, but other entries are enum-like value sets:

- status strings
- rule IDs
- exit codes
- output kind values
- command names
- config modes

Use `enumerations` because it covers both language enums and data-level enumerated values.

Example:

```jsonc
{
  "id": "VERIFY_EXIT_CODES",
  "name": "spec3 verify exit codes",
  "mode": "closed",
  "values": [0, 1, 2]
}
```

Verifier:

- language-specific extractor for code-owned sets
- config/schema parser for data-owned sets
- exact comparison for `mode = "closed"`
- subset comparison for `mode = "required"`
- do not implement enumeration verification before the external verifier fact format is designed

## `schemas`

Structured data contracts.

Use cases:

- SQL table columns
- SQL indexes
- SQL foreign keys
- JSON schema fields
- config schema fields
- parser model fields
- DTO fields

Example:

```jsonc
{
  "id": "LOCK_SCHEMA",
  "format": "json",
  "file": "schemas/spec3-lock.schema.json",
  "requiredFields": ["version", "planHash", "specHash", "verifierHashes"]
}
```

V1 approach:

- SQL and language model extraction starts external
- do not implement broad schema verification before the external verifier fact format is designed, except for spec3's own source and lock models

## `cli` (Deferred)

Defer `cli` as a requirement category in V1.

CLI surface contracts.

Use cases:

- subcommands
- flags
- mutual exclusions
- required one-of groups
- help fragments
- exit codes
- JSON output fields

Example:

```jsonc
{
  "id": "CLI_VERIFY",
  "binary": "spec3",
  "command": "verify",
  "requiredFlags": [],
  "optionalFlags": ["--json"],
  "exitCodes": [0, 1, 2],
  "helpContains": ["verify", "--json"]
}
```

Verifier:

- built-in command execution may eventually handle simple CLI help checks
- broader behavior checks should use fixture tools, not spec3

Reason for deferral:

- CLI checks require executing a binary.
- Command execution as a requirement is already deferred.
- Keeping `cli` active in V1 would blur the boundary between spec conformance and behavior fixtures.
- If needed, CLI surfaces can be represented later after command and external verifier semantics are stable.

## `fixtures`

Fixture infrastructure contracts.

Use cases:

- fixture suite exists
- fixture manifest references expected suite
- fixture coverage entries exist
- behavior approval files are wired

This must not duplicate fixture behavior comparison.

`spec3` verifies fixture infrastructure exists. `fixture3` verifies fixture output behavior.

# Deferred Categories

## `commands`

Defer `commands` as a requirement category.

Reason:

- command execution is necessary for `spec3 verify`, but making command execution itself a first-class requirement category risks turning `spec3` into a generic task runner.
- For now, commands belong under `verifiers`, not under `requirements`.

Keep this concept:

```jsonc
{
  "verifiers": [
    {
      "id": "static",
      "command": ["scripts/verify-static.sh"],
      "files": ["scripts/verify-static.sh"]
    }
  ]
}
```

Do not add this yet:

```jsonc
{
  "requirements": {
    "commands": []
  }
}
```

## `cli`

Defer `cli` together with `commands`.

Do not include `cli` in the V1 typed `requirements` object.

The eventual `cli` category should be designed after command execution semantics are stable.

# Verifier Model

Specs declare verifiers separately from requirements.

Draft:

```jsonc
{
  "verifiers": [
    {
      "id": "tree",
      "type": "builtin",
      "name": "tree"
    },
    {
      "id": "rust-exports",
      "type": "external",
      "command": ["scripts/verify-rust-exports.sh"],
      "files": ["scripts/verify-rust-exports.sh"]
    }
  ]
}
```

Lock hashes all external verifier files.

Built-in verifiers are tied to the `spec3` binary version recorded in the lock.

External verifier fact format:

- Deferred for V1.
- Do not invent one while implementing `tree` and `text`.
- When designed, it should be stable JSON with requirement IDs, extracted facts, and mismatch details.
- Language-specific features must wait for this format.

# Built-In Verifiers For First Implementation

Build only the universal checks first:

- parse JSONC and JSON
- normalize to canonical JSON
- lock hash checks
- tree required/forbidden checks
- text fixed-string required/forbidden checks
- verifier drift checks

Do not build language-specific dependency or export parsers first.

# G3RS And Test Policy

Initialize the Rust workspace with current G3RS adoption:

```text
guardrail3-rs.toml
```

Use G3RS for static validation.

Test policy for this repository:

- Do not add Rust unit tests.
- Do not add Rust integration tests.
- Do not add doc tests.
- Do not add `#[test]`.
- Do not add `#[cfg(test)]`.
- Do not add a `tests/` directory.
- Do not use `cargo test` as project verification.

Verification model:

- `cargo check`
- `cargo clippy`
- `cargo fmt --check`
- `g3rs validate --path . --rules-only`
- `fixture3` behavior fixtures for CLI behavior once there is behavior to check
- `spec3` self-verification once the tool can lock and verify its own spec

Reason:

- This project is specifically about replacing agent-authored ad hoc tests/scripts with locked specs plus fixture behavior checks.
- The test ban must be explicit so an implementation agent does not fall back to standard Rust test scaffolding.

# Relationship To Other Tools

`spec3`:

- verifies planned implementation contracts
- prevents plan/spec/verifier drift
- owns spec parsing, locking, and universal checks

`fixture3`:

- verifies behavior output against approved fixtures
- catches behavior drift

`g3rs` and `g3ts`:

- enforce ongoing architecture and style guardrails
- can be called by external verifier scripts

# Open Decisions

- Whether to store raw JSONC `sourceHash` in the lock for comment-only drift reporting.
- Exact canonical JSON algorithm and key ordering.
- Standard JSON fact format for external verifiers.
- Whether JSON Schema validation is needed for spec source, or whether typed Rust deserialization plus custom validation is enough.
- Whether `cli` returns after V1 as a requirement category or stays delegated to fixture behavior.

# First Implementation Plan

1. Initialize Rust workspace and G3RS policy.
2. Add typed spec model for metadata, requirements, and verifiers.
3. Add JSON and JSONC source loading.
4. Add canonical JSON emission.
5. Add lock file writing and reading.
6. Add `lint`.
7. Add `normalize`.
8. Add `lock`.
9. Add drift-only `status`.
10. Add `verify` preflight drift checks.
11. Add built-in `tree` verifier.
12. Add built-in `text` verifier.
13. Add fixture coverage for spec3 itself through fixture3.

# V1 Definition Of Done

- `spec3 lint` rejects invalid JSONC/JSON and invalid typed specs.
- `spec3 normalize` emits stable strict JSON.
- `spec3 lock` writes a lock with plan/spec/verifier hashes.
- `spec3 status` reports missing lock and drift states.
- `spec3 verify` refuses to run when plan/spec/verifier files drift.
- `spec3 verify` checks built-in `tree` and `text` requirements.
- `commands` is not a requirement category in V1.
- `cli` is not a requirement category in V1.
- `dependencies`, `exports`, `enumerations`, and broad `schemas` are typed but not verified until the external verifier fact format is designed.
- Rust tests are absent.
- The implementation has fixture coverage through `fixture3`.
