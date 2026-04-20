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
