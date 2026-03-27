---
name: promote-spec
description: "Promote a gest spec to a GitHub Discussion (e.g. /promote-spec <id>)."
args: "<gest-id>"
---

# Promote Spec

Promote a gest spec artifact to a GitHub Discussion.

## Instructions

### 1. Read the Spec

```sh
cargo run -- artifact show <id> --json
```

Extract:

- `title` — becomes the discussion title
- `body` — becomes the discussion body (see sanitization rules below)

### 2. Sanitize

**Sanitize the body before promoting.** The discussion body must not contain internal gest
references. Remove or rewrite any gest short IDs (e.g. `ktxolxqz`), `gest task/artifact <id>`
references, or sections that only list gest entities as dependencies. Replace gest IDs with the
entity's title where context is needed. Do not repeat the title as an `# H1` heading in the body —
GitHub already displays the title prominently.

### 3. Confirm and Create

All specs are promoted to the **Ideas** discussion category.

Draft the discussion creation and present it to the user for confirmation. Use the `gh api` to
create the discussion:

```sh
# First, get the repository ID and Ideas category ID
gh api graphql -f query='
  query {
    repository(owner: "<owner>", name: "<repo>") {
      id
      discussionCategories(first: 25) {
        nodes { id name }
      }
    }
  }
'
```

Then create the discussion:

```sh
gh api graphql -f query='
  mutation {
    createDiscussion(input: {
      repositoryId: "<repo-id>",
      categoryId: "<ideas-category-id>",
      title: "<title>",
      body: "<body>"
    }) {
      discussion { url }
    }
  }
'
```

After the user confirms, execute the commands. Extract the discussion URL from the response, then
store it as artifact metadata:

```sh
cargo run -- artifact meta set <id> github-discussion <url>
```

### 4. Report

Print a summary: the GitHub Discussion URL.

Print: `the spec has been promoted and linked via metadata`
