# gest self-update

Update gest to the latest (or a pinned) GitHub release. Downloads the appropriate binary
for your platform and replaces the current installation.

## Usage

```text
gest self-update [OPTIONS]
```

## Options

| Flag                | Description                                           |
|---------------------|-------------------------------------------------------|
| `--target <TARGET>` | Pin to a specific version (bare semver, e.g. `1.2.3`) |
| `-v, --verbose`     | Increase verbosity (repeatable)                       |
| `-h, --help`        | Print help                                            |

## Examples

```sh
# Update to the latest release
gest self-update

# Pin to a specific version
gest self-update --target 0.3.0
```

## Exit Codes

| Code | When                                                     |
|------|----------------------------------------------------------|
| 0    | Success (or already on the requested version)            |
| 64   | Bad flags or invalid `--target` version                  |
| 74   | Could not download, verify, or install the new binary    |
| 78   | Could not load user config                               |

See [Exit Codes](./exit-codes.md) for the full contract.
