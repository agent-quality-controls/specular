# Contributing to Specular

Specular is a Rust CLI for enforcing spec-driven development. The fastest useful
contribution is a precise issue with a plan, expected behavior, and the command
output that proves the current behavior is wrong.

## Development setup

Install Rust 1.85 or newer.

```bash
# Build and run tests.
cargo test

# Check formatting.
cargo fmt --check

# Run strict linting.
cargo clippy --all-targets --all-features -- -D warnings

# Run behavior fixtures.
fixture3 check --all

# Check README prose.
slopless README.md
```

## Spec changes

Changes to the spec format, verifier protocol, report shape, exit codes, or help
text must update all matching surfaces in the same pull request:

- `HELP.txt`
- `README.md`
- fixture inputs and goldens
- relevant `.plans/*.spec.json` files
- relevant coverage maps

Run `specular --help` before editing the format. The CLI help is the source of
truth for tool-interface details.

## Test policy

New behavior needs a deterministic check. Use the smallest useful layer:

- unit tests for pure Rust logic
- Fixture3 for CLI behavior and JSON output
- Specular specs for plan-vs-repo scaffolding
- custom verifier scripts for repository state that the built-ins cannot check

## Release notes

The first crates.io publish for each crate must be manual. After the crate
exists on crates.io, configure Trusted Publishing for this repository and the
`.github/workflows/release.yml` workflow.
