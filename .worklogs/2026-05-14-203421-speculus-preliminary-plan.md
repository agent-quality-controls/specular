Summary
- Created the standalone `speculus` repository and added the preliminary plan.
- The plan sets JSONC/JSON as source formats, canonical JSON as the locked contract, `enumerations` as the finite-value category, and defers `commands` as a requirement category.

Decisions made
- Put `speculus` in `/Users/tartakovsky/Projects/websmasher/speculus` instead of keeping it inside the fixture3 repository.
- Renamed `closedSets` to `enumerations` because the category includes language enum variants and data-level enumerated values such as status strings and exit codes.
- Deferred `commands` as a requirement category because verifier commands are part of the verification mechanism, not the planned implementation contract.

Key files for context
- `.plans/2026-05-14-203402-speculus-preliminary-plan.md`

Verification
- Plan-only change. No build command was needed.

Next steps
- Review the requirement taxonomy before implementing the Rust workspace.
- Decide whether to store raw JSONC `sourceHash` in locks for comment-only drift reporting.
