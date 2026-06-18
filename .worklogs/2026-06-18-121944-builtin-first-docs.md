Summary
- Updated Specular's help text and README to make predefined categories and built-in verifiers the default authoring path.
- The docs now tell agents to use custom entries and external scripts only when no predefined category or built-in verifier can express the check.

Decisions made
- Kept this as documentation and agent guidance, not a lint-rule change. Existing specs that use custom checks remain valid until we explicitly decide to enforce the policy mechanically.
- Used "predefined category" consistently in the README so it matches the spec model and the help text.
- Updated the local Codex Specular skill outside this repository with the same category-first and builtin-first workflow.

Key files for context
- `HELP.txt` - CLI help source, including the priority rule and workflow.
- `README.md` - human-facing usage instructions and agent prompt.
- `/Users/tartakovsky/.codex/skills/spec-driven-development/SKILL.md` - local Codex skill updated outside the repo commit.

Next steps
- If the policy should become enforceable instead of advisory, add lint rules that reject custom or external verifier use where a matching built-in category can judge the requirement.
