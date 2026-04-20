---
title: gest serve
description: gest serve — start the built-in web dashboard to browse tasks, artifacts, iterations, and kanban boards while parallel AI agents run.
keywords: [gest serve, web dashboard, kanban board, agentic workflow CLI, parallel AI agents]
---

# gest serve

Alias: `gest s`

Start the built-in web dashboard for browsing and managing your project's tasks, artifacts, and iterations. While
agents interact with gest through the CLI and `--json` output, the dashboard gives humans a visual overview of
everything that's happening — especially useful when multiple agents are working in parallel across an iteration.

The dashboard provides a status overview, filterable task and artifact lists with rendered Markdown, iteration detail
views with tasks grouped by phase, a kanban board for tracking progress, and full-text search across all entities.

## Usage

```text
gest serve [OPTIONS]
```

## Options

| Flag                          | Description                                                                 |
|-------------------------------|-----------------------------------------------------------------------------|
| `-b, --bind <BIND_ADDRESS>`   | Address to bind to (overrides `[serve].bind_address`, default `127.0.0.1`)  |
| `--debounce-ms <DEBOUNCE_MS>` | File watcher debounce in milliseconds (overrides `[serve].debounce_ms`)     |
| `--no-open`                   | Do not automatically open the browser                                       |
| `-p, --port <PORT>`           | Port to listen on (overrides `[serve].port`, default `2300`)                |
| `-v, --verbose`               | Increase verbosity (repeatable)                                             |
| `-h, --help`                  | Print help                                                                  |

## Views

| Path                    | Description                                              |
|-------------------------|----------------------------------------------------------|
| `/`                     | Dashboard with entity counts and status breakdown        |
| `/tasks`                | Task list with status, priority, tags, and blocking info |
| `/tasks/:id`            | Task detail with description, links, and metadata        |
| `/artifacts`            | Artifact list with tags and archive status               |
| `/artifacts/:id`        | Artifact detail with rendered Markdown body              |
| `/iterations`           | Iteration list with status and phase count               |
| `/iterations/:id`       | Iteration detail with tasks grouped by phase             |
| `/iterations/:id/board` | Kanban board with columns mapped to task status          |
| `/search?q=...`         | Full-text search across tasks, artifacts, and iterations |

## Examples

```sh
# Start with defaults (localhost:2300, auto-open browser)
gest serve

# Custom port, no browser
gest serve --port 8080 --no-open

# Bind to all interfaces
gest serve --bind 0.0.0.0
```
