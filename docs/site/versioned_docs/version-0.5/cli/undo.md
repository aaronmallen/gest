# gest undo

Alias: `gest u`

Undo the most recent mutating command(s) by replaying a transaction log in reverse.

Every mutating CLI command (create, update, archive, link, etc.) is wrapped in a database
transaction whose row-level changes are recorded in the `transactions` and `transaction_events`
tables. `gest undo` reverses the most recent transaction by applying the inverse of each
recorded change. Non-mutating commands (show, list, search, version) are not recorded.

The transaction log lives inside the same SQLite database as your entity data, so undo history
follows the database: local-only if you use the global store, and cross-machine if you point at
a remote libsql database via `[database]`.

## Usage

```text
gest undo [STEPS]
```

## Arguments

| Argument  | Description                               |
|-----------|-------------------------------------------|
| `[STEPS]` | Number of commands to undo (default: `1`) |

## Options

| Flag            | Description                     |
|-----------------|---------------------------------|
| `-v, --verbose` | Increase verbosity (repeatable) |
| `-h, --help`    | Print help                      |

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

Each mutating command runs inside a database transaction. For every row the command inserts,
updates, or deletes, gest captures a `transaction_events` record with the before-state. When
`gest undo` runs, it walks the most recent transaction's events in reverse and applies the
inverse of each:

- **Inserts** become deletes
- **Updates** restore the captured before-row
- **Deletes** re-insert the captured row

Each undo step prints a summary of what was reversed, including the original command and how
long ago it ran.

If you request more undo steps than are available, gest undoes as many as it can and stops
gracefully.

:::tip
The undo command itself is not recorded in the transaction log, so repeated `gest undo` calls
walk backwards through history without creating new entries.
:::
