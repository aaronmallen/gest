# gest migrate

Import legacy flat-file data from a v0.4.x `.gest/` directory into the current
project's SQLite database.

In gest v0.5.0, entity data moved from per-type directories of TOML/Markdown files
into a single SQLite database. Running `gest migrate --from v0.4` walks the old
directory structure and inserts everything into the new database, preserving IDs,
tags, links, iteration assignments, and notes.

## Usage

```text
gest migrate [OPTIONS] --from <FROM>
```

## Options

| Flag             | Description                                                          |
|------------------|----------------------------------------------------------------------|
| `--from <FROM>`  | The source format version to migrate from. Only `v0.4` is supported. |
| `--path <PATH>`  | Path to the legacy data directory (defaults to auto-discovery).      |
| `-v, --verbose`  | Increase verbosity (repeatable).                                     |
| `-h, --help`     | Print help.                                                          |

## Examples

Auto-discover the v0.4.x `.gest/` directory in the current project and migrate
everything into the new database:

```sh
gest migrate --from v0.4
```

Migrate from a specific path:

```sh
gest migrate --from v0.4 --path ~/projects/myapp/.gest
```

## How it works

Migration runs inside a single database transaction so it is all-or-nothing. For
each entity type it:

1. Reads the legacy TOML/Markdown files from the source directory.
2. Parses them with the v0.4.x format.
3. Inserts the data into the corresponding v0.5.0 table with the original ID
   preserved.

After the migration completes, the old `.gest/` directory is left in place so you
can verify the result before deleting it manually. If `storage.sync` is enabled,
the sync layer will rewrite the mirror from the new database on the next command
invocation — the old TOML files will be replaced with fresh JSON/Markdown output
that reflects the current database state.

## Troubleshooting

If migration reports warnings like `could not resolve link ref: tasks/<id>`, it
means the legacy data contained dangling references to entities that no longer
exist. The migration continues past those warnings; the resulting database will
just have fewer links than the source directory claimed.

## See also

For a step-by-step walkthrough — including a pre-migration checklist, a table
of how v0.4.x data maps into v0.5.0 tables, post-migration verification steps,
and rollback instructions — see [Migrating from v0.4.x to v0.5.0](../migration/v0-4-to-v0-5.md).
