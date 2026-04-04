---
name: orchestrate
description: "Take an iteration and execute it: dispatch /implement agents per phase with jj workspace isolation for parallel work (e.g. /orchestrate <iteration-id>)."
args: "<iteration-id>"
---

# Orchestrate (jj profile)

Execute an iteration by dispatching implementation agents phase by phase, using jj workspaces for parallel isolation
when needed.

## Instructions

### 1. Read the Iteration

Retrieve the iteration and visualize the execution plan:

```sh
GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration show --json <id>
GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration status <id> --json
GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration graph <id>
```

Extract:

- Task list grouped by `phase` field
- Blocking dependencies (`blocked-by` links)
- Priority ordering within each phase

### 2. Decide Execution Strategy

Analyze the iteration structure to choose the right execution mode:

- **Single task:** If there is only **1 task** in the entire iteration, run `/implement <task-id>` directly in the main
  workspace. No workspace isolation, no phase logic. Skip to step 5 (Clean Up).
- **Single phase (multiple tasks):** If all tasks belong to a **single phase**, run them **sequentially** in the main
  workspace. No workspace isolation needed. Execute each task with `/implement <task-id>` one after another.
- **Multiple phases with parallel work:** If there are **multiple phases** and **any phase contains more than 1 task**,
  use jj workspaces for parallel execution within those phases. Phases with only 1 task run directly in the main
  workspace.

Present the execution plan (strategy + phase breakdown) to the user for confirmation before proceeding.

### 3. Build Phases

Group tasks by their `phase` field:

- **Phase 1** -- tasks with `phase: 1`
- **Phase 2** -- tasks with `phase: 2`
- **Phase N** -- and so on

### 4. Execute Phases

For each phase:

1. **Claim tasks** using `iteration next`. Use `--json` for structured output or `-q` for bare task ID:

   ```sh
   # Claim next available task (returns JSON with task details):
   GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration next <iteration-id> --claim --agent implement-agent --json

   # Or get just the task ID for scripting:
   GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration next <iteration-id> --claim --agent implement-agent -q
   ```

   Exit code **2** means no tasks are available (distinguishes idle from errors).

2. **If the phase has a single task** (or execution strategy is sequential):
   - Run `/implement <task-id>` directly in the main workspace.

3. **If the phase has multiple tasks and parallel execution is enabled:**

   a. **Create workspaces** for each task in the phase:

   ```sh
   jj workspace add ../gest-<task-id> --name <task-id> -r @
   ```

   b. **Dispatch** `/implement <task-id>` for each task. Each implementation agent works in its respective workspace
   directory (`../gest-<task-id>`).

   c. **Wait** for all agents in the phase to complete.

   d. **Tear down workspaces** after the phase completes:

   ```sh
   # For each workspace created in this phase:
   jj workspace forget <task-id>
   rm -rf ../gest-<task-id>
   ```

   e. **Verify** the main workspace state:

   ```sh
   jj status
   ```

Note: All jj workspaces share the same commit graph. Changes made in any workspace are immediately visible to all other
workspaces. There is no need to merge or cherry-pick -- the commits are already part of the shared history.

1. **Check phase progress:**

   ```sh
   GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration status <iteration-id> --json
   ```

2. **Advance to the next phase** once the current phase is complete:

   ```sh
   GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration advance <iteration-id>
   ```

   Use `--force` to advance past stuck tasks if needed.

3. Report results to the user (successes, failures, tasks needing attention).

Only proceed to the next phase after the user confirms the current phase's results.

### 5. Clean Up

After all phases complete:

1. Check for failed tasks -- any task still `in-progress` represents a failure. Report these to the user with their IDs
   and titles.
2. Update the iteration status using lifecycle shortcuts:
   - If **all tasks** completed successfully (`done`):

     ```sh
     GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- \
       iteration update <iteration-id> --status completed -q
     ```

   - If **any tasks** remain `in-progress`:

     ```sh
     GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- \
       iteration cancel <iteration-id> -q
     ```

     Flag this to the user and list the incomplete tasks.
3. Verify no stale workspaces remain:

   ```sh
   jj workspace list
   ```

   If any task workspaces still exist, clean them up:

   ```sh
   jj workspace forget <task-id>
   rm -rf ../gest-<task-id>
   ```

4. Present a summary of all implemented tasks, including successes and failures.
