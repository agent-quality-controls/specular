Summary
- Added scratch examples comparing the same `driftless` contract in CUE, Pkl, and JSON.
- The scratch file records first-pass observations before deeper tool validation.

Decisions made
- Kept this as scratch material, not a final source-format decision.
- Compared the same semantic contract in each format to make syntax and tooling tradeoffs visible.

Key files for context
- `.plans/2026-05-15-150404-source-format-scratch.md`

Verification
- Confirmed `jq` was installed locally.
- Installed `cue 0.16.1` and `pkl 0.31.1` with Homebrew.
- Validated the CUE example with `cue export`.
- Validated the Pkl example with `pkl eval -f json`.
- Validated the JSON example with `jq`.

Next steps
- Validate each example with actual tools or documentation-backed syntax checks.
- Decide whether the source format should be JSON, CUE, Pkl, or another machine-readable format.
