# Theming

gest exposes a semantic theming system for its terminal UI. You can override
either:

- the **palette**, which updates shared semantic color slots used across many UI tokens
- individual **theme tokens**, which let you change one specific UI element

Theme settings live under the `[colors]` section in your config file.

## How theme resolution works

gest resolves styles in this order:

1. Built-in defaults
2. `[colors.palette]` overrides
3. `[colors.overrides]` token overrides

Palette overrides replace the foreground color for every token mapped to that
semantic slot while preserving the token's built-in modifiers such as `bold`,
`italic`, or `underline`.

Token overrides are the most specific layer. They can replace foreground and
background colors and add text modifiers for a single token.

## Supported color formats

You can use either:

- named ANSI colors such as `red`, `yellow`, or `bright cyan`
- 6-digit hex colors in `#RRGGBB` format

Supported named colors:

`black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`,
`bright black`, `bright red`, `bright green`, `bright yellow`, `bright blue`,
`bright magenta`, `bright cyan`, `bright white`

## Quick example

```toml
[colors.palette]
primary = "#7CC4F0"
warning = "#E3A22B"
text.muted = "#8B93A7"

[colors.overrides]
"log.error" = "#FF6B6B"
"tag" = { fg = "#4EA8E0", italic = true }
"config.heading" = { fg = "#4EA8E0", bold = true, underline = true }
```

## Palette Slots

The palette exposes 11 semantic slots:

| Key             | Default   | Used for          |
|-----------------|-----------|-------------------|
| `accent`        | `#D05830` | warm highlights   |
| `border`        | `#30323A` | rules and borders |
| `error`         | `#D03838` | error states      |
| `primary`       | `#4EA8E0` | brand emphasis    |
| `primary.dark`  | `#3278B0` | dark accents      |
| `primary.light` | `#7CC4F0` | light accents     |
| `success`       | `#36BE78` | success states    |
| `text`          | `#C4C8D4` | body text         |
| `text.dim`      | `#585E6E` | dim text          |
| `text.muted`    | `#7C8294` | secondary text    |
| `warning`       | `#CC9820` | warning states    |

Example:

```toml
[colors.palette]
primary = "#5AB0FF"
success = "#47C97B"
text = "#E3E7F1"
text.muted = "#98A1B3"
```

## Token Overrides

Token overrides belong under `[colors.overrides]`. Each key is a dot-delimited
token name.

### Simple form

The string form sets only the foreground color:

```toml
[colors.overrides]
"log.warn" = "yellow"
"message.success.icon" = "#36BE78"
```

### Table form

The table form can set colors and text modifiers:

```toml
[colors.overrides]
"search.query" = { fg = "#F4F7FF", bold = true }
"markdown.blockquote" = { fg = "#8B93A7", italic = true }
"banner.update.version" = { fg = "#FFCC4D", bold = true }
```

Available fields:

| Field       | Type    | Description        |
|-------------|---------|--------------------|
| `fg`        | string  | Foreground color   |
| `bg`        | string  | Background color   |
| `bold`      | boolean | Enable bold text   |
| `dim`       | boolean | Enable dim text    |
| `italic`    | boolean | Enable italic text |
| `underline` | boolean | Enable underline   |

## Token Reference

These are the token keys currently recognized by gest.

### Core

- `emphasis`
- `error`
- `muted`
- `success`
- `tag`
- `border`

### Artifact Views

- `artifact.detail.label`
- `artifact.detail.separator`
- `artifact.detail.value`
- `artifact.list.archived.badge`
- `artifact.list.tag.archived`
- `artifact.list.title`
- `artifact.list.title.archived`

### Banner

- `banner.author`
- `banner.author.name`
- `banner.gradient.end`
- `banner.gradient.start`
- `banner.shadow`
- `banner.update.command`
- `banner.update.hint`
- `banner.update.message`
- `banner.update.version`
- `banner.version`
- `banner.version.date`
- `banner.version.revision`

### Configuration Views

- `config.heading`
- `config.label`
- `config.no_overrides`
- `config.value`

### Confirm Prompt

- `confirm.description`
- `confirm.indicator`
- `confirm.selected`
- `confirm.unselected`

### IDs And Status Indicators

- `id.prefix`
- `id.rest`
- `indicator.blocked`
- `indicator.blocked_by.id`
- `indicator.blocked_by.label`
- `indicator.blocking`

### Init Output

- `init.command.prefix`
- `init.label`
- `init.section`
- `init.value`

### Iterations

- `iteration.detail.count.blocked`
- `iteration.detail.count.done`
- `iteration.detail.count.in_progress`
- `iteration.detail.count.open`
- `iteration.detail.label`
- `iteration.detail.value`
- `iteration.graph.branch`
- `iteration.graph.phase.icon`
- `iteration.graph.phase.label`
- `iteration.graph.phase.name`
- `iteration.graph.separator`
- `iteration.graph.title`
- `iteration.list.summary`
- `iteration.list.title`
- `iteration.status.label`
- `iteration.status.progress`
- `iteration.status.value`

### Lists

- `list.heading`
- `list.summary`

### Logs

- `log.debug`
- `log.error`
- `log.info`
- `log.timestamp`
- `log.trace`
- `log.warn`

### Markdown Rendering

- `markdown.alert.caution.border`
- `markdown.alert.important.border`
- `markdown.alert.note.border`
- `markdown.alert.tip.border`
- `markdown.alert.warning.border`
- `markdown.blockquote`
- `markdown.blockquote.border`
- `markdown.code.block`
- `markdown.code.border`
- `markdown.code.inline`
- `markdown.emphasis`
- `markdown.heading`
- `markdown.link`
- `markdown.rule`
- `markdown.strong`

### Messages

- `message.created.label`
- `message.success.icon`
- `message.updated.label`

### Meta

- `meta.not_set`
- `meta.value`

### Migrate

- `migrate.count`

### Notes

- `note.detail.label`
- `note.detail.separator`
- `note.detail.value`
- `note.list.body`
- `note.list.id`

### Projects

- `project.list.archived.badge`
- `project.list.root`
- `project.list.root.archived`
- `project.show.value`

### Search

- `search.expand.separator`
- `search.no_results.hint`
- `search.query`
- `search.summary`
- `search.type.label`

### Serve

- `serve.url`

### Tags

- `tag.list.count`
- `tag.list.heading`

### Task And Iteration Status

- `status.cancelled`
- `status.done`
- `status.in_progress`
- `status.open`

### Tasks

- `task.detail.label`
- `task.detail.separator`
- `task.detail.title`
- `task.detail.value`
- `task.list.icon.cancelled`
- `task.list.icon.done`
- `task.list.icon.in_progress`
- `task.list.icon.open`
- `task.list.priority`
- `task.list.title`
- `task.list.title.cancelled`

## Markdown Shorthand Aliases

All `markdown.*` tokens have `md.*` shorthand aliases that you can use in
`[colors.overrides]`:

| Shorthand                   | Resolves to                       |
|-----------------------------|-----------------------------------|
| `md.alert.caution.border`   | `markdown.alert.caution.border`   |
| `md.alert.important.border` | `markdown.alert.important.border` |
| `md.alert.note.border`      | `markdown.alert.note.border`      |
| `md.alert.tip.border`       | `markdown.alert.tip.border`       |
| `md.alert.warning.border`   | `markdown.alert.warning.border`   |
| `md.blockquote`             | `markdown.blockquote`             |
| `md.blockquote.border`      | `markdown.blockquote.border`      |
| `md.code.block`             | `markdown.code.block`             |
| `md.code.border`            | `markdown.code.border`            |
| `md.code`                   | `markdown.code.inline`            |
| `md.emphasis`               | `markdown.emphasis`               |
| `md.heading`                | `markdown.heading`                |
| `md.link`                   | `markdown.link`                   |
| `md.rule`                   | `markdown.rule`                   |
| `md.strong`                 | `markdown.strong`                 |

:::tip
Note that `md.code` maps to `markdown.code.inline`, not `markdown.code`.
:::

## Notes

- `banner.gradient.start`, `banner.gradient.end`, and `banner.shadow` use
  built-in RGB defaults and are not driven by palette slots.
- `markdown.emphasis` and `markdown.strong` are modifier-only defaults, so
  palette overrides do not affect them unless you override the token directly.
- Unknown palette keys and unknown token names are ignored with a warning.
