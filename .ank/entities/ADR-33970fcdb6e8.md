---
id: ADR-33970fcdb6e8
type: adr
slug: the-documentation-is-one-tree-in-docs-and-the-pi
title: The documentation is one tree in docs/, and the pipeline publishes it as a site
created: 2026-09-19T18:25:09Z
author: claude-code/opus-5+docs-audit
status: accepted
scope:
  - docs/**
  - README.md
  - .github/workflows/**
constraint: |
  Documentation a person reads lives in docs/ of this repository, changes by pull request, and is published as a static site that the pipeline builds from the default branch. No wiki repository and no hand-kept copy elsewhere: a page edited outside this tree is a satellite. The files GitHub reads at fixed paths (README.md, CONTRIBUTING.md, SECURITY.md, CODE_OF_CONDUCT.md, the pull request and issue templates) stay where they are and link into the site. skill/, CLAUDE.md and AGENTS.md stay the agent surface: the site links them, never copies them. A normative rule lives in a spec entity; a page links to it or renders it, and never restates it.
ratified: 3cb614299726
verified:
  - by: haksolot@omarchy
    at: 2026-09-20T17:19:35Z
schema: 4
version: 3
---

## Context

A documentation audit on 2026-09-19 replayed every page against ank 0.8.0 and
found the tree drifted in the ~130 code commits since TASK-529f81e51669 closed
"docs/ carries nothing stale". The owner wants to move towards a wiki.

## Options weighed

- **GitHub Wiki.** Edited in the browser, but it is a separate git repository
  with no pull request, no review and no CI. It is exactly "the second
  repository somebody maintains" ADR-8b3045cf11db forbids for the skill, and
  nothing would turn red when a page drifts.
- **docs/ restructured, no site.** Cheap, but pages read on GitHub have no
  navigation, no search, and no place to render generated references.
- **A site built from docs/ (chosen).** The source stays in the tree, reviewed
  and testable (ADR on measured output, proposed alongside); the pipeline
  derives the published copy, the same way npm is an address and not a
  satellite.

## Left to the tasks

The generator is chosen by the task that sets the site up. mdBook is the
recommendation: a single Rust binary, Markdown in, no Node toolchain, and it
matches the stack. Whether the specification entities are rendered into the
site (through `ank show --json` at build time, never by reading .ank/) is an
open question for the restructuring task.

English only, per ADR-d3a8dcf38817.
