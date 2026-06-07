# `spec3 --help` — proposed output

The literal text `spec3 --help` (also `-h`, `help`) should print. It
is the single source the building agent reads; the skill shrinks to "run
`spec3 --help`, then follow the workflow."

---

```
spec3 — deterministic spec-driven development

Verify that a repository matches a machine-readable JSON spec. spec3 starts at
the spec; it does not read prose plans, Markdown, or tickets. It checks
repository state and reports per-requirement evidence. It judges the repo, never
the caller: no roles, no approval, no trust scores.

USAGE
  spec3 lint   <spec.json> [--json]   validate the spec file alone
  spec3 verify <spec.json> [--json]   check the repository against the spec
  spec3 --help                        print this text (also: -h, help)

EXIT CODES
  0  spec valid (lint) / repository conforms (verify)
  1  repository does not conform (verify only)
  2  spec invalid, or a verifier or runtime error
  There are no bypass flags. Nothing softens a refusal.

SPEC FORMAT
  Strict JSON. No comments — use "reason" for explanations. One file is the
  whole contract. Unused categories may be omitted.

  {
    "version": 1,
    "verifiers": { "dependencies": ["scripts/verify-deps.sh"] },
    "requirements": {
      "tree": [
        { "id": "DOCS", "requiredPaths": ["README.md"], "forbiddenGlobs": ["tmp/**"] }
      ],
      "dependencies": [
        { "id": "NO_PG", "manifestGlobs": ["Cargo.toml"], "forbiddenCrates": ["pg"] }
      ]
    }
  }

  Every requirement has:
    "id"      SCREAMING_SNAKE_CASE, unique across ALL categories.
    "reason"  optional; a string or an array of strings (plan citations).

CATEGORIES
  One row per scope. Put every required AND forbidden item for the same scope in
  ONE row — do not split them (the linter rejects a split; see LINT RULES).

  tree            scope: the repository. Builtin verifier.
    requiredPaths        [paths that must exist]
    forbiddenGlobs       [globs no path may match]

  content         scope: files. Builtin verifier.
    files                [globs selecting which files to read]   (required)
    requiredSubstrings   [each must appear in at least one scoped file]
    forbiddenSubstrings  [none may appear in any scoped file]
    Fixed substrings only, no regex. A text scan, not a code parser.

  dependencies    scope: manifestGlobs. Needs a verifier (no builtin yet).
    manifestGlobs            [globs selecting manifests]   (required)
    requiredCrates           [must be declared]
    forbiddenCrates          [must not be declared]
    forbiddenCratePrefixes   [no declared crate may start with these]

  exports         scope: package. Needs a verifier (no builtin yet).
    package              the package checked   (required)
    types / functions    [public names that must exist]

  enumerations    scope: type. Needs a verifier (no builtin yet).
    type                 the enum type name   (required)
    variants             the exact variant set; drift either way fails

  schemas         scope: file. Needs a verifier (no builtin yet).
    file                 a committed artifact that must exist

  PATHS: repo-root-relative, "/" separators, no absolute paths, no "..", no
  empty components. Globs: "*" does not cross "/", "**" does.

VERIFIERS
  A category's verifier judges every row in that category and emits its evidence.

  BUILTIN verifiers ship with spec3 and run automatically: tree and content.
  The other categories have no builtin yet — a row in them needs a verifier set,
  or lint fails (CATEGORY_HAS_NO_VERIFIER).

  Set or override a verifier per category in the top-level "verifiers" map:
    "verifiers": { "dependencies": ["scripts/verify-deps.sh"] }
  Listing a category runs that command instead of any builtin. Overriding tree or
  content this way is allowed; the script then owns that category entirely.

  PROTOCOL: spec3 runs  <command...> <spec.json> <category>  from the repository
  root. The script reads requirements.<category> from the spec and prints one
  JSON object per line to stdout, one per requirement id in that category:
    {"id":"...","status":"pass"|"fail","message":"...",
     "observed":...,"expected":...,"path":"..."}
  Only "id" and "status" are required. Nonzero exit, a missing line for a row, or
  a line for an id outside the category is a runtime error (exit 2).

BEST PRACTICES
  - One row per scope. All Cargo.toml dependency rules in ONE dependencies row,
    all *.rs content rules in ONE content row per file-glob, and so on.
  - Encode only what a script can check deterministically. Leave intent,
    algorithms, and behavior to the prose plan and to spot checks.
  - Give every row a "reason" citing the plan.

LINT RULES (what makes a spec invalid; all are reported at once)
  JSON_SCHEMA              wrong shape, unknown field, or wrong type
  DUPLICATE_ID             an id appears more than once
  ID_FORMAT                an id is not SCREAMING_SNAKE_CASE
  PATH_RULE / GLOB         a path or glob breaks the rules or will not compile
  MERGEABLE_REQUIREMENTS   two rows share a category and scope — merge into one
  VACUOUS_SPEC             no positive assertion; would pass on an empty repo
  CATEGORY_HAS_NO_VERIFIER a category has rows but no builtin and no verifier set
  UNKNOWN_CATEGORY         the verifiers map names something that is not a category

RECOMMENDED AGENTIC DEVELOPMENT WORKFLOW
  1. Brainstorm and write the plan in prose.
  2. Convert the plan into a spec.json covering everything spec3 can check.
  3. Write verifier scripts for categories without a builtin.
  4. Run `spec3 verify` against the not-yet-built repo and confirm it FAILS in
     the right places. This proves the spec and verifiers actually run, and
     surfaces missing or broken verifier scripts before any code is written.
  5. Give the agent BOTH the prose plan and the spec. The prose plan is
     mandatory — it is more detailed than the spec, which only fixes
     high-level, checkable state.
  6. The agent builds until `spec3 verify` exits 0.
  7. Spot-check and otherwise verify by hand: the spec is not exhaustive.

REPORT
  verify prints, per requirement: id, verifier source (builtin:<category> or
  custom:<category>), status, and a concrete message on failure. The header
  stamps the spec file hash, each verifier file hash, the spec3 version, and the
  Git state of those files (diagnostic only). --json emits the report as data.
```
