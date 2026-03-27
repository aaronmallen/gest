---
name: orchestrate
description: "Take a plan and execute it: create workspaces, dispatch /implement agents in parallel (e.g. /orchestrate <gest-id>)."
args: "<gest-id>"
---

# Orchestrate

Execute a multi-issue plan (gest spec artifact) by dispatching parallel implementation agents.

## Instructions

### 1. Read the Plan

Retrieve the spec artifact via `cargo run -- artifact show <id>`. Then find linked tasks via
`cargo run -- task list --json` and filter for tasks whose `links` reference the artifact ID. Extract:

- Task list with dependency ordering
- Parallelization notes
- ADR reference (if any)

### 2. Build Waves

Group tasks into waves using their gest metadata (`wave`, `parallel` keys):

- **Wave 1** -- tasks with `wave: 1` (run in parallel if `parallel: true`)
- **Wave 2** -- tasks with `wave: 2`
- **Wave N** -- and so on

Present the wave plan to the user for confirmation before proceeding.

### 3. Execute Waves

For each wave:

1. Create isolated worktrees for each task in the wave (delegate to **vcs-expert**)
2. Dispatch `/implement <task-id>` into each worktree in parallel
3. Wait for all agents in the wave to complete
4. Report results to the user (successes, failures, tasks needing attention)
5. Merge completed worktrees back (delegate to **vcs-expert**)

Only proceed to the next wave after the user confirms the current wave's results.

### 4. Clean Up

After all waves complete:

1. Remove worktrees (delegate to **vcs-expert**)
2. Check for failed tasks -- any task still `in-progress` represents a failure. Report these to the
   user with their IDs and titles.
3. Archive the spec artifact:
   - If **all tasks** completed successfully (`done`): archive via
     `cargo run -- artifact archive <spec-id>`
   - If **any tasks** remain `in-progress`: do **not** archive the artifact. Flag this to the user
     and list the incomplete tasks.
4. Present a summary of all implemented tasks, including successes and failures
