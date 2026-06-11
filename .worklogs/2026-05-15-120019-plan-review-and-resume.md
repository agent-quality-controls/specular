Summary
- Reviewed the `specular` preliminary plan and corrected it before implementation starts.
- Renamed the root session helper from `code-sessions` to `resume`.

Decisions made
- Kept the V1 plan because it correctly separates structural contract checks from behavior fixture checks.
- Removed broad JSON Schema validation from V1 wording because it conflicted with the deferred external verifier fact-format boundary.
- Recorded local dependency verification for `jsonc-parser` based on `cargo info` before implementation.
- Kept command execution and CLI surface checks deferred.

Key files for context
- `.plans/2026-05-14-203402-specular-preliminary-plan.md`
- `resume`

Verification
- Confirmed the worktree was clean before edits.
- Reviewed the full plan before changing it.
- Verified `jsonc-parser` crate metadata with Cargo.

Next steps
- Initialize the Rust workspace and G3RS policy from the reviewed plan.
- Implement only JSON/JSONC parsing, canonicalization, lock/status/verify drift checks, and built-in `tree` / `text` verification first.
