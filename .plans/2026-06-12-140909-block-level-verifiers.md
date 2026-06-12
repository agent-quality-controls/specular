# Block-level verifiers

## Goal

Migrate Specular from category-level verifier declarations to block-level verifier declarations. Every requirement block must say exactly which verifier checks it, using the field name `verifier`.

This pass migrates the spec format, linting, dispatch, docs, and existing fixtures/specs. It does not add new ecosystem verifiers such as Cargo, npm, Rust exports, TypeScript exports, or enum extraction. Those come after the format migration is complete.

## New spec shape

Specular uses spec `version: 2`.

There is no top-level `verifiers` map in version 2.

Every requirement block has:

```json
"verifier": ["builtin:tree"]
```

or:

```json
"verifier": ["scripts/verify_deps.py"]
```

The verifier value is one command expressed as an argv vector, not a list of
verifiers and not a shell string. Specular never invokes a shell.

The first argv token selects dispatch:

- `builtin:<name>` runs an internal Specular verifier.
- any other first token runs as an external command with the remaining argv tokens.

The initial built-ins are:

- `builtin:tree`, allowed only for `tree`
- `builtin:content`, allowed only for `content`

## Requirement categories

`tree` remains one object and gains `verifier`.

```json
"tree": {
  "verifier": ["builtin:tree"],
  "required": ["src/lib.rs"],
  "forbidden": ["target/**"]
}
```

`content`, `dependencies`, `exports`, and `enumerations` remain arrays of typed blocks. Each block gains `verifier`.

```json
"dependencies": [
  {
    "verifier": ["scripts/verify_deps.py"],
    "manifests": ["Cargo.toml"],
    "required": ["serde"]
  }
]
```

`custom` remains an array of opaque objects, but each object reserves the field `verifier`. All other keys remain custom data.

```json
"custom": [
  {
    "verifier": ["scripts/verify_repo.py"],
    "check": "github-settings",
    "repo": "agent-quality-controls/specular"
  }
]
```

## Verifier protocol

External typed verifier blocks still run once per block:

```text
<command...> <spec.json> <category> <blockIndex>
```

External custom verifier entries also run once per entry:

```text
<command...> <spec.json> custom <blockIndex>
```

The external verifier reads `requirements.<category>[blockIndex]`, ignoring the reserved `verifier` field.

Typed blocks emit exactly one evidence line per item in the block.

Custom blocks emit exactly one evidence line per custom entry. The line must contain `status`. Other fields are copied into the report.

Verifier exit code signals verifier health only. Pass/fail verdicts travel only in evidence.

## Lint rules

Version 2 lint rules:

- `version` must be `2`.
- top-level `verifiers` is not allowed.
- every requirement block, including `tree` and every `custom` entry, must include `verifier`.
- `verifier` must be one non-empty argv array of non-empty strings.
- `builtin:<name>` must exist.
- each builtin must support the category where it is used.
- external verifier commands are not resolved during lint; missing executables remain verify-time errors.

Existing item, path, duplicate target, duplicate item, contradiction, redundancy, vacuity, and custom-shape rules still apply.

Old lint errors tied to category-level verifiers are removed or renamed:

- remove `CATEGORY_HAS_NO_VERIFIER`
- remove `UNKNOWN_CATEGORY` for top-level verifier keys
- remove `DEAD_VERIFIER`
- keep `VERIFIER_COMMAND_EMPTY` for empty block-level command arrays
- add `VERIFIER_MISSING`
- add `UNKNOWN_BUILTIN_VERIFIER`
- add `BUILTIN_CATEGORY_MISMATCH`

## Verify dispatch

Verification resolves each non-empty requirement block independently.

For each block:

1. Read `block.verifier`.
2. If `verifier[0]` starts with `builtin:`, dispatch through the internal builtin registry.
3. Otherwise run the argv command with Specular protocol args.

Built-in evidence uses:

```json
"source": "builtin"
```

External command evidence uses:

```json
"source": "script"
```

Every evidence object includes:

```json
"verifier": "builtin:tree"
```

or the external command label chosen by Specular.

`verifier_files` stamps repo-relative executable/script files that are present in external command argv arrays. Built-ins do not add verifier file stamps.

## Implementation work

Update `src/model.rs`:

- remove `Spec.verifiers`
- add `VerifierCommand`
- add `verifier` to `TreeRequirement`
- add `verifier` to `ContentRequirement`
- add `verifier` to `DependencyRequirement`
- add `verifier` to `ExportRequirement`
- add `verifier` to `EnumerationRequirement`
- replace `custom: Vec<serde_json::Value>` with a typed custom object that requires `verifier` and flattens all other fields

Update `src/lint.rs`:

- require spec version 2
- validate block-level verifier command arrays
- validate builtin names and category compatibility
- remove category-level verifier validation

Update `src/verify.rs`:

- replace category-level verifier selection with block-level dispatch
- add builtin verifier registry
- move current tree and content checks behind `builtin:tree` and `builtin:content`
- run custom verifiers per custom entry
- stamp inline external verifier files

Update `src/evidence.rs`:

- rename external source from `custom` to `script`
- add a serialized `verifier` field to evidence

Update docs and examples:

- `HELP.txt`
- `README.md`
- any skill or fixture text in this repository that shows top-level `verifiers`

Update behavior fixtures:

- migrate valid fixtures to version 2
- update golden reports for `source: script` and evidence `verifier`
- add lint fixtures for missing verifier, unknown builtin, builtin category mismatch, and rejected top-level `verifiers`
- add verify fixtures for script verifier dispatch and custom per-entry dispatch
- add a custom verifier for this migration plan so Specular can report the remaining migration gaps before the implementation starts

Update local Specular plan specs:

- migrate existing `.plans/*.spec.json` files used by this repo to version 2
- update `scripts/verify-repo-quality.py` to accept custom block index protocol

## Key decisions

- Use `verifier`, not `using`, because the field names the verifier directly.
- Do not keep a top-level alias map in version 2. Inline verifier commands make each block locally understandable.
- Keep builtin and external verifier commands in the same argv shape so options can be added later without a second schema.
- Do not add ecosystem built-ins in this migration. The migration creates the dispatch surface that later built-ins will use.
- Do not support shell command strings.
- Prefer a clean version 2 migration over dual v1/v2 semantics in the same model. If compatibility is later required, add a separate v1 loader that lowers to the v2 internal model.

## Files to modify

- `src/model.rs`
- `src/lint.rs`
- `src/verify.rs`
- `src/evidence.rs`
- `HELP.txt`
- `README.md`
- `scripts/verify-repo-quality.py`
- `scripts/verify-block-level-verifiers.py`
- `.plans/*.spec.json`
- `behavior/fixtures/**/repo/spec.json`
- `behavior/fixtures/**/repo/scripts/*`
- `behavior/golden/**/*.json`

## Out of scope

- Cargo dependency builtin verifier.
- npm dependency builtin verifier.
- Rust export verifier.
- TypeScript export verifier.
- Rust or TypeScript enum verifier.
- top-level verifier aliases.
- shell command strings.
