---
id: "0011"
title: "Askama for Web UI Templating"
status: active
tags: [web, ui, templates]
created: 2026-03-31
---

# ADR-0011: Askama for Web UI Templating

## Status

![Active](https://img.shields.io/badge/Active-green?style=for-the-badge)

## Summary

The `gest serve` web UI uses [askama](https://github.com/askama-rs/askama) for server-rendered HTML templates. Askama
compiles Jinja2-style templates at build time into type-safe Rust code, keeping HTML in separate files while catching
errors at compile time.

## Context

The web server spec calls for server-rendered HTML with vanilla JS sprinkles across 6+ views. Two Rust templating crates
are viable:

- **askama** — Jinja2-style templates in separate `.html` files, compiled to Rust at build time. Type-checked,
  auto-escaping, template inheritance (base layouts), block overrides.
- **maud** — Rust macro-based DSL that generates HTML inline. No separate template files, pure Rust, good IDE support
  for the Rust side.

The web UI must faithfully reproduce the design system from `tmp/gest-prototype.html` — a dark, terminal-aesthetic
palette with specific CSS classes, monospace typography, and structured layouts. The kanban board, iteration graph, and
artifact detail views all have distinct HTML structures.

## Decision

Use **askama** with the **askama-derive-axum** crate for axum integration.

### Why askama over maud

**HTML stays as HTML.** The prototype is already written in HTML/CSS. Translating it into maud's Rust macro syntax adds
friction and makes it harder to iterate on markup. With askama, the prototype's HTML structure transfers almost directly
into templates.

**Template inheritance.** Askama's `{% extends "base.html" %}` / `{% block content %}` pattern gives us a single base
layout (nav, head, CSS) with per-view content blocks. Maud requires manual function composition to achieve the same
thing.

**Separation of concerns.** HTML templates in `templates/` are easier to review, diff, and iterate on independently from
Rust handler logic. The Rust structs define the data contract; the templates define presentation.

**Compile-time safety.** Unlike runtime template engines, askama catches missing variables and type mismatches at build
time — the same guarantee maud provides, without embedding HTML in Rust.

### Template file layout

```text
templates/
  base.html           # shared layout: <html>, <head>, nav, CSS/JS includes
  dashboard.html      # extends base
  tasks/
    list.html
    detail.html
  artifacts/
    list.html
    detail.html
  iterations/
    list.html
    detail.html
    board.html        # kanban view
  search.html
```

## Dependencies

| Dependency         | Version | Purpose                                                              |
|--------------------|---------|----------------------------------------------------------------------|
| askama             | 0.15    | Jinja2-style compiled templates                                      |
| askama-derive-axum | 0.1     | Axum responder integration (replacement for deprecated askama\_axum) |

## Consequences

### Positive

- HTML prototype translates directly into templates with minimal rewrite
- Template inheritance eliminates layout duplication across 8+ views
- Compile-time checking catches template errors before runtime
- Clean separation: Rust structs = data, `.html` files = presentation
- Familiar Jinja2 syntax (widely known, well-documented)

### Negative

- Adds a build-time code generation step (askama's derive macro)
- Template files are not Rust — IDE support for the template language is limited to askama-specific plugins
- Slightly more ceremony than maud for very small templates (need a struct + file vs. inline macro)

## References

- [ADR-0010: Atomic UI Architecture](0010-atomic-ui-architecture.md)
- Design prototype: `tmp/gest-prototype.html`
