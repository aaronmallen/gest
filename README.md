# Gest

[![Build][build-badge]][build-link]
[![Crates.io Version][crates-badge]][crates-link]
[![GitHub Sponsors][sponsor-badge]][sponsor-link]
[![Static Badge][discord-badge]][discord-link]

Manage agent-generated artifacts and task backlogs alongside your project.

> [!WARNING]
> Gest is in early development. Commands, file formats, and configuration may change without notice between releases.

Gest is a lightweight task and artifact tracker built for AI-assisted development. It structures agent-generated work
into **phased iterations with dependency tracking**, so independent tasks can be dispatched to separate agents
concurrently — turning sequential, single-context-window workflows into parallel execution pipelines.

Data is stored as plain files — TOML for tasks, Markdown with YAML frontmatter for artifacts — right inside your repo.
No database, no server, no accounts.

Full documentation is available at <https://gest.aaronmallen.dev>.

## Quick Start

```sh
gest init # initialize global store
gest task create "Implement auth middleware"
gest artifact create --source auth-spec.md
gest search "auth"
```

Use `gest init --local` to store data inside the repo (`.gest/`) instead of the global data directory.

## Parallel Execution

The core problem gest solves: an AI agent produces a plan with 15-20 tasks, and you execute them one at a time in one
context window. That's slow. Gest lets you decompose work into **phased iterations** where tasks in the same phase run
concurrently:

```sh
# Phase 1 tasks run in parallel — no dependencies between them
gest task create "Add export data types" --phase 1
gest task create "Add CSV formatter" --phase 1
gest task create "Add JSON formatter" --phase 1

# Phase 2 waits for Phase 1
gest task create "Wire up CLI command" --phase 2

# Group into an iteration and visualize the plan
gest iteration create "Implement export command"
gest iteration add <iteration-id> <task-id>   # repeat for each task
gest iteration graph <iteration-id>
```

The iteration graph shows exactly which tasks can run now and which are waiting. Agents read the graph, pick up
unblocked tasks, and mark them done — the remaining work automatically unblocks.

### Agent Orchestration

Built-in orchestration commands let multiple agents claim and execute tasks concurrently without conflicts:

```sh
gest iteration list --has-available            # find iterations with claimable work
gest iteration next <id> --claim --agent bot1  # atomically claim the next task
gest iteration status <id> --json              # check progress across all phases
gest iteration advance <id>                    # move to the next phase
```

`iteration next` exits with code 2 when no tasks remain, so scripts can distinguish "idle" from "error".

## Web Dashboard

Gest includes a built-in web dashboard for browsing and managing your project's tasks, artifacts, and iterations
without leaving the browser. Run `gest serve` to start it:

```sh
gest serve                    # opens http://localhost:2300
```

The dashboard provides:

- **Status overview** — entity counts and status breakdown at a glance
- **Task and artifact views** — filter, search, and inspect with rendered Markdown
- **Iteration detail** — tasks grouped by phase with dependency visualization
- **Kanban board** — drag-and-drop columns mapped to task status
- **Full-text search** — find anything across tasks and artifacts

## Commands

| Command             | Description                                                    |
|---------------------|----------------------------------------------------------------|
| `gest init`         | Initialize gest (`--local` for in-repo `.gest/` directory)     |
| `gest task`         | Create, list, show, update, tag, link, and manage tasks        |
| `gest artifact`     | Create, list, show, update, tag, archive, and manage artifacts |
| `gest iteration`    | Manage iterations (group tasks into phased execution plans)    |
| `gest tag`          | Add, remove, and list tags across all entity types             |
| `gest search`       | Search across tasks and artifacts                              |
| `gest undo`         | Undo the most recent mutating command(s)                       |
| `gest serve`        | Start the web dashboard for browsing and managing entities     |
| `gest config`       | View and modify configuration                                  |
| `gest generate`     | Generate shell completions and man pages                       |
| `gest self-update`  | Update gest to the latest GitHub release                       |
| `gest version`      | Print version and check for updates                            |

Run `gest --help` or `gest <command> --help` for full details.

## Configuration

Gest loads configuration from global (`~/.config/gest/`) and project-level TOML files. Run `gest config show` to see
the resolved configuration and its source files.

## Installation

### Quick Install (macOS and Linux)

```sh
curl -fsSL https://gest.aaronmallen.dev/install | sh
```

> [!TIP]
> This installs `gest` to `~/.local/bin`. Make sure it's in your `PATH`:
>
>```sh
>export PATH="$HOME/.local/bin:$PATH"
>```

Pin a specific version or change the install directory:

```sh
GEST_VERSION=0.0.1 GEST_INSTALL_PATH=/usr/local/bin \
  curl -fsSL https://gest.aaronmallen.dev/install | sh
```

### Cargo

```sh
cargo install gest
```

Or with [cargo-binstall] for a pre-built binary:

```sh
cargo binstall gest
```

## Status

Early development. See [docs] for design documents and process guides.

## License

[MIT]

[build-badge]:
  https://img.shields.io/github/actions/workflow/status/aaronmallen/gest/build.yml?branch=main&style=for-the-badge
[build-link]: https://github.com/aaronmallen/gest/actions/workflows/build.yml
[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall
[crates-badge]: https://img.shields.io/crates/v/gest?style=for-the-badge
[crates-link]: https://crates.io/crates/gest
[discord-badge]:
  https://img.shields.io/discord/1441938388780585062?style=for-the-badge&logo=discord&logoColor=white&label=Discord&labelColor=%235865F2
[discord-link]: https://discord.gg/PqQdhf9VMF
[docs]: https://github.com/aaronmallen/gest/tree/main/docs
[MIT]: https://github.com/aaronmallen/gest/blob/main/LICENSE
[sponsor-badge]: https://img.shields.io/github/sponsors/aaronmallen?style=for-the-badge
[sponsor-link]: https://github.com/sponsors/aaronmallen
