---
id: "0017"
title: "Exit Code Contract for the gest CLI"
status: active
tags: [cli]
created: 2026-04-09
---

# ADR-0017: Exit Code Contract for the gest CLI

## Status

![Active](https://img.shields.io/badge/Active-green?style=for-the-badge)

## Summary

Adopt the BSD `sysexits.h` convention for gest CLI exit codes. Each `cli::Error` category maps to a well-known Unix
exit code (64–78) via an `exit_code` method on `cli::Error`, which `main` uses to construct the process `ExitCode`.
This replaces the current contract where every failure exits with `1` (and `iteration next` ad-hoc exits with `2`),
giving scripts and AI agents a stable, standards-based way to branch on error category.

## Context

Every gest CLI error currently exits with code `1`, with one ad-hoc exception: `iteration next` uses `exit(2)` for "no
tasks available". Scripts and AI agents wrapping gest cannot programmatically distinguish error categories without
parsing stderr text. This makes automation brittle and forces consumers to string-match on error messages.

The primary beneficiaries are AI agents (already documented consumers of `iteration next`) and shell scripts that wrap
gest in pipelines. The agent profile documentation already treats exit code `2` as a meaningful sentinel for
`iteration next`, confirming that structured exit codes have real value for this project.

The existing `cli::Error` enum has 19 variants, but a few are semantically dirty: `Editor` is used for both true
editor-invocation failures and domain state-transition errors (e.g., `iteration advance`'s "not active", "no tasks",
"phase incomplete"); `Io` is used for both genuine filesystem errors and domain not-found conditions (e.g., `detach`'s
"no workspace found"). These miscategorizations must be cleaned up before any mapping can assign correct codes.

## Decision

Each `cli::Error` category maps to one `sysexits.h` code:

| Code | Name             | Semantic                          | Maps from                                                                   |
|------|------------------|-----------------------------------|-----------------------------------------------------------------------------|
| 0    | —                | Success                           | —                                                                           |
| 64   | `EX_USAGE`       | Command-line usage error          | `Argument`, clap usage errors                                               |
| 65   | `EX_DATAERR`     | User data format error            | `Serialize`, `TomlSerialize`, malformed data                                |
| 66   | `EX_NOINPUT`     | Input entity not found            | `Resolve::NotFound`, repo `NotFound`, `MetaKeyNotFound`, new `NotFound`     |
| 69   | `EX_UNAVAILABLE` | Resource not in required state    | new `InvalidState` variant, UNIQUE constraint violations                    |
| 70   | `EX_SOFTWARE`    | Internal software error           | `Editor` (true editor failures only), catch-all internal                    |
| 74   | `EX_IOERR`       | Filesystem or database I/O error  | `Store`, `Io`, repo `Database`                                              |
| 75   | `EX_TEMPFAIL`    | Try again later (valid empty)     | new `NoTasksAvailable` variant — used by `iteration next`                   |
| 78   | `EX_CONFIG`      | Configuration or setup error      | `Config`, `UninitializedProject`                                            |

The mapping lives as an `exit_code` method on `cli::Error`:

```rust
impl cli::Error {
  pub fn exit_code(&self) -> ExitCode {
    match self {
      Error::Argument(_) => ExitCode::from(64),
      Error::NotFound(_) | Error::Resolve(_) | Error::MetaKeyNotFound(_) => ExitCode::from(66),
      Error::InvalidState(_) => ExitCode::from(69),
      Error::NoTasksAvailable => ExitCode::from(75),
      // ... etc
    }
  }
}
```

`main` returns `std::process::ExitCode` and computes it from the `cli::Error` via `exit_code()`. The existing `die`
function either disappears or becomes a helper that prints the error message and returns an `ExitCode`. No code path
calls `std::process::exit` directly.

To support this mapping, three new variants are added to `cli::Error`:

- `NotFound(String)` — used where `Io(io::Error)` with `ErrorKind::NotFound` was previously used for domain not-found
  conditions
- `InvalidState(String)` — used where `Editor` was previously used for domain state-transition errors
- `NoTasksAvailable` — used by `iteration next` when no unblocked open tasks exist

The existing `Editor` variant is narrowed to true editor-invocation failures only.

### Rationale

Why sysexits over a bespoke scheme? Unix scripters already recognize these codes; sendmail, postfix, and many BSD tools
use them. The 64–78 range doesn't collide with shell conventions (`0` success, `1` generic error, `2` shell builtin
misuse, `126`/`127` exec failures, `128+N` signals). Every gest error category maps cleanly to an existing sysexits
code, so subcode granularity is unnecessary.

Why a method on `cli::Error` rather than a trait or lookup table? Single source of truth co-located with the type
definition. The match is exhaustive, so the compiler enforces that new variants declare their exit code. No separate
registry to keep in sync. Trivial to unit-test.

Why `main` returns `ExitCode` rather than calling `process::exit`? Idiomatic Rust 2021+ pattern; enables normal `Drop`
semantics on the way out. Keeps the exit code path in one place (`main` + `exit_code`) rather than scattered
`process::exit` calls throughout the command tree.

Why introduce new `cli::Error` variants now? The existing taxonomy is dirty — mapping dirty variants to exit codes
would ship known-wrong codes. Cleaning the taxonomy is a prerequisite for a correct mapping, not a parallel concern.

## Consequences

### Positive

- Callers get a stable, standards-based contract they can branch on via `$?` alone.
- Adding a new `cli::Error` variant forces a code assignment via exhaustive match (compile-time safety).
- Cleaner error taxonomy: `Editor` is narrowed to its true meaning; `Io` no longer doubles as domain not-found.
- `iteration next` no longer bypasses the error type system with direct `process::exit` calls.
- `main` follows the idiomatic Rust `ExitCode` pattern.

### Negative

- Exit code `1` → various specific codes is a breaking change under breakver. Must bundle with the next breaking
  release window.
- The documented `iteration next` exit code `2` becomes `75`; agent profile documentation must be updated in lockstep.
- Scripts that hard-code `$? -eq 1` break. (Rare in practice; `$? -ne 0` idioms continue to work.)
- Full audit of `cli::Error` construction sites is required to confirm every call site uses a semantically correct
  variant.

## Future Work

- Subcode granularity (e.g., distinguishing task-not-found from artifact-not-found numerically) is deferred. All
  not-found conditions share code `66`. If a future consumer needs finer granularity, it can be added without breaking
  this contract (e.g., reserving a `22x` range within `EX_NOINPUT`-adjacent space).
- Exit code conventions for the web server, sync daemon, or other non-CLI entry points are out of scope. This ADR
  governs the gest CLI binary only.
- Surfacing exit codes in `--help` output or the README is deferred; this ADR is the authoritative documentation.

## References

- Spec `xtnsuxns` — Exit Code Taxonomy (sysexits.h)
- [BSD sysexits(3)](https://man.openbsd.org/sysexits.3)
- [ADR-0005] — CLI Command Structure and Output Conventions
- `src/main.rs` — current `die` function
- `src/cli.rs` — `cli::Error` enum
- `src/cli/commands/iteration/next.rs` — existing ad-hoc `process::exit` calls

[ADR-0005]: https://github.com/aaronmallen/gest/blob/main/docs/design/0005-cli-command-structure.md
