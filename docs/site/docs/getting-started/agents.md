---
title: Agent Usage
description: How Claude Code, Cursor, Aider, Copilot Workspace, and other AI coding agents use gest to track tasks and run in parallel across isolated workspaces.
keywords: [Claude Code, Cursor, Aider, Copilot Workspace, parallel AI agents, agentic workflow CLI]
---

# Agent Usage

Gest is designed to work with AI coding agents. Any agent that can run shell
commands can use gest to track tasks, store artifacts, and coordinate execution
plans across multiple concurrent workers. To see how the gest project itself
uses agents, browse the
[`.agents/` directory](https://github.com/aaronmallen/gest/tree/main/.agents).

## Workspace setup for parallel agents

If you plan to run agents in parallel across multiple checkouts of the same
repository (jj workspaces, git worktrees, separate clones), you must explicitly
attach each secondary checkout to the same project. Otherwise each checkout
will resolve to its own project row and agents will not see each other's work.

From the primary checkout, get the project id:

```sh
gest project
```

Then from each additional checkout, attach to it:

```sh
cd ../myapp-feature-a
gest project attach <project-id>
```

After attaching, every checkout shares the same tasks, artifacts, and
iterations. See [`gest project`](../cli/project.md) for the full reference.

## Read the work queue

List open tasks to find what needs to be done. Use `--json` for machine-readable
output:

```sh
gest task list --json
```

Filter by status, tag, or assignment:

```sh
gest task list --status open --tag api
gest task list --assigned-to agent-1 --json
```

## Track task progress

Update a task's status as work progresses:

```sh
gest task update <id> --status in-progress
# ... do the work ...
gest task complete <id>
```

Task shortcuts provide concise alternatives to `task update --status`:

```sh
gest task complete <id>          # shortcut for task update <id> --status done
gest task cancel <id>            # shortcut for task update <id> --status cancelled
gest task block <id> <other-id>  # shortcut for task link <id> blocks <other-id>
```

Assign a task to the current agent so other agents know it's taken:

```sh
gest task update <id> --assigned-to agent-1
```

## Store design documents

Save specs, ADRs, RFCs, and other prose as artifacts. Provide the body inline
with `-b` to avoid opening `$EDITOR`:

```sh
gest artifact create "Auth Middleware Design" \
  -b "Token-bucket rate limiting with per-user quotas." \
  --tag spec \
  --tag auth \
  --tag design
```

Or import from a file:

```sh
gest artifact create --source design.md --tag adr --tag architecture
```

Link a task to its source artifact:

```sh
gest task link <task-id> child-of <artifact-id> --artifact
```

## Build execution plans

Group related tasks into an iteration with phased execution. Tasks in the same
phase can run in parallel; lower phases execute first. Phases are the
parallelism boundary: if one task blocks another, they must live in different
phases.

```sh
# Create tasks (no phase yet)
gest task create "Add parser types"
gest task create "Add CLI flag"
gest task create "Integrate parser"

# Create an iteration and assign tasks to phases
gest iteration create "Implement feature X"
gest iteration add <iteration-id> <parser-id>    --phase 1
gest iteration add <iteration-id> <cli-flag-id>  --phase 1
gest iteration add <iteration-id> <integrate-id> --phase 2

# Visualize the plan
gest iteration graph <iteration-id>
```

## Orchestrate multiple agents

Gest provides orchestration primitives that let multiple agents work on the
same iteration concurrently. An orchestrator dispatches work, agents claim
tasks, and gest tracks progress through phased execution.

The basic loop is:

1. **Find work** — list iterations that have claimable tasks
2. **Claim a task** — atomically assign the next available task to an agent
3. **Check progress** — query aggregated status for the iteration
4. **Advance phase** — move to the next phase once the current one completes

### Find work

Use `--has-available` to find iterations that have at least one open,
unassigned, unblocked task in the active phase:

```sh
gest iteration list --has-available
gest iteration list --has-available --json
```

```json
[
  {
    "id": "a1b2c3d4e5f6...",
    "title": "Implement export command",
    "status": "active",
    "tasks": ["aaabbbcccddd...", "eeefffggghhh..."],
    "phase_count": 3,
    "tags": []
  }
]
```

To see what a specific agent is already working on:

```sh
gest task list --assigned-to my-agent --json
```

### Claim a task

`gest iteration next` finds the highest-priority open task in the active phase.
Without `--claim` it peeks without side effects; with `--claim` it atomically
sets the task to `in-progress` (and assigns it to `--agent` when provided):

```sh
# Peek at the next available task
gest iteration next <iteration-id>

# Claim it (assigning to an agent is optional but recommended)
gest iteration next <iteration-id> --claim --agent my-agent

# Claim without naming an agent (assignment is left empty)
gest iteration next <iteration-id> --claim
```

`--agent` requires `--claim`. `--claim` itself can be used standalone.

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

#### Exit code 2

When no tasks are available, `gest iteration next` exits with code **2**
instead of the usual 1 for errors. This lets scripts distinguish "nothing to
do" from "something broke":

```sh
gest iteration next <id> --claim --agent my-agent --json
status=$?
if [ $status -eq 2 ]; then
  echo "No work available -- idle"
elif [ $status -ne 0 ]; then
  echo "Error"
fi
```

#### Task selection order

Candidates are drawn from the active phase (lowest phase with incomplete tasks)
and sorted by:

1. **Phase** — ascending (the active phase wins)
2. **Priority** — lowest value first (P0 before P4)

No further tie-break is applied. Tasks that are already assigned, non-open, or
blocked by unfinished dependencies are excluded.

### Check progress

`gest iteration status` returns aggregated progress for the entire iteration:

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

| Field              | Description                                           |
|--------------------|-------------------------------------------------------|
| `active_phase`     | Lowest phase that still has incomplete tasks          |
| `total_phases`     | Number of distinct phases in the iteration            |
| `phase_progress`   | Done vs. total tasks in the active phase              |
| `blocked`          | Active-phase tasks blocked by unfinished dependencies |
| `in_progress`      | Active-phase tasks currently being worked on          |
| `assignees`        | Agents with in-progress tasks (across all phases)     |
| `overall_progress` | Done vs. total tasks across the entire iteration      |

### Advance phase

Once all tasks in the active phase are in a terminal state (done or
cancelled), advance to the next phase:

```sh
gest iteration advance <iteration-id>
```

If tasks remain incomplete, the command errors. Use `--force` to advance
anyway:

```sh
gest iteration advance <iteration-id> --force
```

On success the command prints how many tasks are now active in the new phase.
When there are no more phases, it prints "All phases complete".

### A minimal orchestrator loop

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

## Search before creating

Before creating a new task or artifact, search for existing ones to avoid
duplicates:

```sh
gest search "rate limiting"
```

Add `--expand` to see full details, or `--json` for machine-readable results:

```sh
gest search "rate limiting" --expand
gest search "rate limiting" --json
```

Resolved tasks and archived artifacts are excluded by default. Pass `--all` to
include them:

```sh
gest search "rate limiting" --all
```

## Scripting tips

### JSON and quiet output

Most commands support `--json` for structured output. Mutation commands
(`create`, `update`, `complete`, `cancel`, `block`, `link`, `tag`, `untag`,
`note add`, `note update`, `meta set`) also support `-q`/`--quiet` to print only
the entity ID:

```sh
gest task show <id> --json
gest task list --json

# Get just the ID for scripting
task_id=$(gest task create "My task" -q)
gest task complete "$task_id" -q
```

### Stdin piping

When `--description` (tasks) or `--body` (artifacts/notes) is omitted and stdin
is a pipe, the piped content is used automatically:

```sh
echo "Detailed description" | gest task create "My task"
cat spec.md | gest artifact create --tag spec
```

### Batch creation

Use `--batch` to create multiple entities from NDJSON (one JSON object per
line):

```sh
cat tasks.ndjson | gest task create --batch
cat artifacts.ndjson | gest artifact create --batch
```

Batch-add tasks to an iteration with per-record phase control:

```sh
cat <<'EOF' | gest iteration add <iteration-id> --batch
{"task":"<task-id-1>","phase":1}
{"task":"<task-id-2>","phase":1}
{"task":"<task-id-3>","phase":2}
{"task":"<task-id-4>"}
EOF
```

When `phase` is omitted, it auto-increments from the current max phase + 1.

### Iteration and link flags

Create tasks that are pre-linked and assigned to an iteration in a single
command:

```sh
gest task create "Add auth" \
  -i <iteration-id> \
  -l child-of:<spec-artifact-id>
```
