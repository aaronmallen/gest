# Architecture Decision Records

This directory contains Architecture Decision Records (ADRs) for the project.

## Index

| ID     | Title                                            | Status               | Date       |
|--------|--------------------------------------------------|----------------------|------------|
| [0001] | Flat Files Over SQLite for Storage               | Superseded by [0013] | 2026-03-26 |
| [0002] | Zero-Config Discovery with Fallback Chain        | Superseded by [0009] | 2026-03-26 |
| [0003] | jj-Style Reverse Hex Change IDs                  | Active               | 2026-03-26 |
| [0004] | Tasks as Freeform Execution Plans                | Active               | 2026-03-26 |
| [0005] | CLI Command Structure and Output Conventions     | Active               | 2026-03-26 |
| [0006] | UI Components as Display+Write Types             | Superseded by [0010] | 2026-03-26 |
| [0007] | Centralized Theme System Using yansi::Style      | Active               | 2026-03-26 |
| [0008] | Custom Logger with Styled Stderr Output          | Active               | 2026-03-26 |
| [0009] | Zero-Config Discovery (TOML Only)                | Active               | 2026-03-29 |
| [0010] | Atomic UI Architecture                           | Superseded by [0015] | 2026-03-30 |
| [0011] | Askama for Web UI Templating                     | Active               | 2026-03-31 |
| [0012] | SQLite for the Event Store                       | Superseded by [0013] | 2026-04-01 |
| [0013] | Global-Only Storage with Project Identity        | Active               | 2026-04-06 |
| [0014] | libsql for Remote Database Support               | Active               | 2026-04-06 |
| [0015] | Atoms/Molecules/Views UI Architecture            | Active               | 2026-04-06 |
| [0016] | Per-entity .gest/ layout as live source of truth | Active               | 2026-04-08 |
| [0017] | Exit Code Contract for the gest CLI              | Active               | 2026-04-09 |

ADRs document significant architectural decisions, the context in which they were made, and their consequences. See
[Writing ADRs] for the process and template.

[0001]: https://github.com/aaronmallen/gest/blob/main/docs/design/0001-flat-files-over-sqlite.md
[0002]: https://github.com/aaronmallen/gest/blob/main/docs/design/0002-zero-config-discovery.md
[0003]: https://github.com/aaronmallen/gest/blob/main/docs/design/0003-reverse-hex-change-ids.md
[0004]: https://github.com/aaronmallen/gest/blob/main/docs/design/0004-tasks-as-execution-plans.md
[0005]: https://github.com/aaronmallen/gest/blob/main/docs/design/0005-cli-command-structure.md
[0006]: https://github.com/aaronmallen/gest/blob/main/docs/design/0006-ui-components-as-display-types.md
[0007]: https://github.com/aaronmallen/gest/blob/main/docs/design/0007-centralized-theme-system.md
[0008]: https://github.com/aaronmallen/gest/blob/main/docs/design/0008-custom-logger-with-styled-stderr.md
[0009]: https://github.com/aaronmallen/gest/blob/main/docs/design/0009-zero-config-discovery-toml-only.md
[0010]: https://github.com/aaronmallen/gest/blob/main/docs/design/0010-atomic-ui-architecture.md
[0011]: https://github.com/aaronmallen/gest/blob/main/docs/design/0011-askama-for-web-ui-templating.md
[0012]: https://github.com/aaronmallen/gest/blob/main/docs/design/0012-sqlite-for-the-event-store.md
[0013]: https://github.com/aaronmallen/gest/blob/main/docs/design/0013-global-only-storage-with-project-identity.md
[0014]: https://github.com/aaronmallen/gest/blob/main/docs/design/0014-libsql-for-remote-database-support.md
[0015]: https://github.com/aaronmallen/gest/blob/main/docs/design/0015-atoms-molecules-views-ui.md
[0016]: https://github.com/aaronmallen/gest/blob/main/docs/design/0016-per-entity-gest-layout-as-live-source-of-truth.md
[0017]: https://github.com/aaronmallen/gest/blob/main/docs/design/0017-exit-code-contract.md
[Writing ADRs]: https://github.com/aaronmallen/gest/blob/main/docs/process/writing-adrs.md
