# fixture3 behavior suite

## Summary

Added the fixture3 behavior suite per the guardrails fixture development guide: two suites (lint, verify), 11 layered fixtures replaying the real driftless binary, goldens approved, `fixture3 check --all` matched. Both gates green: build contract (spec-verify 9/9, exit 0) and behavior (fixture3 matched).

## Decisions made

- Layer ladder per the guide, grouping unit = command (driftless has no policies/families):
  - lint: R00 clean golden (all six categories, custom claims, exit 0); R10 not-JSON (parse hides everything); R11 schema-broken (schema layer hides semantic); R20 semantic (all ten violation codes fired at once, none hide); R21 vacuous (split: cannot coexist with positive rows).
  - verify: R00 clean golden (builtin tree+content + custom verifier pass, stamps, exit 0); R10 every requirement failure mode at once (exit 1); R20 verifier nonzero, R21 protocol violation, R22 coverage miss (runtime errors preempt evidence, each its own layer); R23 invalid spec (lint preflight; exit 2).
- Replay: `scripts/behavior/replay.sh` copies each fixture's `repo/` to a temp dir, runs the binary with `--json`, emits one JSON array of `{fixture, command, exit_code, output}` records sorted by fixture id. Temp dirs have no .git, so git diagnostics are deterministic (`not-a-repository`).
- Fixture repos use .txt files only — no .rs files in fixtures, so the build contract's content tripwires (`**/*.rs`) cannot be tripped by fixture inputs.
- Pollution check done before approval: lint-R20 fires exactly the ten targeted codes, once each.

## Key files for context

- `fixture3.yaml`, `scripts/behavior/replay.sh`
- `behavior/fixtures/{lint,verify}/*/` (fixture.json + repo/ trees)
- `behavior/golden/{lint,verify}/approved.normalized.json`
- `~/Projects/agent-quality-controls/guardrail3/.plans/g3v2-architecture/fixture-development-guide.md`

## Next steps

- The V1 Definition Of Done items are met except ongoing fixture growth; future behavior changes go through `fixture3 check` -> review diff -> `approve --comment`.
- Possible later: `fixture3 reduce` to minimize fixture trees; per-ecosystem builtin verifiers (deps-cargo first) when usage shows recurrence.
