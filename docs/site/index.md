---
layout: home
hero:
  name: gest
  text: Parallel execution for AI-assisted development
  tagline: Decompose agent-generated work into phased iterations, dispatch independent tasks concurrently, and browse everything through a built-in web dashboard.
  actions:
    - theme: brand
      text: Get Started
      link: /getting-started/installation
    - theme: alt
      text: GitHub
      link: https://github.com/aaronmallen/gest
features:
  - title: ⚡ Parallel Execution
    details: Group tasks into phased iterations with dependency tracking. Tasks in the same phase run concurrently across workspaces — no more sequential, single-context-window bottlenecks.
  - title: 🖥️ Web Dashboard
    details: Browse tasks, artifacts, and iterations in a built-in web UI. Inspect status at a glance, view kanban boards, search across everything, and read rendered Markdown — all from gest serve.
  - title: 📄 Artifacts & Specs
    details: Store specs, ADRs, RFCs, and design documents as versioned Markdown with YAML frontmatter. Keep architectural decisions next to the code they describe.
  - title: 🤖 Agent-Native
    details: Every command supports --json output. Agents read the work queue, claim tasks, update status, and store artifacts — all through the CLI without any special integration.
  - title: 🗂️ SQLite-First, Git-Friendly
    details: Entity data lives in a single SQLite database — atomic writes, relational queries, fast dependency graphs. Opt into a .gest/ sync mirror (YAML + Markdown) to commit your data alongside your code.
  - title: 🌐 Global or Local
    details: By default, projects share one SQLite database at ~/.local/share/gest/gest.db. Add --local on init to also materialize a .gest/ mirror inside the repo for version control and collaboration.
---
