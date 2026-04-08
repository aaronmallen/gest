# Why gest?

AI agents are good at generating tasks. They decompose a feature request into dozens of subtasks,
write specs, and propose execution plans. The hard part is turning that output into parallel
workstreams without losing track of what depends on what.

Gest solves this by giving you a lightweight task and artifact store backed by a local SQLite
database (via libsql). The database is the source of truth — atomic writes, relational queries,
fast dependency graphs — and an optional `.gest/` sync mirror writes YAML and Markdown files
alongside your code so the data is inspectable, diffable, and travels with your VCS. No server,
no accounts, no team-facing infrastructure.

## The problem

A typical AI-assisted workflow looks like this:

1. You describe a feature.
2. An agent produces a plan with 15-20 tasks.
3. You execute them one at a time, in one context window, hoping nothing gets lost.

This breaks down quickly. Sequential execution is slow. A single context window can't hold the
full plan plus the code it's modifying. And if the agent loses context mid-way, you're back to
manually piecing together what was done and what remains.

What you need is a way to decompose work into phases, track dependencies, and dispatch
independent tasks in parallel -- without adding infrastructure overhead.

## The workflow

Gest structures work into three layers: **artifacts** capture the design, **tasks** capture
the work, and **iterations** organize tasks into a phased execution plan.

Here's a concrete example. You're adding a new CLI command with several moving parts:

```sh
# Capture the design as an artifact
gest artifact create "Add export command" \
  --tag spec \
  --body "Export tasks and artifacts to JSON or CSV for external tooling."

# Create tasks with phase assignments
gest task create "Add export data types" --phase 1 --priority 0
gest task create "Add CSV formatter" --phase 1 --priority 1
gest task create "Add JSON formatter" --phase 1 --priority 2
gest task create "Wire up CLI command" --phase 2 --priority 0
gest task create "Add integration tests" --phase 3 --priority 0
```

Phase 1 tasks have no dependencies on each other -- the data types, CSV formatter, and JSON
formatter can all be built concurrently. Phase 2 wires them together. Phase 3 validates the
result.

```sh
# Set dependencies
gest task link <wire-up-id> blocked-by <data-types-id>
gest task link <wire-up-id> blocked-by <csv-id>
gest task link <wire-up-id> blocked-by <json-id>

# Group into an iteration
gest iteration create "Implement export command"
gest iteration add <iteration-id> <task-id>  # repeat for each task

# Visualize the plan
gest iteration graph <iteration-id>
```

The iteration graph shows which tasks can run in parallel and which must wait. Agents or
developers can pick up independent tasks concurrently, each in its own workspace.

## What makes it work

The parallelization story depends on two things built into gest's data model:

**Phased tasks.** Every task has an optional `phase` field -- a numeric label that groups tasks
for concurrent execution. Tasks in the same phase have no ordering dependency on each other
and can be dispatched to separate agents or workspaces simultaneously. Lower phases execute
first, higher phases wait. This is the core mechanism that turns a flat task list into a
parallel execution plan.

**Iterations with dependency tracking.** An iteration groups related tasks and overlays
`blocked-by` / `blocks` relationships between them. The `gest iteration graph` command
visualizes exactly which tasks can run now, which are waiting, and what the critical path
looks like. Agents read this graph, pick up unblocked tasks, and mark them done -- the
remaining work automatically unblocks.

Together, phases and iterations give you a structured way to go from "here are 15 tasks" to
"here are 3 waves of concurrent work with explicit dependencies between them."

## A dashboard for humans

Agents interact with gest through the CLI and JSON output. Humans need something more visual.
`gest serve` starts a local web dashboard where you can browse and manage everything gest
tracks:

- **Status overview** — entity counts and status breakdown at a glance.
- **Task and artifact views** — filter, search, and inspect with rendered Markdown.
- **Iteration detail** — tasks grouped by phase with dependency visualization.
- **Kanban board** — columns mapped to task status for tracking iteration progress.
- **Full-text search** — find anything across tasks, artifacts, and iterations without
  memorizing IDs.

The dashboard is read/write — you can update tasks, change statuses, and manage iterations
directly from the browser. It runs entirely local, requires no setup beyond `gest serve`, and
reads and writes the same SQLite database the CLI uses, so changes in one surface show up
immediately in the other.

This matters because parallel agent execution generates a lot of state. When three agents are
working concurrently on phase 1 of an iteration, you want a place to see at a glance what's
in progress, what's blocked, and what's done — without switching between terminal windows.

## Fits your existing stack

Gest is not a replacement for GitHub Issues, Jira, or your team's project tracker. It operates
at a different level -- in-flight development work that moves too fast for a team-facing tool.

Use gest for decomposition and parallel execution during development. When a task needs broader
visibility -- a bug report, a feature request, a cross-team dependency -- promote it to your
team tracker. Every list and search command supports `--json` for scripting this handoff.

```sh
# Example: find all open tasks tagged "blocker" and export for triage
gest task list --tag blocker --json
```

Gest handles the fast, local, developer-side loop. Your team tracker handles coordination and
visibility. They complement each other.
