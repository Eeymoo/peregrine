---
description: Implement tasks from an OpenSpec change (Experimental)
---

Implement tasks from an OpenSpec change.

**Store selection:** If the user names a store (a store is a standalone OpenSpec repo registered on this machine) or the work lives in one, run `openspec store list --json` to discover registered store ids, then pass `--store <id>` on the commands that read or write specs and changes (`new change`, `status`, `instructions`, `list`, `show`, `validate`, `archive`, `doctor`, `context`). Other commands do not take the flag. Hints printed by commands already carry the flag; keep it on follow-ups. Without a store, commands act on the nearest local `openspec/` root.

**Input**: Optionally specify a change name (e.g., `/opsx:apply add-auth`). If omitted, check if it can be inferred from conversation context. If vague or ambiguous you MUST prompt for available changes.

**Steps**

1. **Select the change**

   If a name is provided, use it. Otherwise:
   - Infer from conversation context if the user mentioned a change
   - Auto-select if only one active change exists
   - If ambiguous, run `openspec list --json` to get available changes and use the **AskUserQuestion tool** to let the user select

   Always announce: "Using change: <name>" and how to override (e.g., `/opsx:apply <other>`).

   Read the change's `.openspec.yaml` to get the `issue:` and `branch:` recorded by `/opsx:propose`. If present, the intended branch is `<branch>` (typically `feature/<name>`).

2. **Create a dedicated working branch based on `main` (MANDATORY, before any code change)**

   The rule: **the change's branch must be a fresh branch off `main`** (so it can be pushed, opened as a PR, and merged independently). Never implement a change directly on `main`/`master`, and never stack a change on top of another feature branch unless forced to.

   Use the change's `.openspec.yaml` `branch:` value as the target branch name (typically `feature/<name>`); fall back to `feature/<name>` if absent.

   **If the target branch already exists and is currently checked out**: verify it is the intended branch and continue on it (re-invocation of `/opsx:apply`). Do not recreate it. Skip the rest of this step.
   **If the target branch already exists but is NOT checked out**: `git checkout feature/<name>` to switch to it (warn the user before switching if the working tree is dirty). Skip the rest of this step.

   Otherwise, get to `main` and branch off it:

   a. Determine the current branch and working-tree state:
      ```bash
      git branch --show-current
      git status --porcelain
      ```
   b. **If already on `main` (or `master`)**: proceed straight to step (d).
   c. **If NOT on `main`/`master`** (on a feature/dev branch): **try to switch to `main` automatically** so the new branch is based on `main`:
      - First check `git status --porcelain`. If the output is non-empty (uncommitted changes), do NOT discard anything — STOP and use the **AskUserQuestion tool** to ask the user how to handle the dirty tree (commit / stash / abort). Only proceed once the tree is clean.
      - With a clean tree, switch and update:
        ```bash
        git checkout main 2>/dev/null || git checkout master
        git pull --ff-only
        ```
        If `git pull --ff-only` fails (local `main` has diverged), do NOT force — STOP and ask the user whether to reset `main` to `origin/main` or abort.
   d. **Create the branch off `main`** (now current):
      ```bash
      git checkout -b feature/<name>
      ```
   e. Announce: "Created branch `feature/<name>` based on `main`" before moving on.

   **Why prefer `main`:** a branch based on `main` can be pushed (`git push -u origin HEAD`), opened as a standalone PR (`gh pr create --base main --head feature/<name>`), and merged cleanly via the GitHub merge button. A branch stacked on another feature branch cannot be merged independently until its parent merges, which breaks the `/opsx:archive` PR-merge gate. Only fall back to basing on the current (non-`main`) branch if switching to `main` is impossible (e.g. detached HEAD in CI, offline) AND the user explicitly confirms.

3. **Check status to understand the schema**
   ```bash
   openspec status --change "<name>" --json
   ```
   Parse the JSON to understand:
   - `schemaName`: The workflow being used (e.g., "spec-driven")
   - `planningHome`, `changeRoot`, and `actionContext`: planning scope and edit constraints
   - Which artifact contains the tasks (typically "tasks" for spec-driven, check status for others)

4. **Get apply instructions**

   ```bash
   openspec instructions apply --change "<name>" --json
   ```

   This returns:
   - `contextFiles`: artifact ID -> array of concrete file paths (varies by schema)
   - Progress (total, complete, remaining)
   - Task list with status
   - Dynamic instruction based on current state

   **Handle states:**
   - If `state: "blocked"` (missing artifacts): show message, suggest using `/opsx:continue`
   - If `state: "all_done"`: congratulate, suggest archive (and the PR step below if not done yet)
   - Otherwise: proceed to implementation

5. **Read context files**

   Read every file path listed under `contextFiles` from the apply instructions output.
   The files depend on the schema being used:
   - **spec-driven**: proposal, specs, design, tasks
   - Other schemas: follow the contextFiles from CLI output

6. **Show current progress**

   Display:
   - Schema being used
   - Working branch
   - Progress: "N/M tasks complete"
   - Remaining tasks overview
   - Dynamic instruction from CLI

7. **Implement tasks (loop until done or blocked)**

   For each pending task:
   - Show which task is being worked on
   - Make the code changes required
   - Keep changes minimal and focused
   - Mark task complete in the tasks file: `- [ ]` → `- [x]`
   - Continue to next task

   **Pause if:**
   - Task is unclear → ask for clarification
   - Implementation reveals a design issue → suggest updating artifacts
   - Error or blocker encountered → report and wait for guidance
   - User interrupts

8. **Open the pull request (FINAL step — only when all tasks are complete)**

   When `tasks.md` has all tasks marked `[x]`, push the branch and open a PR, then record the PR number in the change so `/opsx:archive` can gate on it being merged.

   a. **Commit & push** the working branch (if there are uncommitted changes or unpushed commits):
      ```bash
      git add -A
      git commit -m "feat(<name>): <short summary>"
      git push -u origin HEAD
      ```
      Use a conventional-commit message; respect existing commit style in the repo. If the changes were already committed during the loop, just `git push -u origin HEAD`.

   b. **Verify CI/local checks before opening the PR.** If quick checks are available (e.g., `cargo fmt --check`, `cargo clippy`, `cargo test`, `npm run build`), run them and pause if they fail — do NOT open a PR on red.

   c. **Create the PR** referencing the tracking issue. The PR body is auto-filled from `.github/PULL_REQUEST_TEMPLATE.md` by GitHub; do not duplicate that template inline. Pass only the OpenSpec-linking summary:
      ```bash
      gh pr create \
        --base main \
        --head feature/<name> \
        --title "feat(<name>): <one-line summary>" \
        --body "## 关联 OpenSpec change

      - Change: \`<name>\`（\`openspec/changes/<name>/\`）
      - Issue: #<issue-number>
      - 提案 / 设计 / 任务：proposal.md / design.md / tasks.md

      ## 变更摘要

      <auto-filled by PULL_REQUEST_TEMPLATE.md sections below>

      Closes #<issue-number>"
      ```
      - `--base main` unless the repo's default branch is `master` (check `git remote show origin | grep HEAD` if unsure) — use the actual default branch.
      - If `gh` is unavailable or unauthenticated, SKIP PR creation with a clear warning; the change can still be archived later after a manual PR. Leave `.openspec.yaml` without a `pr:` key in that case.
      - Capture the returned PR URL and parse the number (`--json number -q .number` for reliability).

   d. **Write the PR number into `.openspec.yaml`** (new `pr:` key alongside the existing `issue:` / `branch:`):
      ```yaml
      pr: <number>
      ```
      Use the **Edit tool** to insert/merge the `pr:` line. Do not overwrite `issue:` / `branch:`.

   e. **Commit the `.openspec.yaml` update** (and any tasks.md checkbox updates not yet committed) on the same branch and push so the PR reflects the final state:
      ```bash
      git add openspec/changes/<name>/.openspec.yaml
      git commit -m "chore(<name>): record PR #<number> in openspec metadata"
      git push
      ```

   **Guardrails for this step:**
   - Only run this step ONCE per change. If `.openspec.yaml` already has a `pr:` key, skip PR creation and report the existing PR (with its merge status from `gh pr view <number> --json state,merged`).
   - The PR must be open (not draft-only) before `/opsx:archive` can pass its gate.

9. **On completion or pause, show status**

   Display:
   - Tasks completed this session
   - Overall progress: "N/M tasks complete"
   - Working branch and PR link (if created)
   - If all done AND PR created: suggest archive (`/opsx:archive`)
   - If paused: explain why and wait for guidance

**Output During Implementation**

```
## Implementing: <change-name> (schema: <schema-name>)
Working branch: feature/<change-name>

Working on task 3/7: <task description>
[...implementation happening...]
✓ Task complete

Working on task 4/7: <task description>
[...implementation happening...]
✓ Task complete
```

**Output On Completion**

```
## Implementation Complete

**Change:** <change-name>
**Schema:** <schema-name>
**Branch:** feature/<change-name>
**PR:** #<number> — https://github.com/<owner>/<repo>/pull/<number>
**Progress:** 7/7 tasks complete ✓

### Completed This Session
- [x] Task 1
- [x] Task 2
...

All tasks complete and PR opened. Ready to archive this change with `/opsx:archive` (requires the PR to be merged).
```

**Output On Pause (Issue Encountered)**

```
## Implementation Paused

**Change:** <change-name>
**Schema:** <schema-name>
**Branch:** feature/<change-name>
**Progress:** 4/7 tasks complete

### Issue Encountered
<description of the issue>

**Options:**
1. <option 1>
2. <option 2>
3. Other approach

What would you like to do?
```

**Guardrails**
- Create the working branch BEFORE any code change, always based on `main`; never commit implementation work to `main`/`master` or stack it on another feature branch
- Keep going through tasks until done or blocked
- Always read context files before starting (from the apply instructions output)
- If task is ambiguous, pause and ask before implementing
- If implementation reveals issues, pause and suggest artifact updates
- Keep code changes minimal and scoped to each task
- Update task checkbox immediately after completing each task
- Pause on errors, blockers, or unclear requirements - don't guess
- Use contextFiles from CLI output, don't assume specific file names
- Open the PR exactly once; always record `pr:` in `.openspec.yaml`

**Fluid Workflow Integration**

This skill supports the "actions on a change" model:

- **Can be invoked anytime**: Before all artifacts are done (if tasks exist), after partial implementation, interleaved with other actions
- **Allows artifact updates**: If implementation reveals design issues, suggest updating artifacts - not phase-locked, work fluidly
