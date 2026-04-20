---
title: How to use gest
description: A guided walkthrough of gest, from a rough idea to parallel execution, using a real feature from the gest repo.
keywords: [agentic workflow CLI, parallel AI agents, Claude Code, spec tracking for AI coding, AI-assisted development CLI, gest walkthrough]
---

# How to use gest

## What is gest?

**gest** is a local-first work tracker built for AI-assisted development. You
describe a feature; an agent breaks it into pieces; gest stores those pieces
— and the relationships between them — in a single SQLite database on your
machine. No server, no account, no cloud service to sign up for.

Every command speaks JSON, so agents and scripts read the work queue, claim
work, and write back results through the same CLI you use by hand. The data
lives locally (or in a [sync mirror][storage] you can commit alongside your
code), so everything is diffable and travels with your repo.

### The three things gest tracks

- An [**artifact**][artifacts] is a design document — a spec, an
  [ADR][categorizing-artifacts], an RFC, a brainstorm. Stored as Markdown
  with YAML frontmatter.
- A [**task**][tasks] is a unit of work — a thing to build, fix, or check.
  Tasks have a [priority][priority] and a [phase][phase].
- An [**iteration**][iterations] is an execution plan — a group of tasks
  with a [phased dependency graph][dependency-graphs] that agents walk from
  start to finish.

Every artifact, task, and iteration has a short 8-character ID (e.g.
`xtnsuxns`). You'll see these everywhere: the CLI prints them, commands
take them, agents pass them around.

## How we build gest with gest

gest itself is just a CLI and a database — it doesn't ship any agent
integration. But to actually drive the workflow, **we** (the gest
maintainers) use three [Claude Code][claude-code] skills that live in the
gest repo alongside the source, and this page walks through the same three
skills.

Each skill is a text file: a prompt that tells an agent what to do, what
`gest` commands to run, and when to hand off to the next skill. You can
[grab the skills from the repo][gest-skills] to use as-is, adapt the
prompts for whichever agent you prefer, or ignore skills entirely and run
the CLI commands by hand — gest doesn't care.

- **[`/brainstorm`][brainstorm-skill]** turns a rough idea into a spec
  [artifact][artifacts]. You describe the problem in your own words; the
  skill asks clarifying questions, proposes approaches with trade-offs,
  and saves the agreed direction.
- **[`/plan`][plan-skill]** turns a spec into a plan. It decomposes the
  spec into [tasks][tasks], groups them into [phases][phase] that can run
  in parallel, and wraps the result in an [iteration][iterations].
- **[`/orchestrate`][orchestrate-skill]** runs the plan. It walks each
  phase, spins up a [separate worktree][working-across-workspaces] per
  task in the phase, and dispatches an agent into each worktree. Tasks in
  the same phase run at the same time.

## A worked example

Let's walk through a real feature that's being tracked in the gest repo
right now: **giving the gest CLI proper exit codes**. Today every error
exits with `1`, so scripts wrapping `gest` can't tell "no tasks available"
apart from "bad flag" without reading error messages. We'd like each error
category to exit with a distinct, standards-based code.

### Step 1: Start with an idea

Open [Claude Code][claude-code] and run `/brainstorm` with a rough
description of the problem:

```text
/brainstorm every gest CLI error exits with 1, scripts can't branch on category
```

The skill reads the relevant parts of the codebase, asks clarifying
questions ("should we use BSD [`sysexits.h`][sysexits], or invent our own
scheme?"), and once the direction is clear, saves a spec
[artifact][artifacts]. Under the hood, it runs:

```bash
cat spec.md | gest artifact create "Exit Code Taxonomy (sysexits.h)" \
  --tag spec --tag cli -q
```

```text
xtnsuxns
```

That's the spec's ID. You can [view the spec][gest-artifact-show] with
`gest artifact show xtnsuxns` (or in the [built-in web
dashboard][gest-serve] at `gest serve`).

The brainstorm raised an architectural call big enough to warrant an
[ADR][categorizing-artifacts] of its own — narrowing one enum variant and
adding new ones. The skill writes the ADR the same way:

```bash
cat adr.md | gest artifact create \
  "ADR-DRAFT: Exit Code Contract for the gest CLI" \
  --tag adr --tag cli -q
```

```text
prsooyor
```

### Step 2: Plan the work

Pass the spec's ID to `/plan`:

```text
/plan xtnsuxns
```

The skill reads the spec, decides whether the work is one issue or many,
and — for work big enough to split — batches the [tasks][tasks] via
newline-delimited JSON. Each task gets a [phase][phase] number: tasks in
the same phase can run in parallel; tasks in a later phase wait for
earlier ones.

```bash
cat <<'EOF' | gest task create --batch -q
{"title":"Audit cli::Error construction sites","phase":1,"priority":1,"tags":["cli"],"links":["child-of:xtnsuxns"]}
{"title":"Update agent profile docs for new exit codes","phase":1,"priority":2,"tags":["docs","cli"],"links":["child-of:xtnsuxns"]}
{"title":"Refactor cli::Error: add variants, exit_code, ExitCode main","phase":2,"priority":0,"tags":["cli"],"links":["child-of:xtnsuxns"]}
EOF
```

```text
qzkwulpt
sxyqyqlr
ywnqunto
```

Three IDs come back — one per task, in the order they were created.
`/plan` then wraps those tasks in an [iteration][iterations], names it,
and prints the plan as ASCII with [`iteration graph`][gest-iteration]:

```bash
gest iteration create "Exit Code Taxonomy (sysexits.h)" -q
gest iteration graph yzyomkzz
```

```text
Exit Code Taxonomy (sysexits.h)
  4 phases · 8 tasks

  ◆  Phase 1  ──
  ├─╮
  ○ │  qzkwulpt  [P1]  Audit cli::Error construction site…  ○ open  ! blocking
  │ ○  sxyqyqlr  [P2]  Update agent profile documentation…  ○ open
  ├─╯
  │
  ◆  Phase 2  ──
  ⊗  ywnqunto  [P0]  Refactor cli::Error: add variants,…  ⊗ blocked  ! blocking  blocked-by qzkwulpt
  │
  ◆  Phase 3  ──
  ├─╮─╮─╮
  ⊗ │ │ │  pkpopvny  [P1]  Fix additional cli::Error miscateg…  ⊗ blocked
  │ ⊗ │ │  tnlzoutr  [P0]  Fix detach: replace Io misuse with…  ⊗ blocked
  │ │ ⊗ │  nlumrswo  [P0]  Fix iteration advance: replace Edi…  ⊗ blocked
  │ │ │ ⊗  osnqmnyn  [P0]  Fix iteration next: replace proces…  ⊗ blocked
  ├─╯─╯─╯
  │
  ◆  Phase 4  ──
  ⊗  sotpkmsx  [P0]  Add integration test coverage for …  ⊗ blocked
```

Each `◆  Phase N  ──` line is a checkpoint. Tasks under `Phase 1` run
first and can go in parallel; everything in `Phase 2` waits for Phase 1
to finish, and so on. The `blocked-by` labels on later tasks point back
at the tasks they depend on.

### Step 3: Run the plan

Pass the iteration's ID to `/orchestrate`:

```text
/orchestrate yzyomkzz
```

For each phase, `/orchestrate` looks at the unblocked tasks and dispatches
one agent per task. If the phase has more than one task — like Phase 1
above (`qzkwulpt` and `sxyqyqlr`) — it creates a
[separate worktree][working-across-workspaces] for each agent, so they
can edit the code without stepping on each other. Every agent calls
[`iteration next --claim`][gest-iteration]; gest hands out a different
task atomically, so there's no risk of two agents picking up the same
one:

```bash
# agent A                                             # agent B
gest iteration next yzyomkzz --claim \                gest iteration next yzyomkzz --claim \
  --agent agent-a --json                                --agent agent-b --json
```

```json
// agent A                                            // agent B
{"id":"qzkwulpt...","title":"Audit cli::Error...",    {"id":"sxyqyqlr...","title":"Update agent profile...",
 "status":"inprogress","phase":1,"priority":1,         "status":"inprogress","phase":1,"priority":2,
 "assigned_to":"agent-a"}                               "assigned_to":"agent-b"}
```

Once every task in the phase is `done`, `/orchestrate` moves on to Phase
2; then Phase 3; and so on until the iteration is complete. You can check
progress at any time — same JSON that powers the
[web dashboard][gest-serve]:

```bash
gest iteration status yzyomkzz --json
```

```json
{
  "id": "yzyomkzzxrlprqmppxllvouxlooyxxnw",
  "title": "Exit Code Taxonomy (sysexits.h)",
  "status": "active",
  "phases": 4,
  "total_tasks": 8,
  "open": 8,
  "in_progress": 0,
  "done": 0,
  "progress": 0
}
```

### Step 4: Ship it

As each agent finishes its task, it commits the work to your repo. When
the whole iteration is done, a separate [`/changelog`][changelog-skill]
pass reads the new commits and turns them into release notes in your
`CHANGELOG.md`. The first brick of this exit-code work — `iteration
next` returning `2` when no tasks are available — already shipped in
v0.5.4, and you can read it in the real [CHANGELOG][changelog]:

```text
## [v0.5.4] - 2026-04-14

### Changed

- `iteration next` returns proper error codes (exit 2 when no tasks are
  available) instead of calling `process::exit` directly
```

## Where to go next

- **[Install gest][installation]** — one command on macOS or Linux.
- **[Quick start][quick-start]** — the shortest path from zero to a
  working gest project, by hand (no agent required).
- **[Core concepts][concepts]** — deeper reference for artifacts, tasks,
  iterations, phases, links, and storage.
- **[CLI reference][cli-task]** — every flag on every `gest` command.

---

*Output captured from commit [`eb5a2ba`][capture-commit]. IDs are shown
as 8-character short forms; internally they are 32-character lowercase
alphanumeric.*

[artifacts]: ./concepts.md#artifacts
[categorizing-artifacts]: ./concepts.md#categorizing-artifacts
[changelog]: https://github.com/aaronmallen/gest/blob/main/CHANGELOG.md
[changelog-skill]: https://github.com/aaronmallen/gest/blob/main/.agents/profiles/git/skills/changelog/SKILL.md
[capture-commit]: https://github.com/aaronmallen/gest/commit/eb5a2ba0a016ca6c9d332cc7754b81a02a9e5baf
[brainstorm-skill]: https://github.com/aaronmallen/gest/blob/main/.agents/profiles/default/skills/brainstorm/SKILL.md
[claude-code]: https://claude.com/claude-code
[cli-task]: ../cli/task.md
[concepts]: ./concepts.md
[dependency-graphs]: ./concepts.md#dependency-graphs
[gest-artifact-show]: ../cli/artifact.md
[gest-skills]: https://github.com/aaronmallen/gest/tree/main/.agents/profiles
[gest-iteration]: ../cli/iteration.md
[gest-serve]: ../cli/serve.md
[installation]: ./installation.md
[iterations]: ./concepts.md#iterations
[orchestrate-skill]: https://github.com/aaronmallen/gest/blob/main/.agents/profiles/git/skills/orchestrate/SKILL.md
[phase]: ./concepts.md#phase
[plan-skill]: https://github.com/aaronmallen/gest/blob/main/.agents/profiles/default/skills/plan/SKILL.md
[priority]: ./concepts.md#priority
[quick-start]: ./quick-start.md
[storage]: ./concepts.md#storage-modes
[sysexits]: https://man.openbsd.org/sysexits.3
[tasks]: ./concepts.md#tasks
[working-across-workspaces]: ./concepts.md#working-across-workspaces
