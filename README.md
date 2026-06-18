# Specular

[![ci](https://img.shields.io/github/actions/workflow/status/agent-quality-controls/specular/ci.yml?branch=main&label=ci)](https://github.com/agent-quality-controls/specular/actions/workflows/ci.yml)
[![codeql](https://img.shields.io/github/actions/workflow/status/agent-quality-controls/specular/codeql.yml?branch=main&label=codeql)](https://github.com/agent-quality-controls/specular/actions/workflows/codeql.yml)
[![license](https://img.shields.io/github/license/agent-quality-controls/specular)](LICENSE)
[![rust](https://img.shields.io/badge/rust-1.85%2B-orange)](Cargo.toml)

Specular is a CLI for enforcing spec-driven development.

It turns a plan into JSON checks.

Use it when an agent builds code from a plan and "done" is too weak.

Use it for code, docs, deps, and checks.

It is strict. It prints JSON. It fails when work drifts from the spec.

Install with
```bash
cargo install --git https://github.com/agent-quality-controls/specular
```

Tell your agent to use Specular. It should write the spec, add any scripts, and
work until verify exits `0`.

Tell it to use the least custom spec that works. It should use a predefined
category when one fits, use that category's builtin when one exists, and use
custom only when no predefined category fits.

## What it is

Specular uses JSON specs to check a plan. It gives an agent a test loop. The
agent reads the plan, writes the spec, runs it, fixes the repo, and runs it
again.

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
script checks.

## Tell your agent

Use this prompt:

```text
Use Specular for this plan.
Make a JSON spec.
Run specular lint.
Write a coverage map.
Use predefined categories first.
Use built-in verifiers when they exist.
Add verifier scripts only when no built-in fits.
Use custom only when no predefined category fits.
Run specular verify before coding.
Keep working until specular verify exits 0.
Use specular --help for the spec shape and script calls.
```

## Spec patterns

There are three patterns:

1. Predefined categories with built-ins: `tree`, `content`, Cargo
   `dependencies`, and Rust `enumerations`. Use these first.
2. Predefined categories with no built-in for your stack: `dependencies`,
   `exports`, `enumerations`. Use one script per block.
3. Custom categories: put any JSON under `custom`, then write the script. Use
   this only when no predefined category fits.

Every non-empty block has one block-level verifier command. Built-ins are
explicit:

```json
{
  "version": 4,
  "requirements": {
    "tree": {
      "verifier": ["builtin:tree"],
      "required": ["src/lib.rs"]
    },
    "content": [
      {
        "verifier": ["builtin:content"],
        "files": ["README.md"],
        "required": ["Specular is a CLI"]
      }
    ],
    "dependencies": [
      {
        "verifier": ["builtin:cargo-dependencies"],
        "files": ["Cargo.toml"],
        "required": ["serde"],
        "forbidden": ["openssl"],
        "forbiddenGlobs": ["g3*"]
      }
    ],
    "enumerations": [
      {
        "verifier": ["builtin:rust-enumerations"],
        "files": ["src/**/*.rs"],
        "name": "Category",
        "values": ["Tree", "Content", "Dependencies"]
      }
    ]
  }
}
```

Predefined categories have fixed fields. Custom entries can hold any JSON except
`verifier`, which Specular owns.

Do not put tree, content, Cargo package, or Rust enum checks in `custom`. Do
not write a script for checks the built-ins can judge.

For `builtin:cargo-dependencies`, `required`, `exists`, and `forbidden` use
exact Cargo package names. `forbiddenGlobs` uses package-name globs. Renamed
deps use Cargo package identity, so `serde_json = { package = "serde-json",
version = "..." }` is checked by `serde-json`.

For `builtin:rust-enumerations`, `files` selects Rust files, `name` is an enum
name or an inline-module-qualified name such as `wire::Status`, and `values` is
the exact variant set.

Run `specular --help` before writing a spec. It has full examples and script
call rules.

## Verifier scripts

A verifier is one command. It is encoded as argv, not as a shell string or a
list of verifiers. Scripts can use any language; examples use Python.

```json
"verifier": ["python3", "scripts/verify_deps.py", "--workspace"]
```

Typed script blocks are called once per block:

```text
<command...> <spec.json> <category> <blockIndex>
```

Custom scripts are called once per entry:

```text
<command...> <spec.json> custom <blockIndex>
```

Verifier scripts print JSON proof lines. Their exit code says whether the script
ran cleanly. The proof carries pass and fail.
