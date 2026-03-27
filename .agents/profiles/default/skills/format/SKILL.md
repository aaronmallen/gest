---
name: format
description: Format and lint the project, auditing changed files against code style guidelines.
---

# Format

Format and lint the project, auditing changed files against code style guidelines.

## Instructions

### 1. Run Formatter and Linter

run `mise format && mise lint`

### 2. Audit Changed Files

Delegate to the **vcs-expert** agent to get the current diff. Identify all changed files.

For each changed file, dispatch an agent to audit that file against
`docs/dev/code-style.md`. Launch all agents in parallel.

Each agent should read the full file and check for style violations, reporting any issues with file
paths and line numbers.

### 3. Fix Violations

Fix any violations the agents report.

### 4. Re-lint

run `mise lint` to verify that the project is still linted.

### 5. Run Tests

run `mise test` to confirm nothing is broken.
