# Configuration

gest uses hierarchical TOML configuration files that are merged together at
runtime. Settings closer to your working directory take precedence over global
defaults, giving you fine-grained control per project.

## Config file locations

gest searches for configuration in two layers: a **global** config and
**project-level** configs.

### Global config

The global config lives at your platform's config home:

| Platform | Path |
| --- | --- |
| Linux | `~/.config/gest/config.toml` |
| macOS | `~/Library/Application Support/gest/config.toml` |

You can override this location with the `GEST_CONFIG` environment variable.

### Project config

Within each directory from the filesystem root down to your working directory,
gest checks for project config files in the following order (first match wins):

1. `.config/gest.toml`
2. `.gest/config.toml`
3. `.gest.toml`

When multiple config files are found at different directory levels, they are
deep-merged with files closer to the working directory taking precedence.

## Data storage: global vs local

gest stores its data (tasks, artifacts, iterations) in a **data directory**.
The data directory is resolved with this precedence:

1. `$GEST_DATA_DIR` environment variable (must be an absolute path)
2. `storage.data_dir` in config (must be an absolute path)
3. A `.gest/` or `gest/` directory found by walking up from the working directory
4. The platform's global data home: `~/.local/share/gest/<hash>/`

### Global store (default)

By default, `gest init` sets up the **global store**. Data is kept under
`~/.local/share/gest/` in a subdirectory derived from a hash of your project
path. This keeps your project directory clean and works well for personal use.

```sh
# Initialize with the global store (default)
gest init
```

### Local store

Use `gest init --local` to create a `.gest/` directory inside your project.
This is useful when you want to commit gest data alongside your code or share
it with collaborators.

```sh
# Initialize with a local .gest/ directory
gest init --local
```

## Per-entity directory overrides

By default, all entity types (artifacts, tasks, iterations) are stored under
the resolved data directory. You can override the storage location for each
entity type independently using environment variables or config settings.

The resolution precedence for each entity type is:

1. Entity-specific environment variable (e.g. `GEST_ARTIFACT_DIR`)
2. Entity-specific config setting (e.g. `storage.artifact_dir`)
3. `<data_dir>/<entity>/` (default fallback)

For example, to keep artifacts in your project's `docs/` directory and tasks in
`tasks/` while letting iterations use the default:

```toml
[storage]
artifact_dir = "./docs"
task_dir = "./tasks"
```

This produces the following layout:

```text
docs/<id>.md
docs/archive/<id>.md
tasks/<id>.toml
tasks/resolved/<id>.toml
<data_dir>/iterations/<id>.toml
<data_dir>/iterations/resolved/<id>.toml
```

`gest init` respects these overrides and creates directories at the resolved
paths.

## Configuration settings

### `[storage]`

| Key | Type | Default | Description |
| --- | --- | --- | --- |
| `data_dir` | string (absolute path) | _(auto-resolved)_ | Override the data directory. Must be an absolute path to an existing directory. |
| `artifact_dir` | string (path) | `<data_dir>/artifacts` | Override the artifact storage directory. |
| `iteration_dir` | string (path) | `<data_dir>/iterations` | Override the iteration storage directory. |
| `task_dir` | string (path) | `<data_dir>/tasks` | Override the task storage directory. |

### `[serve]`

| Key | Type | Default | Description |
| --- | --- | --- | --- |
| `bind_address` | string (IP address) | `"127.0.0.1"` | IP address the web server binds to. |
| `port` | integer | `2300` | Port the web server listens on. |
| `open` | boolean | `true` | Whether to automatically open the browser when the server starts. |

### `[log]`

| Key | Type | Default | Description |
| --- | --- | --- | --- |
| `level` | string | `"warn"` | Log level filter. Valid values: `"error"`, `"warn"`, `"info"`, `"debug"`, `"trace"`. |

### `[colors]`

The `[colors]` section lets you override semantic color tokens used in the UI.
Each key is a dot-delimited token name (e.g. `"log.error"`), and the value is
either a color string or a table with detailed style options.

**Simple form** -- set the foreground color with a string:

```toml
[colors]
"log.error" = "#D23434"
"log.warn" = "yellow"
```

**Table form** -- control foreground, background, and text modifiers:

```toml
[colors.emphasis]
fg = "#9448C7"
bold = true
```

Available fields in the table form:

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `fg` | string | _(none)_ | Foreground color (named color or `#RRGGBB` hex). |
| `bg` | string | _(none)_ | Background color (named color or `#RRGGBB` hex). |
| `bold` | boolean | `false` | Enable bold text. |
| `dim` | boolean | `false` | Enable dim/faint text. |
| `italic` | boolean | `false` | Enable italic text. |
| `underline` | boolean | `false` | Enable underlined text. |

**Supported named colors:** `black`, `red`, `green`, `yellow`, `blue`,
`magenta`, `cyan`, `white`, and their `bright` variants (e.g. `bright cyan`).

## Example config file

```toml
[storage]
data_dir = "/home/user/projects/myapp/.gest"
artifact_dir = "./docs"
task_dir = "./tasks"

[serve]
port = 8080
open = false

[log]
level = "info"

[colors]
"log.error" = "#D23434"
"log.warn" = "yellow"

[colors.emphasis]
fg = "#9448C7"
bold = true
```

## Environment variables

| Variable | Description |
| --- | --- |
| `GEST_CONFIG` | Override the path to the global config file. |
| `GEST_DATA_DIR` | Override the data storage directory (must be an absolute path). |
| `GEST_ARTIFACT_DIR` | Override the artifact storage directory. |
| `GEST_ITERATION_DIR` | Override the iteration storage directory. |
| `GEST_TASK_DIR` | Override the task storage directory. |
| `GEST_LOG_LEVEL` | Override the log level filter (e.g. `debug`, `trace`). Takes precedence over the config file. |
| `VISUAL` | Preferred editor for interactive editing (checked before `EDITOR`). |
| `EDITOR` | Fallback editor for interactive editing. |

## Managing config from the CLI

gest provides subcommands to inspect and modify configuration without editing
files by hand.

### `gest config show`

Displays the merged configuration and the config file sources that were
discovered:

```sh
gest config show
```

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
