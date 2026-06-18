# Coverage: builtin Cargo dependency verifier

Extraction was run locally rather than independently agent-isolated because the
current tool contract only permits sub-agents when explicitly requested.

## Goal

- Covered by `tree.required`: required files and new `src/cargo_dependencies.rs`.
- Covered by custom checks: `model`, `lint`, `verify`, `docs`, `dependencies`,
  and `fixtures`.

## Current Dependency State

- Covered by custom check `dependencies`: verifies crates.io AQC dependencies
  are used directly and Guardrail3 is not used.
- Covered by custom check `verify`: verifies the Cargo checker imports and uses
  AQC Cargo/file-engine vocabulary.

## Semantics

- Covered by custom checks `model`, `lint`, `verify`, and `fixtures`.
- Fixture coverage is required for exact package checks, renamed dependencies,
  forbidden package globs, target dependency tables, workspace dependencies,
  missing matches, and invalid TOML.

## Approach

- Covered by custom checks `model`, `lint`, `verify`, `docs`, `dependencies`,
  `fixtures`, `golden`, and `dogfood`.
- The verifier checks implementation presence and behavior fixtures; it does
  not prove every internal helper name from the prose plan.

## Key Decisions

- Covered by custom checks `dependencies`, `model`, `lint`, `verify`, and
  `docs`.
- The split between exact `forbidden` and glob `forbiddenGlobs` is checked in
  model, lint, docs, and fixtures.

## Files To Modify

- Covered by `tree.required` and the custom checks for source, docs, fixtures,
  goldens, Cargo metadata, and dogfood files.
