# Gest

[![Build][build-badge]][build-link]
[![Crates.io Version][crates-badge]][crates-link]
[![GitHub Sponsors][sponsor-badge]][sponsor-link]
[![Discord][discord-badge]][discord-link]

> [!WARNING]
> Gest is pre-1.0; expect breaking changes between minor versions until 1.0.

Gest is a lightweight task and artifact tracker built for AI-assisted development. It structures agent-generated work
into **phased iterations with dependency tracking**, so independent tasks can be dispatched to separate agents
concurrently — turning sequential, single-context-window workflows into parallel execution pipelines.

## Quick Start

```sh
gest init                                      # register the current repo as a project
gest task create "Implement auth middleware"
gest artifact create --source auth-spec.md
gest iteration create "Ship auth"
gest search "auth"
```

Open the web dashboard at any time with `gest serve` (defaults to <http://localhost:2300>).

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
unblocked tasks via `gest iteration next --claim`, and mark them done — remaining work automatically unblocks.

### Sharing a project across worktrees

When dispatching parallel agents into separate git worktrees or jj workspaces, attach each workspace to the same
project so all agents read and write the same task and artifact data:

```sh
# In the main checkout, capture the project ID:
gest project
# → wrolrrvn

# In each new worktree, attach to that project:
cd ../gest-feature-branch
gest project attach wrolrrvn

# When the worktree is done, detach before removing it:
gest project detach
```

## How it works

Gest keeps your tasks, artifacts, and iterations in a local SQLite database that powers fast queries, full-text
search, undo, and the web dashboard. Projects can additionally keep a `.gest/` mirror of that data as human-readable
YAML and Markdown files so it can travel with your code through git — run `gest init --local` to opt in. The on-disk
mirror is imported on start and exported on exit, keeping both representations in sync without manual steps.

## Installation

### Quick install (macOS and Linux)

```sh
curl -fsSL https://gest.aaronmallen.dev/install | sh
```

> [!TIP]
> This installs `gest` to `~/.local/bin`. Make sure it's in your `PATH`:
>
> ```sh
> export PATH="$HOME/.local/bin:$PATH"
> ```

Pin a specific version or change the install directory:

```sh
GEST_VERSION=0.5.0 GEST_INSTALL_PATH=/usr/local/bin \
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

### Upgrading from v0.4.x

v0.5.0 replaced the file-backed store with SQLite. Run `gest migrate` once after upgrading; see the
[v0.4 → v0.5 migration guide](https://gest.aaronmallen.dev/migration/v0-4-to-v0-5) for details.

## Documentation

Full documentation, guides, and reference are available at <https://gest.aaronmallen.dev>.

## Status

Pre-1.0, under active development. See [docs] for design documents and process guides.

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
