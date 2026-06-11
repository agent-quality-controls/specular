# Specular repo quality hardening

## Goal

Make Specular a public, installable, release-ready Rust CLI repository with the same quality posture Slopless uses where it applies: public repo metadata, community health files, CI, static analysis, supply-chain scanning, release workflow, README badges, and a Specular contract that checks the scaffolding mechanically.

OpenSSF Best Practices registration is out of scope for this pass.

## Approach

1. Make public AQC repositories visible.
   - Make `agent-quality-controls/specular` public.
   - Make `agent-quality-controls/aqc-shared` public because Specular depends on AQC shared crates.
   - Enable Issues, Discussions, repo topics, secret scanning, push protection, and Dependabot security updates for Specular where GitHub exposes the setting.

2. Fix the public dependency boundary.
   - Keep Specular buildable outside the local monorepo by removing hard local-only dependency assumptions.
   - Publish the AQC shared crates first.
   - Use crates.io dependencies for Specular.

3. Add repository health files.
   - Add `LICENSE`.
   - Add `SECURITY.md`.
   - Add `.github/CONTRIBUTING.md`.
   - Add `.github/CODE_OF_CONDUCT.md`.
   - Add `.github/CODEOWNERS`.
   - Add `.github/PULL_REQUEST_TEMPLATE.md`.
   - Add issue templates for bugs and spec/verifier requests.

4. Add CI and static-analysis workflows.
   - Add `.github/workflows/ci.yml`.
   - CI runs `cargo fmt --check`, Clippy with denied warnings, `cargo test`, `cargo run -- --help`, `python3 scripts/verify-cli-version.py`, `cargo package --no-verify`, `fixture3 check --all`, and `slopless README.md`.
   - Add `.github/workflows/codeql.yml` for Rust CodeQL scanning.
   - Add `.github/workflows/scorecard.yml` for OpenSSF Scorecard SARIF upload.
   - Pin third-party actions by commit SHA.

5. Add dependency automation.
   - Add `.github/dependabot.yml` for Cargo and GitHub Actions.

6. Add release workflow.
   - Add `.github/workflows/release.yml`.
   - Verify release tag matches `Cargo.toml`.
   - Run the same Rust gates as CI before publish.
   - Use crates.io Trusted Publishing through `rust-lang/crates-io-auth-action`.
   - Document that the first crates.io publish for each crate must be manual before Trusted Publishing can be configured.

7. Update Cargo metadata.
   - Add description, license, repository, readme, rust-version, keywords, categories, and include list to `Cargo.toml`.
   - Keep `unsafe_code = "forbid"` and strict Clippy lints.

8. Update README badges and install path.
   - Add CI, CodeQL, license, Rust, and crates.io-ready badges.
   - Keep install instructions accurate for the current release state.
   - Avoid Scorecard and crates.io version badges until the workflow and crate page exist.

9. Add Specular verification for this plan.
   - Add `<plan>.spec.json`.
   - Add `<plan>.spec.coverage.md`.
   - Add a Python custom verifier for GitHub repository state, release workflow shape, action pinning, Cargo metadata, and publish-readiness checks that built-in tree/content checks cannot express.

10. Verify locally.
    - Run `specular lint <spec>`.
    - Run `specular verify <spec>`.
    - Run `cargo fmt --check`.
    - Run `cargo clippy --all-targets --all-features -- -D warnings`.
    - Run `cargo test`.
    - Run `fixture3 check --all`.
    - Run `slopless README.md`.

## Key decisions

- Make `aqc-shared` public because a public Specular repo with private local path dependencies is not installable or CI-friendly.
- Do not register OpenSSF Best Practices in this pass because the user explicitly excluded it.
- Use crates.io Trusted Publishing for the release workflow. Official crates.io docs describe Trusted Publishing as tokenless CI publishing through OIDC, but the first release of each crate still needs manual publishing before the trusted publisher can be configured.
- Do not require the two advanced secret-scanning features that GitHub left disabled after the API PATCH. Require secret scanning, push protection, and Dependabot security updates.
- Treat `cargo package --no-verify` as a CI packaging smoke test now that the AQC shared crates are published on crates.io.

## Files to modify

- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `LICENSE`
- `SECURITY.md`
- `.github/CONTRIBUTING.md`
- `.github/CODE_OF_CONDUCT.md`
- `.github/CODEOWNERS`
- `.github/PULL_REQUEST_TEMPLATE.md`
- `.github/ISSUE_TEMPLATE/bug.yml`
- `.github/ISSUE_TEMPLATE/spec-verifier-request.yml`
- `.github/ISSUE_TEMPLATE/config.yml`
- `.github/dependabot.yml`
- `.github/workflows/ci.yml`
- `.github/workflows/codeql.yml`
- `.github/workflows/scorecard.yml`
- `.github/workflows/release.yml`
- `scripts/verify-repo-quality.py`
- `scripts/verify-cli-version.py`
- `.plans/2026-06-11-171235-repo-quality-hardening.md.spec.json`
- `.plans/2026-06-11-171235-repo-quality-hardening.md.spec.coverage.md`
- `.worklogs/<timestamp>-repo-quality-hardening.md`

## Out of scope

- OpenSSF Best Practices registration.
- Manual crates.io first publish, if no local crates.io credentials are available.
- Branch protection that requires a second reviewer, because this is not useful for a single-maintainer phase.
