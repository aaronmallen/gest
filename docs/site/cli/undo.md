# gest undo

Undo the most recent mutating command(s) by restoring file snapshots.

Every mutating CLI command (create, update, archive, link, etc.) is automatically recorded in a
local event store. `gest undo` reverses the most recent operation by restoring files to their
prior state. Non-mutating commands (show, list, search, version) are not recorded.

The event store lives outside version control in the system state directory
(`~/.local/state/gest/<project-hash>/`), so undo history is local to each machine.

## Usage

```text
gest undo [STEPS]
```

## Arguments

| Argument | Description |
| --- | --- |
| `[STEPS]` | Number of commands to undo (default: `1`) |

## Options

| Flag | Description |
| --- | --- |
| `-v, --verbose` | Increase verbosity (repeatable) |
| `-h, --help` | Print help |

## Examples

Undo the last command:

```sh
gest undo
```

Undo the last 3 commands:

```sh
gest undo 3
```

## How it works

Before each mutating command runs, gest snapshots all files in the data directory. After the
command completes, any files that changed are recorded as events grouped under a single
transaction. `gest undo` reverses the most recent transaction:

- **Created files** are deleted
- **Modified files** are restored to their prior content
- **Deleted files** are recreated

Each undo step prints a summary of what was reversed, including the original command and how
long ago it ran.

If you request more undo steps than are available, gest undoes as many as it can and stops
gracefully.

::: tip
The undo command itself is not recorded in the event store, so repeated `gest undo` calls walk
backwards through history without creating new entries.
:::
