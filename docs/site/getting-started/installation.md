# Installation

## Quick Install (macOS and Linux)

The fastest way to get gest is with the install script, which downloads a pre-built binary for
your platform:

```sh
curl -fsSL https://gest.aaronmallen.dev/install | sh
```

This installs `gest` to `~/.local/bin`. Make sure it is on your `PATH`:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

To pin a specific version or change the install directory:

```sh
GEST_VERSION=0.3.0 GEST_INSTALL_PATH=/usr/local/bin \
  curl -fsSL https://gest.aaronmallen.dev/install | sh
```

## Install via Cargo

If you already have a Rust toolchain, install from [crates.io](https://crates.io/crates/gest):

```sh
cargo install gest
```

Or with [cargo-binstall](https://github.com/cargo-bins/cargo-binstall) for a pre-built binary:

```sh
cargo binstall gest
```

## Download from GitHub Releases

Pre-built binaries for macOS (Apple Silicon and Intel) and Linux (x86_64 and aarch64) are
published with every release. Download the archive for your platform from the
[Releases page](https://github.com/aaronmallen/gest/releases), extract it, and move the
binary somewhere on your `PATH`:

```sh
# Example for macOS Apple Silicon
tar xzf gest-v0.3.0-aarch64-apple-darwin.tar.gz
mv gest ~/.local/bin/
```

## Verify Installation

After installing, confirm gest is available:

```sh
gest version
```

You should see output similar to:

```text
v0.3.4 macos-aarch64 (2026-03-31 revision 9644667)
```

The `version` command also checks for available updates. To update an existing install, run:

```sh
gest self-update
```
