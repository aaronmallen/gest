# gest serve

Start a local web server for browsing gest entities in a browser. The server provides a web UI with a dashboard,
task/artifact/iteration views, a kanban board, and full-text search.

## Usage

```text
gest serve [OPTIONS]
```

## Options

| Flag | Description |
| --- | --- |
| `-b, --bind <ADDRESS>` | Address to bind to (overrides config, default `127.0.0.1`) |
| `--port <PORT>` | Port to listen on (overrides config, default `2300`) |
| `--no-open` | Do not automatically open the browser |
| `-v, --verbose` | Increase verbosity (repeatable) |
| `-h, --help` | Print help |

## Views

| Path | Description |
| --- | --- |
| `/` | Dashboard with entity counts and status breakdown |
| `/tasks` | Task list with status, priority, tags, and blocking info |
| `/tasks/:id` | Task detail with description, links, and metadata |
| `/artifacts` | Artifact list with kind, tags, and archive status |
| `/artifacts/:id` | Artifact detail with rendered Markdown body |
| `/iterations` | Iteration list with status and phase count |
| `/iterations/:id` | Iteration detail with tasks grouped by phase |
| `/iterations/:id/board` | Kanban board with columns mapped to task status |
| `/search?q=...` | Full-text search across tasks and artifacts |

## Configuration

The server reads its settings from the `[serve]` section in your config file. CLI flags override config values.

```toml
[serve]
bind_address = "127.0.0.1"
port = 2300
open = true
```

See [Configuration](/configuration/#serve) for the full reference.

## Examples

```sh
# Start with defaults (localhost:2300, auto-open browser)
gest serve

# Custom port, no browser
gest serve --port 8080 --no-open

# Bind to all interfaces
gest serve --bind 0.0.0.0
```
