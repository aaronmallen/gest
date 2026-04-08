# Code Style

This document describes the formatting and organization conventions used in Gest. Most formatting is handled
automatically by tools -- you generally just need to run `mise run format` before committing.

The code organization rules are the main things to keep in mind when writing new code. Project-specific rules will be
added to this document as the project matures.

## Running Formatters and Linters

```bash
mise run format       # Format all files
mise run lint         # Lint all files
```

## General Principles

These principles apply across all file types in the project:

| Principle            | Convention                                                    |
|----------------------|---------------------------------------------------------------|
| Maximum line width   | 120 characters                                                |
| Indentation          | 2 spaces (no tabs)                                            |
| Trailing whitespace  | None                                                          |
| Final newline        | All files end with a single newline                           |
| Import/include order | Alphabetical, grouped by origin (stdlib, external, local)     |
| Declaration ordering | Alphabetical within visibility groups (public before private) |

These conventions are enforced by `.editorconfig` and the project's linting tools.

## Import Style

Prefer importing named types (structs, enums, traits) directly rather than using fully-qualified paths, unless there is
a name conflict. Functions and free-standing helpers may use the fully-qualified path.

```rust
// Good: import the trait and type, qualify only where there's a conflict (fmt::Result vs std::Result)
use std::fmt::{self, Display, Formatter};

impl Display for Foo {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    // ...
  }
}

// Bad: unnecessarily qualified types
impl fmt::Display for Foo {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    // ...
  }
}

// Fine: free functions can stay qualified
std::fs::create_dir_all(path)?;
```

## Code Organization

### Module-Level Ordering

Order items within a module by:

1. **Constants**: All constant declarations first
2. **Type groups**: Each type definition immediately followed by its implementation blocks
3. **Free functions**: Any standalone helper functions after all type groups

Type groups are ordered **alphabetically** by type name, with **public types before private types** (each visibility
group sorted alphabetically).

### Derive Attributes

Traits listed in `#[derive(...)]` attributes should be ordered **alphabetically**.

```rust
// Good
#[derive(Clone, Debug, Eq, PartialEq)]
struct Foo;

// Bad
#[derive(Debug, Clone, PartialEq, Eq)]
struct Foo;
```

### Enumeration Variants

Enumeration variants should be ordered **alphabetically**.

### Struct/Record Fields

Struct or record fields should be ordered **alphabetically**, unless field order is semantically significant (e.g.,
positional arguments in a CLI framework).

### Implementation Block Ordering

Order functions and methods within implementation blocks by:

1. **Static vs Instance**: Static/associated functions first, then instance methods
2. **Visibility**: Public items first, then private items
3. **Alphabetical**: Within each group, sort alphabetically

In test modules, fall back to purely alphabetical ordering when the static/instance/public/private structure doesn't
apply. See [testing] for test-specific conventions.

## Documentation Comments

Doc comments explain *why* code exists and how it fits together -- not what a reader can already see from the name
and signature. Keep them concise and idiomatic.

### `//!` vs `///`

- Use `//!` for **module-level** context: what the module does, why it exists, and how its pieces fit together.
- Use `///` for **items**: structs, enums, traits, functions, methods, and fields.

### One-line summary style

The first line of any doc comment is a single concise sentence. Additional detail follows after a blank line only
when non-obvious behavior warrants it.

```rust
// Good
/// Returns the canonical path to the gest data directory.
pub fn data_dir() -> PathBuf { ... }

// Bad: restates the name without adding information
/// Gets the data directory.
pub fn data_dir() -> PathBuf { ... }
```

### Avoid name-restating redundancy

Don't write `/// The name of the user` above `pub name: String`. Docs should add information the name alone doesn't
convey -- constraints, units, invariants, lifetimes, or context.

### `cli/commands/` leaf rule

Single-purpose command leaf files (one `Command` struct, one `call()` impl) do **not** need a `//!` header. The
command struct's `///` comment carries the description. Subsystem roots (e.g. `cli/commands/artifact.rs`) and
shared-helper modules **do** get `//!` headers.

### `pub` item coverage

Every `pub` struct, enum, trait, function, method, and field should have a `///` comment. Private items get docs only
where they genuinely help a reader.

### No divider comments

Don't use Unicode box-drawing dividers or other section separators inside source files. If a file needs visual
structure beyond normal item grouping, consider splitting it.

[testing]: https://github.com/aaronmallen/gest/blob/main/docs/dev/testing.md
