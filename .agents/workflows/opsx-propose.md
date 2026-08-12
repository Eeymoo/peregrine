---
description: Propose a new change - create it, generate all artifacts, and open a tracking issue in one step
---

Propose a new change - create the change, generate all artifacts, and open a tracking GitHub issue in one step.

I'll create a change with artifacts:
- proposal.md (what & why)
- design.md (how)
- tasks.md (implementation steps)

I'll also open a GitHub tracking issue and record its number in the change's `.openspec.yaml`.

When ready to implement, run /opsx:apply

---

**Store selection:** If the user names a store (a store is a standalone OpenSpec repo registered on this machine) or the work lives in one, run `openspec store list --json` to discover registered store ids, then pass `--store <id>` on the commands that read or write specs and changes (`new change`, `status`, `instructions`, `list`, `show`, `validate`, `archive`, `doctor`, `context`). Other commands do not take the flag. Hints printed by commands already carry the flag; keep it on follow-ups. Without a store, commands act on the nearest local `openspec/` root.

**Input**: The argument after `/opsx:propose` is the change name (kebab-case), OR a description of what the user wants to build.

**Steps**

1. **If no input provided, ask what they want to build**

   Use the **AskUserQuestion tool** (open-ended, no preset options) to ask:
   > "What change do you want to work on? Describe what you want to build or fix."

   From their description, derive a kebab-case name (e.g., "add user authentication" → `add-user-auth`).

   **IMPORTANT**: Do NOT proceed without understanding what the user wants to build.

2. **Create the change directory**
   ```bash
   openspec new change "<name>"
   ```
   This creates a scaffolded change in the planning home resolved by the CLI with `.openspec.yaml`.

3. **Get the artifact build order**
   ```bash
   openspec status --change "<name>" --json
   ```
   Parse the JSON to get:
   - `applyRequires`: array of artifact IDs needed before implementation (e.g., `["tasks"]`)
   - `artifacts`: list of all artifacts with their status and dependencies
   - `planningHome`, `changeRoot`, `artifactPaths`, and `actionContext`: path and scope context. Use these instead of assuming repo-local paths.

4. **Create artifacts in sequence until apply-ready**

   Use the **TodoWrite tool** to track progress through the artifacts.

   Loop through artifacts in dependency order (artifacts with no pending dependencies first):

   a. **For each artifact that is `ready` (dependencies satisfied)**:
      - Get instructions:
        ```bash
        openspec instructions <artifact-id> --change "<name>" --json
        ```
      - The instructions JSON includes:
        - `context`: Project background (constraints for you - do NOT include in output)
        - `rules`: Artifact-specific rules (constraints for you - do NOT include in output)
        - `template`: The structure to use for your output file
        - `instruction`: Schema-specific guidance for this artifact type
        - `resolvedOutputPath`: Resolved path or pattern to write the artifact
        - `dependencies`: Completed artifacts to read for context
      - Read any completed dependency files for context
      - Create the artifact file using `template` as the structure and write it to `resolvedOutputPath`
      - Apply `context` and `rules` as constraints - but do NOT copy them into the file
      - Show brief progress: "Created <artifact-id>"

   b. **Continue until all `applyRequires` artifacts are complete**
      - After creating each artifact, re-run `openspec status --change "<name>" --json`
      - Check if every artifact ID in `applyRequires` has `status: "done"` in the artifacts array
      - Stop when all `applyRequires` artifacts are done

   c. **If an artifact requires user input** (unclear context):
      - Use **AskUserQuestion tool** to clarify
      - Then continue with creation

5. **Create the tracking GitHub issue**

   This step binds the change to a GitHub issue so `/opsx:archive` can gate on the linked PR and the team has a durable record. Do this AFTER the artifacts are generated so the issue body can summarize them.

   a. Read the just-generated `proposal.md` from `changeRoot` to extract motivation / goals / non-goals / impact.

   b. Compose the issue body referencing the change path. Minimal structure:
      ```markdown
      ## 动机 / Why
      <from proposal.md>

      ## 目标 / Goals
      <from proposal.md>

      ## 非目标 / Non-goals
      <from proposal.md>

      ## 影响范围 / Impact
      <from proposal.md>

      ---
      OpenSpec change: `openspec/changes/<name>/`
      用 `/opsx:apply <name>` 开始实施。
      ```

   c. Create the issue with the `gh` CLI. Required flags:
      ```bash
      gh issue create \
        --title "[Feature] <name>: <one-line summary from proposal>" \
        --label "feature,openspec" \
        --body "<body>"
      ```
      - If the `feature` or `openspec` label does not exist, create it first with `gh label create <name> --force` (ignore "already exists" errors).
      - Capture the returned URL and parse the issue number from it (the trailing integer, or use `--json number -q .number` instead of the URL form for reliable parsing).

   d. **Write the issue number (and intended branch name) into `.openspec.yaml`** so `/opsx:apply` and `/opsx:archive` can read them. Merge the two new keys into the existing file without disturbing `schema` / `created` / `status` / `note`:
      ```yaml
      schema: spec-driven
      created: <existing>
      status: active
      issue: <number>
      branch: feature/<name>
      ```
      Use the **Edit tool** to insert the `issue:` and `branch:` lines (preserve any existing `note:` block).

   e. **Add a tracking reference to the top of `proposal.md`**: prepend a blockquote line such as `> **跟踪 issue：#<number>**（https://github.com/<owner>/<repo>/issues/<number>）` immediately after the existing status blockquote (or at the very top if none). Use the **Edit tool**, do not rewrite the file.

   **Guardrails for this step:**
   - If `gh` is not authenticated, run `gh auth status`; if it still fails, SKIP the issue creation with a clear warning and continue (do not block the whole propose on it). Leave `.openspec.yaml` without an `issue:` key in that case.
   - Never create an issue twice: if `.openspec.yaml` already has an `issue:` key, skip creation and report the existing one.
   - Keep the issue body bilingual-friendly but prefer Simplified Chinese per project convention.

6. **Show final status**
   ```bash
   openspec status --change "<name>"
   ```

**Output**

After completing all artifacts, summarize:
- Change name and location
- List of artifacts created with brief descriptions
- Tracking issue: `#<number>` with URL (or "issue creation skipped - gh unavailable")
- What's ready: "All artifacts created! Ready for implementation."
- Prompt: "Run `/opsx:apply` to start implementing."

**Artifact Creation Guidelines**

- Follow the `instruction` field from `openspec instructions` for each artifact type
- The schema defines what each artifact should contain - follow it
- Read dependency artifacts for context before creating new ones
- Use `template` as the structure for your output file - fill in its sections
- **IMPORTANT**: `context` and `rules` are constraints for YOU, not content for the file
  - Do NOT copy `<context>`, `<rules>`, `<project_context>` blocks into the artifact
  - These guide what you write, but should never appear in the output

**Guardrails**
- Create ALL artifacts needed for implementation (as defined by schema's `apply.requires`)
- Always read dependency artifacts before creating a new one
- If context is critically unclear, ask the user - but prefer making reasonable decisions to keep momentum
- If a change with that name already exists, ask if user wants to continue it or create a new one
- Verify each artifact file exists after writing before proceeding to next
- Always create exactly ONE tracking issue per change, and always record its number in `.openspec.yaml`
