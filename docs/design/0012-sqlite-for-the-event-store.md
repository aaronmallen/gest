---
id: "0012"
title: "SQLite for the Event Store"
status: active
tags: [storage, architecture, event-store]
created: 2026-04-01
---

# ADR-0012: SQLite for the Event Store

## Status

![Active](https://img.shields.io/badge/Active-green?style=for-the-badge)

## Summary

Use SQLite for the event store — a local, per-machine mutation log that powers
`gest undo`. This creates a scoped exception to ADR-0001's flat-file-only
storage policy. Entity data remains flat files; only operational state (mutation
history) uses SQLite.

## Context

ADR-0001 chose flat files for entity storage based on committability, concurrent
access, and greppability. None of these constraints apply to the event store:

- **Not committable** — the event store is local operational state, never
  checked into version control. It lives in `state_home`, outside the repo.
- **Concurrent writes matter** — multi-agent workflows may produce simultaneous
  mutations. An append-only flat file has no safe concurrent-write story;
  SQLite's WAL mode handles this natively.
- **Querying matters** — undo needs to efficiently find the most recent
  non-undone transaction and its associated events. Ordered retrieval and
  filtering are SQLite's strength.
- **Greppability is irrelevant** — users don't grep their undo history with
  Unix tools.

An append-only NDJSON file was considered but rejected: it lacks concurrent-write
safety, requires full-file scanning for queries, and has no built-in transaction
semantics.

## Decision

The event store uses SQLite (via `rusqlite`) with WAL mode, stored at
`<state_home>/gest/<project_hash>/events.db`. The project hash uses the same
`path_hash` logic as the global data directory.

This exception is narrowly scoped: **only non-entity operational state** (mutation
history, undo log) may use SQLite. Entity storage (tasks, artifacts, iterations)
remains flat files per ADR-0001.

## Dependencies

| Dependency | Version | Purpose              |
|------------|---------|----------------------|
| rusqlite   | latest  | SQLite connection    |

## Consequences

### Positive

- Safe concurrent writes from multi-agent workflows via WAL mode
- Efficient ordered queries for undo/redo operations
- No impact on entity storage — ADR-0001 remains fully in effect for tasks,
  artifacts, and iterations
- Event store is invisible to version control

### Negative

- Introduces a new dependency class (C library via `rusqlite`/`libsqlite3-sys`)
- Binary state that doesn't sync between machines — undo history is local only
- Increases binary size due to bundled SQLite

## References

- ADR-0001: Flat Files Over SQLite for Storage (remains active for entity data)
- Spec: Event Store and Undo Command (`quvwpvrx`)
