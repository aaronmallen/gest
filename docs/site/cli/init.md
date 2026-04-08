# gest init

Initialize gest for the current directory. This registers a project row in the global
SQLite database (`<data_dir>/gest.db`, typically `~/.local/share/gest/gest.db`) keyed on
the current working directory, so any `gest` command run inside the tree resolves to the
same project.

By default, no per-project files are created on disk — your entity data lives in the
shared global database alongside every other project on the machine. Pass `--local` to
also materialize a `.gest/` directory inside the current project; when a `.gest/`
directory exists and `storage.sync` is enabled (the default), gest bidirectionally syncs
the database with YAML and Markdown files inside `.gest/` on every invocation.

## Usage

```text
gest init [OPTIONS]
```

## Options

| Flag            | Description                                                                                                |
|-----------------|------------------------------------------------------------------------------------------------------------|
| `--local`       | Also create a `.gest/` directory in the current project to materialize the sync mirror alongside your code |
| `-v, --verbose` | Increase verbosity (repeatable)                                                                            |
| `-h, --help`    | Print help                                                                                                 |

## Examples

Initialize with global storage only (default):

```sh
gest init
```

Initialize with a local `.gest/` sync mirror:

```sh
gest init --local
```

## See also

- [Configuration](/configuration/) — how `data_dir`, `storage.sync`, and the
  `[database]` section interact
- [FAQ: global vs local stores](/faq#whats-the-difference-between-global-and-local-stores)
