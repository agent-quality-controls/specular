# CLI version and trusted publishing

## Summary

Added `specular --version` and `specular -v` with the same early package-version behavior Slopless uses. Added a CI verifier script that checks the CLI output against `Cargo.toml`, and configured crates.io Trusted Publishing for the `specular` crate.

## Decisions made

- Implemented version output in `src/main.rs` with `env!("CARGO_PKG_VERSION")` so the CLI reports Cargo package metadata directly.
- Added `scripts/verify-cli-version.py` rather than a shell snippet so CI has the same explicit version gate style as Slopless' `scripts/verify-cli-version.mjs`.
- Updated the existing repo-quality plan/spec/coverage files so Specular requires the version verifier script and CI step.
- Configured crates.io Trusted Publishing through the crates.io API for crate `specular`, repository `agent-quality-controls/specular`, workflow file `release.yml`, and no GitHub Actions environment.

## Key files for context

- `src/main.rs`
- `HELP.txt`
- `scripts/verify-cli-version.py`
- `.github/workflows/ci.yml`
- `.plans/2026-06-11-171235-repo-quality-hardening.md`
- `.plans/2026-06-11-171235-repo-quality-hardening.md.spec.json`
- `.plans/2026-06-11-171235-repo-quality-hardening.md.spec.coverage.md`

## Verification

- `cargo fmt --check`
- `python3 scripts/verify-cli-version.py`
- `cargo run --quiet -- --version`
- `cargo run --quiet -- -v`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `specular lint .plans/2026-06-11-171235-repo-quality-hardening.md.spec.json`
- `cargo run --quiet -- verify .plans/2026-06-11-171235-repo-quality-hardening.md.spec.json`
- `fixture3 check --all`
- `slopless README.md`
- `cargo package --no-verify --allow-dirty`
- crates.io Trusted Publishing config list for `specular`

## Next steps

- For the next Specular release, bump `Cargo.toml`, commit, tag/release on GitHub, and let `.github/workflows/release.yml` publish through Trusted Publishing.
- Do not create a GitHub Release for `v0.2.0`; that version was already published manually, so the release workflow would try to republish an existing crate version.
