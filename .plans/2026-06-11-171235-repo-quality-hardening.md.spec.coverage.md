# Coverage Map: Specular repo quality hardening

Extraction note: this spec was not produced by three isolated sub-agent passes because the user did not explicitly authorize sub-agent work. I performed the extraction in one local pass after reading `specular --help`, the Slopless OpenSSF plan, the Slopless promotion plan, and the current Specular repository state.

## Goal

- Coverage: tree, content, custom verifier, hand review
- Requirement targets:
  - Public GitHub state through `scripts/verify-repo-quality.py`
  - Community files through `tree`
  - CI, CodeQL, Scorecard, release, Dependabot through `tree` and `content`
  - Cargo and README metadata through `content` and custom verifier

## Approach

- Coverage: tree, content, custom verifier, hand review
- Requirement targets:
  - Step 1 GitHub visibility/settings: custom checks `github-*`
  - Step 2 dependency boundary: `Cargo.toml` content plus `no-local-aqc-path-deps`
  - Step 3 health files: `tree` plus content checks for `SECURITY.md`, contributing, and PR template
  - Step 4 workflows: `tree`, workflow content checks, and `workflow-actions-pinned`
  - Step 5 Dependabot: `tree` and `.github/dependabot.yml` content
  - Step 6 release workflow: `.github/workflows/release.yml` content plus `release-trusted-publishing`
  - Step 7 Cargo metadata: `Cargo.toml` content plus `cargo-metadata-complete`
  - Step 8 README badges/install: README content checks
  - Step 9 Specular verification files: `tree`
  - Step 10 local verification: checked by command output, not fully encoded in the spec

## Key decisions

- Coverage: custom verifier, content, hand review
- Requirement targets:
  - Public `aqc-shared`: `github-aqc-shared-public`
  - OpenSSF Best Practices excluded: plan text and absence from required workflow/files
  - Trusted Publishing: release workflow content and `release-trusted-publishing`
  - Advanced secret-scanning subfeatures not required: hand review of GitHub API result
  - `cargo package --no-verify`: CI and release workflow content

## Files to modify

- Coverage: tree
- Requirement targets:
  - Every listed file is in `requirements.tree.required`

## Out of scope

- Coverage: hand review
- Reason:
  - OpenSSF registration is intentionally excluded.
  - Manual crates.io first publish cannot be proven by repository state unless credentials and registry ownership exist.
  - Branch protection with second-review requirements is an external repo setting and is intentionally deferred.
