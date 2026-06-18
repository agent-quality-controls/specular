# Plan: builtin Cargo dependency verifier

## Goal

Add a Cargo-aware dependency verifier to Specular:

- `requirements.dependencies[*].verifier` accepts `["builtin:cargo-dependencies"]`.
- Dependency blocks use `files`, not `manifests`.
- `required` and `exists` are exact Cargo package names.
- `forbidden` accepts exact Cargo package names.
- `forbiddenGlobs` accepts Cargo package-name globs.
- Matching uses Cargo package identity, so renamed dependencies are checked by
  `package`, not by the left-side local key.

This is a public spec-format change. The implementation should bump the spec
format version and crate version before release.

## Current Dependency State

AQC now has the required lower-level support:

- `aqc-shared` commit `4f48e9a Rename forbidden glob requirements`
- `DependencyPackageGlob { glob: String }`
- `CargoTomlRequirements.forbidden_dependency_package_globs`
- `CargoTomlRequirements.forbidden_workspace_dependency_package_globs`
- `CargoTomlRequirements.forbidden_patch_dependency_package_globs`
- `aqc-cargo-toml-engine` tests cover exact package identity, renamed
  dependencies, forbidden package globs, target scopes, workspace dependencies,
  patch tables, and expanded dependency subtables.

Specular should depend on the Cargo file engine crates directly, not on
Guardrail3. Guardrail3 is a policy/adapter system; Specular needs the
Cargo.toml file-engine vocabulary and reconcile behavior.

Implementation dependency rule:

- Use crates.io versions of `aqc-file-engine-core` and
  `aqc-cargo-toml-engine` once they are published.
- Use local path dependencies only for a short local integration pass before
  those crates are published.

## Semantics

For each dependency block:

- `files` is a list of repo-relative globs selecting Cargo files to inspect.
- A matched file should normally be a `Cargo.toml`.
- An unreadable or invalid TOML matched file fails every item in the block
  because Specular cannot prove presence or absence from that file.
- `required`: every matched file must declare the exact package in at least one
  dependency-shaped Cargo table.
- `exists`: at least one matched file must declare the exact package in at least
  one dependency-shaped Cargo table.
- `forbidden`: no matched file may declare the exact package identity.
- `forbiddenGlobs`: no matched file may declare a package identity matching the
  glob.

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

1. Add Cargo file engine dependencies.
   - Add crates.io dependencies:
     - `aqc-file-engine-core`
     - `aqc-cargo-toml-engine`
     - `toml_edit` only if Specular needs direct table discovery beyond what
       the file engine exposes.
   - Import the file-engine API directly:
     ```rust
     use aqc_cargo_toml_engine::{
         CargoTomlEngine, CargoTomlRequirements, DependencyKind,
         DependencyPackageGlob, DependencyRequirement as CargoDependencyRequirement,
         DependencyScope, DependencySpec,
     };
     use aqc_file_engine_core::{
         Engine, EngineRequirement, Finding, ForbiddenGlobRequirements,
         ItemRequirements, Provenance,
     };
     ```
   - Do not depend on `g3rs-cargo-adapter`.

2. Migrate the dependency block target field.
   - In `src/model.rs`, rename `DependencyRequirement.manifests` to `files`.
   - In `src/model.rs`, add `DependencyRequirement.forbiddenGlobs` with
     `#[serde(default)]`.
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
     Cargo file. Let AQC perform package identity checks and forbidden-glob
     checks.
   - Build AQC requirements with:
     - `DependencyRequirement { file_key: None, value.package = Some(name) }`
       for exact required, exists, and forbidden items.
     - `DependencyPackageGlob { glob }` inside `ForbiddenGlobRequirements` for
       `forbiddenGlobs` items.
   - Use the Cargo file engine as the package-identity oracle:
     - For an exact positive item, run a check-only package requirement against
       each dependency-shaped table scope in a file. The file passes when any
       scope returns no finding for that item.
     - For exact forbidden items, run `ItemRequirements.banned` across every
       discovered dependency-shaped table scope and fail if any finding is
       reported.
     - For `forbiddenGlobs` items, run `ForbiddenGlobRequirements.globs` across
       every discovered dependency-shaped table scope and fail if any finding
       is reported.

5. Structure `src/cargo_dependencies.rs`.
   - Public entry point:
     - `check_cargo_dependencies(block, tree, root, verifier, out)`.
   - Internal data:
     - `DependencyTableTarget::Scope(DependencyScope)`
     - `DependencyTableTarget::Workspace`
   - Internal helpers:
     - `matched_cargo_files(block.files, tree) -> Vec<FileTreeEntry>`
     - `read_cargo_file(path) -> Result<Vec<u8>, VerifyError>`
     - `discover_dependency_tables(bytes) -> Vec<DependencyTableTarget>`
     - `exact_requirement(package, message) -> CargoDependencyRequirement`
     - `exact_items(required_or_banned) -> ItemRequirements<CargoDependencyRequirement>`
     - `glob_items(globs) -> ForbiddenGlobRequirements<DependencyPackageGlob>`
     - `run_cargo_engine(bytes, CargoTomlRequirements) -> Vec<Finding>`
     - `file_has_package(bytes, tables, package) -> Result<bool, VerifyError>`
     - `file_exact_forbidden_hits(bytes, tables, package) -> Result<Vec<String>, VerifyError>`
     - `file_glob_forbidden_hits(bytes, tables, glob) -> Result<Vec<String>, VerifyError>`
   - Keep all Cargo-specific helper code in this module. `src/verify.rs`
     should only dispatch to it.

6. Preserve Specular evidence shape.
   - Emit one evidence item per declared spec item.
   - Use `source: "builtin"` and verifier `builtin:cargo-dependencies`.
   - Use the dependency block's `files` array as the evidence target.
   - For `required`, fail with the list of matched files where no dependency
     table declares the package.
   - For `exists`, fail when no matched file declares the package.
   - For `forbidden`, fail with file path plus Cargo table/key hits when AQC
     reports exact package matches.
   - For `forbiddenGlobs`, fail with file path plus Cargo table/key hits when
     AQC reports glob package matches.
   - Treat AQC requirement conflicts as failing evidence, not as silent pass.

7. Lint rules for package items.
   - `required`, `exists`, and `forbidden` must be non-empty, trimmed, and
     non-glob.
   - `forbiddenGlobs` items must be non-empty, trimmed, contain a glob
     metacharacter, and compile with `globset` during lint.
   - Keep exact required-vs-exact forbidden contradictions as lint errors.
   - Reject exact `required` or `exists` items that match a `forbiddenGlobs`
     item as contradictions.

8. Behavior coverage.
   - Add lint fixtures for:
     - `builtin:cargo-dependencies` accepted on dependencies blocks.
     - builtin category mismatch rejected.
     - `files` accepted and `manifests` rejected.
     - `required` and `exists` reject globs.
     - `forbidden` rejects glob metacharacters.
     - `forbiddenGlobs` accepts a valid glob.
     - `forbiddenGlobs` rejects non-glob exact strings and invalid globs.
     - exact `required` or `exists` matching `forbiddenGlobs` is rejected.
   - Add verify fixtures for:
     - exact required package passes.
     - exact required package fails when absent.
     - renamed dependency satisfies exact package identity.
     - exact forbidden catches renamed dependency.
     - `forbiddenGlobs` catches plain dependency key.
     - `forbiddenGlobs` catches renamed dependency package.
     - `forbiddenGlobs` catches expanded dependency subtable.
     - required checks every matched file.
     - exists checks at least one matched file.
     - no matched files fails positive items.
     - invalid matched Cargo file fails every item in the block.

9. Documentation.
   - Update `HELP.txt` first because agents rely on it.
   - Update `README.md` with the short human-facing install/use section and a
     Cargo builtin example.
   - Document:
     - exact package names for `required` and `exists`
     - exact package names for `forbidden`
     - glob package names for `forbiddenGlobs`
     - renamed dependency behavior
     - Python custom verifier examples remain valid for non-builtin categories

10. Dogfood contract.
   - Create a Specular spec for this plan after the prose plan is accepted.
   - The spec should check:
     - `builtin:cargo-dependencies` appears in help/docs and builtin registry.
     - `manifests` is gone from user-facing current docs and model code.
     - `files` is present for dependency blocks.
     - `forbiddenGlobs` appears in docs, model code, lint coverage, and verify
       fixtures.
     - AQC dependency crates are present in `Cargo.toml`.
     - behavior fixture files for exact, renamed, glob, subtable, required, and
       exists cases exist.
   - Use a custom Python verifier where text checks cannot prove fixture
     behavior.

11. Verification gates.
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

- Use the Cargo file engine directly instead of Guardrail3 because Specular
  needs Cargo.toml file-engine facts, not Guardrail policy machinery.
- Keep Specular dependency semantics package-oriented. The local Cargo key is
  not part of the public item unless a later format adds a structured item.
- Split exact bans from glob bans so Specular does not infer verifier semantics
  from punctuation inside a package item.
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
