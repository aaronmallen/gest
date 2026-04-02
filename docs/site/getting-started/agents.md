# Agent Usage

Gest is designed to work with AI coding agents. Any agent that can run shell commands can use
gest to track tasks, store artifacts, and coordinate execution plans. To see how the gest
project itself uses agents, browse the
[`.agents/` directory](https://github.com/aaronmallen/gest/tree/main/.agents).

## Read the Work Queue

List open tasks to find what needs to be done. Use `--json` for machine-readable output:

```sh
gest task list --json
```

Filter by status, tag, or assignment:

```sh
gest task list --status open --tag api
gest task list --assigned-to agent-1 --json
```

## Track Task Progress

Update a task's status as work progresses:

```sh
gest task update <id> --status in-progress
# ... do the work ...
gest task complete <id>
```

Task shortcuts provide concise alternatives to `task update --status`:

```sh
gest task complete <id>   # shortcut for task update <id> --status done
gest task cancel <id>     # shortcut for task update <id> --status cancelled
gest task block <id> <other-id>  # shortcut for task link <id> blocks <other-id>
```

Assign a task to the current agent so other agents know it's taken:

```sh
gest task update <id> --assigned-to agent-1
```

## Store Design Documents

Save specs, ADRs, RFCs, and other prose as artifacts. Provide the body inline with `-b` to
avoid opening `$EDITOR`:

```sh
gest artifact create \
  -t "Auth Middleware Design" \
  -b "Token-bucket rate limiting with per-user quotas." \
  -k spec \
  --tag "auth,design"
```

Or import from a file:

```sh
gest artifact create --source design.md --type adr --tag "architecture"
```

Link a task to its source artifact:

```sh
gest task link <task-id> child-of <artifact-id> --artifact
```

## Build Execution Plans

Group related tasks into an iteration with phased execution. Tasks in the same phase can run
in parallel; lower phases execute first:

```sh
# Create tasks with phase assignments
gest task create "Add parser types" --phase 1 --priority 1
gest task create "Add CLI flag" --phase 1 --priority 2
gest task create "Integrate parser" --phase 2 --priority 0

# Set dependencies
gest task link <integrate-id> blocked-by <parser-id>

# Create an iteration and add tasks
gest iteration create "Implement feature X"
gest iteration add <iteration-id> <task-id>

# Visualize the plan
gest iteration graph <iteration-id>
```

## Orchestrate Multiple Agents

When running multiple agents concurrently, use the orchestration commands to coordinate work
without conflicts:

```sh
# Find iterations with available work
gest iteration list --has-available

# Claim the next task (atomic -- safe for concurrent agents)
gest iteration next <iteration-id> --claim --agent my-agent --json

# Check overall progress
gest iteration status <iteration-id> --json

# Advance to the next phase when the current one is done
gest iteration advance <iteration-id>
```

`iteration next` exits with code **2** when no tasks are available, letting scripts
distinguish "idle" from "error":

```sh
task=$(gest iteration next "$ITER" --claim --agent worker-1 --json 2>/dev/null)
if [ $? -eq 2 ]; then
  echo "Nothing to do"
fi
```

See the [Agent Orchestration guide](./agent-orchestration.md) for a complete
walkthrough.

## Use JSON and Quiet Output

Most commands support `--json` for structured output. Mutation commands also support
`-q`/`--quiet` to print only the entity ID:

```sh
gest task show <id> --json
gest task list --json
gest artifact list --json
gest search "auth" --json

# Get just the ID for scripting
task_id=$(gest task create "My task" -q)
gest task complete "$task_id" -q
```

### Stdin Piping

When `--description` (tasks) or `--body` (artifacts/notes) is omitted and stdin is a pipe,
the piped content is used automatically:

```sh
echo "Detailed description" | gest task create "My task"
cat spec.md | gest artifact create -k spec
```

### Batch Creation

Use `--batch` to create multiple entities from NDJSON (one JSON object per line):

```sh
cat tasks.ndjson | gest task create --batch
```

### Iteration and Link Flags

Create tasks pre-linked and assigned to an iteration in a single command:

```sh
gest task create "Add auth" -i <iteration-id> -l child-of:<spec-id>
```

## Search Before Creating

Before creating a new task or artifact, search for existing ones to avoid duplicates:

```sh
gest search "rate limiting"
```

Add `--expand` to see full details, or `--json` for machine-readable results:

```sh
gest search "rate limiting" --expand
gest search "rate limiting" --json
```

Resolved tasks and archived artifacts are excluded by default. Pass `--all` to include them:

```sh
gest search "rate limiting" --all
```
