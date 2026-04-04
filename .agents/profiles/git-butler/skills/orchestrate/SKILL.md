---
name: orchestrate
description: "Take an iteration and execute it: dispatch /implement agents per phase, always sequentially (e.g. /orchestrate <iteration-id>)."
args: "<iteration-id>"
---

# Orchestrate (git-butler profile)

Execute an iteration by dispatching implementation agents phase by phase. All tasks are executed sequentially in the
main workspace.

Git Butler's virtual branch model does not support parallel worktree isolation. Virtual branches operate within a single
working directory, and multiple worktrees would conflict with Git Butler's state management. Therefore, all tasks are
executed sequentially regardless of phase structure.

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

Analyze the iteration structure:

- **Single task:** If there is only **1 task** in the entire iteration, run `/implement <task-id>` directly. No phase
  logic needed. Skip to step 5 (Clean Up).
- **Multiple tasks (any number of phases):** All tasks are executed **sequentially** in phase order, respecting blocking
  dependencies. Within each phase, tasks are executed one after another in priority order.

Present the execution plan (phase breakdown and task order) to the user for confirmation before proceeding.

### 3. Build Phases

Group tasks by their `phase` field:

- **Phase 1** -- tasks with `phase: 1`
- **Phase 2** -- tasks with `phase: 2`
- **Phase N** -- and so on

Even though execution is always sequential, phase ordering and blocking dependencies must still be respected. A task in
phase 2 must not start until all phase 1 tasks are complete.

### 4. Execute Phases

For each phase:

1. **Claim tasks** using `iteration next`. Use `--json` for structured output or `-q` for bare task ID:

   ```sh
   # Claim next available task in priority order (returns JSON with task details):
   GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration next <iteration-id> --claim --agent implement-agent --json

   # Or get just the task ID for scripting:
   GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration next <iteration-id> --claim --agent implement-agent -q
   ```

   Exit code **2** means no tasks are available (distinguishes idle from errors).

2. For each task in the phase (in priority order):
   - Run `/implement <task-id>` in the main workspace.
   - Wait for the task to complete before starting the next.

3. **Check phase progress:**

   ```sh
   GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration status <iteration-id> --json
   ```

4. **Advance to the next phase** once the current phase is complete:

   ```sh
   GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration advance <iteration-id>
   ```

   Use `--force` to advance past stuck tasks if needed.

5. Report results to the user (successes, failures, tasks needing attention).

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
3. Present a summary of all implemented tasks, including successes and failures.
