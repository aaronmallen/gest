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
  - title: 📁 Plain Files, No Infrastructure
    details: TOML for tasks, Markdown for artifacts — no database, no server, no accounts. Data is inspectable, diffable, and travels with your VCS.
  - title: 🌐 Global or Local
    details: Store data globally in ~/.local/share/gest/ for cross-project tracking, or locally in .gest/ to version-control everything inside a single repo.
---
