Summary
- Updated the `specular` plans so the library boundary starts at the machine-readable spec.
- Removed prose plan, Markdown description, plan hash, and plan drift from the `specular` runtime model.

Decisions made
- The spec file is the source contract.
- The lock file is a generated receipt for the canonical spec, checker map, verifier files, and metadata.
- Verification phases are input validity, lock validity, requirement conformance, and evidence validity.
- Source format remains open: JSON, JSONC, CUE, Pkl, HCL, Dhall, or another stable machine-readable format can be considered.

Key files for context
- `.plans/2026-05-14-203402-specular-preliminary-plan.md`
- `.plans/2026-05-15-132431-specular-validation-model-research.md`

Verification
- Grepped the plans for stale plan-hash and prose-plan pipeline references.
- Confirmed remaining prose mentions only describe artifacts outside the library boundary.

Next steps
- Research the source format decision separately from the internal typed model and lock format.
- Define the exact lock JSON shape and evidence JSON shape.
