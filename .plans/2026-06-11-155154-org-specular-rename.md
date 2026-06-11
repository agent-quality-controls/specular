# Org Specular Rename

## Goal

Finish the Specular rename across the Agent Quality Controls GitHub organization and local checkouts.

## Approach

- Rename the GitHub repository from its previous project name to `agent-quality-controls/specular`.
- Update the local `specular` origin URL to `https://github.com/agent-quality-controls/specular.git`.
- Search local AQC checkouts for previous project names.
- Rename old project-name references in `fixture3`, `aqc-shared`, and `guardrail3` to Specular / `specular`.
- Keep `fixture3` references when they refer to the fixture testing product or command, not this project.
- Add a worklog in each repo that receives committed file changes.
- Commit each changed repo and push the changed branches.

## Key Decisions

- The `fixture3` repository and command stay named `fixture3`; only stale references to this project are renamed.
- The `specular` project folder is already renamed locally, so no filesystem directory rename is needed.
- `.git` internals are not edited by text replacement. Git remotes are updated through `git remote set-url`.
- The existing `resume` change in this repo stays unstaged.

## Files To Modify

- `specular`: remote URL, this plan, and a worklog.
- `fixture3`: old project plan/worklog files and filenames.
- `aqc-shared`: docs/comments/worklogs mentioning the previous project name.
- `guardrail3`: docs/plans/worklogs mentioning the previous project name.
