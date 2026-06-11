# Goal

Rename the product from its old name to Specular CLI while keeping the repository folder path unchanged.

# Approach

- Rename the Rust crate and binary in `Cargo.toml`, then update Rust imports and runtime binary paths.
- Update user-facing help, docs, plans, worklogs, fixture metadata, golden outputs, and script comments.
- Preserve internal JSON spec terminology where it means "spec file" rather than the product name.
- Re-run `cargo fmt`, `cargo check`, `cargo clippy`, `fixture3 check --all`, and the repository self-verification contracts.

# Key decisions

- CLI command and package name: `specular`.
- Display name in prose: `Specular CLI`.
- Report field: rename the old version field to `specular_version` and update goldens.
- Folder name `/specular` is left unchanged per request.

# Files to modify

- `Cargo.toml`, `Cargo.lock`
- `src/main.rs`, `src/lib.rs`, `src/model.rs`, `src/evidence.rs`, `src/verify.rs`
- `HELP.txt`
- `scripts/behavior/replay.sh`
- `fixture3.yaml`, `behavior/**`
- `.plans/**`, `.worklogs/**`
