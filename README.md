# Specular is a CLI for enforcing spec-driven development.
It turns a prose plan into JSON checks. Then it tests the repo.
Use it when an agent builds code from a plan and "done" is too weak.
Install with `cargo install --git https://github.com/agent-quality-controls/specular`, run `specular --help`, then run `specular verify <spec.json>`.
Tell your agent to use Specular, make the spec and coverage map, add needed scripts, and keep working until verify exits `0`.

## What it is

Specular gives an agent a checked loop. The agent reads the plan, writes the
spec, runs it, fixes the repo, and runs it again.

The plan stays easy to read. The JSON spec names the parts a program can check.
The result is JSON proof, one item at a time.

Exit codes:

- `0`: the spec is valid, or the repo matches it.
- `1`: the repo does not match it.
- `2`: the spec, verifier, call shape, timeout, or run failed.

Agents can read the JSON result. Status is a pass or fail, not a prose claim.

## Why use it

An agent can say the plan is done. Specular makes it prove the work.

Use this loop:

1. Turn the plan into a typed JSON spec.
2. Run the spec before coding.
3. Confirm it fails where work is still missing.
4. Build the code.
5. Run `specular verify` until it exits `0`.

Use it for plans with clear parts: files, text, exports, deps, named cases, or
checks a script can test.

## Install

From this repo:

```bash
# Install the local checkout.
cargo install --path .
```

From GitHub:

```bash
# Install the latest main branch from GitHub.
cargo install --git https://github.com/agent-quality-controls/specular
```

Check the install:

```bash
# Print the current spec format, verifier calls, and report shape.
specular --help
```

The help text owns the spec shape, field names, script calls, lint errors, exit
codes, and report JSON.

## Quick start

Start with a prose plan. Put a JSON spec and coverage map next to it:

```text
.plans/my-change.md
.plans/my-change.md.spec.json
.plans/my-change.md.spec.coverage.md
```

Then run the loop:

```bash
# Check that the spec is well formed.
specular lint .plans/my-change.md.spec.json

# Run before coding. Missing work should fail here.
specular verify .plans/my-change.md.spec.json

# Build, then run again until the command exits 0.
specular verify .plans/my-change.md.spec.json
```

If the first verify passes too early, the spec is too weak.

## Tell your agent

Use this prompt:

```text
Use Specular for this plan.
Make a JSON spec.
Write a coverage map.
Add any needed verifier scripts.
Run specular verify before coding.
Keep working until specular verify exits 0.
Use specular --help for the spec shape and script calls.
```

For larger plans, ask for three passes:

```text
Use Specular.
Run three separate spec drafts from the plan.
Lint each draft.
Merge conflicts.
Write the accepted spec and coverage map.
Build until specular verify exits 0.
```

The coverage map shows which plan headings Specular checks, which belong in
behavior fixtures, which need custom verifiers, and which are not covered.

## Spec patterns

There are three patterns:

1. Predefined categories with built-in checks: `tree`, `content`.
2. Predefined categories that need your script: `dependencies`, `exports`,
   `enumerations`.
3. Custom categories: put any JSON under `custom`, then write the script.

Predefined categories have fixed fields. Custom entries can hold any JSON.

Run `specular --help` before writing a spec. It includes full examples and the
script call rules.

## Verifier scripts

Verifier commands can be any script the OS can run. Python is usually the least
annoying choice.

Typed script blocks are called once per block:

```text
<command...> <spec.json> <category> <blockIndex>
```

Custom scripts are called once:

```text
<command...> <spec.json> custom
```

Verifier scripts print JSON proof lines. Their exit code only says whether the
script ran cleanly; the proof carries pass and fail results.
