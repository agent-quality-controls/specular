# Org Specular Rename

## Summary

Finished the organization-facing Specular rename: the GitHub repository was renamed to `agent-quality-controls/specular`, this checkout's `origin` now points there, local AQC checkouts were scanned, and stale references in neighboring repos were updated.

## Decisions made

- Left Fixture3 as Fixture3; it is a separate product and command.
- Updated the GitHub repository description to "Deterministic spec-driven development CLI".
- Kept `.git` internals out of text edits and updated the remote with `git remote set-url`.
- Left the unrelated `resume` change unstaged.

## Key files for context

- `.plans/2026-06-11-155154-org-specular-rename.md`
- `fixture3/.plans/2026-05-14-120328-specular-machine-checkable-specs.md`
- `aqc-shared/README.md`
- `guardrail3/.plans/g3v2-architecture/2026-05-21-195830-repo-workspace-plugin-generation-model.md`

## Verification

- Local stale-name scan across AQC checkouts.
- Tracked filename scan for previous project names.
- GitHub repository metadata check for `agent-quality-controls/specular`.

## Next steps

- Push updated branches.
