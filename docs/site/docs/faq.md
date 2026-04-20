---
title: FAQ
description: Frequently asked questions about gest — the agentic workflow CLI for parallel AI agents. How it differs from Jira and GitHub Issues, agent compatibility, and more.
keywords: [gest FAQ, agentic workflow CLI, AI agent task tracker, AI-assisted development CLI, Claude Code task manager]
---

# FAQ

## What is gest?

Gest is a CLI tool for tracking tasks and artifacts generated during AI-assisted development.
It stores data in a local SQLite database (via libsql) with an optional sync layer that
mirrors everything to a `.gest/` directory as YAML and Markdown files so your data stays
inspectable, portable, and version-control friendly. See the
[quick start](./getting-started/quick-start.md) to get up and running.

## How does gest differ from GitHub Issues or Jira?

Gest is designed for fast, local, developer-centric tracking -- not team-wide project management. Key differences:

- **No server or network required.** Everything lives in a local SQLite database; remote sync is opt-in via
  `[database]`.
- **Optimized for AI workflows.** Tasks and artifacts map to the outputs that AI agents produce during development.
- **Parallel execution.** Phased iterations with dependency tracking let agents work concurrently across workspaces.
- **Built-in web dashboard.** `gest serve` gives you a visual overview, kanban boards, and full-text search
  — no external tools needed.
- **Scriptable.** JSON output (`--json`) and quiet mode (`-q`) on all commands make it easy to integrate with other
  tools.

Gest complements issue trackers rather than replacing them. Use `gest` for in-flight
development work and promote items to GitHub Issues when they need broader visibility.

## Where is my data stored?

Entity data lives in a SQLite database at `<data_dir>/gest.db` — by default
`~/.local/share/gest/gest.db`. Projects are rows inside that database rather than
separate subdirectories. Run `gest config show` to see the resolved data dir.

If you initialized with `--local`, a `.gest/` directory is also created inside your
project. When `storage.sync` is enabled (the default), gest bidirectionally syncs
the database with YAML and Markdown files in `.gest/` on every invocation — so you
can commit the mirror alongside your code and still have inspectable, diff-friendly
history.

## What's the difference between global and local stores?

**Global only** (`gest init`):

- One SQLite database at `~/.local/share/gest/gest.db` shared across all projects on the machine
- Not shared via VCS
- Best for personal task tracking across projects

**Local sync** (`gest init --local`):

- Same SQLite database, plus a `.gest/` directory inside your project
- The sync layer mirrors the database to `.gest/` as YAML/Markdown files so the data is version-controlled
- Best for project-specific tasks you want to share or track in git

The database is always the source of truth. The `.gest/` mirror is rewritten from
the database on export and re-imported when files change on disk.

## Can I use gest with CI/CD?

Yes. Every list/search command supports `--json`, so you can script gest in CI
pipelines. For example:

```sh
# Fail the build if there are open tasks tagged "blocker"
if gest task list --json | jq -e '.[] | select(.tags[] == "blocker")' > /dev/null 2>&1; then
  echo "Blocker tasks remain"; exit 1
fi
```

Use `gest init --local` so the `.gest/` mirror is available in your CI checkout —
gest will import it into a fresh SQLite database on first run.

## How do I move data between machines?

Three options, depending on what you need:

1. **Local sync (`gest init --local`)** — commit the `.gest/` directory. On another machine,
   checkout the repo and run any `gest` command; the sync layer imports the mirror into that
   machine's database automatically.
2. **Remote libsql database** — set `database.url` and `database.auth_token` in your config to
   point at a shared libsql instance. Every machine that uses the same URL sees the same data.
3. **Manual SQLite copy** — `gest.db` is a regular SQLite file, so you can `rsync` or `cp` it
   between machines when gest is not running.

## How do I migrate from v0.4.x flat files?

Run `gest migrate --from v0.4`. It walks your existing `.gest/` directory, reads the old
TOML/Markdown files, and imports everything into the new SQLite database. For a
step-by-step walkthrough — pre-migration checklist, data mapping table,
post-migration verification, and rollback — see the
[v0.4 → v0.5 migration guide](/docs/0.5/migration/v0-4-to-v0-5). The short-form CLI
reference lives at [gest migrate](./cli/migrate.md).

## What does the `.gest/` sync mirror look like?

When `storage.sync` is enabled and a `.gest/` directory exists in the project root, gest
writes a per-entity layout with one file per row, grouped into singular subdirectories:

```text
.gest/
├── project.yaml
├── artifact/       # <id>.md (body + YAML frontmatter)
│   └── notes/      # <note_id>.yaml
├── author/         # <id>.yaml
├── event/          # <yyyy-mm>/<id>.yaml (sharded by month)
├── iteration/      # <id>.yaml
├── relationship/   # <id>.yaml
├── tag/            # <id>.yaml
└── task/           # <id>.yaml
    └── notes/      # <note_id>.yaml
```

Most files are YAML; only artifact bodies are Markdown with YAML frontmatter. This mirror
is regenerated from the database on every command that mutates state and re-imported on
every command that reads state (if the files are newer than the database row).

Configuration files (`.config/gest.toml`, `.gest.toml`, and the global config) are plain
TOML and are loaded from the config search paths described in
[Configuration](./configuration/index.md) — they are never stored in the database, and gest does
**not** look for a config file inside `.gest/`.

## How does search work?

`gest search <QUERY>` performs a case-insensitive text match against titles, descriptions, and
body content across tasks, artifacts, and iterations. See [gest search](./cli/search.md) for the
full reference.

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

Yes. Two patterns work:

1. **Local sync via git** — run `gest init --local` and commit the `.gest/` directory. Each
   person's database imports and exports through the sync mirror, and merge conflicts only
   happen when two people edit the same entity at the same time. Files are keyed by unique IDs
   so unrelated edits rarely collide.
2. **Shared remote database** — point `database.url` at a shared libsql instance. Everyone
   writes to the same backing store in real time; no mirror, no merge.

## How do I back up my gest data?

For a **local sync mirror**, your VCS handles it — just commit the `.gest/` directory.

For the **database itself**, back up `<data_dir>/gest.db`. It is a standard SQLite file,
so any backup tool (rsync, Time Machine, `sqlite3 .backup`, etc.) works. If you use a remote
libsql database, back it up via whatever your hosting provider offers.

## Does gest have a web UI?

Yes. Run `gest serve` to start a local web dashboard at `http://localhost:2300`. It provides a
status overview, task and artifact views with rendered Markdown, iteration detail with tasks
grouped by phase, a kanban board for tracking progress, and full-text search. The dashboard
reads and writes the same SQLite database the CLI uses, so changes made on the command line
show up in the dashboard immediately (and vice versa). See [gest serve](./cli/serve.md) for options.

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
