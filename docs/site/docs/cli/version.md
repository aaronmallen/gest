# gest version

Print the current version, platform info, and check for available updates.

## Usage

```text
gest version
```

## Options

| Flag            | Description                     |
|-----------------|---------------------------------|
| `-v, --verbose` | Increase verbosity (repeatable) |
| `-h, --help`    | Print help                      |

## Examples

```sh
gest version
```

## Exit Codes

| Code | When                                                  |
|------|-------------------------------------------------------|
| 0    | Success                                               |
| 64   | Bad flags                                             |
| 74   | Could not reach the update endpoint                   |
| 78   | Could not load user config                            |

See [Exit Codes](./exit-codes.md) for the full contract.
