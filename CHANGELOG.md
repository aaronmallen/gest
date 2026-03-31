# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Break Versioning].

## [Unreleased]

## [v0.3.3] - 2026-03-31

### Added

- Task and artifact create/edit forms in the web UI with markdown preview, relationship management, and inline search
- Dashboard status cards now link to filtered task views
- Navigation buttons (new/edit) on list and detail pages

### Changed

- Short ID prefixes are now displayed with clickable links, tags are clickable, and artifact type label is used
  consistently across the web UI

### Fixed

- Store write functions now respect entity location (active vs. archive/resolved), preventing duplicates on mutation
- CLI flags (`--version`, `--verbose`) and shell completion variants now have help text (see [#24])

## [v0.3.2] - 2026-03-31

### Added

- Built-in web server (`gest serve`) with dashboard, task/artifact/iteration views, kanban board, and full-text search
  (see [#13], [#14], [#15], [#16], [#17], [#18], [#19], [#20], [#21], [#22], [#23])
- `--port`, `--bind`, and `--no-open` flags on `gest serve` for controlling the local server (see [#15])
- `[serve]` config section with `port`, `bind_address`, and `open` settings (see [#14])

### Changed

- Cached iteration phase counts to avoid recomputing on each read
- `resolve_dot_path` now walks by reference instead of cloning, reducing allocations
- Consolidated shared helpers for store operations, CLI metadata handling, and tag/untag commands

### Fixed

- Multi-word `EDITOR` values (e.g. `code --wait`) are now parsed correctly via shell tokenization
- Nested metadata `set_nested` is now bounds-checked with a depth limit to prevent stack overflows
- Version line no longer appears in the `--help` banner
- `gest search --expand` now shows full content instead of truncated snippets (see [#11])

## [v0.3.1] - 2026-03-30

### Added

- Per-entity directory overrides via `GEST_ARTIFACT_DIR`, `GEST_TASK_DIR`, `GEST_ITERATION_DIR` environment variables
  and corresponding `storage.*_dir` config fields

### Changed

- Batched disk reads when resolving blocked-by status for task lists, replacing per-task I/O with a single pass over
  unique references
- Faster search filtering by avoiding redundant string allocations during case-insensitive matching
- Early-exit prefix matching stops scanning after two matches instead of collecting all candidates
- Skip metadata serialization during search when metadata maps are empty
- Store functions now receive the full application config, allowing future access to any setting without plumbing
  changes
- Consolidated shared helpers for editor invocation, TOML metadata operations, iteration task-loading, and status
  display across CLI commands

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
[#11]: https://github.com/aaronmallen/gest/issues/11
[#13]: https://github.com/aaronmallen/gest/issues/13
[#14]: https://github.com/aaronmallen/gest/issues/14
[#15]: https://github.com/aaronmallen/gest/issues/15
[#16]: https://github.com/aaronmallen/gest/issues/16
[#17]: https://github.com/aaronmallen/gest/issues/17
[#18]: https://github.com/aaronmallen/gest/issues/18
[#19]: https://github.com/aaronmallen/gest/issues/19
[#20]: https://github.com/aaronmallen/gest/issues/20
[#21]: https://github.com/aaronmallen/gest/issues/21
[#22]: https://github.com/aaronmallen/gest/issues/22
[#23]: https://github.com/aaronmallen/gest/issues/23
[#24]: https://github.com/aaronmallen/gest/issues/24

[Unreleased]: https://github.com/aaronmallen/gest/compare/0.3.3...main
[v0.2.0]: https://github.com/aaronmallen/gest/compare/0.1.0...0.2.0
[v0.2.1]: https://github.com/aaronmallen/gest/compare/0.2.0...0.2.1
[v0.2.2]: https://github.com/aaronmallen/gest/compare/0.2.1...0.2.2
[v0.2.3]: https://github.com/aaronmallen/gest/compare/0.2.2...0.2.3
[v0.3.0]: https://github.com/aaronmallen/gest/compare/0.2.3...0.3.0
[v0.3.1]: https://github.com/aaronmallen/gest/compare/0.3.0...0.3.1
[v0.3.2]: https://github.com/aaronmallen/gest/compare/0.3.1...0.3.2
[v0.3.3]: https://github.com/aaronmallen/gest/compare/0.3.2...0.3.3
