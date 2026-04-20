---
title: gest init
description: gest init — register a project with gest in the global SQLite database, or pass --local to mirror data into a .gest/ directory for version control.
keywords: [gest init, initialize gest project, SQLite setup, local sync, agentic workflow CLI]
---

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

## Exit Codes

| Code | When                                                |
|------|-----------------------------------------------------|
| 0    | Success                                             |
| 64   | Bad flags                                           |
| 74   | Could not create the database or `.gest/` mirror    |
| 78   | Could not load user config or resolve `data_dir`    |

See [Exit Codes](./exit-codes.md) for the full contract.

## See also

- [Configuration](../configuration/index.md) — how `data_dir`, `storage.sync`, and the
  `[database]` section interact
- [FAQ: global vs local stores](../faq.md#whats-the-difference-between-global-and-local-stores)
