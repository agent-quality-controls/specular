# Specular checks whether code matches a plan.
Use it when an agent builds from prose and "done" is not enough.
Install: `cargo install --git https://github.com/agent-quality-controls/specular`, then run `specular --help`.
Make a plan, make `<plan>.spec.json`, run `specular verify <spec.json>`, and build until it exits `0`.
Tell your agent: "Use Specular: make the spec, add needed verifier scripts, run verify before coding, and keep going until verify exits 0."

## What it does

Specular reads a JSON spec next to your plan. It checks the repo and prints
JSON proof for each item.

Exit codes:

- `0`: the spec is valid, or the repo matches it.
- `1`: the repo does not match it.
- `2`: the spec, verifier, call shape, timeout, or run failed.

Agents can read the JSON result. That makes status less vague than a prose
claim.

## Why use it

An agent can say the plan is done. Specular makes it prove the work.

Use this loop:

1. Turn the plan into a typed JSON spec.
2. Run the spec before coding.
3. Confirm it fails where work is still missing.
4. Build the code.
5. Run `specular verify` until it exits `0`.

Use it for plans with clear parts: files, text, exports, deps, named cases,
or checks a script can test.

## Install

From this repo:

```bash
cargo install --path .
```

From GitHub:

```bash
cargo install --git https://github.com/agent-quality-controls/specular
```

Check the install:

```bash
specular --help
```

The help text owns the spec shape, field names, verifier calls, lint errors,
exit codes, and report JSON.

## Use

Start with a prose plan. Put a JSON spec and coverage map next to it:

```text
.plans/my-change.md
.plans/my-change.md.spec.json
.plans/my-change.md.spec.coverage.md
```

Lint the spec:

```bash
specular lint .plans/my-change.md.spec.json
```

Run it before coding:

```bash
specular verify .plans/my-change.md.spec.json
```

The first verify should fail where work is missing. If it passes too early,
the spec is too weak.

Build until verify exits `0`:

```bash
specular verify .plans/my-change.md.spec.json
```

## Tell your agent

Use this prompt:

```text
Use Specular for this plan. Make a JSON spec, write a coverage map, add any needed verifier scripts, run specular verify before coding, and keep working until specular verify exits 0. Use specular --help for the spec shape and verifier calls.
```

For larger plans, ask for three passes:

```text
Use Specular. Run three separate passes from the plan, lint each draft, merge disagreements, write the accepted spec and coverage map, then build until specular verify exits 0.
```

The coverage map shows which plan headings Specular checks, which belong in
behavior fixtures, which need custom verifiers, and which are not covered.

## Spec patterns

There are three patterns:

1. Predefined categories with built-in verifiers: `tree`, `content`.
2. Predefined categories that need your verifier: `dependencies`, `exports`,
   `enumerations`.
3. Custom categories: put any JSON under `custom`, then write the verifier.

Predefined categories have fixed fields. Custom entries can hold any JSON.

Run `specular --help` before writing a spec. It includes full examples and the
verifier call rules.

## Verifiers

Verifier commands can be any script the OS can run. Python is usually the least
annoying choice.

Typed verifier blocks are called once per block:

```text
<command...> <spec.json> <category> <blockIndex>
```

Custom verifiers are called once:

```text
<command...> <spec.json> custom
```

Verifier scripts print JSON proof lines. Their exit code only says whether the
script ran cleanly; the proof carries pass and fail results.
