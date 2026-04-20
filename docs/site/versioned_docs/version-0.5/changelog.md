# Changelog

What's new in gest — told version by version.

## v0.5.6

<span style={{opacity: 0.5}}>2026-04-19</span>

### Pager on Detail Views

`gest artifact show` and `gest iteration show` now route their output through the pager, the same
way `list` and `search` already did. Long descriptions, timelines, and relationship blocks scroll
under your `$PAGER` instead of dumping into scrollback. `--no-pager`, the `pager.enabled` config
field, and the TTY/terminal-height policy all apply uniformly.

### Project JSON Envelope

`project archive`, `project delete`, and `project unarchive` now emit the same `Envelope<Project>`
JSON shape that the other entity commands use — the project row plus sidecar arrays for
relationships, tags, and notes. If you're scripting against the older `--json` output (added in
v0.5.4), update your parsers to read `envelope.entity` for the project body; the sidecar arrays
are additive.

### Purge Performance

`gest purge` now batches cascade deletes. Instead of issuing one `SELECT`/`DELETE` per row per
child table, it emits one `IN (...)` query per child table per entity type (tasks, iterations,
artifacts). On large cleanups this collapses thousands of round-trips into a handful, while the
child-first ordering and per-row audit events that `undo` relies on are preserved.

### Transaction Replay Hardening

`gest undo` now runs every replayed `table_name` and `before_data` column key through a closed
allowlist before interpolating it into the rendered SQL. A corrupt, tampered, or forward-
incompatible audit row surfaces as a clear `InvalidValue` error instead of being pasted into a
`DELETE`/`UPDATE`/`INSERT`. Well-formed rows replay identically.

### Web UI Navigation Guard

The `data-nav-select` selector in the web UI now rejects any option value that isn't a same-
origin relative path (must start with `/`, must not start with `//`). This closes off a defensive
hole where a selector populated from untrusted input could have fired a `javascript:` URI or a
protocol-relative redirect to an external host. Today all option values come from server-rendered
routes, so this is a belt-and-braces hardening rather than a fix for an exploitable bug.

### Internal Consolidation

Behind the CLI surface, a round of shared-helper extraction landed: title inference from piped
markdown, status-transition lifecycle commands, unlink audit payloads, short-ID `--quiet` output,
and summary rendering each collapsed from N copies to one. No user-visible behavior change, but
command handlers are now noticeably thinner.

## v0.5.5

<span style={{opacity: 0.5}}>2026-04-17</span>

### Removing Relationships

`gest task unlink` and `gest iteration unlink` close a long-standing gap: there was no way to remove a single
relationship between two entities once it had been created. The commands take a source and target ID, with an optional
`--rel <type>` filter to disambiguate when multiple edge types connect the same pair and `--artifact` when the target
is an artifact. Iteration-to-iteration unlinks delete both the forward and reciprocal rows atomically, so `gest undo`
restores both halves.

- `task unlink <id> <target> [--rel <type>] [--artifact]` ([#63](https://github.com/aaronmallen/gest/issues/63))
- `iteration unlink <id> <target> [--rel <type>] [--artifact]`

### Iteration Link Flag Parity

`iteration link` gained a `--rel <type>` flag matching the `task link` shape. The old positional form
(`iteration link <id> <rel> <target>`) still works but now emits a deprecation warning on stderr — switch to
`iteration link <id> <target> --rel <type>` going forward.

### Web UI

A new global kanban board surfaces every task in the project in a single view, independent of iteration membership.

## v0.5.4

<span style={{opacity: 0.5}}>2026-04-14</span>

### Mermaid Diagrams in the Web UI

Fenced ` ```mermaid ` code blocks now render as interactive diagrams on all detail pages, timeline notes, and the
markdown preview pane. The Mermaid v11 library is loaded from the jsDelivr CDN — if the CDN is unreachable the blocks
display as raw code, same as before. The CSP was updated to allow the CDN script origin and Mermaid's runtime style
injection.

Iteration descriptions also now render full markdown (headings, lists, code blocks, etc.) on the web detail page,
matching the behavior already present on artifact and task pages.

### Performance

List commands for tasks, artifacts, and iterations now batch tag, status, and phase queries into bulk lookups instead of
issuing one query per row. `iteration next` also eliminates per-task relationship fan-out by using batch-loaded data.

### CLI Polish

- `project delete`, `project archive`, and `project unarchive` gained `--json` and `--quiet` output flags
- `iteration next` returns exit code 2 (instead of a hard `process::exit`) when no tasks are available, making it
  composable in scripts
- `purge` no longer prompts for confirmation twice

## v0.5.3

<span style={{opacity: 0.5}}>2026-04-12</span>

### Bug Fixes

- `gest self-update` was downloading the `.sha256` checksum file instead of the `.tar.gz` release archive — the asset
  identifier filter was dropped during the v0.5.0 rewrite, broken since v0.5.0

## v0.5.2

<span style={{opacity: 0.5}}>2026-04-12</span>

### Project Lifecycle Management

Projects now have a full archive/delete lifecycle. `gest project archive` soft-archives a project — it disappears from
`project list` by default (use `--all` to reveal it with an `[archived]` badge) and detaches from any workspaces.
`gest project unarchive` reverses the operation. For permanent removal, `gest project delete` performs a non-undoable
cascade deletion of the project and all its owned tasks, iterations, and artifacts, with a confirmation prompt showing
exactly what will be removed.

- Archive and unarchive with entity count display and workspace reporting
  ([#38](https://github.com/aaronmallen/gest/issues/38))
- Cascade delete with tombstone fan-out for sync safety
  ([#38](https://github.com/aaronmallen/gest/issues/38))

### Bulk Purge

The new `gest purge` command consolidates cleanup into a single operation. It can remove terminal tasks, terminal
iterations, archived artifacts, archived projects, dangling relationships, and orphan tombstones — individually via
selector flags or all at once. `--dry-run` previews what would be removed without touching the database, and `--yes`
bypasses the confirmation prompt for scripted use.

- Selector flags: `--tasks`, `--iterations`, `--artifacts`, `--projects`, `--relationships`,
  `--tombstones` ([#39](https://github.com/aaronmallen/gest/issues/39))

### Smarter ID Prefixes

List and search output now displays the shortest unique prefix per entity instead of a single shared minimum length
across all IDs of the same type. This means shorter, easier-to-type prefixes for most entities while still guaranteeing
uniqueness.

## v0.5.1

<span style={{opacity: 0.5}}>2026-04-10</span>

### Web Security and Error Handling

The web dashboard received a comprehensive security and reliability pass. A CSP nonce middleware replaces
the previous `'unsafe-inline'` script/style policy, and new CSRF protection via double-submit signed
cookies guards all mutating requests. Gravatar avatars are now proxied through a first-party
`/avatars/{hash}` endpoint, so the dashboard no longer leaks browsing activity to an external origin and
works in air-gapped environments. Additional security headers — `Referrer-Policy`, `Permissions-Policy`,
`frame-ancestors`, `base-uri`, and `form-action` — round out the hardening.

Error handling has been unified under a single error type. Store "not found" errors now surface as proper
HTTP 404 responses instead of generic 500s, and validation failures return 400 with an HTML error page.

### Web Forms and Usability

Task create and edit forms gained a markdown preview toggle (matching the existing artifact form), and the
priority field is now a dropdown instead of a free-text numeric input — out-of-range values surface as a
user-friendly error with the form fields preserved.

Iteration tags now render as clickable `#` links, matching the pattern used in task and artifact views.

### Performance

Web list pages (tasks, iterations, artifacts) now batch tag and relationship lookups into two queries
instead of issuing per-row fan-out, and a new status counts query replaces the double-pass
count-then-filter pattern on the dashboard.

### Priority Labels

`--priority` on `task create` and `task update` now accepts `critical`, `high`, `medium`, `low`, and
`lowest` (case-insensitive) in addition to the 0–4 integer form.

### Iteration Graph

The iteration graph view has been rebuilt with phase headers, `◆` icons, parallel column indicators with
branch-drawing connectors, priority badges, and blocked/blocking dependency markers — restoring and
extending the detail level from v0.4.4.

### CLI Improvements

- `--json` output on entity commands now uses an Envelope format that includes relationships, tags, and
  notes alongside the entity data
- `--batch` flag on `iteration add` supports bulk NDJSON task addition with per-record phase control and
  all-or-nothing rollback semantics
- `--tag` and `--tags` arguments accept comma-separated values (e.g. `--tag a,b`)
- Confirmation prompts use an interactive Yes/No selector with arrow-key navigation instead of text input
- Rendered markdown headings in terminal output now show `#` level markers for visual distinction

## v0.5.0

v0.5.0 is the storage rewrite. Entity data moves from flat TOML/Markdown files under `.gest/` into a
single SQLite database (via libsql), and project identity becomes explicit so multiple worktrees of
the same repo can share a live view of the same tasks, artifacts, and iterations. Existing v0.4.x
projects import in one shot — see the
[v0.4 → v0.5 migration guide](./migration/v0-4-to-v0-5.md) for the full upgrade path, data mapping, and
rollback procedure.

### SQLite-First Storage

Entity data now lives in `<data_dir>/gest.db` — a single libsql database that holds tasks, artifacts,
iterations, notes, relationships, tags, events, and undo history in one place. Multi-entity writes
are atomic (so a task plus its tags plus its relationships land together or not at all), and
concurrent agents can hit the same store without fighting over flat-file locks.

For local-mode projects, `.gest/` sticks around as a **sync mirror**: the database is the source of
truth, but every mutation rewrites a set of human-readable YAML/Markdown files under `.gest/task/`,
`.gest/artifact/`, and `.gest/iteration/` (singular subdirectories, a small rename from the v0.4.x
plural layout) so the data stays inspectable and git-committable. Disable the mirror with
`storage.sync = false` or `GEST_STORAGE__SYNC=false` if you want the database only.

A new `[database]` config section lets you point gest at a remote libsql/Turso instance. Set `url`
directly, or provide `scheme`, `host`, `port`, `username`, `password`, and `auth_token`
individually:

```toml
[database]
url = "libsql://my-project.turso.io"
auth_token = "..."
```

Existing v0.4.x projects are imported with `gest migrate --from v0.4`. The migrator auto-discovers
both the local-mode `.gest/` layout and the global-mode `~/.local/share/gest/<hash>/` layout, preserves
every ID so external references (commit messages, PR bodies) keep working, and prints a summary of
what it imported. The guide walks through the details.

### Workspaces

In v0.4.x, two worktrees of the same repo produced two entirely separate stores: local mode wrote to
the worktree's own `.gest/`, and global mode keyed the project row by hashing the current working
directory, so each checkout got a different hash. Parallel agents couldn't share a project.

v0.5.0 makes project identity explicit. A project is a row in the database, identified by an assigned
ID. Additional checkouts opt in by **attaching** as a workspace:

```sh
# In the original checkout, find the project ID
gest project

# In a second worktree, attach to it
cd ../myapp-feature-a
gest project attach <project-id>
```

After attaching, both checkouts resolve to the same project row and share the same tasks, artifacts,
and iterations in real time. `gest project detach` reverses the operation, and `gest project list`
shows all known projects across your machine.

### Artifact Notes

Artifacts now support the same note management surface as tasks. Use `gest artifact note add`,
`list`, `show`, `update`, and `delete` to attach human- or agent-authored annotations to a spec, ADR,
or RFC over time. Notes appear in `artifact show` output and on the web UI artifact detail page,
with the same Gravatar avatars and agent badges you already get on task notes.

### Entity Deletion

Tasks and artifacts can now be hard-deleted from the store. `gest task delete <id>` and
`gest artifact delete <id>` remove the entity and all its dependent rows (notes, relationships, tags,
events). Both commands prompt for confirmation interactively; pass `--yes` to skip the prompt in
scripts. `task delete` additionally requires `--force` when the task is still a member of one or more
iterations — this keeps active sprint members from being removed by accident.

Destructive note commands (`task note delete`, `artifact note delete`) now prompt by default as well,
with the same `--yes` escape hatch.

### Theme Overhaul

The theme system has been rebuilt around a new token hierarchy and richer palette semantics, and the
default palette has been retuned across the CLI and web UI. Rather than inlining the full token list
here, see the [theming reference](./configuration/theming.md) for the complete set of tokens, the palette
slots they cascade from, and examples for overriding them in your `gest.toml`.

### New Flags and Ergonomics

- **`gest task claim <id>`** assigns the task to the current author and marks it in-progress in a
  single step — handy when picking up the next available task from an iteration
- **`--raw` (`-r`) output flag** on entity commands produces script-friendly plain output without
  themed styling, giving you a third output mode alongside the default human-readable and `--json`
  shapes
- **`--limit <N>`** on `task list`, `iteration list`, `artifact list`, and `gest search` caps result
  counts for faster scanning of large projects
- **`--metadata-json`** on create and update commands merges a JSON object into metadata on top of
  any `--metadata key=value` pairs, so you can set structured values in one shot without repeated
  `-m` flags
- **`gest artifact create`** gained `-i`/`--iteration` (link a new artifact to an iteration inline)
  and `-s`/`--source` (read the body from a file path instead of stdin or `$EDITOR`)
- **`gest config get storage.data_dir`** returns the resolved path instead of `null` when no explicit
  override is set

### Breaking Changes

- **Storage format changed to SQLite.** Existing v0.4.x projects must be imported with
  `gest migrate --from v0.4`. The migrator never touches the source files, so your old `.gest/` stays
  intact as a backup until you remove it yourself. Follow the
  [migration guide](./migration/v0-4-to-v0-5.md) for details.
- **Artifact `kind` field removed.** Categorization is tag-driven now. The migrator converts
  `kind: "spec"` into a `spec` tag automatically.
- **`artifact create` CLI reshaped.** The command takes a positional `[TITLE]` argument; the `--title`
  and `--type` flags are gone, and `-t` is now a shortcut for `--tag` instead of `--type`.
- **`type:` search filter removed.** Use `tag:<name>` instead (e.g. `gest search 'tag:spec'`). If you
  had `type:` queries in personal aliases or scripts, update them.
- **Environment variables renamed or removed.** `GEST_LOG_LEVEL` is now `GEST_LOG__LEVEL` (double
  underscore). `GEST_DATA_DIR` is now `GEST_STORAGE__DATA_DIR`. `GEST_PROJECT_DIR`, `GEST_STATE_DIR`,
  `GEST_ARTIFACT_DIR`, `GEST_TASK_DIR`, and `GEST_ITERATION_DIR` are all gone — the new storage model
  has no per-entity directory overrides.
- **Config fields removed.** `storage.project_dir`, `storage.state_dir`, `storage.artifact_dir`,
  `storage.task_dir`, and `storage.iteration_dir` are no longer honored; delete them from existing
  `gest.toml` files.
- **Undo state lives in the database.** There is no longer a separate state directory; undo history
  follows whichever database the command ran against.

## v0.4.4

<span style={{opacity: 0.5}}>2026-04-04</span>

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

<span style={{opacity: 0.5}}>2026-04-03</span>

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

<span style={{opacity: 0.5}}>2026-04-02</span>

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

<span style={{opacity: 0.5}}>2026-04-01</span>

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

<span style={{opacity: 0.5}}>2026-04-01</span>

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

<span style={{opacity: 0.5}}>2026-03-31</span>

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

<span style={{opacity: 0.5}}>2026-03-31</span>

### Security

ID prefix validation now rejects invalid characters, closing a potential path injection vector via crafted entity IDs.

## v0.3.3

<span style={{opacity: 0.5}}>2026-03-31</span>

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

<span style={{opacity: 0.5}}>2026-03-31</span>

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

<span style={{opacity: 0.5}}>2026-03-30</span>

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

<span style={{opacity: 0.5}}>2026-03-30</span>

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

<span style={{opacity: 0.5}}>2026-03-29</span>

### Iterations

A new entity type for planning multi-phase work. Iterations have their own storage, UI components, CLI commands, and a
graph visualization that shows how tasks flow through phases.

### Richer Task Fields

Tasks now support `priority`, `assigned_to`, and `phase` fields for more structured project tracking.

### Search Improvements

`--expand` on `gest search` now works without `--json`, showing full detail blocks directly in the terminal.

## v0.2.2

<span style={{opacity: 0.5}}>2026-03-27</span>

### Update Notifications

`gest version` now checks for newer releases and suggests `gest self-update` when one is available.

### Search Improvements

The `--expand` (`-e`) flag on `gest search` enriches `--json` output with full item details.

### Theme Tokens

New `indicator.blocked` and `indicator.blocking` theme tokens let you customize how task list status indicators look.

### Performance

Reduced unnecessary memory allocations in UI rendering and search, plus extracted shared helpers and removed dead code.

## v0.2.1

<span style={{opacity: 0.5}}>2026-03-26</span>

### Bug Fixes

- Install script now downloads the correct checksum file during installation
- Self-update no longer fails with a 404 when fetching the target release

## v0.2.0

<span style={{opacity: 0.5}}>2026-03-26</span>

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

<span style={{opacity: 0.5}}>2026-03-26</span>

The first release of gest — a lightweight CLI for tracking AI-agent-generated tasks and artifacts alongside your
project. Ships with `artifact` and `task` commands, TOML/JSON/YAML configuration, search, and editor integration.
