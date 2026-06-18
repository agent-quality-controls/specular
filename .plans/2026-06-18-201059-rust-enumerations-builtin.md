# Plan: Rust enum syntax facts and builtin enumeration verifier

## Goal

Add a Rust enum verifier to Specular without duplicating Guardrail3 source
analysis or creating another policy-shaped parser package.

End state:

- `aqc-shared` provides a small Rust syntax fact crate for one source file.
- The first shared fact surface is enum declarations and variants only.
- Specular adds `builtin:rust-enumerations` for file-scoped enum checks.
- The verifier uses predefined `enumerations` blocks and does not use custom
  checks for Rust enum variant sets.

## Current Evidence

Guardrail3 already uses `syn` in several places, but the parsing is mixed into
rule-specific packages:

- `g3rs-apparch-ingestion` walks crate modules and public items.
- `g3rs-arch-ingestion` extracts facade exports from `lib.rs` and `mod.rs`.
- `g3rs-code-source-checks` visits Rust types and enums for code rules.
- `g3rs-garde-ingestion` visits enums for Garde validation rules.

Those packages prove `syn` is the local Rust parser choice, but they are not
safe dependencies for Specular. They expose Guardrail3 concepts such as rule
inputs, policy-specific facts, check results, architectural layers, and source
families.

`aqc-shared` currently has file, tree, git, and TOML engines. It does not have
a Rust source syntax fact crate.

## Key Boundary

Do not combine Rust source parsing with crate walking.

The first shared crate is file-local syntax only:

- Input: one Rust source string and an optional file label.
- Output: syntax facts or a parse error.
- No filesystem access.
- No Cargo access.
- No workspace or crate model.
- No module resolution.
- No public API claims.
- No Guardrail3 or Specular types.
- No findings, severity, waivers, rule ids, or messages.
- No regex fallback.

Crate/module/public-surface work belongs in a later crate or layer if exports
need it. It must consume the syntax fact crate instead of reparsing source text
itself.

## Proposed AQC Crate

Create `aqc-rust-syntax` in `aqc-shared`.

Initial public API:

```rust
pub fn parse_rust_syntax(source: &str) -> Result<RustFileSyntax, RustSyntaxError>;

pub struct RustFileSyntax {
    pub enums: Vec<RustEnumDecl>,
}

pub struct RustEnumDecl {
    pub name: String,
    pub module_path: Vec<String>,
    pub variants: Vec<String>,
    pub visibility: RustVisibility,
    pub line: usize,
}

pub enum RustVisibility {
    Private,
    Public,
    Crate,
    Restricted(String),
}

pub struct RustSyntaxError {
    pub message: String,
}
```

The exact names can change during implementation, but the roles cannot:

- `RustFileSyntax` is a fact container, not a rule result.
- `RustEnumDecl` preserves duplicate enum names as separate entries.
- `module_path` is only for inline modules present in the same parsed file. It
  is not resolved through `mod foo;` files.
- `visibility` is raw syntax visibility. It does not imply exported API status.

## AQC Implementation

1. Add `packages/file-types/rust/aqc-rust-syntax` or
   `packages/source/rust/aqc-rust-syntax` after checking the existing
   `aqc-shared` package layout.
2. Depend on `syn = { version = "2", features = ["full", "parsing", "visit"] }`
   and `proc-macro2 = { version = "1", features = ["span-locations"] }`.
3. Parse with the existing local convention:
   `syn::parse_file(source.strip_prefix('\u{feff}').unwrap_or(source))`.
4. Visit:
   - top-level enums
   - enums inside inline `mod` blocks
5. Do not follow `mod foo;` declarations.
6. Do not interpret `cfg`, feature flags, derives, serde attributes, or Garde
   attributes.
7. Preserve variant names exactly as Rust identifiers.
8. Return parse errors instead of fallback text scanning.
9. Add fixtures for:
   - unit variants
   - tuple variants
   - struct variants
   - public, private, `pub(crate)`, and restricted visibility
   - inline nested modules
   - duplicate enum names in one file
   - byte-order mark prefix
   - malformed Rust
   - attributes on enums and variants, proving they do not affect variant facts

## Specular Format Change

Add `files` to `EnumerationRequirement`:

```json
{
  "verifier": ["builtin:rust-enumerations"],
  "files": ["src/model.rs"],
  "name": "Category",
  "values": ["Tree", "Content", "Dependencies", "Exports", "Enumerations", "Custom"]
}
```

Semantics:

- `files` is required for `builtin:rust-enumerations`.
- `files` is a list of repo-relative globs selecting Rust files to parse.
- `name` is the enum name or inline-module-qualified name, such as
  `wire::Status`.
- `values` is the exact variant set. Drift in either direction fails.
- If zero matched files parse to a matching enum name, the block fails.
- If more than one matched enum has the same `name` and different variants, the
  block fails with ambiguity instead of choosing one.
- If all matching enum declarations have the same variant set, the verifier may
  compare that set once and emit one evidence item per value plus one drift
  evidence item for extra variants if needed.
- Unreadable or unparseable matched files fail the block because Specular cannot
  prove the enum set.

This is a public spec-format change and should bump the spec format version.

## Specular Implementation

1. Add the crates.io `aqc-rust-syntax` dependency after it is published.
2. Update `src/model.rs`:
   - add `files: Vec<String>` to `EnumerationRequirement`
   - keep `name` and `values`
3. Update `src/lint.rs`:
   - accept `builtin:rust-enumerations` only on `Category::Enumerations`
   - validate `enumerations[*].files` globs when the builtin is selected
   - require non-empty `files` for the builtin
   - keep `values` duplicate checks
4. Add `src/rust_enumerations.rs`:
   - match files with `aqc-filetree`
   - read text with `aqc-fs-utils`
   - parse with `aqc-rust-syntax`
   - compare exact variant sets
   - emit Specular evidence, not AQC findings
5. Update `src/verify.rs` to dispatch `builtin:rust-enumerations`.
6. Update `HELP.txt` first, then `README.md`.
7. Add lint fixtures:
   - builtin accepted on enumerations
   - builtin rejected on other categories
   - `files` required for builtin
   - invalid `files` glob rejected
   - duplicate enum values rejected
8. Add verify fixtures:
   - clean enum passes
   - missing enum fails
   - missing variant fails
   - extra variant fails
   - duplicate enum name with same variants passes
   - duplicate enum name with different variants fails as ambiguous
   - nested inline module enum can be addressed by `module::Enum`
   - malformed matched Rust file fails

## Guardrail3 Reuse Requirement

Do not accept `aqc-rust-syntax` as complete only because Specular uses it.

Before treating it as the shared boundary, migrate one Guardrail3 enum-related
consumer to use it for enum discovery. Good candidates:

- `g3rs-code-source-checks` large type inventory enum discovery.
- `g3rs-garde-ingestion` enum target discovery.

That migration proves the crate is a shared syntax fact boundary rather than a
Specular-specific adapter.

## Explicit Non-Goals

- Rust export verifier.
- Crate walking.
- Public API resolution.
- `pub use` resolution.
- `mod foo;` resolution.
- Rustdoc JSON integration.
- `cargo-public-api` integration.
- TypeScript support.
- Regex scanning.
- Guardrail3 rule extraction.
- Any Specular custom verifier changes.

## Key Decisions

- Build enum checking before Rust export checking because enum checks can be
  file-scoped and syntax-only.
- Add `files` to enumeration blocks because file-scoped syntax checks need an
  explicit file target. Relying on only `name` would be ambiguous across files.
- Keep syntax parsing in AQC and Specular verdict mapping in Specular. AQC
  emits facts; Specular emits evidence.
- Do not reuse Guardrail3 packages directly. Extract the syntax fact logic and
  leave policy logic behind.

## Files to modify later

In `aqc-shared`:

- workspace manifest
- new `aqc-rust-syntax` package
- package fixtures/tests
- one Guardrail3 consumer after the AQC crate is available

In `specular`:

- `Cargo.toml`
- `Cargo.lock`
- `src/model.rs`
- `src/lint.rs`
- `src/verify.rs`
- new `src/rust_enumerations.rs`
- `HELP.txt`
- `README.md`
- `behavior/fixtures/**`
- `behavior/golden/**`
- `.plans/*.spec.json` if old specs need the new version

## Work Order

1. Plan and implement `aqc-rust-syntax` in `aqc-shared`.
2. Publish `aqc-rust-syntax`.
3. Migrate one Guardrail3 enum consumer to prove reuse.
4. Create a Specular dogfood spec for `builtin:rust-enumerations`.
5. Implement the Specular builtin.
6. Bump Specular spec format and crate version.
7. Release and install Specular.
