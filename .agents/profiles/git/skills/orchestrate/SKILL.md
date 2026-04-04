---
name: orchestrate
description: "Take an iteration and execute it: dispatch /implement agents per phase with git worktree isolation for parallel work (e.g. /orchestrate <iteration-id>)."
args: "<iteration-id>"
---

# Orchestrate (git profile)

Execute an iteration by dispatching implementation agents phase by phase, using git worktrees for parallel isolation
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
  worktree. No worktree isolation, no phase logic. Skip to step 5 (Clean Up).
- **Single phase (multiple tasks):** If all tasks belong to a **single phase**, run them **sequentially** in the main
  worktree. No worktree isolation needed. Execute each task with `/implement <task-id>` one after another.
- **Multiple phases with parallel work:** If there are **multiple phases** and **any phase contains more than 1 task**,
  use git worktrees for parallel execution within those phases. Phases with only 1 task run directly in the main
  worktree.

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
   - Run `/implement <task-id>` directly in the main worktree.

3. **If the phase has multiple tasks and parallel execution is enabled:**

   a. **Create worktrees** for each task in the phase:

   ```sh
   git worktree add -b implement/<task-id> ../gest-<task-id> HEAD
   ```

   Each worktree gets its own branch (`implement/<task-id>`) based on the current HEAD.

   b. **Dispatch** `/implement <task-id>` for each task. Each implementation agent works in its respective worktree
   directory (`../gest-<task-id>`).

   c. **Wait** for all agents in the phase to complete.

   d. **Tear down worktrees** after the phase completes:

   ```sh
   # For each worktree created in this phase:
   git worktree remove ../gest-<task-id>
   git branch -d implement/<task-id>
   ```

Note: Use `git branch -d` (lowercase) to safely delete merged branches. If the branch has unmerged changes that need to
be kept, merge or cherry-pick them into the main branch first.

   e. **Verify** worktree cleanup:

   ```sh
   git worktree list
   ```

Important: Unlike jj workspaces, git worktrees have independent branches. After parallel work completes, you may need to
merge the worktree branches back into the main branch before proceeding to the next phase. Ensure all changes from the
current phase are integrated before starting the next phase.

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
3. Verify no leftover worktrees remain:

   ```sh
   git worktree list
   ```

   If any task worktrees still exist, clean them up:

   ```sh
   git worktree remove ../gest-<task-id>
   git branch -d implement/<task-id>
   ```

4. Present a summary of all implemented tasks, including successes and failures.
