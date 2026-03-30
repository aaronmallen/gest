---
layout: home
hero:
  name: gest
  text: Project-aware task and artifact tracking
  tagline: Manage agent-generated artifacts and task backlogs alongside your code — stored as plain files, versioned with your project.
  actions:
    - theme: brand
      text: Get Started
      link: /getting-started/installation
    - theme: alt
      text: GitHub
      link: https://github.com/aaronmallen/gest
features:
  - title: Tasks
    details: Create and manage tasks with priorities, phases, tags, and links. Built for AI-assisted workflows where agents generate and refine work items as you go.
  - title: Artifacts
    details: Store specs, ADRs, RFCs, and design documents as versioned Markdown with YAML frontmatter. Keep architectural decisions next to the code they describe.
  - title: Iterations
    details: Group tasks into iterations with dependency graphs and phase-based execution. Plan work in ordered phases so agents and humans stay in sync.
  - title: Search
    details: Full-text search across every task and artifact. Pipe JSON output into scripts or use it interactively to find what you need fast.
  - title: Plain Files
    details: TOML for tasks, Markdown for artifacts — no database, no server. Everything is inspectable, diffable, and friendly to version control.
  - title: Global or Local
    details: Store data globally in ~/.local/share/gest/ for cross-project tracking, or locally in .gest/ to keep everything inside a single repo.
---
