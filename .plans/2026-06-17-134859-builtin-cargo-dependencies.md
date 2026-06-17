# Plan: builtin Cargo dependency verifier

## Goal

Add a Cargo-aware dependency verifier to Specular:

- `requirements.dependencies[*].verifier` accepts `["builtin:cargo-dependencies"]`.
- Dependency blocks use `files`, not `manifests`.
- `required` and `exists` are exact Cargo package names.
- `forbidden` accepts exact Cargo package names and package-name globs.
- Matching uses Cargo package identity, so renamed dependencies are checked by
  `package`, not by the left-side local key.

This is a public spec-format change. The implementation should bump the spec
format version and crate version before release.

## Current Dependency State

AQC now has the required lower-level support:

- `aqc-shared` commit `184db42 Add cargo dependency package pattern bans`
- `DependencyPackagePattern { pattern: String }`
- `CargoTomlRequirements.banned_dependency_package_patterns`
- `CargoTomlRequirements.banned_workspace_dependency_package_patterns`
- `CargoTomlRequirements.banned_patch_dependency_package_patterns`
- `aqc-cargo-toml-engine` tests cover exact package identity, renamed
  dependencies, package-pattern bans, target scopes, workspace dependencies,
  patch tables, and expanded dependency subtables.

Guardrail3 also exposes the fields:

- `guardrail3` commit `8e55be9d1 Expose cargo dependency pattern bans`

Specular should depend on the AQC engine crates directly, not on Guardrail3.
Guardrail3 is a policy/adapter system; Specular only needs the Cargo file
engine vocabulary.

Release blocker:

- `aqc-file-engine-core` and `aqc-cargo-toml-engine` must be pushed and
  published before Specular can publish to crates.io with this builtin.
- A local path dependency is acceptable only while developing the change.

## Semantics

For each dependency block:

- `files` is a list of repo-relative globs selecting Cargo files to inspect.
- A matched file should normally be a `Cargo.toml`; unreadable or invalid TOML
  files produce failing evidence for affected positive items.
- `required`: every matched file must declare the exact package in at least one
  dependency-shaped Cargo table.
- `exists`: at least one matched file must declare the exact package in at least
  one dependency-shaped Cargo table.
- `forbidden`: no matched file may declare a package whose package identity
  equals the exact item or matches the glob item.

Dependency-shaped Cargo tables for the initial builtin:

- `[dependencies]`
- `[dev-dependencies]`
- `[build-dependencies]`
- `[target.'cfg(...)'.dependencies]`
- `[target.'cfg(...)'.dev-dependencies]`
- `[target.'cfg(...)'.build-dependencies]`
- `[workspace.dependencies]`

Do not include `[patch.<registry>]` in Specular's generic dependency category
yet. A patch entry is a dependency override, not a dependency declaration. AQC
supports patch bans, but Specular needs an explicit target field before it can
expose that without surprising users.

## Approach

1. Publish or locally stage AQC engine crates.
   - Push `aqc-shared` commit `184db42`.
   - Remove `publish = false` from `aqc-file-engine-core` and
     `aqc-cargo-toml-engine` if publishing is part of this run.
   - Publish compatible versions before the final Specular release.
   - During local implementation, use path dependencies only if crates.io
     versions are not available yet.

2. Migrate the dependency block target field.
   - In `src/model.rs`, rename `DependencyRequirement.manifests` to `files`.
   - In `src/lint.rs`, validate `dependencies[*].files` as globs.
   - In `src/verify.rs`, use `files` for script protocol block targets.
   - Update `HELP.txt`, `README.md`, fixtures, goldens, and dogfood specs.
   - Bump the spec format version because `manifests` removal is breaking.

3. Add the builtin registry entry.
   - In `src/lint.rs`, map `builtin:cargo-dependencies` to
     `Category::Dependencies`.
   - In `src/verify.rs`, dispatch dependency blocks with that verifier to a new
     Cargo dependency checker.
   - Keep custom script verifiers working for `dependencies` blocks.

4. Add a Cargo dependency checking module.
   - Add a module such as `src/cargo_dependencies.rs`.
   - Use `aqc-cargo-toml-engine` and `aqc-file-engine-core` directly.
   - Use `toml_edit` only to enumerate dependency-shaped tables in a matched
     Cargo file. Let AQC perform package identity checks and pattern checks.
   - Build AQC requirements with:
     - `DependencyRequirement { file_key: None, value.package = Some(name) }`
       for exact required, exists, and forbidden items.
     - `DependencyPackagePattern { pattern }` inside
       `PatternBanRequirements` for glob forbidden items.
   - For positive checks, evaluate all dependency-shaped tables in a file and
     pass when any table satisfies the package requirement.
   - For forbidden checks, evaluate all dependency-shaped tables in matched
     files and fail when any exact or pattern match is found.

5. Preserve Specular evidence shape.
   - Emit one evidence item per declared spec item.
   - Use `source: "builtin"` and verifier `builtin:cargo-dependencies`.
   - Use the dependency block's `files` array as the evidence target.
   - Include matched file paths and Cargo keys in `message` or `observed` when
     an item fails.
   - Treat AQC requirement conflicts as failing evidence, not as silent pass.

6. Lint rules for package items.
   - `required` and `exists` must be non-empty, trimmed, and non-glob.
   - `forbidden` must be non-empty and trimmed.
   - If a `forbidden` item contains glob metacharacters, compile it with
     `globset` during lint.
   - Keep exact required-vs-exact forbidden contradictions as lint errors.
   - Do not try to lint every pattern-vs-required overlap; AQC detects that
     when the builtin verifier builds requirements.

7. Behavior coverage.
   - Add lint fixtures for:
     - `builtin:cargo-dependencies` accepted on dependencies blocks.
     - builtin category mismatch rejected.
     - `files` accepted and `manifests` rejected.
     - `required` and `exists` reject globs.
     - `forbidden` accepts a valid glob and rejects an invalid glob.
   - Add verify fixtures for:
     - exact required package passes.
     - exact required package fails when absent.
     - renamed dependency satisfies exact package identity.
     - exact forbidden catches renamed dependency.
     - forbidden glob catches plain dependency key.
     - forbidden glob catches renamed dependency package.
     - forbidden glob catches expanded dependency subtable.
     - required checks every matched file.
     - exists checks at least one matched file.
     - no matched files fails positive items.

8. Documentation.
   - Update `HELP.txt` first because agents rely on it.
   - Update `README.md` with the short human-facing install/use section and a
     Cargo builtin example.
   - Document:
     - exact package names for `required` and `exists`
     - exact or glob package names for `forbidden`
     - renamed dependency behavior
     - Python custom verifier examples remain valid for non-builtin categories

9. Dogfood contract.
   - Create a Specular spec for this plan after the prose plan is accepted.
   - The spec should check:
     - `builtin:cargo-dependencies` appears in help/docs and builtin registry.
     - `manifests` is gone from user-facing current docs and model code.
     - `files` is present for dependency blocks.
     - AQC dependency crates are present in `Cargo.toml`.
     - behavior fixture files for exact, renamed, glob, subtable, required, and
       exists cases exist.
   - Use a custom Python verifier where text checks cannot prove fixture
     behavior.

10. Verification gates.
    - `cargo fmt --check`
    - `cargo clippy --all-targets --all-features -- -D warnings`
    - `cargo test`
    - behavior fixture replay
    - `specular lint <plan>.spec.json`
    - `specular verify <plan>.spec.json`
    - existing dogfood specs
    - `slopless README.md`
    - `cargo package --no-verify --allow-dirty`

## Key Decisions

- Use AQC directly instead of Guardrail3 because Specular needs file-engine
  facts, not Guardrail policy machinery.
- Keep Specular dependency semantics package-oriented. The local Cargo key is
  not part of the public item unless a later format adds a structured item.
- Use `files` rather than `manifests` because the selected inputs are files, and
  Cargo is only one verifier for the dependency category.
- Keep patch-table checks out of the initial Specular builtin because the
  current dependency block has no way to say "check patches" explicitly.
- Bump the spec format version because removing `manifests` is a breaking
  schema change.

## Files To Modify

- `Cargo.toml`
- `Cargo.lock`
- `src/model.rs`
- `src/lint.rs`
- `src/verify.rs`
- new `src/cargo_dependencies.rs` or equivalent
- `src/evidence.rs` only if observed data needs a richer shape
- `HELP.txt`
- `README.md`
- behavior fixtures and goldens under `behavior/`
- current dogfood plans/specs that still use `manifests`
- release/version files when the implementation is ready to publish
