# gest purge

Purge terminal, archived, and orphaned data from the store.

By default operates on the current project. Pass `--all-projects` to sweep
the entire store. When no selector flags are given, all selectors are applied
(the confirmation prompt is the safety net).

## Usage

```text
gest purge [OPTIONS]
```

## Selectors

Each flag enables a specific category of data to purge. When none are given,
all categories are selected.

| Flag              | Description                                      |
|-------------------|--------------------------------------------------|
| `--tasks`         | Purge terminal tasks (done/cancelled)            |
| `--iterations`    | Purge terminal iterations (completed/cancelled)  |
| `--artifacts`     | Purge archived artifacts                         |
| `--projects`      | Purge archived projects (non-undoable)           |
| `--relationships` | Purge dangling relationship rows                 |
| `--tombstones`    | Purge orphan tombstone files                     |

## Options

| Flag              | Description                                                  |
|-------------------|--------------------------------------------------------------|
| `--all-projects`  | Operate across every project instead of just the current one |
| `--dry-run`       | Show what would be purged without making any changes         |
| `--yes`           | Skip the interactive confirmation prompt                     |

## Examples

Preview what would be purged in the current project:

```sh
gest purge --dry-run
```

Purge all categories with confirmation:

```sh
gest purge
```

Purge only terminal tasks and iterations, skipping the prompt:

```sh
gest purge --tasks --iterations --yes
```

Purge across all projects:

```sh
gest purge --all-projects
```

Purge only orphan tombstone files:

```sh
gest purge --tombstones --yes
```

## Exit Codes

| Code | When                                                   |
|------|--------------------------------------------------------|
| 0    | Success (including dry-run with no-op result)          |
| 64   | Bad flags                                              |
| 66   | A target project id did not resolve                    |
| 74   | Store I/O error                                        |
| 78   | Not a gest project (run `gest init`)                   |

See [Exit Codes](./exit-codes.md) for the full contract.

## Notes

- Archived-project deletions use the cascade delete path and are **not**
  reversible via `gest undo`. All other purge operations are undoable.
- When no selector flags are given, all selectors are applied.
- `--dry-run` prints the summary and exits before any prompt or mutation.
