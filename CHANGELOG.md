# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Break Versioning].

## [Unreleased]

## [v0.3.0] - 2026-03-30

### Changed

- **Breaking:** Configuration files are now TOML only — JSON and YAML configs are no longer supported
- **Breaking:** `gest init` now creates the global data store by default; use `gest init --local`
  for in-repo `.gest/` mode
- Complete UI overhaul with an atomic architecture (atoms, composites, views) for consistent,
  aligned rendering across all commands

## [v0.2.3] - 2026-03-29

### Added

- Iteration entity type with storage, UI components, CLI commands, and graph visualization for planning multi-phase work
- `priority`, `assigned_to`, and `phase` fields on tasks for richer project tracking
- `--expand` flag on `gest search` now works without `--json`, showing full detail blocks directly in the terminal

## [v0.2.2] - 2026-03-27

### Added

- `gest version` now checks for newer releases and suggests `gest self-update` when one is available
- `--expand` (`-e`) flag on `gest search` enriches `--json` output with full item details
- `indicator.blocked` and `indicator.blocking` theme tokens for task list status indicators

### Changed

- Reduced unnecessary memory allocations in UI rendering and search
- Extracted shared helpers and removed dead code for cleaner internals

## [v0.2.1] - 2026-03-26

### Fixed

- Install script now downloads the correct checksum file during installation
- Self-update no longer fails with a 404 when fetching the target release

## [v0.2.0] - 2026-03-26

### Added

- Shell completions and man-page generation commands (see [#1])
- Markdown rendering in the terminal with styled headings, code blocks, lists, blockquotes, and more (see [#2], [#3])
- Artifact and task detail views now render descriptions as styled markdown (see [#4], [#5])
- Verbose logging across commands, store operations, and config discovery

### Changed

- `artifact list --include-archived` renamed to `--all` (`-a`), matching task and search commands (**breaking**)
- Faster unique ID prefix computation and shorter ID encoding

### Fixed

- Log level semantics now correctly follow debug=why, trace=result convention

## v0.1.0 - 2026-03-26

Initial release

[Break Versioning]: https://www.taoensso.com/break-versioning
[Keep a Changelog]: https://keepachangelog.com/en/1.1.0/

[#1]: https://github.com/aaronmallen/gest/issues/1
[#2]: https://github.com/aaronmallen/gest/issues/2
[#3]: https://github.com/aaronmallen/gest/issues/3
[#4]: https://github.com/aaronmallen/gest/issues/4
[#5]: https://github.com/aaronmallen/gest/issues/5

[Unreleased]: https://github.com/aaronmallen/gest/compare/0.3.0...main
[v0.2.0]: https://github.com/aaronmallen/gest/compare/0.1.0...0.2.0
[v0.2.1]: https://github.com/aaronmallen/gest/compare/0.2.0...0.2.1
[v0.2.2]: https://github.com/aaronmallen/gest/compare/0.2.1...0.2.2
[v0.2.3]: https://github.com/aaronmallen/gest/compare/0.2.2...0.2.3
[v0.3.0]: https://github.com/aaronmallen/gest/compare/0.2.3...0.3.0
