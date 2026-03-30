# gest init

Initialize gest for the current project. This creates the data directory structure where tasks, artifacts, and iterations are stored.

By default, gest stores data in a global directory (`~/.local/share/gest/<project-hash>/`). Use the `--local` flag to create a `.gest/` directory inside the current project instead.

## Usage

```
gest init [OPTIONS]
```

## Options

| Flag | Description |
|------|-------------|
| `--local` | Initialize a `.gest` directory in the current project instead of the global data directory |
| `-v, --verbose` | Increase verbosity (repeatable) |
| `-h, --help` | Print help |

## Examples

Initialize with global storage (default):

```sh
gest init
```

Initialize with a local `.gest/` directory:

```sh
gest init --local
```
