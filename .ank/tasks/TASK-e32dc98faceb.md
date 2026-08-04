---
id: TASK-e32dc98faceb
type: task
slug: the-documentation-splits-by-audience
title: The documentation splits by audience
created: 2026-08-01T18:30:10Z
author: seanl@sean-laptop
status: in_progress
scope:
  - docs/**
  - README.md
blocked_by: [TASK-ff1c20395929]
done_criteria: |
  docs/getting-started.md takes a newcomer from install to a first claim and a first done without reading the specification. docs/format.md documents the file format and canonical form for third-party tools and points at the specification as the source of truth instead of restating it. README.md routes the documents by audience. The specification stops serving as tutorial.
criteria_by: creator
schema: 2
version: 5
---

The specification stays the single source of truth per ADR-63b59c5c26f7; the two new documents point at it and never restate normative content. Blocked by the spec revision so the newcomer document does not teach a surface about to change.

## Log
- 2026-08-04T04:15:29Z seanl@sean-laptop — Walked the newcomer path in a scratch repo rather than writing it from the spec, and two first-run refusals only that walk would have found. In a fresh local-only repository accept exits 9: default_branch is absent from the config init writes, and refs/remotes/origin/HEAD does not exist without a remote, so the very first accept fails until the key is added by hand. And new --verify <name> refuses at creation time (code 7) when the verifier is not declared in config.yml, so declaring the verifier precedes creating the task that names it. getting-started.md orders the steps around both.
- 2026-08-04T04:17:39Z seanl@sean-laptop — The walk also turned up a defect outside this scope, filed as TASK-c1783c841710: init writes no ignore rule, so .ank/index.db lands in the first commit of every repository it creates, while section 6 calls the index gitignored. This repository hides it -- its own .gitignore carries the line by hand. getting-started.md documents the behaviour as it is and tells the reader to add the line.
- 2026-08-04T04:24:07Z seanl@sean-laptop — Three readings the criterion left open, decided and written into the documents rather than left implicit. First, 'stops serving as tutorial' is a change of role, not a deletion: the argued examples in sections 3, 4 and 5 are what a normative document owes a reader who has to revise it, and cutting them would weaken the source of truth to satisfy a clause about routing. The spec now carries a 'Who this document is for' section that says it is not a tutorial and names the two documents that are, and the README no longer sends a newcomer to it. Second, format.md documents the field order, the emission rules and the quoting predicate -- none of which the specification enumerates, since section 3 states canonical form as a property -- and defers every rule it depends on with a section pointer plus a stated precedence: where the two disagree, the specification is right and format.md is a bug. Third, getting-started.md quotes real output only. Every block in it was produced by a scratch repository, which is how the two first-run refusals were found.
