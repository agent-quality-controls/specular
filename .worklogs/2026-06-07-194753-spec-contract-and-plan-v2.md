# Speculus CLI plan v2 (lock-free) + build contract

## Summary

Replaced the 2026-05-14 preliminary plan with a lock-free V1 plan, then produced the build contract for it with the spec-driven-development skill: a 9-requirement spec extracted by three independent passes, adjudicated, plus the bash lint/verify harness and five category verifiers. Verify currently fails on exactly the 6 requirements describing the unbuilt workspace — that is the build to-do list.

## Decisions made

- Lock file removed from V1 (tamper evidence = input-closure stamps in reports + git; lock returns later as a thin additive layer if ever needed). `diff` command also rejected — git diff of typed rows suffices.
- Custom verifiers are V1 core: declared in spec, JSON-lines evidence protocol (`status` field), claimed-ID coverage enforced; nonzero exit = runtime error.
- One noun: verifier (builtin/custom are adjectives). No trust types; `VerifierSource { Builtin(Category), Custom(VerifierId) }` records origin; the library never judges.
- `Status { Pass, Fail }` — same name as its wire field `status` (Verdict rejected: one concept, one name).
- Public API pinned in plan (Public Library Surface): `lint`/`verify` functions; 15 types.
- aqc backends attach in V1 (crates landed in aqc-shared 2026-06-07); spec glob matching uses speculus's own GlobSet (literal separator on), not `FileTree::glob`.
- Guardrail3 boundary pinned concretely: forbidden crate prefixes `g3`/`guardrail` + aqc-shared engine crates; allowed aqc crates are exactly the three fact crates.
- Granularity is derived (scope+polarity merge rule), MERGEABLE_REQUIREMENTS and VACUOUS_SPEC are lint rules; coverage map is a required artifact (0 UNCOVERED).
- Test policy: no Rust tests; behavior is fixture3's lane per the guardrails fixture development guide.

## Key files for context

- `.plans/2026-06-07-124603-speculus-plan-v2-lock-free.md` — the plan (single source of truth)
- `.plans/...md.spec.json` — the build contract (9 requirements)
- `.plans/...md.spec.coverage.md` — plan-section -> coverage mapping
- `scripts/spec-lint.sh`, `scripts/spec-verify.sh`, `scripts/verify-{tree,content,dependencies,exports,enumerations}.sh`
- `~/Projects/agent-quality-controls/guardrail3/.plans/g3v2-architecture/fixture-development-guide.md` — fixture doctrine for the behavior suite
- `~/Projects/agent-quality-controls/aqc-shared/packages/{aqc-filetree,aqc-fs-utils,aqc-git-helpers}` — the fact crates V1 wires

## Next steps

- Build the workspace until `scripts/spec-verify.sh` exits 0 (Cargo.toml, guardrail3-rs.toml, src/lib.rs, src/main.rs, the 10 crates, public API, 3 enums).
- Smoke-test the binary by hand (lint/verify against a sample spec).
- fixture3 behavior suite per the fixture development guide: layered fixtures (R00 clean golden -> R10 breakage layers), `fixture3.yaml` at repo root, replay = run `speculus` against fixture trees, one JSON record per command.
