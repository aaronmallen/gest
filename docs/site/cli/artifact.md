# gest artifact

Create, update, list, and manage artifacts. Artifacts store documents such as specs, ADRs, RFCs, and notes alongside your project.

## Usage

```
gest artifact <COMMAND> [OPTIONS]
```

## Subcommands

| Command | Description |
|---------|-------------|
| [`create`](#artifact-create) | Create a new artifact |
| [`list`](#artifact-list) | List artifacts with optional filters |
| [`show`](#artifact-show) | Display an artifact's full details |
| [`update`](#artifact-update) | Update an artifact's fields |
| [`tag`](#artifact-tag) | Add tags to an artifact |
| [`untag`](#artifact-untag) | Remove tags from an artifact |
| [`archive`](#artifact-archive) | Archive an artifact |
| [`meta`](#artifact-meta) | Read or write metadata fields |

---

## artifact create

Create a new artifact from inline text, a source file, an editor, or stdin.

```
gest artifact create [OPTIONS]
```

### Options

| Flag | Description |
|------|-------------|
| `-t, --title <TITLE>` | Artifact title (auto-extracted from first `#` heading if omitted) |
| `-b, --body <BODY>` | Body content as an inline string (skips editor and stdin) |
| `-k, --type <KIND>` | Artifact type (e.g. `spec`, `adr`, `rfc`, `note`) |
| `-m, --metadata <METADATA>` | Key=value metadata pairs (repeatable) |
| `-s, --source <SOURCE>` | Read body content from a file path |
| `--tags <TAGS>` | Comma-separated list of tags |

### Examples

```sh
# Create from inline body
gest artifact create -t "Auth Spec" -k spec -b "## Overview\nAuth flow details..."

# Create from a file
gest artifact create -k adr -s docs/decisions/001-storage.md

# Create interactively (opens $EDITOR)
gest artifact create -k rfc --tags "backend,v2"
```

---

## artifact list

List artifacts, optionally filtered by type, tag, or archive status.

```
gest artifact list [OPTIONS]
```

### Options

| Flag | Description |
|------|-------------|
| `-a, --all` | Include archived artifacts alongside active ones |
| `--archived` | Show only archived artifacts |
| `-j, --json` | Output as JSON |
| `-k, --type <KIND>` | Filter by artifact type (e.g. `spec`, `adr`, `rfc`) |
| `--tag <TAG>` | Filter by tag |

### Examples

```sh
# List active artifacts
gest artifact list

# Filter by type
gest artifact list -k spec

# Include archived
gest artifact list --all
```

---

## artifact show

Display an artifact's full details and rendered body.

```
gest artifact show [OPTIONS] <ID>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Artifact ID or unique prefix |

### Options

| Flag | Description |
|------|-------------|
| `-j, --json` | Output as JSON instead of formatted detail |

### Examples

```sh
gest artifact show abc123
gest artifact show abc123 --json
```

---

## artifact update

Update an artifact's title, body, type, or tags.

```
gest artifact update [OPTIONS] <ID>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Artifact ID or unique prefix |

### Options

| Flag | Description |
|------|-------------|
| `-b, --body <BODY>` | Replace the body content |
| `-k, --type <KIND>` | Artifact type (e.g. `spec`, `adr`, `rfc`, `note`) |
| `--tags <TAGS>` | Replace all tags with this comma-separated list |
| `-t, --title <TITLE>` | New title |

### Examples

```sh
gest artifact update abc123 -t "Updated Title"
gest artifact update abc123 -k adr --tags "approved,backend"
```

---

## artifact tag

Add tags to an artifact.

```
gest artifact tag <ID> [TAGS]...
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Artifact ID or unique prefix |
| `[TAGS]...` | Tags to add (space-separated) |

### Examples

```sh
gest artifact tag abc123 approved reviewed
```

---

## artifact untag

Remove tags from an artifact.

```
gest artifact untag <ID> [TAGS]...
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Artifact ID or unique prefix |
| `[TAGS]...` | Tags to remove (space-separated) |

### Examples

```sh
gest artifact untag abc123 draft
```

---

## artifact archive

Move an artifact to the archive by setting its `archived_at` timestamp.

```
gest artifact archive <ID>
```

### Arguments

| Argument | Description |
|----------|-------------|
| `<ID>` | Artifact ID or unique prefix |

### Examples

```sh
gest artifact archive abc123
```

---

## artifact meta

Read or write artifact metadata fields. Metadata uses dot-delimited key paths for nested values.

```
gest artifact meta <COMMAND>
```

### meta get

Retrieve a single metadata value.

```
gest artifact meta get <ID> <PATH>
```

| Argument | Description |
|----------|-------------|
| `<ID>` | Artifact ID or unique prefix |
| `<PATH>` | Dot-delimited key path (e.g. `outer.inner`) |

### meta set

Set a metadata value. Strings, numbers, booleans, and null are auto-detected.

```
gest artifact meta set <ID> <PATH> <VALUE>
```

| Argument | Description |
|----------|-------------|
| `<ID>` | Artifact ID or unique prefix |
| `<PATH>` | Dot-delimited key path (e.g. `config.timeout`) |
| `<VALUE>` | Value to set |

### Examples

```sh
gest artifact meta set abc123 status "approved"
gest artifact meta get abc123 status
```
