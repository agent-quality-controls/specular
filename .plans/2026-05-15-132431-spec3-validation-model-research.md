# Goal

Explain which verification approaches matter for `spec3`, what each one is for, what risks it creates, and what system we should build first.

# Short Decision

`spec3` should not start as a generic formal-methods tool, a CUE wrapper, an OPA wrapper, or a JSON Schema wrapper.

It should start as a small contract checker with these rules:

- The reviewed `.spec3.json` or `.spec3.jsonc` file is the source of truth.
- Every active requirement has a stable ID.
- Every active requirement is either checked by a known checker or rejected as unsupported.
- Every checker result points back to the requirement ID it checked.
- `verify` first checks that the locked inputs did not drift.
- Only after that does `verify` check repository facts against requirements.
- Unsupported non-empty requirement categories fail early. They are not silently ignored.

This borrows from requirements traceability and Design by Contract without making the user write safety-critical-process paperwork.

# What We Are Building

`spec3` is a conformance checker for implementation structure.

It answers:

- Did the implementation build what the reviewed spec says must exist?
- Did the implementation avoid what the reviewed spec says must not exist?
- Did we run those checks against the same plan, spec, and verifier files that were reviewed?
- Can every failure be traced to the exact requirement that failed?

It does not answer:

- Is the software correct in all possible executions?
- Is the prose plan wise?
- Did behavior output change?
- Does every function satisfy a mathematical proof?
- Did an LLM implement the best architecture?

# Core Model

Use these terms internally:

- Contract: the reviewed spec.
- Requirement: one checked statement in the contract.
- Checker: code that evaluates requirements.
- Fact: observed repository state.
- Evidence: a pass or fail result for one requirement.
- Precondition: something that must be true before verification can be trusted.
- Postcondition: something that must be true after repository facts are checked.
- Invariant: something that must always be true for the repo to be valid.

Example:

```jsonc
{
  "requirements": {
    "text": [
      {
        "id": "NO_RUST_TESTS",
        "scope": ["**/*.rs", "Cargo.toml"],
        "forbidden": ["#[test]", "#[cfg(test)]", "cargo test"]
      }
    ]
  }
}
```

Meaning:

- Requirement ID: `NO_RUST_TESTS`
- Checker: built-in `text`
- Fact input: file contents under the scoped paths
- Evidence output: pass if forbidden strings are absent, fail with matching path and location if present
- No prose-plan source reference required

# Answers To Current Comments

## Source reference back to prose plan

Decision:

- Do not require it.

Reason:

- Once reviewed, the machine-readable spec is the contract.
- Requiring every requirement to point to a prose line creates maintenance work and weakens adoption.
- The plan can still be locked and hashed as the parent document.

Consequence:

- `spec3` detects spec/code drift, not prose/spec disagreement.
- Prose/spec review remains a human step.

## Verification method per requirement

Decision:

- Do not require a separate `verificationMethod` field in V1.

Reason:

- The requirement category already implies the checker.
- `requirements.text` is checked by the text checker.
- `requirements.tree` is checked by the tree checker.

Consequence:

- The spec stays shorter.
- `spec3 lint` must reject non-empty categories that have no checker in the current implementation.

## Every requirement must have a verification route

Decision:

- Yes, for every non-empty active requirement category.

Plain meaning:

- If the spec says something must be checked, `spec3` must know what code checks it.
- If no checker exists, `spec3` must fail before pretending verification happened.

Example:

```jsonc
{
  "requirements": {
    "exports": [
      {
        "id": "PUBLIC_API",
        "required": { "types": ["Spec"] }
      }
    ]
  }
}
```

In V1, this must fail with "exports requirements are not supported yet."

It must not pass, skip, or warn only.

## Every finding maps back to a requirement ID

Decision:

- Yes.

Plain meaning:

- Every pass or fail result must say which requirement it checked.

Example failure:

```json
{
  "requirementId": "NO_RUST_TESTS",
  "checker": "text",
  "status": "fail",
  "path": "crates/spec3/src/lib.rs",
  "message": "forbidden text found: #[test]"
}
```

Why this matters:

- Without requirement IDs, output becomes a loose lint report.
- With requirement IDs, a future agent can tell exactly which contract item broke.

## Orphan requirements and orphan checker outputs

Decision:

- Orphan requirements are fatal.
- Orphan checker outputs are fatal.

Plain meaning:

- Orphan requirement: the spec contains a requirement that no checker evaluated.
- Orphan checker output: a checker reported something that is not tied to a requirement in the spec.

Example orphan requirement:

```jsonc
{
  "requirements": {
    "tree": [
      { "id": "ROOT_README", "required": { "files": ["README.md"] } }
    ]
  }
}
```

If `verify` finishes without a result for `ROOT_README`, verification is invalid.

Example orphan checker output:

```json
{
  "requirementId": "UNKNOWN_REQUIREMENT",
  "checker": "tree",
  "status": "fail"
}
```

If `UNKNOWN_REQUIREMENT` is not in the locked spec, verification is invalid.

## Lock stores requirement-to-checker mapping

Simpler wording:

- The lock should record which checker is responsible for each requirement ID.

Decision:

- Yes, but keep it simple.

Example:

```json
{
  "checks": [
    { "requirementId": "NO_RUST_TESTS", "checker": "text" },
    { "requirementId": "ROOT_README", "checker": "tree" }
  ]
}
```

Why this matters:

- If a later edit changes which checker owns a requirement, the lock changes.
- That prevents an agent from silently rerouting a requirement to a weaker checker.

# Borrowed Approaches

## Requirements traceability

What it is:

- A way to connect requirements to implementation and verification evidence.

What we should borrow:

- Stable requirement IDs.
- Every requirement must be checked.
- Every check result must point back to a requirement.
- Orphans matter.

What we should not borrow:

- Heavy source-to-design-to-test matrices.
- Mandatory prose-line references.
- Safety-critical documentation workflow.

Consequence for `spec3`:

- Build requirement coverage into `lint`, `lock`, and `verify`.
- Do not make users maintain a full traceability matrix.

Sources:

- NASA SWE-059 says bidirectional traceability helps ensure no requirements are lost and no extra design elements exist without a parent requirement: https://swehb.nasa.gov/pages/viewpage.action?pageId=16451354
- ISO/IEC/IEEE 29148 is the requirements-engineering standard to evaluate later. Public summaries describe requirement attributes and verification links: https://standards.ieee.org/ieee/29148/6937/

## Design by Contract

What it is:

- A model where correctness is expressed as preconditions, postconditions, and invariants.

What we should borrow:

- Preconditions: checks that must pass before verification can be trusted.
- Postconditions: checks over repository facts.
- Invariants: checks that must always hold.

How this maps:

- Precondition: locked plan hash matches current plan.
- Precondition: locked spec hash matches current spec.
- Precondition: locked verifier file hashes match current verifier files.
- Postcondition: required file exists.
- Postcondition: forbidden text is absent.
- Invariant: no Rust test scaffolding exists in this repo.

What we should not borrow:

- Do not expose these words as required JSON fields in V1.
- Do not make users model every rule in contract theory terms.

Consequence for `spec3`:

- Use the model to keep the implementation clean.
- Keep the user-facing spec simple.

Sources:

- Eiffel documentation describes preconditions, postconditions, and invariants as explicit behavioral contracts: https://www.eiffel.com/values/design-by-contract/

## Policy as code with OPA

What it is:

- A policy engine that evaluates structured input data against policies.

What it is good for:

- Arbitrary rules over structured facts.
- Cases where policies change more often than the host program.
- Cross-domain rules that should not be hard-coded into a CLI.

What it would give us:

- A mature model for separating fact collection from policy decisions.
- A path for future arbitrary rules over repository facts.

Problems:

- Requires Rego or generated Rego.
- Makes `spec3` depend on a second policy language.
- Harder to keep every output tied to a simple requirement ID.
- Too much for V1.

Decision:

- Do not integrate OPA in V1.
- Borrow the architecture: collect structured facts, then evaluate requirements against those facts.

Sources:

- OPA describes itself as separating policy decisions from policy enforcement and evaluating structured JSON-like input: https://www.openpolicyagent.org/docs

## CUE

What it is:

- A data constraint language for validation, configuration, querying, and code generation.

What it is good for:

- Combining schema and data constraints.
- Validating structured data with richer constraints than JSON Schema.
- Making invalid states fail during data unification.

What it would give us:

- A mature constraint language for the spec shape.
- A possible replacement for JSONC plus custom validation.

Problems:

- Requires users or generated files to interact with CUE.
- Adds a second language before we know our model.
- Rust integration likely means shelling out or carrying bindings.
- Overkill before V1 requirement categories stabilize.

Decision:

- Do not use CUE in V1.
- Reconsider CUE if the spec shape becomes hard to validate with Rust types and JSON Schema.

Sources:

- CUE describes itself as a data validation language with logic-programming roots and constraints that allow schema and policy constraints to coexist: https://cuelang.org/docs/

## JSON Schema

What it is:

- A standard vocabulary for validating JSON document shape.

What it is good for:

- Validating the syntax and structure of `.spec3.json`.
- Publishing a machine-readable schema for editors and tools.

What it is not good for:

- Checking repository state.
- Checking Git drift.
- Explaining spec lifecycle.
- Replacing `tree` and `text` checkers.

Decision:

- Use JSON Schema later for spec file shape if a maintained Rust validator passes dependency review.
- Do not use JSON Schema as the main verification engine.

Sources:

- JSON Schema documents separate Core and Validation specs: https://json-schema.org/specification

## TLA+

What it is:

- A formal specification language and model-checking ecosystem for checking system state transitions.

What it is good for:

- State machines.
- Concurrency.
- "This bad state must never be reachable" questions.

What it would help here:

- Prove the lock/status/verify lifecycle cannot accept drifted inputs.
- Clarify trusted and untrusted states before code exists.

Problems:

- Separate formal language.
- More machinery than the V1 implementation needs if the state machine is small.
- Does not prove the Rust implementation matches the model unless we maintain the mapping.

Decision:

- Do not integrate TLA+ into the runtime.
- Use a plain state table first.
- Consider TLA+ only if the state table becomes ambiguous or concurrent.

Sources:

- TLA+ is used for formal specification and model checking; AWS describes using formal specs after prose design to find design errors before implementation: https://cacm.acm.org/research/how-amazon-web-services-uses-formal-methods/

## Alloy

What it is:

- A lightweight formal modeling language and analyzer for relational constraints.

What it is good for:

- Checking small logical models.
- Finding counterexamples in relationship-heavy designs.

What it would help here:

- Requirement-to-checker mapping.
- Orphan requirement/output constraints.
- Lock relation constraints.

Problems:

- Separate language and tool.
- More useful for design review than runtime verification.
- Adds cost if the relation model is simple enough to state directly.

Decision:

- Do not integrate Alloy into V1.
- Use its way of thinking: define the relations clearly.
- Consider Alloy if requirement/checker/evidence relations become complex.

Sources:

- Alloy docs describe the Analyzer as the tool that checks a spec by converting the model to a SAT formula: https://alloy.readthedocs.io/en/latest/tooling/analyzer.html

## RFC 8785 JSON Canonicalization

What it is:

- A JSON canonicalization scheme for creating stable bytes for hashing and signing.

What it is good for:

- Lock hashes.
- Avoiding hand-rolled sorted JSON.

Problems:

- Rust crates checked so far fail the current star threshold.
- Full RFC behavior includes edge cases around duplicate keys, Unicode, and number serialization.
- Implementing it ourselves is risky.

Decision:

- Prefer strict JSON subset plus a well-reviewed canonicalization path.
- If no Rust crate passes the dependency gate, either narrow the spec to avoid numbers and reject duplicate keys, or allow a documented exception for a small audited crate.

Sources:

- RFC 8785 defines JCS and requires duplicate property names not appear in input data, Unicode-expressible strings, and deterministic property sorting: https://www.rfc-editor.org/rfc/rfc8785.html

# Proposed V1 Algorithm

## `lint`

Steps:

1. Parse spec source.
2. Reject duplicate keys.
3. Decode into typed Rust model.
4. Validate requirement IDs are unique.
5. Validate active categories are supported.
6. Validate every active requirement has a checker.
7. Validate paths are repo-root-relative UTF-8 paths.
8. Validate glob patterns compile.
9. Validate no non-empty unsupported category exists.

Output:

- Spec is structurally valid, or a list of precise errors.

## `lock`

Steps:

1. Run `lint`.
2. Check Git dirty state for plan, spec, and verifier files.
3. Build the requirement-to-checker map.
4. Canonicalize the spec.
5. Hash the plan, canonical spec, verifier files, and requirement-to-checker map.
6. Write lock metadata.

Output:

- Lock file that freezes what was reviewed and how it will be checked.

## `status`

Steps:

1. Read spec and lock.
2. Recompute plan/spec/verifier hashes.
3. Compare against lock.
4. Report drift.
5. Report unsupported populated categories.
6. Report coverage problems if the requirement-to-checker map cannot be built.

Output:

- Trusted or untrusted state, with exact reason.

## `verify`

Steps:

1. Run `status` preconditions.
2. Stop if plan, spec, verifier, or checker mapping drifted.
3. Extract repository facts.
4. Run checkers against facts.
5. Require one result set for every active requirement.
6. Reject orphan checker results.
7. Emit evidence.

Output:

- Pass only if all active requirements are checked and pass.

# What We Should Build

Build V1 as:

- Rust CLI.
- Strict typed spec model.
- Active categories only: `tree`, `text`.
- Empty unsupported categories allowed only if we decide they help forward compatibility.
- Non-empty unsupported categories fail.
- Requirement IDs required.
- Requirement-to-checker map built automatically from category.
- Lock includes that map.
- Every result includes requirement ID.
- `verify` fails on orphan requirements and orphan outputs.
- JSON output is designed as evidence, not just messages.

# What We Should Not Build First

Do not build first:

- OPA integration.
- CUE integration.
- TLA+ or Alloy runtime integration.
- Export parsing.
- Dependency parsing.
- Arbitrary external verifier protocol.
- Broad JSON Schema validation.
- Generic command runner.
- Full source-to-prose traceability.

# Concrete Changes To Main Plan

The main plan should be revised so:

- User-facing questions use plain terms.
- `source reference back to prose plan` is answered no.
- `verification method per requirement` is answered no for V1.
- `every active requirement has a checker` is answered yes.
- `every result maps to a requirement ID` is answered yes.
- `orphan requirements and orphan checker outputs` are fatal.
- `lock stores requirement-to-checker map` replaces the jargon phrase "requirement-to-verifier links as first-class data".
- V1 explicitly rejects non-empty unsupported categories.
- V1 includes a common evidence JSON shape.
