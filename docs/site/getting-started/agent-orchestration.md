# Agent Orchestration

Gest provides orchestration primitives that let multiple AI agents work on the same iteration
concurrently. An orchestrator dispatches work, agents claim tasks, and gest tracks progress
through phased execution.

## Workflow Overview

1. **Find work** -- list iterations that have claimable tasks
2. **Claim a task** -- atomically assign the next available task to an agent
3. **Check progress** -- query aggregated status for the iteration
4. **Advance phase** -- move to the next phase once the current one completes

## Step 1: Find Work

Use `--has-available` to find iterations that have at least one open, unassigned, unblocked
task in the active phase:

```sh
gest iteration list --has-available
```

Add `--json` for machine-readable output:

```sh
gest iteration list --has-available --json
```

```json
[
  {
    "id": "a1b2c3d4e5f6...",
    "title": "Implement export command",
    "status": "active",
    "tasks": ["tasks/aaa...", "tasks/bbb..."],
    "phase_count": 3,
    "tags": []
  }
]
```

To see what a specific agent is already working on:

```sh
gest task list --assigned-to my-agent --json
```

## Step 2: Claim a Task

`gest iteration next` finds the highest-priority open task in the active phase. Without
`--claim` it peeks without side effects; with `--claim` it atomically sets the task to
`in-progress` and assigns it to the named agent:

```sh
# Peek at the next available task
gest iteration next <iteration-id>

# Claim it
gest iteration next <iteration-id> --claim --agent my-agent
```

`--agent` is required when `--claim` is used.

Add `--json` to get structured output:

```sh
gest iteration next <iteration-id> --claim --agent my-agent --json
```

```json
{
  "id": "f7e8d9c0...",
  "title": "Add CSV formatter",
  "status": "in-progress",
  "assigned_to": "my-agent",
  "phase": 1,
  "priority": 1
}
```

### Exit Code 2

When no tasks are available, `gest iteration next` exits with code **2** instead of the
usual 1 for errors. This lets scripts distinguish "nothing to do" from "something broke":

```sh
gest iteration next <id> --claim --agent my-agent --json
status=$?
if [ $status -eq 2 ]; then
  echo "No work available -- idle"
elif [ $status -ne 0 ]; then
  echo "Error"
fi
```

### Task Selection Order

Tasks are selected from the active phase (lowest phase with incomplete tasks) using:

1. **Priority** -- lowest value first (P1 before P5)
2. **Created date** -- oldest first (tie-breaker)

Tasks that are already assigned, non-open, or blocked by unfinished dependencies are excluded.

## Step 3: Check Progress

`gest iteration status` returns aggregated progress for the entire iteration:

```sh
gest iteration status <iteration-id>
```

With `--json`:

```sh
gest iteration status <iteration-id> --json
```

```json
{
  "active_phase": 1,
  "total_phases": 3,
  "phase_progress": {
    "done": 2,
    "total": 4
  },
  "blocked": 0,
  "in_progress": 1,
  "assignees": ["agent-1", "agent-2"],
  "overall_progress": {
    "done": 2,
    "total": 12
  }
}
```

Fields:

| Field              | Description                                           |
|--------------------|-------------------------------------------------------|
| `active_phase`     | Lowest phase that still has incomplete tasks          |
| `total_phases`     | Number of distinct phases in the iteration            |
| `phase_progress`   | Done vs. total tasks in the active phase              |
| `blocked`          | Active-phase tasks blocked by unfinished dependencies |
| `in_progress`      | Active-phase tasks currently being worked on          |
| `assignees`        | Agents with in-progress tasks (across all phases)     |
| `overall_progress` | Done vs. total tasks across the entire iteration      |

## Step 4: Advance Phase

Once all tasks in the active phase are in a terminal state (done or cancelled), advance to
the next phase:

```sh
gest iteration advance <iteration-id>
```

If tasks remain incomplete, the command errors. Use `--force` to advance anyway:

```sh
gest iteration advance <iteration-id> --force
```

On success the command prints how many tasks are now active in the new phase. When there are
no more phases, it prints "All phases complete".

## Putting It Together

A minimal orchestrator loop:

```sh
ITER_ID="a1b2c3d4"

while true; do
  # Try to claim work
  task=$(gest iteration next "$ITER_ID" --claim --agent worker-1 --json 2>/dev/null)
  status=$?

  if [ $status -eq 2 ]; then
    # No tasks in this phase -- try advancing
    gest iteration advance "$ITER_ID" 2>/dev/null || break
    continue
  elif [ $status -ne 0 ]; then
    echo "Error claiming task" >&2
    break
  fi

  task_id=$(echo "$task" | jq -r '.id')

  # ... do the work ...

  gest task complete "$task_id" -q
done
```

## Scripting Tips

### Machine-Readable Output

All mutation commands (`create`, `update`, `complete`, `cancel`, `block`, `link`, `tag`,
`untag`, `note add`, `note update`, `meta set`) support `--json` for structured output and
`-q`/`--quiet` for printing only the entity ID:

```sh
# Get the new task's full JSON after creation
gest task create "Implement parser" --json

# Get just the ID for scripting
task_id=$(gest task create "Implement parser" -q)
```

### Stdin Piping

When `--description` (tasks) or `--body` (artifacts/notes) is omitted and stdin is a pipe,
the piped content is used automatically:

```sh
echo "Design notes here" | gest task create "Design task"
cat spec.md | gest artifact create -k spec
```

### Batch Creation

Use `--batch` to create multiple entities from NDJSON (one JSON object per line):

```sh
cat tasks.ndjson | gest task create --batch
cat artifacts.ndjson | gest artifact create --batch
```

### Iteration and Link Flags

Create tasks that are pre-linked and assigned to an iteration in a single command:

```sh
gest task create "Add auth" \
  -i <iteration-id> \
  -l child-of:<spec-artifact-id>
```
