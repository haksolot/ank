---
id: TASK-b50b340c0bb1
type: task
slug: the-reader-acts-and-every-refusal-it-shows-is-th
title: The reader acts, and every refusal it shows is the CLI's own
created: 2026-08-24T22:01:41Z
author: claude-code/opus-5+planning
status: in_progress
scope:
  - crates/ank-tui/**
  - crates/ank-cli/tests/tui.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
blocked_by: [TASK-49746735127f]
done_criteria: |
  From a selected entity the reader may claim, log, release with a reason, finish with done and a proof, and amend, each performed by running the corresponding CLI verb and never by writing a file or a ref. Where the CLI refuses, the interface shows that refusal with its exit code and the command the CLI named as the way out, unaltered. No action is taken without an explicit keystroke, and nothing is re-run on a timer. A test drives the built binary against a temporary corpus and shows three things: a claim taken through the interface produces the same ref a shell claim produces, a done refused for a missing proof leaves the task untouched and displays the code the table declares, and a session left idle renews no claim. cargo test is green and cargo fmt --check passes.
criteria_by: creator
schema: 4
version: 3
---

The writing half. It adds no capability to ank; it removes the step of typing an
id, which is the step people get wrong.

**Passing the refusal through unaltered is the whole design.** Every refusal here
is a refusal on state, and each one names the exact command that resolves it. A
reader that rewrote those into its own wording would be inventing a second
vocabulary for the same conditions, and the first divergence would be a screen
telling a person something the CLI would not have said.

**"Nothing on a timer" is a constraint with a name.** ADR-0bb7ea8991bc says a
claim is renewed by working, not by reporting, and a long-lived screen is the
most natural place in this project to break that: a refresh loop that happens to
call a renewing verb would keep a claim alive for somebody who went home. The
criterion asks for the idle case to be measured, not reasoned about.

Intersecting claims need nothing new here. `claim` names a live claim whose scope
overlaps and takes the task anyway, which ADR-052accd6e3b2 makes a fact to read
rather than an error, and the interface shows what the verb said.
