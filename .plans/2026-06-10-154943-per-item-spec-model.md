# Plan: per-item spec model

Supersedes the format described in `2026-06-07-202732-help-and-verifier-model-plan.md`
and the committed ID-based model.

## Goal

- No requirement IDs. The atom is the item (a path, a substring, a package name);
  each item gets its own pass/fail entry in the report.
- Uniform quantifier keys across typed categories: `required` (in every matched
  place), `exists` (in at least one), `forbidden` (in none).
- One verifier model: every category's verifier is declared in the `verifiers`
  map; every verifier script judges its category and emits the same JSON
  evidence lines that speculus's own builtin verifiers produce.
- A `custom` category whose entries are free-form dicts; speculus never interprets
  them — the custom verifier reads the spec and does its own checks.
- All output is JSON. No flags, no text format.

## The format

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
      "reason": "plan: repository layout",
      "required": ["src/lib.rs", "src/main.rs"],
      "forbidden": ["tests/**"]
    },
    "content": [
      { "files": ["**/*.rs"], "required": ["// SPDX"], "forbidden": ["#[test]"],
        "reason": "plan: licensing, test policy" },
      { "files": ["docs/*.txt"], "exists": ["INVOICE"] }
    ],
    "dependencies": [
      { "manifests": ["Cargo.toml"], "required": ["serde"],
        "forbidden": ["openssl", "guardrail*", "g3*"] }
    ],
    "exports": [
      { "package": "speculus", "required": ["Spec", "lint", "verify"] }
    ],
    "enumerations": [
      { "name": "Status", "values": ["Pass", "Fail"] }
    ],
    "custom": [
      { "check": "version-sync", "files": ["README.md", "Cargo.toml"],
        "reason": "plan: release process" }
    ]
  }
}
```

Shape rules:

- `tree` is a single object (cannot be duplicated by construction). `content`,
  `dependencies`, `exports`, `enumerations` are arrays of typed blocks.
- `custom` is an array of free-form objects. No schema beyond "is an object";
  speculus passes them through untouched. The example's `check`/`files` keys are
  the author's own vocabulary, not speculus's.
- Block target keys are descriptive per category: `files` (content),
  `manifests` (dependencies), `package` (exports), `name` (enumerations).
  `tree` has no target: the repository.
- Any category may be omitted. Any list field defaults to `[]`.
- `reason` is optional on every typed block: string or array of strings. The
  extraction skill mandates filling it; the schema does not.
- `schemas` and `forbiddenPrefixes` no longer exist. File existence is
  `tree.required`; prefix bans are forbidden globs (`guardrail*`).

## Semantics (typed categories)

- `required`: each item present in every matched place. Strict default — a
  misread fails loudly instead of passing silently.
- `exists`: each item present in at least one matched place.
- `forbidden`: no item present in any matched place.
- `required`/`exists` items are exact strings. `forbidden` items may be globs in
  `tree` and `dependencies` (`globset`, literal separator on). `content` items
  are fixed substrings both ways.
- `enumerations`: exact set — the named set's values equal `values`, nothing
  missing, nothing extra.
- Zero matched places + non-empty `required`/`exists` = fail (no vacuous pass),
  message names the glob that matched nothing.
- `exists` is rejected by lint on `tree` and `exports` (single place; it would
  equal `required`).

## Verifier model

- One rule for every category: the verifier declared in the `verifiers` map
  judges that category. `tree` and `content` have builtins used when no map
  entry exists; the other categories require a map entry.
- Every verifier script — typed or custom — does its own checking and emits
  evidence lines (below). speculus does not re-judge script output; for typed
  categories it only checks bookkeeping (every item answered exactly once).

## Wire protocol (one, for all scripts)

Evidence line = the same object that appears in speculus's report:

```json
{"item": "serde", "status": "pass"}
{"item": "guardrail*", "status": "fail", "message": "guardrail3 declared", "path": "Cargo.toml"}
```

- `status` ("pass" | "fail") is mandatory on every line.
- Typed categories: `item` is mandatory and must be one of the block's items.
  Optional: `message`, `observed`, `expected`, `path`.
- Custom: only `status` is mandatory; all other fields are the script's own
  (e.g. `{"check": "version-sync", "status": "fail", "message": "..."}`).
  Lines are included in the report verbatim, labeled as custom.

Invocation:

- Typed categories: once per block — `<command...> <spec.json> <category>
  <blockIndex>` from the repo root. The script reads its block from the spec.
  speculus enforces: one line per item in the block, no unknown items, no
  duplicates. Violations = exit 2 with expected-vs-got.
- Custom: once — `<command...> <spec.json> custom`. The script reads the whole
  custom array and emits whatever evidence lines it produces.
  Zero lines + exit 0 = runtime error (a verifier that says nothing is broken).
- Script exit code signals health only: nonzero = runtime error (exit 2).
  Verdicts travel in the lines, never in the exit code.
- Timeout: 60 seconds per invocation, constant. Expiry = exit 2.

## Lint rules

Kept: JSON_SCHEMA, VACUOUS_SPEC (at least one `required`/`exists`/`values`
anywhere — custom entries do not count, they are opaque),
CATEGORY_HAS_NO_VERIFIER (includes custom), UNKNOWN_CATEGORY, PATH_RULE, GLOB.

Removed: DUPLICATE_ID, ID_FORMAT (no IDs), MERGEABLE_REQUIREMENTS for tree
(object shape makes it impossible).

New:

- DUPLICATE_TARGET: two blocks in one category with the same target
  (same `files` set / `manifests` set / `package` / `name`).
- DUPLICATE_ITEM: the same item twice within one block's lists.
- CONTRADICTION: same item in `required` and `forbidden` of one block.
- REDUNDANT: same item in `required` and `exists` of one block.
- ITEM_FORMAT: empty item, leading/trailing whitespace, or glob metacharacters
  (`*`, `?`, `[`) in `required`/`exists`.
- EXISTS_SINGLE_PLACE: `exists` used on `tree` or `exports`.
- DEAD_VERIFIER: a `verifiers` map entry for a category with no requirements.
- CUSTOM_SHAPE: a custom entry that is not a JSON object.

## Report (JSON only)

The only output format, for both commands. No flags.

```json
{
  "speculus_version": "0.2.0",
  "spec": {"path": "spec.json", "sha256": "..."},
  "verifier_files": [{"path": "scripts/verify-deps.sh", "sha256": "..."}],
  "git": [{"path": "spec.json", "state": "clean"}],
  "evidence": [
    {"category": "tree", "polarity": "required", "item": "src/main.rs",
     "status": "fail", "message": "missing"},
    {"category": "dependencies", "target": ["Cargo.toml"],
     "polarity": "forbidden", "item": "guardrail*", "status": "pass",
     "source": "custom"},
    {"category": "custom", "check": "version-sync", "status": "fail",
     "message": "README says 0.2, Cargo.toml says 0.3", "source": "custom"}
  ],
  "conforms": false
}
```

- Builtin-produced evidence: `source: "builtin"`. Script-produced: `source:
  "custom"`. Per item; no tallies.
- `lint` output: `{"result": "pass"}` or `{"result": "fail", "violations":
  [{"code": ..., "message": ...}]}`.
- Exit codes unchanged: 0 conform/valid, 1 nonconform, 2 error.
- Deterministic order: categories in fixed order, blocks and items in spec
  order; custom lines in emission order.

## Code changes

1. `src/model.rs` — rewrite: `Tree` object struct; typed block structs;
   `custom: Vec<serde_json::Value>`; `verifiers` map unchanged. Delete ID
   fields. `#[serde(default)]` on all lists and categories.
2. `src/lint.rs` — the rule set above; delete ID checks.
3. `src/verify.rs` — per-item judgment for builtin tree/content; per-block
   script invocation with item bookkeeping; custom invocation (verbatim lines,
   zero-lines check); 60s timeout on every spawn; expected-vs-got errors.
4. `src/evidence.rs` — evidence atom rework (category/target/polarity/item or
   custom fields; `source` builtin|custom); wire parsing.
5. `src/main.rs` — JSON-only printing; drop `--json`; help text swap.
6. `HELP.txt` — full rewrite: the format example above, quantifier semantics,
   the one wire protocol with a complete copy-paste typed script and a complete
   custom script, lint rules, workflow (unchanged), report shape.

## Repo artifacts

7. Build-contract spec — rewrite in the new format; the three bash scripts
   rewritten to emit evidence lines per block.
8. Coverage map — re-point entries at category/target instead of IDs.
9. Fixtures — rewrite all `repo/spec.json`; lint suite gains DUPLICATE_TARGET,
   CONTRADICTION, ITEM_FORMAT, EXISTS_SINGLE_PLACE, DEAD_VERIFIER fixtures;
   verify suite: typed-script protocol errors (unknown item, missing item),
   custom pass/fail/silent/broken; replay drops `--json`. Timeout not
   fixture-tested (too slow); noted in fixture docs. Re-approve goldens.
10. Skill + codex copies — extraction guidance: items self-name (no ID
    invention), reason per typed block, typed-first / custom for everything
    else.
11. Plan doc (`2026-06-07-124603-*.md`) — sync Spec Model, Verifiers, Evidence;
    note supersession of the ID model.

## Order of work

1. Library (1-5); cargo check/clippy/fmt clean.
2. HELP.txt (6); embedded JSON example must parse and lint.
3. Contract + scripts (7-8); `speculus verify` on own contract green.
4. Fixtures (9); `fixture3 check --all` re-approved.
5. Skill + plan sync (10-11).
6. Worklog + commit per unit.

## Out of scope

- New builtin verifiers.
- Configurable timeout.
- Open/user-defined typed categories (custom covers the need).
- Regex anywhere.
```
