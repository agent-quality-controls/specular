# Plan: per-category verifiers + front-loaded help

Goal: verifiers become per-category (builtin or an overriding script); merge
required+forbidden into one row per scope; omit empty categories; move all format
docs into `spec3 --help`; shrink the skill to a pointer.

Companion: `2026-06-07-202732-help-output-draft.md` is the exact help text.

## The model (decided)

- Top-level `verifiers` map: category -> command. Omitted category uses the
  builtin (tree, content) or, for the others, fails lint.
- A category's verifier judges every row in that category. No per-row ownership,
  no claims, no requirementIds.
- Override = list a builtin category in the map; the script replaces the builtin.
- Empty categories may be omitted from `requirements`.
- Categories are the closed set of six (open categories not adopted).

## Change 1 — model.rs

- `verifiers: BTreeMap<Category, Vec<String>>`, `#[serde(default)]`. Delete
  `VerifierDecl` and `requirement_ids`.
- Each `Requirements` category field gets `#[serde(default)]` so it can be
  omitted; absent = empty.

## Change 2 — verify.rs

- Per category with rows: builtin fn for tree/content unless the map overrides;
  otherwise run the mapped command as `<command...> <spec.json> <category>` from
  the repo root.
- A missing command file is NOT pre-checked: the spawn fails -> `VerifyError`
  -> exit 2 (case D). Lint never touches disk for this.
- Coverage per category: every row id gets exactly one evidence line; a line for
  an id outside the category, or a missing line for a row, is a runtime error.
- `VerifierSource::Custom(Category)`; source label `custom:<category>`.

## Change 3 — lint.rs

- `check_mergeable`: group by `(category, scope)` only (drop polarity), so
  required+forbidden of one scope collide -> `MERGEABLE_REQUIREMENTS`.
- Add `CATEGORY_HAS_NO_VERIFIER`: a category has rows but no builtin and no map
  entry.
- Add `UNKNOWN_CATEGORY`: a verifiers-map key is not one of the six. Deserialize
  the map with string keys and check, so it is a clean violation collected with
  the rest, not a serde parse abort.
- Delete: `UNCLAIMED_REQUIREMENT`, `UNKNOWN_CLAIM`, `OVERLAPPING_CLAIM`,
  `BUILTIN_COVERED_CLAIM`, `VERIFIER_COMMAND_MISSING` (the last is disk state,
  not a spec defect — case C and D are byte-identical JSON; it belongs at verify).
- Keep: JSON_SCHEMA, DUPLICATE_ID, ID_FORMAT, PATH_RULE, GLOB, VACUOUS_SPEC,
  garde FIELD_RULE.
- One shared `Category::has_builtin` used by both lint and verify.

## Change 4 — evidence.rs / main.rs

- `VerifierSource::Custom` carries `Category`, not a verifier id.
- Remove `Report::source_counts`; report summary drops the builtin/custom tally,
  keeps per-row lines and a plain count + conforms flag.
- `main.rs`: `help` / `--help` / `-h` print `include_str!("../HELP.txt")`; bare
  no-args prints short usage + "run spec3 help". Add `HELP.txt` at repo root.

## Change 5 — build-contract spec + coverage map

- `.plans/2026-06-07-124603-...md.spec.json`: merge the split tree rows into one
  and the split dependency rows into one; move custom verifiers into the
  `verifiers` map (dependencies/exports/enumerations -> the bash scripts); omit
  empty categories; add `HELP.txt` to the tree row.
- Hand-merge + adjudicate; update the coverage map.

## Change 6 — fixtures

- Rewrite `behavior/fixtures/*/repo/spec.json` to the map model; merge split
  rows; omit empty categories.
- lint suite: add `MERGEABLE_REQUIREMENTS` (split scope), `CATEGORY_HAS_NO_VERIFIER`,
  `UNKNOWN_CATEGORY`; drop fixtures for deleted codes.
- verify suite: add case D (verifier file missing -> exit 2) replacing the old
  command-missing-at-lint path; keep protocol/coverage error fixtures.
- Add a `help` suite: golden of `spec3 --help`.
- Re-approve goldens (report loses the tally; sources now `custom:<category>`).

## Change 7 — skill + plan doc

- SKILL.md (+ codex): cut format/field/category/verifier prose now in
  `spec3 --help`; keep the 3-pass extraction protocol, the coverage-map artifact,
  adjudication, and "run `spec3 --help`". The workflow line "run verify before
  coding to confirm it fails in the right places" mirrors help step 4.
- `.plans/2026-06-07-124603-...md`: Spec Model (one row per scope, omittable
  categories), Custom Verifiers (per-category map, override), Evidence Model (no
  tally), lint rule list.

## Order

1. Changes 1-4 (library), one build, cargo check/clippy/fmt.
2. Change 5-6 (contract + fixtures); spec3 verify self-check green;
   fixture3 check --all re-approved.
3. Change 7 (docs/skill).
4. Commit per agreed unit.

## Out of scope

- New builtin verifiers (deps-cargo etc.) — arrive when usage recurs.
- Open / user-defined categories — not adopted; six stay closed.
```
