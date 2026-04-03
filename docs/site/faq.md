# FAQ

## What is gest?

Gest is a CLI tool for tracking tasks and artifacts generated during AI-assisted development.
It stores data as plain files (TOML for tasks, Markdown with YAML frontmatter for artifacts)
so everything is inspectable, portable, and version-control friendly. See the
[quick start](/getting-started/quick-start) to get up and running.

## How does gest differ from GitHub Issues or Jira?

Gest is designed for fast, local, developer-centric tracking -- not team-wide project management. Key differences:

- **No server or network required.** Everything lives on disk as plain files.
- **Optimized for AI workflows.** Tasks and artifacts map to the outputs that AI agents produce during development.
- **Parallel execution.** Phased iterations with dependency tracking let agents work concurrently across workspaces.
- **Built-in web dashboard.** `gest serve` gives you a visual overview, kanban boards, and full-text search
  — no external tools needed.
- **Scriptable.** JSON output (`--json`) and quiet mode (`-q`) on all commands make it easy to integrate with other
  tools.

Gest complements issue trackers rather than replacing them. Use `gest` for in-flight
development work and promote items to GitHub Issues when they need broader visibility.

## Where is my data stored?

By default, gest uses a project-specific subdirectory under the global data root at
`~/.local/share/gest/<project-hash>/`. If you initialize with `--local`, data goes into a
`.gest/` directory inside your project. Run `gest config show` to see where your resolved
project directory is.

The data directory contains three subdirectories:

```text
artifacts/        # Markdown files with YAML frontmatter
  archive/        # Archived artifacts
tasks/            # TOML files
  resolved/       # Done or cancelled tasks
iterations/       # TOML files
  resolved/       # Completed or cancelled iterations
```

## What's the difference between global and local stores?

**Global** (`~/.local/share/gest/<project-hash>/`):

- Created with `gest init`
- Not shared via VCS
- Best for personal task tracking across projects

**Local** (`.gest/`):

- Created with `gest init --local`
- Shared via VCS if you commit `.gest/`
- Best for project-specific tasks you want to share or version

When a local `.gest/` directory exists, gest uses it automatically. Otherwise it falls back to the global store.

## Can I use gest with CI/CD?

Yes. Because data is plain files and every list/search command supports `--json`, you can
script gest in CI pipelines. For example:

```sh
# Fail the build if there are open tasks tagged "blocker"
if gest task list --json | jq -e '.[] | select(.tags[] == "blocker")' > /dev/null 2>&1; then
  echo "Blocker tasks remain"; exit 1
fi
```

Use `gest init --local` so the `.gest/` directory is available in your CI checkout.

## How do I move data between global and local stores?

Gest does not have a built-in migration command yet. Since data is plain files, you can copy them directly:

```sh
# Global to local (use `gest config show` to find your project directory)
cp -r ~/.local/share/gest/<project-hash>/ .gest/

# Local to global
cp -r .gest/ ~/.local/share/gest/<project-hash>/
```

Be careful not to overwrite files with the same ID in the destination.

## What file formats does gest use?

- **Tasks** are stored as TOML files (e.g., `tasks/abc123.toml`).
- **Artifacts** are stored as Markdown files with YAML frontmatter (e.g., `artifacts/def456.md`).
- **Iterations** are stored as TOML files (e.g., `iterations/ghi789.toml`).
- **Configuration** supports TOML, JSON, and YAML. See [configuration](/configuration/) for details.

All formats are human-readable and editable with any text editor.

## How does search work?

`gest search <QUERY>` performs a case-insensitive text match against titles, descriptions, and
body content across both tasks and artifacts. See [gest search](/cli/search) for the full
reference.

Useful flags:

- `--expand` / `-e` -- show full detail for each result.
- `--json` / `-j` -- emit results as JSON for scripting.
- `--all` / `-a` -- include archived and resolved items (excluded by default).

## Are there shortcuts for common task operations?

Yes. Gest provides shortcut commands for frequent status changes:

- `gest task complete <id>` — mark a task as done (shortcut for `task update <id> --status done`)
- `gest task cancel <id>` — cancel a task (shortcut for `task update <id> --status cancelled`)
- `gest task block <id> <other-id>` — mark one task as blocking another
  (shortcut for `task link <id> blocks <other-id>`)

All shortcuts support `--json` and `-q` for machine-readable output.

## How do I get machine-readable output?

Most commands support `--json` for structured JSON output. Mutation commands also support `-q` / `--quiet` to print
only the entity ID, which is useful for scripting:

```sh
task_id=$(gest task create "My task" -q)
gest task complete "$task_id" -q
```

Read commands like `meta get` support `--json` and `--raw` (bare value, no styling).

## Can multiple people use gest on the same project?

Yes, if you use a local store (`gest init --local`) and commit the `.gest/` directory. Each
person's changes to task and artifact files merge through your normal VCS workflow. Because
files are keyed by unique IDs, merge conflicts are rare -- they only happen when two people
edit the same entity concurrently.

## How do I back up my gest data?

For a **local store**, your VCS handles it -- just commit the `.gest/` directory.

For the **global store**, back up `~/.local/share/gest/`. Since it is plain files, any backup
tool (rsync, Time Machine, etc.) works.

## Does gest have a web UI?

Yes. Run `gest serve` to start a local web dashboard at `http://localhost:2300`. It provides a
status overview, task and artifact views with rendered Markdown, iteration detail with tasks
grouped by phase, a kanban board for tracking progress, and full-text search. The dashboard
reads and writes the same plain-file store the CLI uses — no separate database or sync layer.
See [gest serve](/cli/serve) for options.

## What are iterations and when should I use them?

Iterations group related tasks into an execution plan with ordered phases. They are useful
when you have a set of tasks that should be completed together in a specific sequence -- for
example, implementing a feature that spans multiple files or steps.

Create an iteration when:

- You have several related tasks with dependencies between them.
- You want to track progress across a multi-step plan.
- You are coordinating AI agent execution across phases.

For standalone tasks with no particular ordering, plain tasks are sufficient.

## How do I update gest?

Run the built-in self-update command:

```sh
gest self-update
```

This downloads the latest release from GitHub. You can also reinstall via Cargo:

```sh
cargo install gest
```

Or use the install script with a pinned version:

```sh
GEST_VERSION=0.x.x curl -fsSL https://gest.aaronmallen.dev/install | sh
```
