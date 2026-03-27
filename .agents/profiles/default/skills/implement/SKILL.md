---
name: implement
description: "Implement a single issue: write code, verify, review, format, and commit (e.g. /implement <gest-id>)."
args: "<gest-id>"
---

# Implement

Implement a single issue (gest task) from start to commit.

## Instructions

### 1. Read the Issue

Retrieve the task via `cargo run -- task show <id>`. Understand:

- User story (the "why")
- Acceptance criteria (the "what" -- each must be satisfied)
- Dependencies and notes

Then mark the task as in-progress:

```sh
cargo run -- task update <id> --status in-progress
```

### 2. Implement

Write the code to satisfy all acceptance criteria. Follow these principles:

- **Tests are source of truth** -- never modify existing tests unless the issue explicitly calls for
  a behavioral change. If a test fails, your implementation is wrong.
- **Add tests** for new behavior and edge cases
- **Respect existing patterns** -- follow conventions already established in the codebase
- **Minimal changes** -- only change what the issue requires
- **Never modify existing integration tests** -- integration tests in `tests/integration/` are the
  strongest behavioral contract. Do not modify them unless the issue explicitly requires a behavioral
  change.

### 3. Verify

Run the full verification suite:

1. `mise run format` -- format all files
2. `mise run lint` -- check for style violations
3. `mise run check` -- compile/type-check
4. `mise run test` -- run all tests

All four must pass before proceeding.

### 3.5. Integration Tests

For CLI-facing changes (anything that modifies files under `src/cli/`), dispatch the
**integration-tester** agent to write or update integration tests covering the changed behavior.

### 4. Review

Invoke `/code-review` to review the changeset. Address any blocking findings.

### 5. Format

Invoke `/format` to ensure code style compliance.

### 6. Commit

Invoke `/commit` to create the commit. Reference the issue in the commit footer if applicable.

### 7. Close the Task

Mark the task as done:

```sh
cargo run -- task update <id> --status done
```

Then check if the task links to a spec artifact (look for artifact links in the task's `links`
field). If it does, query all tasks that link to the same artifact via `cargo run -- task list --json`
and check whether any remain `open` or `in-progress`. If none do, archive the artifact:

```sh
cargo run -- artifact archive <spec-id>
```

**On failure:** If any prior step fails and cannot be resolved, leave the task as `in-progress` --
do not mark it `done` or reset it to `open`. This signals incomplete work to the user or to
`/orchestrate`.
