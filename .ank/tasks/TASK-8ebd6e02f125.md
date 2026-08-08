---
id: TASK-8ebd6e02f125
type: task
slug: color-on-a-terminal-bytes-unchanged-in-a-pipe
title: Color on a terminal, bytes unchanged in a pipe
created: 2026-08-05T04:06:09Z
author: seanl@sean-laptop
status: in_progress
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
  - docs/**
  - crates/ank-cli/src/style.rs
  - crates/ank-cli/src/main.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/graph.rs
  - crates/ank-cli/src/edit.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  Output is colored — hand-written ANSI, no new dependency — only when stdout is
  a terminal and NO_COLOR is unset. Captured in a pipe, every command's output
  is byte-for-byte identical to today's, and the existing golden corpus does not
  move. --json is never colored. Behaviour is tested through the binary,
  including the piped case.
criteria_by: creator
schema: 2
version: 9
---

Execution of ADR-962c25797569, presentation half. The guarantee that matters
is negative: agents read pipes, and a pipe must never see an escape sequence.
TTY detection goes through what the tree already has (libc is present as a
transitive dependency; if a direct dependency became necessary the ADR's
no-new-dependency line wins and the feature shrinks). Restraint over
decoration: status markers, section headers, the error line — not a theme
engine.

## Log
- 2026-08-05T04:06:58Z seanl@sean-laptop — amended: -blocked_by ADR-962c25797569
- 2026-08-08T18:11:16Z seanl@sean-laptop — amended: +scope crates/ank-cli/src/style.rs, +scope crates/ank-cli/src/main.rs, +scope crates/ank-cli/src/commands.rs, +scope crates/ank-cli/src/human.rs, +scope crates/ank-cli/src/done.rs, +scope crates/ank-cli/src/claim.rs, +scope crates/ank-cli/src/graph.rs, +scope crates/ank-cli/src/edit.rs, +scope crates/ank-cli/src/init.rs, +scope crates/ank-cli/tests/cli.rs
- 2026-08-08T18:14:04Z seanl@sean-laptop — released: Reclaiming under the agent's own ANK_AGENT: taken while seanl@sean-laptop already held TASK-1613794deccf, and two live claims on one identity make HEAD ambiguous between verbs.
- 2026-08-08T18:15:41Z claude-code@ank — Correction to the release note above: HEAD was never ambiguous between verbs. A second session finished TASK-1613794deccf at 18:12:14Z, between this session's ank status and its ank done, so done saw one remaining claim and correctly asked for a proof. The release was taken on a misread and cost nothing but this entry. The real lesson stands and is the one the claim warning names: an unset ANK_AGENT makes two sessions on one machine a single agent, and one-claim-per-agent then arbitrates between them.
- 2026-08-08T18:31:47Z claude-code@ank — Falsified the new tests rather than trusting them green: forcing style::COLOR unconditionally in dispatch turns 7 of 69 integration tests red, three of them the new ones. A negative guarantee asserted by a test that cannot fail is not a guarantee. Also measured: std::io::IsTerminal (stable 1.70, floor here 1.95) answers the terminal question with no dependency at all, so the libc route the task body anticipated was never needed.
- 2026-08-08T18:41:29Z claude-code@ank — Implementation complete and locally green: 318 tests, fmt clean, ank check 0 faults. Design that mattered: the Style rides on Invocation, so parse() leaves it PLAIN (every unit test stays uncoloured with no edit) and dispatch() is the single place --json forces it back off -- which covers the three sites that print a non-JSON line while --json is set (done's 'running:', log's and amend's takeover warnings) without touching any of them. Budget accounting now counts visible characters, not bytes: otherwise a terminal would truncate the log one entry earlier than a pipe, which is the same command answering differently by who is watching. Follow-up filed as TASK-21031b516bb2 for the SKILL.md sentence, which falls outside this criterion.
