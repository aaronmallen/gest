---
name: brainstorm
description: Explore a rough idea with the user, clarify requirements, and draft a spec. Also decomposes large specs into smaller ones (e.g. /brainstorm "offline mode", /brainstorm <gest-id>).
args: "<rough idea or gest artifact ID to decompose>"
---

# Brainstorm

Explore a problem space with the user and produce a spec.

## Instructions

### 1. Explore the Problem Space

Read the input provided by the user. Determine the mode:

- **Rough idea** -- the user has a new idea to explore
- **Existing spec** -- the user provides a gest artifact ID to decompose

For rough ideas, delegate to the **brainstormer** agent with the idea. The agent will research the
codebase and return a structured context report covering existing patterns, constraints, complexity,
and open questions. Use the research report to ground the conversation.

For existing specs, retrieve the spec via `cargo run -- artifact show <id>` and identify natural
decomposition boundaries.

### 2. Clarify Requirements

Ask clarifying questions **one at a time**. Do not front-load a list of questions. Wait for the
user's answer before asking the next. Use the brainstormer's open questions as a starting point for
rough ideas, or scope boundaries for decomposition.

### 3. Propose Approaches

Once the problem is well-understood, propose **2-3 approaches** with trade-offs and complexity
estimates. Ground each approach in the constraints and existing patterns surfaced by the brainstormer.
Ask the user which approach to pursue.

### 4. Draft the Spec

Invoke `/write-spec` with the agreed-upon approach. The skill will output a gest artifact ID for the
new spec.

For decomposition, invoke `/write-spec` for each sub-spec, linking them back to the parent.

Print next-step hint: `invoke /plan <id> when you're ready for the next step` (using the spec's gest
ID).
