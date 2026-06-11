# Design: front-loaded help + per-category verifiers (not yet implemented)

## Summary

Two design docs for the next change set. No source touched. Decides: verifiers
become per-category (builtin or an overriding script via a top-level `verifiers`
map), required+forbidden merge into one row per scope, empty categories are
omittable, and all format docs move into `specular help` so the skill shrinks to a
pointer.

## Decisions made (this session)

- Verifier is per category, not per requirement. Drop the `verifiers` array +
  `requirementIds`; replace with a `verifiers` map (category -> command).
  Override = list a builtin category in the map.
- Keep "builtin" (no rename to "default").
- One row per scope: `MERGEABLE_REQUIREMENTS` groups by category+scope only.
- Report drops the builtin/custom tally (a rolled-up count is a soft trust score,
  which the model forbids).
- `VERIFIER_COMMAND_MISSING` removed from lint: case C (file exists) and case D
  (file missing) are byte-identical JSON, so it is disk state, not a spec defect.
  D fails at verify when the command will not spawn (exit 2).
- `UNVERIFIED_CATEGORY` renamed `CATEGORY_HAS_NO_VERIFIER`. `UNKNOWN_CATEGORY`
  added as a clean lint violation for bad map keys (not a serde abort).
- Categories stay the closed six. Analysis: open categories cannot be
  scope-merge-enforced and cannot converge in 3-pass extraction — the two
  properties that justify the library over the skill. The open-world need is
  already served by the skill, outside the library.
- Workflow gains: run `specular verify` before coding to confirm it fails in the
  right places (also surfaces missing verifier files).

## Key files for context

- `.plans/2026-06-07-202732-help-output-draft.md` — exact `specular help` text
- `.plans/2026-06-07-202732-help-and-verifier-model-plan.md` — the change list

## Next steps

- Implement per the plan: model.rs (verifiers map, omittable categories),
  verify.rs (per-category dispatch, spawn-fail = exit 2), lint.rs (scope-only
  merge, CATEGORY_HAS_NO_VERIFIER, UNKNOWN_CATEGORY; drop claim codes +
  VERIFIER_COMMAND_MISSING), evidence/main (drop tally, HELP.txt + help command).
- Then update build-contract spec, fixtures (incl. help suite), skill, plan doc.
