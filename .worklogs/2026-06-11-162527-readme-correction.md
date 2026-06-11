Summary:
- Corrected the README opening so the first line states that Specular is a CLI for enforcing spec-driven development.
- Reworked the quick start into normal README prose and added comments to shell command blocks.
- Ran `slopless README.md` after the edits and it passed.

Decisions made:
- Kept the first section to five lines while making the product claim explicit.
- Used commented Bash blocks for install and verify commands because that is where comments help the reader.
- Kept verifier examples language-neutral in command shape and kept Python named as the normal script choice.

Key files for context:
- `README.md`
- `.worklogs/2026-06-11-162107-readme.md`

Next steps:
- Push the branch when ready.
