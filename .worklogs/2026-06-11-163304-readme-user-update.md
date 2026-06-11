Summary:
- Committed the user's README revision with minimal cleanup.
- Fixed a typo, removed trailing whitespace, and simplified a few dense phrases so `slopless README.md` passes.
- Preserved the shorter README structure and the user's cuts to the install and quick-start sections.

Decisions made:
- Kept the first line as "Specular is a CLI for enforcing spec-driven development."
- Kept the install command in the opening section as a fenced Bash block.
- Changed "categories" to "groups" in the short pattern list to reduce style-gate complexity without changing the three supported use cases.

Key files for context:
- `README.md`
- `.worklogs/2026-06-11-162527-readme-correction.md`

Next steps:
- Push this commit with the existing unpushed commits on `main`.
