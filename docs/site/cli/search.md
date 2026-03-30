# gest search

Search across tasks and artifacts by keyword. The query is matched against titles, descriptions, and body content.

## Usage

```
gest search [OPTIONS] <QUERY>
```

## Arguments

| Argument | Description |
|----------|-------------|
| `<QUERY>` | Text matched against titles, descriptions, and body content |

## Options

| Flag | Description |
|------|-------------|
| `-a, --all` | Include archived and resolved items |
| `-e, --expand` | Show full detail for each result |
| `-j, --json` | Emit results as JSON |
| `-v, --verbose` | Increase verbosity (repeatable) |
| `-h, --help` | Print help |

## Examples

```sh
# Basic keyword search
gest search "authentication"

# Include archived/resolved items
gest search "login" --all

# Expanded detail view
gest search "migration" --expand

# JSON output for scripting
gest search "api" --json
```
