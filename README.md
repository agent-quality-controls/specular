# Specular is a CLI for enforcing spec-driven development.

[![ci](https://img.shields.io/github/actions/workflow/status/agent-quality-controls/specular/ci.yml?branch=main&label=ci)](https://github.com/agent-quality-controls/specular/actions/workflows/ci.yml)
[![codeql](https://img.shields.io/github/actions/workflow/status/agent-quality-controls/specular/codeql.yml?branch=main&label=codeql)](https://github.com/agent-quality-controls/specular/actions/workflows/codeql.yml)
[![license](https://img.shields.io/github/license/agent-quality-controls/specular)](LICENSE)
[![rust](https://img.shields.io/badge/rust-1.85%2B-orange)](Cargo.toml)

It turns a prose plan into JSON checks that a machine can enforce.

Use it when an agent builds code from a plan and "done" is too weak.

Install with
```bash
cargo install --git https://github.com/agent-quality-controls/specular
```

Tell your agent to use Specular, make the spec and coverage map, add needed scripts, and keep working until verify exits `0`.

## What it is

Specular uses JSON spec files to enforce spec-driven development. It gives an agent a test loop instead of a prose-only plan. The agent reads the plan, writes the spec, runs it, fixes the repo, and runs it again.

Exit codes:

- `0`: the spec is valid, or the repo matches it.
- `1`: the repo does not match it.
- `2`: the spec, verifier, call shape, timeout, or run failed.

Agents can read the JSON result. Status is a pass or fail.

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

## Tell your agent

Use this prompt:

```text
Use Specular for this plan.
Make a JSON spec.
Run specular lint.
Write a coverage map.
Add any needed verifier scripts.
Run specular verify before coding.
Keep working until specular verify exits 0.
Use specular --help for the spec shape and script calls.
```

## Spec patterns

There are three patterns:

1. Predefined groups with built-in checks: `tree`, `content`.
2. Predefined groups that need your script: `dependencies`, `exports`,
   `enumerations`.
3. Custom groups: put any JSON under `custom`, then write the script.

Predefined groups have fixed fields. Custom entries can hold any JSON.

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
