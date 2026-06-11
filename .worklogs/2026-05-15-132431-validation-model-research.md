Summary
- Added a research memo explaining the validation models relevant to `driftless`.
- Resolved the user's inline comments in the main plan with plain-language decisions and examples.

Decisions made
- Chose a small contract-checker model for V1 instead of a generic OPA, CUE, TLA+, Alloy, or JSON Schema-based system.
- Kept requirements traceability only where it directly helps: stable requirement IDs, requirement coverage, checker ownership, and evidence mapping.
- Rejected mandatory prose-line references and per-requirement verification-method fields for V1.
- Required non-empty unsupported categories, orphan requirements, and orphan checker outputs to fail.

Key files for context
- `.plans/2026-05-14-203402-driftless-preliminary-plan.md`
- `.plans/2026-05-15-132431-driftless-validation-model-research.md`

Verification
- Read the user's plan comments from the working-tree diff.
- Researched primary documentation for NASA traceability, Design by Contract, OPA, CUE, JSON Schema, Alloy, AWS formal methods, and RFC 8785.

Next steps
- Review the memo and then update the V1 implementation plan to include the exact evidence JSON shape and lock structure.
