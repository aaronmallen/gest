---
title: Exit Codes
description: The BSD sysexits.h contract that gest CLI commands use so scripts and agents can branch on error category via $? alone.
keywords: [gest exit codes, sysexits, CLI error codes, scripting, shell exit code]
---

# Exit Codes

Every `gest` command returns a process exit code drawn from the BSD
[`sysexits.h`](https://man.openbsd.org/sysexits.3) taxonomy. Scripts and agents
can branch on the category of failure via `$?` alone, without parsing stderr.

The per-command pages list the codes each command can emit. Codes and their
meanings are defined once here.

## Contract

| Code | Name             | Meaning                                          |
|------|------------------|--------------------------------------------------|
| 0    | —                | Success                                          |
| 64   | `EX_USAGE`       | Command-line usage error                         |
| 65   | `EX_DATAERR`     | User data format error                           |
| 66   | `EX_NOINPUT`     | Input entity not found                           |
| 69   | `EX_UNAVAILABLE` | Resource not in the required state               |
| 70   | `EX_SOFTWARE`    | Internal software error                          |
| 74   | `EX_IOERR`       | Filesystem or database I/O error                 |
| 75   | `EX_TEMPFAIL`    | Try again later (valid empty result)             |
| 78   | `EX_CONFIG`      | Configuration or setup error                     |

Source of truth: [ADR-0017](https://github.com/aaronmallen/gest/blob/main/docs/design/0017-exit-code-contract.md).

## Code reference

### 0 — Success

The command completed as requested. Mutations were applied, output was
printed, and no error path was taken.

### 64 — `EX_USAGE`

An application-level command-line usage error occurred after parsing
succeeded: an argument is missing, malformed, or incompatible with another
flag. Typical causes:

- A positional ID argument was omitted where required.
- Batch input (NDJSON) that fails to parse as JSON per record.
- A relationship spec like `<rel>:<target>` that cannot be parsed.
- `--agent` was passed to `iteration next` without `--claim`.

Note: clap/parser-level failures — an unknown flag, missing required
argument, or invalid subcommand — exit with clap's own default parse-error
code (typically `2`), not `EX_USAGE`. Those failures are outside the
sysexits contract.

### 65 — `EX_DATAERR`

Structured data (JSON/TOML) could not be serialized or deserialized outside
store/database I/O handling. Typical causes:

- A store row contains a value that fails to serialize to JSON for `--json`
  output.
- A non-configuration TOML file (for example, metadata) is syntactically
  invalid.

Invalid configuration files route through `Error::Config` and surface as
`EX_CONFIG` (78), not `EX_DATAERR`. Store-layer decode failures (e.g. a
malformed id parsed from a database column) route through `Error::Store`
and surface as `EX_IOERR` (74).

### 66 — `EX_NOINPUT`

A named entity could not be resolved. Typical causes:

- An id or prefix argument did not match any entity in the store.
- `config get <key>` where the dotted path does not exist.
- `gest migrate --from v0.4` where the source `.gest/` directory is missing.
- `iteration next <id>` where the iteration id does not resolve.
- `project detach` where the current working directory has no attached
  workspace.
- `meta get <id> <key>` where the key is not present on the entity.

### 69 — `EX_UNAVAILABLE`

The target entity exists but is not in a state that accepts the requested
transition. Typical causes:

- `iteration advance` on an iteration that is terminal, has no tasks, or
  whose current phase still has non-terminal work.
- `iteration next` on an iteration that is not active.
- `task delete <id>` where the task still belongs to one or more iterations
  (use `--force`).
- `undo` when no undoable transactions are recorded for the current project.
- UNIQUE constraint violations from the store layer.

### 70 — `EX_SOFTWARE`

An internal software failure. In practice this is reserved for true editor-
invocation failures on commands that launch `$EDITOR`:

- The editor could not be spawned, exited non-zero, or produced an empty
  body where non-empty content was required.
- `$EDITOR` is unset and no fallback is available.

### 74 — `EX_IOERR`

A filesystem or database I/O error. Typical causes:

- The store (`gest.db`) cannot be opened, read, written, or locked.
- Reading `stdin` or writing to `stdout`/`stderr` fails.
- `self-update` cannot download or install the new binary.
- `migrate` cannot read the legacy `.gest/` tree.

### 75 — `EX_TEMPFAIL`

A valid "empty result" that is distinct from an error. In gest, this is
emitted by a single command:

- `iteration next` when no unblocked open tasks remain in the active phase.

Orchestrator loops branch on `$? -eq 75` to decide whether to advance the
iteration or idle.

### 78 — `EX_CONFIG`

A configuration or setup error. Typical causes:

- The user config file could not be loaded or is invalid.
- A command that requires an initialized project was run in a directory
  with no `gest.db` and no attached parent workspace. Run `gest init` or
  `gest project attach <id>`.
- XDG directories could not be resolved.

## Scripting patterns

### "Is there work to do?"

Use `iteration next` as a claim attempt and branch on 75:

```sh
gest iteration next "$ITER_ID" --claim --agent worker-1 --json
case $? in
  0)  echo "claimed a task" ;;
  75) echo "no work available -- try advancing or idling" ;;
  *)  echo "error" >&2; exit 1 ;;
esac
```

### "Did that command fail because of missing input, or something worse?"

```sh
gest task show "$id" --json
case $? in
  0)  : ;;
  66) echo "no such task: $id" ;;
  *)  echo "unexpected error" >&2 ;;
esac
```

### "Is this directory initialized?"

```sh
if ! gest task list >/dev/null 2>&1 && [ $? -eq 78 ]; then
  gest init
fi
```
