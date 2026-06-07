# Coverage map

Plan: `2026-06-07-124603-spec3-plan-v2-lock-free.md`
Spec: `2026-06-07-124603-spec3-plan-v2-lock-free.md.spec.json` (9 requirements)

Purpose of this spec: confirm the building agent delivered everything the plan pins as repository state — files, dependencies, public API, closed sets — and nothing forbidden. `fixture3` = binary behavior, outside this spec's reach (no fixtures exist yet; those entries are open work).

- Goal — WORKSPACE_CORE_FILES_PRESENT, PUBLIC_LIBRARY_SURFACE; behavior: fixture3
- Problem statement — REQUIRED_CRATES_PRESENT (thin over aqc-*); guardrail3 coupling: see Boundary entry
- Design principles / Mechanism, not policy — RUST_FORBIDDEN_SUBSTRINGS (bypass flags); read-only, --json, exit codes: fixture3
- Design principles / Source recorded, never judged — ENUM_VERIFIER_SOURCE; report labeling: fixture3
- Design principles / Closed-world core, open-world escape hatch — fixture3 (lint behavior)
- Repository — WORKSPACE_CORE_FILES_PRESENT
- Non-Goals — NO_RUST_TEST_TREE, RUST_FORBIDDEN_SUBSTRINGS, FORBIDDEN_CRATES_ABSENT; feature absences: reviewed, not verified
- V1 Source Format — REQUIRED_CRATES_PRESENT; the stack running: fixture3
- JSON Shape Rules — fixture3 (lint behavior)
- Spec Model — ENUM_CATEGORY (six categories closed); lint behavior: fixture3
- Custom Verifiers — fixture3 (execution protocol)
- Requirement Coverage — fixture3
- Public Library Surface — PUBLIC_LIBRARY_SURFACE, ENUM_CATEGORY, ENUM_VERDICT, ENUM_VERIFIER_SOURCE
- Commands (lint, verify) — fixture3
- Spec Change Review — not-applicable (decision record)
- Evidence Model — ENUM_VERDICT, ENUM_VERIFIER_SOURCE (typed fields); report shape, stamps: fixture3
- Verification Phases — fixture3
- V1 Requirement Categories (tree, content, ownership, backend attachment) — REQUIRED_CRATES_PRESENT (aqc-* attached); verifier behavior: fixture3; "no placeholder walk/read": review-enforced for library code (adjudicated 2026-06-07: throwaway verifier scripts are exempt; the rule governs how the library is built)
- Deferred Categories — not-applicable (future work)
- Deferred: Lock Layer — not-applicable (decision record)
- Path, Glob, And Walk Semantics — REQUIRED_CRATES_PRESENT (camino, globset); lint path rules: fixture3
- Git State Diagnostics — REQUIRED_CRATES_PRESENT (aqc-git-helpers); output: fixture3
- Boundary With Guardrail3 v2 — FORBIDDEN_CRATES_ABSENT (adjudicated 2026-06-07: prefixes g3/guardrail banned, plus the aqc-shared engine crates; pinned in the plan's Boundary section)
- Relationship To Other Tools — not-applicable (narrative)
- Dependency Health Gate — REQUIRED_CRATES_PRESENT, FORBIDDEN_CRATES_ABSENT
- G3RS And Test Policy — WORKSPACE_CORE_FILES_PRESENT (policy file), NO_RUST_TEST_TREE, RUST_FORBIDDEN_SUBSTRINGS
- Implementation Order — not-applicable (sequencing)
- V1 Definition Of Done — repository-state items covered by the nine IDs; every behavioral item: fixture3

## Summary

9 requirements cover all repository-state facts the plan pins: 4 core files, test-tree ban, 4 forbidden substrings, 10 required + 7 forbidden crates + 2 forbidden prefixes, 15 public types + 2 functions, 3 closed enums. Behavior is fixture3's lane. UNCOVERED: 0. One rule is review-enforced by adjudication (placeholder-walk ban, library code only).
