# Gest

[![Build][build-badge]][build-link]
[![Crates.io Version][crates-badge]][crates-link]
[![GitHub Sponsors][sponsor-badge]][sponsor-link]

Manage agent-generated artifacts and task backlogs alongside your project.

> [!WARNING]
> Gest is in early development. Commands, file formats, and configuration may change without notice between releases.

Gest is a CLI tool for tracking tasks and artifacts generated during AI-assisted development. Data is stored as plain
files — TOML for tasks, Markdown with YAML frontmatter for artifacts — and can live inside your repo (`.gest/`) or in an
external data directory.

## Quick Start

```sh
gest init                                    # initialize global store
gest task create "Implement auth middleware"
gest artifact create --source auth-spec.md
gest search "auth"
```

Use `gest init --local` to store data inside the repo (`.gest/`) instead of the global data directory.

## Documentation

Full documentation is available at <https://gest.aaronmallen.dev>.

## Commands

| Command             | Description                                                    |
|---------------------|----------------------------------------------------------------|
| `gest init`         | Initialize gest (`--local` for in-repo `.gest/` directory)     |
| `gest task`         | Create, list, show, update, tag, link, and manage tasks        |
| `gest artifact`     | Create, list, show, update, tag, archive, and manage artifacts |
| `gest iteration`    | Manage iterations (group tasks into phased execution plans)    |
| `gest search`       | Search across tasks and artifacts                              |
| `gest serve`        | Start a local web dashboard for browsing and managing entities |
| `gest config`       | View and modify configuration                                  |
| `gest generate`     | Generate shell completions and man pages                       |
| `gest self-update`  | Update gest to the latest GitHub release                       |
| `gest version`      | Print version and check for updates                            |

Run `gest --help` or `gest <command> --help` for full details.

## Configuration

Gest loads configuration from global (`~/.config/gest/`) and project-level TOML files. Run `gest config show` to see
the resolved configuration and its source files.

## Installation

### Quick Install (macOS and Linux)

```sh
curl -fsSL https://gest.aaronmallen.dev/install | sh
```

> [!TIP]
> This installs `gest` to `~/.local/bin`. Make sure it's in your `PATH`:
>
>```sh
>export PATH="$HOME/.local/bin:$PATH"
>```

Pin a specific version or change the install directory:

```sh
GEST_VERSION=0.0.1 GEST_INSTALL_PATH=/usr/local/bin \
  curl -fsSL https://gest.aaronmallen.dev/install | sh
```

### Cargo

```sh
cargo install gest
```

Or with [cargo-binstall] for a pre-built binary:

```sh
cargo binstall gest
```

## Status

Early development. See [docs] for design documents and process guides.

## License

[MIT]

[build-badge]:
  https://img.shields.io/github/actions/workflow/status/aaronmallen/gest/build.yml?branch=main&style=for-the-badge
[build-link]: https://github.com/aaronmallen/gest/actions/workflows/build.yml
[cargo-binstall]: https://github.com/cargo-bins/cargo-binstall
[crates-badge]: https://img.shields.io/crates/v/gest?style=for-the-badge
[crates-link]: https://crates.io/crates/gest
[docs]: https://github.com/aaronmallen/gest/tree/main/docs
[MIT]: https://github.com/aaronmallen/gest/blob/main/LICENSE
[sponsor-badge]: https://img.shields.io/github/sponsors/aaronmallen?style=for-the-badge
[sponsor-link]: https://github.com/sponsors/aaronmallen
