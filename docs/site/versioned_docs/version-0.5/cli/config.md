# gest config

View and modify gest configuration. Configuration is loaded from TOML files at the global
and project levels, with project-level values taking precedence.

## Usage

```text
gest config <COMMAND> [OPTIONS]
```

## Subcommands

| Command                | Description                           |
|------------------------|---------------------------------------|
| [`get`](#config-get)   | Retrieve a single configuration value |
| [`set`](#config-set)   | Persist a configuration value         |
| [`show`](#config-show) | Display the merged configuration      |

---

## config get

Retrieve a single configuration value by dot-delimited key.

```text
gest config get <KEY>
```

### Arguments

| Argument | Description                                        |
|----------|----------------------------------------------------|
| `<KEY>`  | Dot-delimited config key (e.g. `storage.data_dir`) |

### Examples

```sh
gest config get storage.data_dir
gest config get log.level
gest config get database.url
```

---

## config set

Persist a configuration value to a TOML config file. By default, writes to the
project-level config. Use `--global` to write to the user-level config instead.

```text
gest config set [OPTIONS] <KEY> <VALUE>
```

### Arguments

| Argument  | Description                                 |
|-----------|---------------------------------------------|
| `<KEY>`   | Dot-delimited config key (e.g. `log.level`) |
| `<VALUE>` | Value to assign                             |

### Options

| Flag           | Description                                                           |
|----------------|-----------------------------------------------------------------------|
| `-g, --global` | Write to the global (user-level) config instead of the project config |

### Examples

```sh
# Set project-level config
gest config set log.level debug

# Set global config
gest config set log.level info --global
```

---

## config show

Display the merged configuration and discovered config file sources.

```text
gest config show
```

### Examples

```sh
gest config show
```
