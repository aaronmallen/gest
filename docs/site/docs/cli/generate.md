---
title: gest generate
description: gest generate — produce shell completions and man pages for the gest CLI, the agentic workflow CLI for parallel AI agents.
keywords: [gest generate, shell completions, man pages, agentic workflow CLI]
---

# gest generate

Generate shell completions and man pages for gest.

## Usage

```text
gest generate <COMMAND> [OPTIONS]
```

## Subcommands

| Command                                | Description                              |
|----------------------------------------|------------------------------------------|
| [`completions`](#generate-completions) | Print shell completion scripts to stdout |
| [`man-pages`](#generate-man-pages)     | Write man page files to a directory      |

## Exit Codes

| Code | When                                                     |
|------|----------------------------------------------------------|
| 0    | Success                                                  |
| 64   | Bad flags or unknown shell target                        |
| 74   | Could not write completions or man pages to the target   |

See [Exit Codes](./exit-codes.md) for the full contract.

---

## generate completions

Print shell completion scripts to stdout. Pipe the output to the appropriate location for your shell.

```text
gest generate completions --shell <SHELL>
```

### Options

| Flag              | Description                                                 |
|-------------------|-------------------------------------------------------------|
| `--shell <SHELL>` | Target shell: `bash`, `elvish`, `fish`, `powershell`, `zsh` |

### Examples

```sh
# Bash
gest generate completions --shell bash > ~/.local/share/bash-completion/completions/gest

# Zsh
gest generate completions --shell zsh > ~/.zfunc/_gest

# Fish
gest generate completions --shell fish > ~/.config/fish/completions/gest.fish
```

---

## generate man-pages

Write roff man page files for all commands to a directory.

```text
gest generate man-pages --output-dir <OUTPUT_DIR>
```

### Options

| Flag                        | Description                            |
|-----------------------------|----------------------------------------|
| `--output-dir <OUTPUT_DIR>` | Directory to write man page files into |

### Examples

```sh
gest generate man-pages --output-dir ./man
```
