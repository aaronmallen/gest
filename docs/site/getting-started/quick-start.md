# Quick Start

This guide walks through gest's core workflow: initializing a store, creating tasks and
artifacts, linking them together, and searching.

## Initialize a Store

Before using gest you need to initialize a data store. By default, gest uses a global store in
your system data directory (`~/.local/share/gest/` on Linux, `~/Library/Application Support/gest/`
on macOS):

```sh
gest init
```

To keep data inside your repository instead, use the `--local` flag. This creates a `.gest/`
directory in the current project:

```sh
gest init --local
```

## Create a Task

Tasks track units of work. Create one by providing a title:

```sh
gest task create "Implement auth middleware"
```

You can add details inline:

```sh
gest task create "Add rate limiting" \
  -d "Implement token-bucket rate limiting on API endpoints" \
  -p 1 \
  -s open \
  --tag "api,security" \
  --phase 2
```

- `-d` sets the description (omit it to open your `$EDITOR`)
- `-p` sets priority (`0` is highest, `4` is lowest)
- `-s` sets the initial status (`open`, `in-progress`, `done`, or `cancelled`)
- `--tag` attaches tags (repeatable, or comma-separated)
- `--phase` assigns an execution phase for parallel grouping

List your tasks to see what you have:

```sh
gest task list
```

## Create an Artifact

Artifacts store documents like specs, ADRs, RFCs, and notes. You can create one from a file:

```sh
gest artifact create --source auth-spec.md --type spec --tag "auth"
```

Or write the body inline:

```sh
gest artifact create \
  -t "Rate Limiting Design" \
  -b "Token-bucket algorithm with per-user quotas." \
  -k adr \
  --tag "api,design"
```

- `-t` sets the title (auto-extracted from the first `#` heading if omitted)
- `-b` provides inline body content
- `-k` / `--type` sets the artifact type (e.g. `spec`, `adr`, `rfc`, `note`)
- `--source` reads body content from a file

List artifacts to see them:

```sh
gest artifact list
```

## Link Entities

Tasks can be linked to other tasks or to artifacts. Gest supports several relationship types:
`blocks`, `blocked-by`, `child-of`, `parent-of`, and `relates-to`.

Link two tasks (reciprocal links are created automatically):

```sh
# Use the task ID or a unique prefix
gest task link <task-id> blocked-by <other-task-id>
```

Link a task to an artifact with the `--artifact` flag:

```sh
gest task link <task-id> relates-to <artifact-id> --artifact
```

View a task's details to see its links:

```sh
gest task show <task-id>
```

## Search

Search across all tasks and artifacts by keyword:

```sh
gest search "auth"
```

Add `--expand` to see full details for each result:

```sh
gest search "auth" --expand
```

Use `--json` to get machine-readable output for scripting:

```sh
gest search "auth" --json
```

By default, resolved tasks and archived artifacts are excluded. Pass `--all` to include them:

```sh
gest search "auth" --all
```

## Next Steps

- Read [Core Concepts](./concepts) to understand the data model in depth
- Explore the [CLI reference](/cli/task) for the full set of commands
