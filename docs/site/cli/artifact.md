# gest artifact

Create, update, list, and manage artifacts. Artifacts store documents such as specs, ADRs,
RFCs, and notes alongside your project.

## Usage

```text
gest artifact <COMMAND> [OPTIONS]
```

## Subcommands

| Command                        | Aliases | Description                               |
|--------------------------------|---------|-------------------------------------------|
| [`archive`](#artifact-archive) |         | Archive an artifact                       |
| [`create`](#artifact-create)   | `new`   | Create a new artifact                     |
| [`delete`](#artifact-delete)   | `rm`    | Delete an artifact and its dependent rows |
| [`list`](#artifact-list)       | `ls`    | List artifacts with optional filters      |
| [`show`](#artifact-show)       | `view`  | Display an artifact's full details        |
| [`update`](#artifact-update)   | `edit`  | Update an artifact's fields               |
| [`tag`](#artifact-tag)         |         | Add tags to an artifact                   |
| [`untag`](#artifact-untag)     |         | Remove tags from an artifact              |
| [`meta`](#artifact-meta)       |         | Read or write metadata fields             |
| [`note`](#artifact-note)       |         | Manage notes attached to an artifact      |

---

## artifact archive

Move an artifact to the archive by setting its `archived_at` timestamp.

```text
gest artifact archive [OPTIONS] <ID>
```

### Arguments

| Argument | Description                  |
|----------|------------------------------|
| `<ID>`   | Artifact ID or unique prefix |

### Options

| Flag          | Description                |
|---------------|----------------------------|
| `-j, --json`  | Output as JSON             |
| `-q, --quiet` | Print only the artifact ID |

### Examples

```sh
gest artifact archive abc123
```

---

## artifact create

Create a new artifact from inline text, a source file, an editor, or stdin.

When `--body` and `--source` are both omitted and stdin is a terminal, `$EDITOR` opens for
interactive editing. When stdin is a pipe, the piped content is used as the body.

```text
gest artifact create [OPTIONS] [TITLE]
```

### Arguments

| Argument  | Description                                                                                 |
|-----------|---------------------------------------------------------------------------------------------|
| `[TITLE]` | Artifact title (auto-extracted from the first `#` heading when piping stdin or `--source`)  |

### Options

| Flag                          | Description                                                                         |
|-------------------------------|-------------------------------------------------------------------------------------|
| `--batch`                     | Read NDJSON from stdin (one artifact per line)                                      |
| `-b, --body <BODY>`           | Body content as an inline string (skips editor and stdin)                           |
| `-i, --iteration <ITERATION>` | Add the artifact to an iteration (ID or prefix)                                     |
| `-j, --json`                  | Output the created artifact as JSON                                                 |
| `-m, --metadata <KEY=VALUE>`  | Set a metadata key=value pair (repeatable; supports dot-paths and scalar inference) |
| `--metadata-json <JSON>`      | Merge a JSON object into metadata (repeatable; applied after `--metadata` pairs)    |
| `-q, --quiet`                 | Print only the artifact ID                                                          |
| `-s, --source <SOURCE>`       | Read body content from a file path                                                  |
| `-t, --tag <TAG>`             | Tag (repeatable). Use tags like `spec`, `adr`, `rfc`, `note` to categorize.         |

Artifact categorization is tag-driven in v0.5.0. The `--type`/`-k` flag and `kind`
field were removed — tag your artifacts with `spec`, `adr`, `rfc`, etc. and filter
listings with `--tag`.

### Examples

```sh
# Create from inline body
gest artifact create "Auth Spec" --tag spec --body "## Overview\nAuth flow details..."

# Create from a file (title extracted from the first heading)
gest artifact create --tag adr --source docs/decisions/001-storage.md

# Create interactively (opens $EDITOR)
gest artifact create "My RFC" --tag rfc --tag backend --tag v2

# Add to an iteration
gest artifact create "Sprint 3 Notes" --tag note -i iter123

# Pipe body from stdin
echo "# My Spec\nDetails..." | gest artifact create --tag spec

# Batch-create artifacts from NDJSON
cat artifacts.ndjson | gest artifact create --batch

# Machine-readable output
gest artifact create "Quick note" --tag note --json
gest artifact create "Quick note" --tag note -q
```

---

## artifact delete

Permanently delete an artifact and its dependent rows (tags, metadata, notes, links).
This is irreversible; prefer [`archive`](#artifact-archive) when you only want to hide
an artifact from active listings.

```text
gest artifact delete [OPTIONS] <ID>
```

### Arguments

| Argument | Description                  |
|----------|------------------------------|
| `<ID>`   | Artifact ID or unique prefix |

### Options

| Flag          | Description                                                                     |
|---------------|---------------------------------------------------------------------------------|
| `--yes`       | Skip the interactive confirmation prompt                                        |
| `--force`     | Reserved for future guards; currently a no-op (artifacts have no guards today)  |
| `-j, --json`  | Output as JSON                                                                  |
| `-q, --quiet` | Suppress normal output                                                          |

### Examples

```sh
# Interactive (prompts for confirmation)
gest artifact delete abc123

# Non-interactive (scripts, CI)
gest artifact delete abc123 --yes
```

---

## artifact list

List artifacts, optionally filtered by tag or archive status.

```text
gest artifact list [OPTIONS]
```

### Options

| Flag              | Description                                      |
|-------------------|--------------------------------------------------|
| `-a, --all`       | Include archived artifacts alongside active ones |
| `--archived`      | Show only archived artifacts                     |
| `-j, --json`      | Output as JSON                                   |
| `-t, --tag <TAG>` | Filter by tag                                    |

### Examples

```sh
# List active artifacts
gest artifact list

# Filter by tag (use category tags like `spec`, `adr`, `rfc`)
gest artifact list --tag spec

# Include archived
gest artifact list --all

# JSON output for scripting
gest artifact list --json
```

---

## artifact show

Display an artifact's full details and rendered body.

```text
gest artifact show [OPTIONS] <ID>
```

### Arguments

| Argument | Description                  |
|----------|------------------------------|
| `<ID>`   | Artifact ID or unique prefix |

### Options

| Flag         | Description                                |
|--------------|--------------------------------------------|
| `-j, --json` | Output as JSON instead of formatted detail |

### Examples

```sh
gest artifact show abc123
gest artifact show abc123 --json
```

---

## artifact update

Update an artifact's title, body, metadata, or tags.

```text
gest artifact update [OPTIONS] <ID>
```

### Arguments

| Argument | Description                  |
|----------|------------------------------|
| `<ID>`   | Artifact ID or unique prefix |

### Options

| Flag                         | Description                                                                         |
|------------------------------|-------------------------------------------------------------------------------------|
| `-b, --body <BODY>`          | Replace the body content                                                            |
| `-e, --edit`                 | Open `$EDITOR` pre-filled with the current body                                     |
| `-j, --json`                 | Output as JSON                                                                      |
| `-m, --metadata <KEY=VALUE>` | Set a metadata key=value pair (repeatable; supports dot-paths and scalar inference) |
| `--metadata-json <JSON>`     | Merge a JSON object into metadata (repeatable; applied after `--metadata` pairs)    |
| `-q, --quiet`                | Print only the artifact ID                                                          |
| `-t, --tag <TAG>`            | Replace all tags (repeatable)                                                       |
| `-T, --title <TITLE>`        | New title                                                                           |

### Examples

```sh
gest artifact update abc123 -T "Updated Title"
gest artifact update abc123 --tag approved --tag backend

# Machine-readable output
gest artifact update abc123 -T "New Title" --json
```

---

## artifact tag

Add tags to an artifact, deduplicating with any existing tags.

```text
gest artifact tag [OPTIONS] <ID> [TAGS]...
```

### Arguments

| Argument    | Description                            |
|-------------|----------------------------------------|
| `<ID>`      | Artifact ID or unique prefix           |
| `[TAGS]...` | Tags to add (space or comma-separated) |

### Options

| Flag          | Description                               |
|---------------|-------------------------------------------|
| `-j, --json`  | Output the artifact as JSON after tagging |
| `-q, --quiet` | Output only the artifact ID               |

### Examples

```sh
gest artifact tag abc123 approved reviewed
gest artifact tag abc123 approved,reviewed
```

---

## artifact untag

Remove tags from an artifact.

```text
gest artifact untag [OPTIONS] <ID> [TAGS]...
```

### Arguments

| Argument    | Description                               |
|-------------|-------------------------------------------|
| `<ID>`      | Artifact ID or unique prefix              |
| `[TAGS]...` | Tags to remove (space or comma-separated) |

### Options

| Flag          | Description                                 |
|---------------|---------------------------------------------|
| `-j, --json`  | Output the artifact as JSON after untagging |
| `-q, --quiet` | Output only the artifact ID                 |

### Examples

```sh
gest artifact untag abc123 draft
```

---

## artifact meta

Read or write artifact metadata fields. Metadata uses dot-delimited key paths for nested values.

```text
gest artifact meta <COMMAND>
```

### meta get

Retrieve a single metadata value.

```text
gest artifact meta get [OPTIONS] <ID> <PATH>
```

| Argument | Description                                 |
|----------|---------------------------------------------|
| `<ID>`   | Artifact ID or unique prefix                |
| `<PATH>` | Dot-delimited key path (e.g. `outer.inner`) |

| Flag     | Description                           |
|----------|---------------------------------------|
| `--json` | Output as a JSON object               |
| `--raw`  | Output the bare value with no styling |

### meta set

Set a metadata value. Strings, numbers, booleans, and null are auto-detected.

```text
gest artifact meta set [OPTIONS] <ID> <PATH> <VALUE>
```

| Argument  | Description                                    |
|-----------|------------------------------------------------|
| `<ID>`    | Artifact ID or unique prefix                   |
| `<PATH>`  | Dot-delimited key path (e.g. `config.timeout`) |
| `<VALUE>` | Value to set                                   |

| Flag          | Description              |
|---------------|--------------------------|
| `-j, --json`  | Output as JSON           |
| `-q, --quiet` | Print only the entity ID |

### Examples

```sh
# Set a metadata field
gest artifact meta set abc123 status "approved"

# Read it back
gest artifact meta get abc123 status

# JSON output
gest artifact meta get abc123 status --json

# Raw value (no styling)
gest artifact meta get abc123 status --raw
```

---

## artifact note

Manage notes attached to an artifact. Notes are lightweight markdown bodies that hang
off an artifact and are a good fit for running commentary, review threads, or agent
annotations that should live alongside the artifact without altering its body.

```text
gest artifact note <COMMAND>
```

### note add

Add a new note to an artifact. Use `--body -` to open `$EDITOR` for interactive entry.

```text
gest artifact note add [OPTIONS] --body <BODY> <ID>
```

| Argument | Description                  |
|----------|------------------------------|
| `<ID>`   | Artifact ID or unique prefix |

| Flag                | Description                                      |
|---------------------|--------------------------------------------------|
| `-b, --body <BODY>` | Note body (required; use `-` to open `$EDITOR`)  |
| `--agent <AGENT>`   | Set the author (agent) identifier for this note  |
| `-j, --json`        | Output as JSON                                   |
| `-q, --quiet`       | Print only the note ID                           |

### note list

List all notes attached to an artifact, newest first.

```text
gest artifact note list [OPTIONS] <ID>
```

| Argument | Description                  |
|----------|------------------------------|
| `<ID>`   | Artifact ID or unique prefix |

| Flag            | Description                            |
|-----------------|----------------------------------------|
| `--limit <N>`   | Cap the number of items returned       |
| `-j, --json`    | Output as JSON                         |
| `-q, --quiet`   | Print only note IDs                    |

### note show

Display a single note.

```text
gest artifact note show [OPTIONS] <ID>
```

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Note ID or unique prefix |

| Flag         | Description    |
|--------------|----------------|
| `-j, --json` | Output as JSON |

### note update

Replace a note's body.

```text
gest artifact note update [OPTIONS] <ID>
```

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Note ID or unique prefix |

| Flag                | Description                                           |
|---------------------|-------------------------------------------------------|
| `-b, --body <BODY>` | New body text (use `-` to open `$EDITOR`)             |
| `-j, --json`        | Output as JSON                                        |
| `-q, --quiet`       | Print only the note ID                                |

### note delete

Delete a note from an artifact.

```text
gest artifact note delete [OPTIONS] <ID>
```

| Argument | Description              |
|----------|--------------------------|
| `<ID>`   | Note ID or unique prefix |

| Flag          | Description                                    |
|---------------|------------------------------------------------|
| `--yes`       | Skip the interactive confirmation prompt       |
| `-j, --json`  | Output as JSON                                 |
| `-q, --quiet` | Suppress normal output                         |

### Examples

```sh
# Add a note inline
gest artifact note add abc123 --body "Reviewed by security team; approved."

# Add a note via $EDITOR
gest artifact note add abc123 --body -

# Attribute a note to an agent
gest artifact note add abc123 --body "Drafted outline" --agent implement-agent

# List notes for an artifact
gest artifact note list abc123
gest artifact note list abc123 --limit 5 --json

# Show and update a single note
gest artifact note show note456
gest artifact note update note456 --body "Updated commentary"

# Delete a note (non-interactive)
gest artifact note delete note456 --yes
```
