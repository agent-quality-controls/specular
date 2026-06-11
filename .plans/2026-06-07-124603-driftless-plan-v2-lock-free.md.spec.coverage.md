# Coverage map

Plan: `2026-06-07-124603-driftless-plan-v2-lock-free.md`
Spec: `2026-06-07-124603-driftless-plan-v2-lock-free.md.spec.json`

Purpose: verify the repository-state facts pinned by the product plan after the per-item redesign. Behavior is covered by `fixture3`; the spec covers files, dependencies, exported API, closed enums, scripts, and key content tripwires.

- Goal: `tree` required source files; `exports` public API; behavior in fixture3.
- Problem statement: dependency items for `aqc-filetree`, `aqc-fs-utils`, and `aqc-git-helpers`; no Guardrail3 coupling through forbidden dependency globs.
- Design principles / Mechanism, not policy: `src/main.rs` forbids flags, fixture3 checks JSON-only command behavior.
- Design principles / Source recorded, never judged: `VerifierSource` enum, report evidence source fields in fixture3.
- Design principles / Closed-world core, open-world escape hatch: `Category` enum and fixture3 lint behavior for missing verifiers.
- Repository: `tree` required files.
- Non-Goals: forbidden test directories, forbidden bypass flag strings, forbidden dependency globs; remaining non-goals are review-only.
- V1 Source Format: dependency items for serde, serde_json, schemars, jsonschema, camino, globset.
- JSON Shape Rules: fixture3 lint suite.
- Spec Model: `src/model.rs` content checks, `Category` enum, HELP format example.
- Verifiers (per category): script tree items and verify fixtures for typed/custom verifier behavior.
- Requirement Coverage: fixture3 verify suite for missing, unknown, duplicate, and silent evidence failures.
- Commands: `src/main.rs` JSON-only content checks and fixture3 command outputs.
- Spec Change Review: not-applicable, decision record.
- Public Library Surface: `exports` item list plus enum checks.
- Evidence Model: `src/evidence.rs` exported types and fixture3 report goldens.
- Verification Phases: fixture3 lint and verify suites.
- V1 Requirement Categories: dependency items for platform crates; fixture3 covers tree/content behavior.
- Deferred Categories: `dependencies`, `exports`, and `enumerations` use script verifiers; custom covers opaque checks.
- Deferred: Lock Layer: not-applicable, decision record.
- Path, Glob, And Walk Semantics: dependency items for camino/globset and fixture3 lint path/glob behavior.
- Git State Diagnostics: dependency item for `aqc-git-helpers`; fixture3 report goldens show diagnostics.
- Boundary With Guardrail3 v2: forbidden dependency globs `guardrail*` and `g3*`.
- Relationship To Other Tools: not-applicable, narrative.
- Dependency Health Gate: dependency required/forbidden items.
- G3RS And Test Policy: forbidden test directories and forbidden Rust content strings.
- Implementation Order: not-applicable, sequencing.
- V1 Definition Of Done: repository-state items in the spec; behavior in fixture3.

UNCOVERED: 0.
