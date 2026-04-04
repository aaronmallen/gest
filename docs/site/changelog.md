# Changelog

What's new in gest — told version by version.

## v0.4.4

<span style="opacity: 0.5">2026-04-04</span>

### Security Headers

The web server now sets `Content-Security-Policy`, `X-Frame-Options`, and `X-Content-Type-Options` on
every response, hardening the default setup against clickjacking and content-type sniffing.

### Bug Fixes

- The `-q` flag on mutation commands was printing the full 32-character ID — it now outputs the
  8-character short ID, matching how IDs are displayed everywhere else
- Store writes use the `tempfile` crate for proper temporary files, avoiding collisions when multiple
  agents write concurrently
- `config get storage.project_dir` now returns the resolved path instead of `null` when no explicit
  override is set ([#31](https://github.com/aaronmallen/gest/issues/31))

### Performance

Iteration orchestration operations (`all_phases`, `assignees`, `phases_with_incomplete`) now use
sort+dedup instead of HashSet round-trips, and an unnecessary double-clone has been eliminated.

## v0.4.3

<span style="opacity: 0.5">2026-04-03</span>

### Iteration Cancel and Reopen

Iterations can now be cancelled and reopened with dedicated commands. `gest iteration cancel <id>`
marks the iteration as cancelled and automatically cascades to all non-terminal tasks — open and
in-progress tasks become cancelled, while done tasks are left untouched. `gest iteration reopen <id>`
reverses the operation, restoring the iteration and its cancelled tasks. The old `failed` status
remains as a deprecated alias for backward compatibility.

### Live Reload

The web UI now updates in real time. When you change project files on disk, pages connected to the
web server receive updates via server-sent events and swap in fresh content without a full reload.
The file watcher debounce defaults to 2000ms and is configurable via `serve.debounce_ms`.

### Request Logging

Every HTTP request to the web server is now logged with method, path, response status, and elapsed
time. Control the verbosity with the `serve.log_level` setting (default: info).

### Web Dashboard

The dashboard task card now shows only actionable work (open + in-progress) rather than all tasks.
A new iteration status breakdown row shows active, completed, and cancelled counts with links to
filtered views.

### Bug Fixes

- `read_from_editor` no longer drops content due to a missing return statement

## v0.4.2

<span style="opacity: 0.5">2026-04-02</span>

### Ergonomics Everywhere

This release is all about reducing friction. Nearly every common workflow got shorter.

**Command aliases** let you move faster: `gest t` instead of `gest task`, `gest a` for `gest artifact`,
`gest i` for `gest iteration`, `gest grep` for `gest search`. Subcommands gained natural aliases too —
`new`, `ls`, `view`, `edit`, and `rm` all work where you'd expect. Top-level `u` and `s` shortcuts
round out `undo` and `serve`.

**Lifecycle shortcuts** — `task complete`, `task cancel`, and `task block` — handle the most common
status transitions in a single command instead of `task update --status`.

**Tags got simpler.** A unified `--tag` flag works on both create and update commands, and tag/untag
commands accept comma-separated values (`gest task tag <id> rust,cli`) so you can apply multiple tags
in one shot.

### Machine-Readable Output

Every mutation command now accepts `--json` for structured output and `-q` for bare IDs — making gest
a first-class citizen in shell pipelines and automation scripts. `meta get` gained `--json` and `--raw`
variants for programmatic value access.

### Batch and Pipeline Support

The new `--batch` flag reads NDJSON from stdin to create multiple tasks or artifacts in a single
invocation. Body and description fields can also be piped from stdin implicitly, so `echo "notes" |
gest task create "my task"` just works.

Create commands gained `-i`/`--iteration` to add the new entity to an iteration inline, and `task
create` supports `-l`/`--link` for creating relationships at creation time — no follow-up commands
needed.

### Search Paging

Search output is now paged through `$PAGER` (falling back to `less -R`) when stdout is a terminal.
Piped and redirected output bypasses the pager automatically.

### Under the Hood

Entity operations are now backed by generic trait-based action functions, and catch-all error variants
have been replaced with domain-specific types that produce clearer diagnostics. Tag list tables now
render with proper theme formatting.

## v0.4.1

<span style="opacity: 0.5">2026-04-01</span>

### Search Filters

`gest search` now understands structured queries. Use `is:`, `tag:`, `status:`, and `type:` prefixes to narrow results —
negate any filter with a `-` prefix (e.g. `-status:done`). Filters of the same type OR-combine, and different filter
types AND-combine, giving you precise control without complex syntax.

### Reliability and Safety

A batch of fixes hardens the store and web server:

- Entity file moves are now atomic (write-to-temp then rename), closing a TOCTOU race that could leave partial files on
  disk
- Unrecognized event types produce a clear error instead of panicking
- HTML error responses are sanitized to prevent reflected XSS
- Dashboard errors are logged instead of silently swallowed
- Task link mutations use a dedicated patch field for atomicity
- CLI metadata values are parsed as typed TOML (numbers, booleans) rather than always strings
- Native TOML datetimes are deserialized correctly
- Integer casts in orchestration replaced with correct types
- UI text truncation no longer panics on edge-case inputs

## v0.4.0

<span style="opacity: 0.5">2026-04-01</span>

### Undo

You can now reverse the most recent mutating command with `gest undo`. Under the hood, a new SQLite-backed event store
captures before/after snapshots of every file mutation. If something goes wrong, `gest undo` restores the previous state
in one step. The event store location is configurable via `GEST_STATE_DIR` or `storage.state_dir`.

### Tags

New `gest tag add|remove|list` subcommands let you manage tags on any entity. `gest tags` lists all tags across the
project with optional entity-type filtering. Tags are a lightweight way to categorize and find related work.

### Activity Timeline

Task and iteration status, phase, and priority changes are now recorded automatically as events with author attribution.
The `show` views merge events and notes into a single chronological activity timeline — in both the CLI and the web UI —
so you can see the full history of an entity at a glance.

### Theming Overhaul

Color configuration has been restructured into two tiers: `[colors.palette]` defines 11 semantic color slots, and
`[colors.overrides]` provides per-token customization. Palette colors cascade through to all tokens that reference a
given slot, making it easy to restyle the entire UI by changing a few palette values. `config show` now displays palette
and override counts separately.

### Web UI Accessibility

The web UI received a comprehensive accessibility pass to meet WCAG 2.1 AA requirements:

- Semantic HTML landmarks and a skip-to-content link for screen readers
- Heading hierarchy with proper `h1`–`h6` elements
- Font sizes converted to `rem` units respecting browser preferences
- Color contrast updated to meet the 4.5:1 minimum ratio
- Focus-visible outlines on all interactive elements for keyboard navigation
- ARIA labels on form fields and the relationship modal
- Keyboard-accessible relationship modal with focus trap and Escape-to-close

### Cross-Entity IDs

ID generation now checks for prefix collisions across all entity types before assigning a short ID, and a new
cross-entity resolver can look up any entity by prefix with ambiguity detection. The `--no-color` global flag is also
available for scripting.

### Breaking Changes

This release bundles several breaking changes:

- **`GEST_DATA_DIR` semantics changed.** It now points to the global root (e.g. `~/.local/share/gest`) instead of the
  project-specific directory. Use the new `GEST_PROJECT_DIR` env var or `storage.project_dir` config field for the old
  behavior.
- **Color config restructured.** The flat `[colors]` section is replaced by `[colors.palette]` and `[colors.overrides]`.
- **Project discovery tightened.** Only `.gest/` directories are matched during walk-up discovery; unrelated directories
  named `gest/` are no longer false positives.
- **Empty datetime fields omitted.** Task and iteration TOML files no longer write `resolved_at = ""` or
  `completed_at = ""`. Existing files with empty strings are still read correctly.

### Bug Fixes

- `config set` now writes typed TOML values instead of wrapping everything as strings
- Event recording no longer silently skipped when git `user.name` is unset

## v0.3.5

<span style="opacity: 0.5">2026-03-31</span>

### Task Notes

Tasks can now carry threaded notes with author attribution. Human authors are identified automatically from your git
config, and agent-authored notes are tagged with `--agent`. Each note includes a timestamp and renders its body as
markdown.

- Add, list, show, update, and delete notes via `gest task note` subcommands
- Notes appear in `task show` output and on the web UI task detail page
- Gravatar avatars and agent badges distinguish human from machine authors

### Iteration Orchestration

New commands make it easier to drive multi-phase work through an iteration without manually juggling task status.

- `iteration status` shows progress at a glance
- `iteration next` peeks at or claims the next available task
- `iteration advance` signals that a phase is complete

### Filtering and Search

- `task list --assigned-to` narrows results by assignee
- `iteration list --has-available` finds iterations with claimable work
- `gest search` now includes iterations in its results

## v0.3.4

<span style="opacity: 0.5">2026-03-31</span>

### Security

ID prefix validation now rejects invalid characters, closing a potential path injection vector via crafted entity IDs.

## v0.3.3

<span style="opacity: 0.5">2026-03-31</span>

### Web UI Forms

The web UI gained create and edit forms for tasks and artifacts, complete with markdown preview, relationship
management, and inline search — so you can manage your project without leaving the browser.

- Dashboard status cards now link directly to filtered task views
- New/edit buttons on list and detail pages for quick navigation

### Display Improvements

Short ID prefixes are now clickable links throughout the web UI. Tags are clickable for quick filtering, and artifact
type labels are used consistently across all views.

### Bug Fixes

- Store writes now respect whether an entity lives in the active or archive/resolved directory, preventing duplicates
  on mutation
- CLI flags (`--version`, `--verbose`) and shell completion variants now include help text
  ([#24](https://github.com/aaronmallen/gest/issues/24))

## v0.3.2

<span style="opacity: 0.5">2026-03-31</span>

### Built-in Web Server

`gest serve` launches a local web UI with a dashboard, task/artifact/iteration views, a kanban board, and full-text
search — everything you need to browse your project visually
([#13](https://github.com/aaronmallen/gest/issues/13)–[#23](https://github.com/aaronmallen/gest/issues/23)).

- `--port`, `--bind`, and `--no-open` flags control the local server
  ([#15](https://github.com/aaronmallen/gest/issues/15))
- `[serve]` config section with `port`, `bind_address`, and `open` settings
  ([#14](https://github.com/aaronmallen/gest/issues/14))

### Performance

- Iteration phase counts are now cached instead of recomputed on every read
- `resolve_dot_path` walks by reference instead of cloning, reducing allocations
- Shared helpers consolidated across store operations, CLI metadata handling, and tag/untag commands

### Bug Fixes

- Multi-word `EDITOR` values (e.g. `code --wait`) are now parsed correctly via shell tokenization
- Nested metadata `set_nested` is bounds-checked with a depth limit to prevent stack overflows
- Version line no longer appears in the `--help` banner
- `gest search --expand` now shows full content instead of truncated snippets
  ([#11](https://github.com/aaronmallen/gest/issues/11))

## v0.3.1

<span style="opacity: 0.5">2026-03-30</span>

### Per-Entity Directory Overrides

You can now control where each entity type is stored on disk via `GEST_ARTIFACT_DIR`, `GEST_TASK_DIR`, and
`GEST_ITERATION_DIR` environment variables — or the corresponding `storage.*_dir` config fields.

### Performance

- Blocked-by status resolution for task lists now uses a single batched read pass instead of per-task I/O
- Search filtering avoids redundant string allocations during case-insensitive matching
- Prefix matching exits early after two matches instead of collecting all candidates
- Metadata serialization is skipped during search when maps are empty
- Store functions receive the full application config, removing the need for plumbing changes when new settings are
  accessed

## v0.3.0

<span style="opacity: 0.5">2026-03-30</span>

### Breaking Changes

This release includes two breaking changes:

- **TOML-only configuration.** JSON and YAML config files are no longer supported. Rename your config to `gest.toml`
  and convert the contents to TOML.
- **Global store by default.** `gest init` now creates the global data store. Use `gest init --local` if you want an
  in-repo `.gest/` directory.

### New UI

The entire terminal UI has been rebuilt with an atomic architecture (atoms, composites, views) for consistent, aligned
rendering across all commands.

## v0.2.3

<span style="opacity: 0.5">2026-03-29</span>

### Iterations

A new entity type for planning multi-phase work. Iterations have their own storage, UI components, CLI commands, and a
graph visualization that shows how tasks flow through phases.

### Richer Task Fields

Tasks now support `priority`, `assigned_to`, and `phase` fields for more structured project tracking.

### Search Improvements

`--expand` on `gest search` now works without `--json`, showing full detail blocks directly in the terminal.

## v0.2.2

<span style="opacity: 0.5">2026-03-27</span>

### Update Notifications

`gest version` now checks for newer releases and suggests `gest self-update` when one is available.

### Search Improvements

The `--expand` (`-e`) flag on `gest search` enriches `--json` output with full item details.

### Theme Tokens

New `indicator.blocked` and `indicator.blocking` theme tokens let you customize how task list status indicators look.

### Performance

Reduced unnecessary memory allocations in UI rendering and search, plus extracted shared helpers and removed dead code.

## v0.2.1

<span style="opacity: 0.5">2026-03-26</span>

### Bug Fixes

- Install script now downloads the correct checksum file during installation
- Self-update no longer fails with a 404 when fetching the target release

## v0.2.0

<span style="opacity: 0.5">2026-03-26</span>

### Shell Completions and Man Pages

New generation commands produce shell completions and man pages for your preferred shell
([#1](https://github.com/aaronmallen/gest/issues/1)).

### Markdown Rendering

Descriptions are no longer plain text in the terminal. Headings, code blocks, lists, blockquotes, and more are rendered
with styled formatting ([#2](https://github.com/aaronmallen/gest/issues/2),
[#3](https://github.com/aaronmallen/gest/issues/3)). Artifact and task detail views use this automatically
([#4](https://github.com/aaronmallen/gest/issues/4), [#5](https://github.com/aaronmallen/gest/issues/5)).

### Verbose Logging

Every command, store operation, and config discovery step now emits verbose logs for easier debugging.

### Breaking Change

`artifact list --include-archived` has been renamed to `--all` (`-a`), matching the convention used by task and search
commands.

### Performance

Faster unique ID prefix computation and shorter ID encoding.

### Bug Fixes

Log level semantics now correctly follow the debug=why, trace=result convention.

## v0.1.0

<span style="opacity: 0.5">2026-03-26</span>

The first release of gest — a lightweight CLI for tracking AI-agent-generated tasks and artifacts alongside your
project. Ships with `artifact` and `task` commands, TOML/JSON/YAML configuration, search, and editor integration.
