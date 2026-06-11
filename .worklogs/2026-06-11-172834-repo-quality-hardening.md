Summary:
- Made Specular public and added a Specular-checked quality-hardening plan, spec, coverage map, and custom verifier.
- Added community health files, CI, CodeQL, Scorecard, Dependabot, release workflow, Cargo metadata, README badges, and crates.io-ready package metadata.
- Published the three AQC shared crates that Specular depends on: `aqc-filetree`, `aqc-fs-utils`, and `aqc-git-helpers` v0.1.0.

Decisions made:
- Made `agent-quality-controls/specular` and `agent-quality-controls/aqc-shared` public.
- Removed local-only `../aqc-shared` dependencies from Specular and resolved AQC crates from crates.io.
- Used crates.io Trusted Publishing in the release workflow, with the first manual publish handled outside that workflow.
- Added a custom Python verifier for GitHub repo state, security settings, Cargo metadata, workflow action pinning, and release workflow shape.
- Did not register OpenSSF Best Practices because the user explicitly excluded it.

Key files for context:
- `.plans/2026-06-11-171235-repo-quality-hardening.md`
- `.plans/2026-06-11-171235-repo-quality-hardening.md.spec.json`
- `.plans/2026-06-11-171235-repo-quality-hardening.md.spec.coverage.md`
- `scripts/verify-repo-quality.py`
- `Cargo.toml`
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`

Verification:
- `specular lint .plans/2026-06-11-171235-repo-quality-hardening.md.spec.json`
- `specular verify .plans/2026-06-11-171235-repo-quality-hardening.md.spec.json`
- `cargo fmt --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test`
- `cargo package --no-verify --allow-dirty`
- `fixture3 check --all`
- `slopless README.md`

Next steps:
- Push this commit.
- Manually publish `specular` v0.2.0 to crates.io.
- Configure crates.io Trusted Publishing for `specular`, `aqc-filetree`, `aqc-fs-utils`, and `aqc-git-helpers` against their release workflows.
