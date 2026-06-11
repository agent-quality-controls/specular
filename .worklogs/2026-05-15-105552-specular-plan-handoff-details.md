Summary
- Updated the `specular` preliminary plan with missing handoff details for a fresh implementation agent.
- The plan now explicitly covers G3RS setup, the no-Rust-tests policy, dirty worktree lock rules, deferred CLI requirements, and deferred external verifier fact formats.

Decisions made
- Deferred `cli` as a V1 requirement category because CLI checks require command execution, and `commands` is already deferred as a requirement category.
- Required `specular lock` to fail on dirty worktrees by default so uncommitted spec/verifier changes cannot become hidden contract inputs.
- Required `specular verify` to fail when locked plan, spec, or verifier files are dirty, while allowing dirty implementation files in V1.
- Made language-specific requirement categories blocked on a stable external verifier fact format.
- Added an explicit Rust test ban for this repo.

Key files for context
- `.plans/2026-05-14-203402-specular-preliminary-plan.md`

Verification
- Plan-only change. No build command was needed.

Next steps
- Start implementation from the updated plan.
- Build only JSON/JSONC parsing, canonicalization, lock/status/verify drift checks, and built-in tree/text verification first.

