# `specular --help` — output

The literal text `specular --help` (also `-h`, `help`) prints. Committed as `HELP.txt`, embedded via `include_str!`.

```
specular — deterministic spec-driven development

Verify a repository against a machine-readable JSON spec. specular reads the spec,
checks repository state, and reports per-requirement evidence.

USAGE
  specular lint   <spec.json> [--json]   validate the spec file alone
  specular verify <spec.json> [--json]   check the repository against the spec
  specular --help                        print this text (also: -h, help)

EXIT CODES
  0  spec valid (lint) / repository conforms (verify)
  1  repository does not conform (verify only)
  2  spec invalid, or a verifier/runtime error
  No bypass flags exist.

SPEC FORMAT
  Strict JSON, no comments. This example uses every field; copy and delete what
  you do not need. Any category may be omitted; any list field defaults to [].

  {
    "version": 1,
    "verifiers": {
      "dependencies": ["scripts/verify-deps.sh"],
      "exports":      ["scripts/verify-exports.sh"],
      "enumerations": ["scripts/verify-enums.sh"],
      "schemas":      ["scripts/verify-schemas.sh"]
    },
    "requirements": {
      "tree": [
        {
          "id": "CORE_FILES",
          "reason": "plan: Repository layout",
          "requiredPaths": ["src/lib.rs", "README.md"],
          "forbiddenGlobs": ["**/tests/**"]
        }
      ],
      "content": [
        {
          "id": "NO_TEST_CODE",
          "reason": ["plan: Test policy", "plan: No bypass"],
          "files": ["**/*.rs"],
          "requiredSubstrings": [],
          "forbiddenSubstrings": ["#[test]", "--force"]
        }
      ],
      "dependencies": [
        {
          "id": "CRATES",
          "reason": "plan: Dependency gate",
          "manifestGlobs": ["Cargo.toml"],
          "requiredCrates": ["serde"],
          "forbiddenCrates": ["openssl"],
          "forbiddenCratePrefixes": ["guardrail"]
        }
      ],
      "exports": [
        {
          "id": "PUBLIC_API",
          "reason": "plan: Public surface",
          "package": "specular",
          "types": ["Spec", "Report"],
          "functions": ["lint", "verify"]
        }
      ],
      "enumerations": [
        {
          "id": "STATUS_SET",
          "reason": "plan: Public surface",
          "type": "Status",
          "variants": ["Pass", "Fail"]
        }
      ],
      "schemas": [
        {
          "id": "WIRE_SCHEMA",
          "reason": "plan: Durable formats",
          "file": "schemas/evidence.schema.json"
        }
      ]
    }
  }

FIELDS  (* required; every other field defaults to [])
  every row    id*, reason            reason is a string or array of strings
  tree         requiredPaths, forbiddenGlobs
  content      files*, requiredSubstrings, forbiddenSubstrings
  dependencies manifestGlobs*, requiredCrates, forbiddenCrates, forbiddenCratePrefixes
  exports      package*, types, functions
  enumerations type*, variants*
  schemas      file*

RULES
  - id: SCREAMING_SNAKE_CASE, unique across ALL categories.
  - One row per category+scope. Scope is: tree=the repo, content=files,
    dependencies=manifestGlobs, exports=package, enumerations=type, schemas=file.
    Two rows with the same category and scope are rejected — put required and
    forbidden in the SAME row.
  - At least one positive assertion must exist (requiredPaths / requiredSubstrings
    / requiredCrates / exports / enumerations / schemas).
  - Paths: repo-root-relative, "/" separators, no absolute, no "..".
    Globs: "*" stays within one path segment, "**" crosses segments.

VERIFIERS
  Each category is judged by exactly one verifier.

    builtin (run automatically, declare nothing):  tree, content
    no builtin (you MUST declare one):             dependencies, exports,
                                                   enumerations, schemas

  To supply a missing verifier, or to replace a builtin, add the category to the
  top-level "verifiers" map with a command:

    "verifiers": { "dependencies": ["scripts/verify-deps.sh"] }

  Writing a verifier — three steps:
    1. Make an executable command. specular runs it as:
         <command...> <spec.json> <category>
       Read requirements.<category> from the spec file (argv 1).
    2. Print one JSON object per line to stdout, one per requirement in that
       category. Required keys id, status; the rest optional:
         {"id":"CRATES","status":"pass"}
         {"id":"CRATES","status":"fail","message":"openssl present",
          "observed":"openssl 0.10","expected":"absent","path":"Cargo.toml"}
    3. Add the category to "verifiers" and run `specular verify`.
  Exit nonzero, a missing line for a requirement, or a line for an id outside the
  category is a runtime error (exit 2).

LINT ERRORS (all reported at once)
  JSON_SCHEMA              wrong shape, unknown field, or wrong type
  DUPLICATE_ID             id used more than once
  ID_FORMAT                id not SCREAMING_SNAKE_CASE
  PATH_RULE / GLOB         bad path, or a glob that will not compile
  MERGEABLE_REQUIREMENTS   two rows with the same category and scope
  VACUOUS_SPEC             no positive assertion
  CATEGORY_HAS_NO_VERIFIER a non-builtin category with no verifier declared
  UNKNOWN_CATEGORY         a "verifiers" key that is not a category

WORKFLOW
  1. Write the plan in prose (kept as the detailed source).
  2. Write spec.json for everything specular can check.
  3. Write verifier scripts for the non-builtin categories you use.
  4. Run `specular verify` on the unbuilt repo; confirm it fails where expected.
  5. Build until `specular verify` exits 0.
  6. Spot-check by hand; the spec is not exhaustive.

REPORT
  Per requirement: id, source (builtin:<category> or custom:<category>), status,
  and a message on failure. Header stamps the spec hash, each verifier file hash,
  the specular version, and the Git state of those files. --json emits it as data.
```
