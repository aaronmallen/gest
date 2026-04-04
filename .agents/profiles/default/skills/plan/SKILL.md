---
name: plan
description: "Take a spec and create an implementation plan: tasks, ADRs for large features, and dependency ordering (e.g. /plan <gest-id>)."
args: "<gest-id>"
---

# Plan

Assess a spec and produce an implementation plan.

## Instructions

### 1. Read and Understand the Spec

Read the spec via `GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- artifact show <id>` (add `--json`
for structured parsing). Identify:

- Acceptance criteria (these become the basis for tasks)
- Components or subsystems affected
- Whether an architectural decision is involved (warrants an ADR)
- Natural parallelization boundaries

### 2. Scope Assessment

Determine the right approach:

- **Single issue** -- small, focused work that can be completed in one session. Few acceptance criteria, one component
  affected.
- **Multi-issue** -- large work spanning multiple components or requiring parallel effort. Many acceptance criteria,
  clear decomposition boundaries.

Present the assessment to the user and ask which path to take.

### 3. Draft ADR (if needed)

If the work involves an architectural decision (new patterns, significant trade-offs, dependency choices), invoke
`/write-adr` before creating tasks.

### 4. Create Tasks

For **single issue**: invoke `/write-issue` with the spec and acceptance criteria.

For **multi-issue**:

1. Identify the natural breakdown (by component, by layer, by acceptance criteria groups)
2. Determine dependencies between tasks (what must be done first)
3. For issues that affect CLI behavior (commands, flags, output format), include integration test acceptance criteria in
   the task description. Example: "Integration test verifies `gest task create --description` outputs the created task
   ID."
4. **Batch-create all tasks** using NDJSON via `--batch`. Include phase, priority, tags, and the `child-of` link to the
   spec inline. Each line is a JSON object:

   ```sh
   cat <<'EOF' | GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task create --batch -q
   {"title":"First task","description":"...","phase":1,"priority":0,"tags":["storage"],"links":["child-of:<spec-id>"]}
   {"title":"Second task","description":"...","phase":1,"priority":1,"tags":["config"],"links":["child-of:<spec-id>"]}
   {"title":"Third task","description":"...","phase":2,"priority":0,"tags":["storage"],"links":["child-of:<spec-id>"]}
   EOF
   ```

   Links use the format `"rel:target_id"` (e.g. `"child-of:abcd1234"`, `"blocked-by:efgh5678"`). Artifact vs task
   targets are auto-detected.

   The NDJSON schema for tasks:

   ```json
   {
     "title": "string (required)",
     "description": "string",
     "assigned_to": "string",
     "phase": 1,
     "priority": 0,
     "status": "open",
     "tags": ["tag1", "tag2"],
     "links": ["rel:target_id"],
     "iteration": "iteration-id",
     "metadata": {"key": "value"}
   }
   ```

   With `-q`, each created task's bare ID is printed on its own line, in input order. Capture these for linking.

   A phase is a parallelization boundary: **every task in the same phase runs concurrently in its own workspace**, so
   tasks within a phase must be fully independent. If task A blocks task B, they **must** be in different phases (A in
   an
   earlier phase, B in a later one). Put tasks that share no dependencies in the same phase to maximize parallelism.

5. **Set blocking dependencies** for tasks that span phases:

   ```sh
   GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- task link <task-id> blocked-by <other-task-id> -q
   ```

6. **Create an iteration**, link it to the spec, and add all tasks:

   ```sh
   # Create iteration and capture ID
   GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration create "<plan title>" -q

   # Link iteration to spec
   GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration link <iteration-id> child-of <spec-id> --artifact -q

   # Add each task
   GEST_PROJECT_DIR=$XDG_DATA_HOME/gest/2f8de7bc06014bd7 cargo run -- iteration add <iteration-id> <task-id> -q
   ```

### 5. Output

Present the task list with IDs to the user:

- Link to the source spec
- Link to ADR (if created)
- Iteration ID
- Task list with titles, IDs, phases, and dependency ordering
- Parallelization notes (which tasks share a phase and can be worked simultaneously)

### 6. Next Step

For multi-issue plans, print: `invoke /orchestrate <iteration-id> when you're ready`

For single issue plans, print: `invoke /implement <task-id> when you're ready`
