# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog], and this project adheres to
[Break Versioning].

## [Unreleased]

## [v0.4.3] - 2026-04-03

### Added

- `iteration cancel` and `iteration reopen` shortcut commands for iteration lifecycle management,
  with automatic cascade — cancelling an iteration cancels all non-terminal tasks, reopening restores
  them
- `Cancelled` iteration status with `failed` retained as a deprecated alias for backward
  compatibility
- Live reload in the web UI via server-sent events — pages update automatically when project files
  change on disk, with configurable debounce (`serve.debounce_ms`, default 2000ms)
- Per-request HTTP logging with method, path, status, and elapsed time (`serve.log_level`, default
  info)

### Changed

- Dashboard now shows open + in-progress task counts instead of all tasks, with a new iteration
  status breakdown row
- Web server handler I/O is now offloaded to blocking threads, preventing file reads from starving
  async connection handling

### Fixed

- Editor content is no longer silently discarded when using `read_from_editor`

## [v0.4.2] - 2026-04-02

### Added

- Command aliases for faster navigation: top-level shortcuts (`a`, `t`, `i`, `grep`), subcommand
  aliases (`new`, `ls`, `view`, `edit`, `rm`), and single-letter aliases (`u` for undo, `s` for
  serve)
- `task complete`, `task cancel`, and `task block` shortcut commands for common lifecycle transitions
- `--json` and `-q` output flags on all mutation commands (create, update, status, tag, untag, link,
  meta set, note) for machine-readable and script-friendly output
- `--json` and `--raw` flags on `meta get` for structured and raw value output
- `--batch` flag on create commands for NDJSON bulk creation of tasks and artifacts via stdin
- `-i`/`--iteration` flag on create commands to add new entities to an iteration inline
- `-l`/`--link` flag on `task create` for inline link creation (e.g. `--link blocks:abc`)
- Implicit stdin support for body and description fields — pipe content directly without flags
- Unified `--tag` flag on create and update commands and comma-separated positional tags on tag/untag
  commands (e.g. `gest task tag <id> rust,cli`)
- Search output is now paged through `$PAGER` (falling back to `less -R`) when stdout is a terminal
- `-j` short flag for `--json` on `task note list` and `task note show`

### Changed

- Consolidated entity operations behind generic trait-based action functions, reducing duplication
  across task, artifact, and iteration command handlers
- Replaced catch-all error variants with domain-specific error types for clearer diagnostics

### Fixed

- Tag list table now renders with themed formatting

## [v0.4.1] - 2026-04-01

### Added

- Search queries now support structured filters: `is:`, `tag:`, `status:`, and `type:` prefixes with negation via `-`
  prefix, OR-combination within the same filter type, and AND-combination across different types

### Changed

- Unified metadata handling across tasks and artifacts via a shared trait, reducing code duplication in the store layer

### Fixed

- Entity file moves are now atomic (write-to-temp then rename), eliminating partial-read and TOCTOU race conditions
- Unrecognized event types in the database now produce a user-visible error instead of panicking
- Web view tables no longer overflow the container on narrow viewports
- HTML error responses in the web server are sanitized to prevent XSS
- Dashboard handler now logs errors instead of silently swallowing them
- Task link updates are now atomic via a dedicated `links` patch field
- Metadata values set via the CLI are parsed as typed TOML values (integers, booleans, floats) instead of always strings
- Native TOML datetime values are handled correctly during deserialization
- Project directory resolution no longer contains an unreachable code branch
- Integer casts in iteration orchestration replaced with correct `usize` types
- UI text truncation no longer panics on certain inputs

## [v0.4.0] - 2026-04-01

### Added

- `gest undo` command reverses the most recent mutating transaction by restoring file snapshots, backed by a new SQLite
  event store
- `GEST_STATE_DIR` environment variable and `storage.state_dir` config field for controlling event store location
- `gest tag add|remove|list` subcommands for managing tags across tasks, artifacts, and iterations
- `gest tags` command for listing all tags with optional entity-type filtering
- `--no-color` global flag for disabling colored output
- Cross-entity ID resolution with ambiguity detection and collision-safe ID generation
- Automatic event generation on task and iteration status, phase, and priority changes with author attribution
- Merged event/note activity timeline in `task show`, `iteration show`, and the web task detail page
- Two-tier color configuration: `[colors.palette]` for 11 semantic color slots and `[colors.overrides]` for per-token
  customization, with palette values cascading through to all referencing tokens
- `config show` now displays palette and override counts separately
- Web UI accessibility improvements: semantic landmarks and skip navigation, rem-based font sizes, WCAG 2.1 AA color
  contrast, focus-visible keyboard styles, form ARIA labels, semantic heading hierarchy, and keyboard-accessible
  relationship modal

### Changed

- **Breaking:** `GEST_DATA_DIR` now points to the global root directory instead of the project-specific path; use the
  new `GEST_PROJECT_DIR` env var (or `storage.project_dir` config field) for the old behavior
- **Breaking:** The flat `[colors]` config section is replaced by `[colors.palette]` and `[colors.overrides]`
- **Breaking:** Project discovery no longer matches undotted `gest/` directories; use `.gest/` or set
  `$GEST_PROJECT_DIR`
- **Breaking:** Task and iteration TOML files no longer write `resolved_at = ""` or `completed_at = ""` for unset
  datetime fields; existing files with empty strings are still read correctly

### Fixed

- `config set` now writes typed TOML values (integers, booleans, floats) instead of wrapping all values as strings
- Event recording no longer silently skipped when git `user.name` is unset — falls back to "unknown" author

## [v0.3.5] - 2026-03-31

### Added

- Task notes with author attribution (human via git config, agent via `--agent` flag), timestamps, and rendered markdown
  body — managed via `gest task note add|list|show|update|delete` subcommands
- Notes appear in `task show` output and the web UI task detail page with Gravatar avatars and agent badges
- Iteration orchestration commands: `iteration status` for progress tracking, `iteration next` for peeking at or
  claiming the next available task, and `iteration advance` for signaling phase completion
- `--assigned-to` filter on `task list` for narrowing tasks by assignee
- `--has-available` filter on `iteration list` for finding iterations with claimable work
- `gest search` now includes iterations in search results

## [v0.3.4] - 2026-03-31

### Fixed

- ID prefix validation now rejects invalid characters, preventing potential path injection via crafted entity IDs

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

[Unreleased]: https://github.com/aaronmallen/gest/compare/0.4.3...main
[v0.2.0]: https://github.com/aaronmallen/gest/compare/0.1.0...0.2.0
[v0.2.1]: https://github.com/aaronmallen/gest/compare/0.2.0...0.2.1
[v0.2.2]: https://github.com/aaronmallen/gest/compare/0.2.1...0.2.2
[v0.2.3]: https://github.com/aaronmallen/gest/compare/0.2.2...0.2.3
[v0.3.0]: https://github.com/aaronmallen/gest/compare/0.2.3...0.3.0
[v0.3.1]: https://github.com/aaronmallen/gest/compare/0.3.0...0.3.1
[v0.3.2]: https://github.com/aaronmallen/gest/compare/0.3.1...0.3.2
[v0.3.3]: https://github.com/aaronmallen/gest/compare/0.3.2...0.3.3
[v0.3.4]: https://github.com/aaronmallen/gest/compare/0.3.3...0.3.4
[v0.3.5]: https://github.com/aaronmallen/gest/compare/0.3.4...0.3.5
[v0.4.0]: https://github.com/aaronmallen/gest/compare/0.3.5...0.4.0
[v0.4.1]: https://github.com/aaronmallen/gest/compare/0.4.0...0.4.1
[v0.4.2]: https://github.com/aaronmallen/gest/compare/0.4.1...0.4.2
[v0.4.3]: https://github.com/aaronmallen/gest/compare/0.4.2...0.4.3
