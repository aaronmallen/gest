# gest search

Search across tasks and artifacts by keyword. The query is matched against titles, descriptions, and body content.
Queries support structured filter prefixes to narrow results by type, status, or tag.

## Usage

```text
gest search [OPTIONS] <QUERY>
```

## Arguments

| Argument | Description |
| --- | --- |
| `<QUERY>` | Text and filter expressions matched against titles, descriptions, and body content |

## Query Syntax

A query is made up of **free text** and **filter expressions**, separated by spaces. Free text is matched against
titles, descriptions, and body content. Filter expressions use a `prefix:value` format to constrain results by
specific fields.

### Filter Prefixes

| Prefix | Description | Example |
| --- | --- | --- |
| `is:` | Filter by entity type (`task`, `artifact`, `iteration`) | `is:task` |
| `tag:` | Filter by tag name | `tag:urgent` |
| `status:` | Filter by status | `status:open` |
| `type:` | Filter by artifact kind | `type:spec` |

Both prefixes and values are **case-insensitive** -- `IS:Task`, `is:task`, and `Is:TASK` all behave the same.

### Negation

Prefix any filter with `-` to **exclude** matching items:

```text
-tag:wip          # exclude items tagged "wip"
-status:done      # exclude items with status "done"
-is:artifact      # exclude artifacts from results
```

### Combination Rules

Filters combine using these rules:

- **Same prefix** filters are **OR-combined** -- `is:task is:artifact` matches tasks *or* artifacts.
- **Different prefix** filters are **AND-combined** -- `is:task tag:urgent` matches tasks that are *also* tagged
  "urgent".
- **Free text** is **AND-combined** with filters -- `is:task fix login` matches tasks whose content contains both "fix"
  and "login".
- **Negation** filters are AND-combined with everything else -- `tag:urgent -status:done` matches urgent items that are
  *not* done.

## Options

| Flag | Description |
| --- | --- |
| `-a, --all` | Include archived and resolved items |
| `-e, --expand` | Show full detail for each result |
| `-j, --json` | Emit results as JSON |
| `-v, --verbose` | Increase verbosity (repeatable) |
| `-h, --help` | Print help |

## Examples

```sh
# Basic keyword search
gest search "authentication"

# Filter to tasks only
gest search "is:task"

# Tasks tagged "urgent"
gest search "is:task tag:urgent"

# Tasks or artifacts matching "login"
gest search "is:task is:artifact login"

# Exclude work-in-progress items
gest search "tag:api -tag:wip"

# Open tasks about migrations
gest search "is:task status:open migration"

# Artifacts of type "spec"
gest search "type:spec"

# Combine negation with free text
gest search "is:task -status:done fix auth"

# Include archived/resolved items
gest search "login" --all

# Expanded detail view
gest search "is:task tag:urgent" --expand

# JSON output for scripting
gest search "status:open" --json
```
