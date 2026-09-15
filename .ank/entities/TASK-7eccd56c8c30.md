---
id: TASK-7eccd56c8c30
type: task
slug: the-guide-states-what-level-1-needs-a-remote-nam
title: "The guide states what level 1 needs: a remote named origin, and nothing else"
created: 2026-09-15T09:21:01Z
author: claude-code/fable-5.1+distributed-review
status: in_progress
scope:
  - docs/getting-started.md
blocked_by: []
done_criteria: |
  In the section of docs/getting-started.md that covers the default branch and remotes, beside the refs/remotes/origin/HEAD refusal, the guide states that coordination between clones needs only a remote named origin, that a bare repository reachable over file:// or ssh on the same network is one, and that two clones with no common origin are not arbitrated: both claims of one task succeed and nothing reports it. The tests that read the guide, crates/ank-cli/tests/adopt.rs and the guide-reading tests in crates/ank-cli/tests/cli.rs, stay green.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 2
---

The architecture review of 2026-09-15 took "machines without a common remote"
as a scenario needing a peer protocol. SPEC-15a56aeedcfd already states what
the scenario needs and what it lacks: level 1 is "the repository has a remote
named origin", GitHub or not, and two clones without one are not arbitrated.
The guide says neither, so an operator on a LAN reads "no GitHub" as "no
coordination". One paragraph closes the scenario with no code.
