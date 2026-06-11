# Help usage patterns

## Summary

Reworked `HELP.txt` so the first spec-format explanation separates the three usage patterns: built-in categories with built-in verifiers, built-in categories that need a verifier command, and opaque custom checks. Replaced verifier examples with Python and described verifier commands as arbitrary executables.

## Decisions made

- Put the category-choice guide before the full JSON example so agents see the decision tree first.
- Kept predefined category fields strict and called out that unknown predefined fields fail lint.
- Described `custom` as opaque JSON objects interpreted only by the user's verifier.
- Removed shell verifier examples from HELP and used Python examples for both typed and custom verifier protocols.

## Key files for context

- `HELP.txt`

## Verification

- `cargo fmt --check`
- `cargo check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `fixture3 check --all`
- `specular verify .plans/2026-06-10-154943-per-item-spec-model.md.spec.json` -> `conforms: true`

## Next steps

- Reinstall `specular` after commit so the globally installed help text includes this change.
