# Configuration

gest uses hierarchical TOML configuration files that are merged together at
runtime. Settings closer to your working directory take precedence over global
defaults, giving you fine-grained control per project.

## Config file locations

gest searches for configuration in two layers: a **global** config and
**project-level** configs.

### Global config

The global config lives at your platform's config home:

| Platform | Path                                             |
|----------|--------------------------------------------------|
| Linux    | `~/.config/gest/config.toml`                     |
| macOS    | `~/Library/Application Support/gest/config.toml` |

You can override this location with the `GEST_CONFIG` environment variable.

### Project config

Within each directory from the filesystem root down to your working directory,
gest checks for project config files in the following order:

1. `.config/gest.toml`
2. `.gest.toml`

Both paths are checked at every directory level from the filesystem root down to
the current working directory, and all matching files are deep-merged with files
closer to the working directory taking precedence.

## Data storage: global vs local

gest stores entity data (tasks, artifacts, iterations, notes, events, and
relationships) in a single SQLite database at `<data_dir>/gest.db`. Projects are
rows inside that database, not separate directories on disk.

### Global data root

The global data root is the directory that contains `gest.db`. It is resolved with
this precedence:

1. `$GEST_STORAGE__DATA_DIR` environment variable (must be an absolute path)
2. `storage.data_dir` in config (must be an absolute path)
3. The platform's global data home: `~/.local/share/gest/`

### Project resolution

Projects are tracked in the `projects` table of the database, keyed on their root
path. `gest init` records a new project row for the current working directory; after
that, any `gest` invocation inside the tree walks up from `cwd` and matches against
the `projects` table to find the active project.

There is no longer a separate "project directory" setting — entity data lives
inside the single SQLite database, not in per-project subdirectories.

### Global store (default)

By default, `gest init` records the project with no `.gest/` directory. All entity
data lives in the global SQLite database at `<data_dir>/gest.db`, which is shared
across every project on the machine. This keeps your project directory clean and
works well for personal use.

```sh
# Initialize a project that uses the global store only (default)
gest init
```

### Local store (sync mirror)

Use `gest init --local` to also create a `.gest/` directory inside your project.
When this directory exists and `storage.sync` is enabled (the default), gest
bidirectionally syncs the SQLite database with JSON/markdown files inside `.gest/`
on every command invocation. This is useful when you want to commit gest data
alongside your code or share it with collaborators — but the database is still the
source of truth, not the files.

```sh
# Initialize with a local .gest/ sync mirror
gest init --local
```

## State storage

There is no separate state directory in v0.5.0. The undo log, transaction history,
sync digests, and every other piece of operational state live inside the main
SQLite database at `<data_dir>/gest.db`. Undo history is local to each database —
if you point `gest` at a remote libsql URL via `[database]`, the undo log follows
the database across machines.

## Configuration settings

For a dedicated guide to terminal UI color customization, see
[Theming](./theming.md).

### `[storage]`

| Key         | Type                   | Default           | Description                                                                                                                                                                               |
|-------------|------------------------|-------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `cache_dir` | string (absolute path) | _(auto-resolved)_ | Override the global cache directory used for regeneratable artifacts such as the avatar cache. Must be an absolute path. Defaults to `$XDG_CACHE_HOME/gest` (`~/.cache/gest` when unset). |
| `data_dir`  | string (absolute path) | _(auto-resolved)_ | Override the global data root directory. Must be an absolute path.                                                                                                                        |
| `sync`      | boolean                | `true`            | Enable bidirectional sync between the SQLite database and a `.gest/` directory when one exists in the project root.                                                                       |

### `[database]`

gest v0.5.0 stores entity data in a SQLite database (via libsql). By default the database lives at
`<data_dir>/gest.db`. For multi-device sync, you can point at a remote libsql database instead.

A connection can be provided either as a complete `url` or as individual components
(`scheme`, `host`, `port`, `username`, `password`). When both a `url` and components
are present, the explicit `url` takes precedence. When only components are given, a URL
is assembled in the form `scheme://[user[:pass]@]host[:port]`, with `scheme` defaulting
to `sqlite`.

| Key          | Type   | Default  | Description                                                                     |
|--------------|--------|----------|---------------------------------------------------------------------------------|
| `auth_token` | string | _none_   | Bearer or API token for the remote database.                                    |
| `host`       | string | _none_   | Database server hostname or IP address (used to synthesize a URL).              |
| `password`   | string | _none_   | Password for username/password authentication.                                  |
| `port`       | integer| _none_   | Port number the database server listens on.                                     |
| `scheme`     | string | `sqlite` | Connection scheme used when synthesizing a URL from components.                 |
| `url`        | string | _none_   | Explicit connection URL. When set, component fields are ignored.                |
| `username`   | string | _none_   | Username for database authentication.                                           |

### `[serve]`

Controls the built-in web dashboard (`gest serve`). CLI flags on `gest serve` override
these values at runtime.

| Key            | Type    | Default       | Description                                                                      |
|----------------|---------|---------------|----------------------------------------------------------------------------------|
| `bind_address` | string  | `127.0.0.1`   | IP address the server should bind to.                                            |
| `debounce_ms`  | integer | `2000`        | File watcher debounce window in milliseconds.                                    |
| `log_level`    | string  | _none_        | Optional log level override applied only while `gest serve` is running.          |
| `open`         | boolean | `true`        | Whether to automatically open the browser when the server starts.                |
| `port`         | integer | `2300`        | Port the server should listen on.                                                |

### `[log]`

| Key     | Type   | Default  | Description                                                                          |
|---------|--------|----------|--------------------------------------------------------------------------------------|
| `level` | string | `"warn"` | Log level filter. Valid values: `"error"`, `"warn"`, `"info"`, `"debug"`, `"trace"`. |

### `[pager]`

| Key       | Type    | Default | Description                                                                                                     |
|-----------|---------|---------|-----------------------------------------------------------------------------------------------------------------|
| `enabled` | boolean | `true`  | Whether long output is piped through a pager. Set to `false` to always print directly to stdout.                |
| `command` | string  | _none_  | Optional pager command override (e.g. `"less -FR"`). An empty string is treated as unset and disables override. |

### `[colors]`

The `[colors]` section controls terminal UI colors through palette slots and
per-token overrides. See the [Theming](./theming.md) guide for the
full reference, including palette slots, token overrides, color formats, and
the complete token list.

## Example config file

```toml
[storage]
data_dir = "/home/user/.local/share/gest"
sync = true

[database]
# Optional: point at a remote libsql database for multi-device sync.
# url = "libsql://my-project.turso.io"
# auth_token = "eyJhbGci..."

[log]
level = "info"

[colors.palette]
primary = "#5AB0FF"

[colors.overrides]
"log.error" = "#D23434"
```

## Environment variables

gest recognizes the following environment variables. They take precedence over
values from config files.

| Variable                  | Description                                                                                   |
|---------------------------|-----------------------------------------------------------------------------------------------|
| `GEST_CONFIG`             | Override the path to the global config file.                                                  |
| `GEST_LOG__LEVEL`         | Override the log level filter (e.g. `debug`, `trace`). Takes precedence over the config file. |
| `GEST_STORAGE__CACHE_DIR` | Override the global cache directory for regeneratable artifacts (must be an absolute path).   |
| `GEST_STORAGE__DATA_DIR`  | Override the global data root directory (must be an absolute path).                           |
| `GEST_STORAGE__SYNC`      | Enable or disable bidirectional sync to `.gest/` (`true`/`false`).                            |
| `VISUAL`                  | Preferred editor for interactive editing (checked before `EDITOR`).                           |
| `EDITOR`                  | Fallback editor for interactive editing.                                                      |
| `PAGER`                   | Preferred pager program (falls back to `less`).                                               |

## Managing config from the CLI

gest provides subcommands to inspect and modify configuration without editing
files by hand.

### `gest config show`

Displays the merged configuration and the config file sources that were
discovered:

```sh
gest config show
```

#### Options

| Flag          | Description                              |
|---------------|------------------------------------------|
| `-j, --json`  | Emit output as JSON                      |
| `-q, --quiet` | Suppress normal output                   |
| `-r, --raw`   | Emit script-friendly plain output        |

### `gest config get <KEY>`

Retrieve a single value by its dot-delimited key:

```sh
gest config get storage.data_dir
gest config get log.level
```

### `gest config set <KEY> <VALUE>`

Persist a value to the project config file (or use `--global` for the global
config):

```sh
# Set in the project config
gest config set log.level debug

# Set in the global config
gest config set --global log.level warn
```
