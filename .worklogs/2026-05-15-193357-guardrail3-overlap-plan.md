Summary
- Updated the active driftless plan to make guardrail3/shared AQC fact reuse a blocker before V1 checkers.
- Removed `normalize` from the public V1 command surface and kept canonicalization as internal lock/hash behavior.

Decisions made
- Use `garde` for struct-level validation because guardrail3 already standardizes on it and it covers nested/contextual validation.
- Do not add `validator` unless `garde` has a concrete blocker.
- Do not implement repository crawling, ignore handling, file reading semantics, config parsing, or source AST parsing inside driftless checkers.
- Treat `tree` and `text` as thin checkers over shared repository facts.

Key files for context
- `.plans/2026-05-14-203402-driftless-preliminary-plan.md`
- `/Users/tartakovsky/Projects/agent-quality-controls/guardrail3/packages/shared/g3-workspace-crawl`
- `/Users/tartakovsky/Projects/agent-quality-controls/guardrail3/packages/rs/code/g3rs-code-ingestion`
- `/Users/tartakovsky/Projects/agent-quality-controls/guardrail3/packages/rs/test/g3rs-test-ingestion`

Verification
- Grepped the plan for `normalize`, `garde`, `validator`, shared fact, crawler, and parser-stack references.
- Reviewed the final diff before commit.

Next steps
- Decide whether driftless depends directly on existing guardrail3 shared crates or first extracts them into AQC-neutral shared crates.
- Only after that decision, implement `tree` and `text` checkers over the selected shared fact boundary.
