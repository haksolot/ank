---
id: TASK-86babab0eb1b
type: task
slug: amend-takes-a-scope-with-a-leading-and-check-fin
title: amend takes a scope with a leading + and check finds it four commands later
created: 2026-08-25T03:26:36Z
author: claude-code/opus-5+daemon-refs
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  ank amend <id> --scope "+p" and ank amend <id> --drop-scope "-p" both exit non-zero and write nothing to the entity: the refusal names the leading marker as a marker, names --drop-scope as the way to remove a glob, and names a leading ./ as the way to write a path whose own name begins with that character, so ank amend <id> --scope "./+p" succeeds and stores +p. A test in crates/ank-cli/tests/cli.rs drives the binary for each of those cases.
criteria_by: claimer
proof:
  - type: commit
    ref: 2ebb949d6e3b50cf04d0d4659eb0e7215d2a0f00
    criteria: 71999546f5f8
    via: submitted
schema: 4
version: 6
---

`ank amend <id> --scope "+crates/ank-cli/src/context.rs"` stores the `+` as part
of the glob. Nothing refuses it, nothing warns, and `--list`-style feedback says
`+scope +crates/...`, which reads as the tool acknowledging an add-marker rather
than as a glob that begins with a plus sign. The mistake is easy to make
precisely because this CLI's own idiom is `--scope` to add and `--drop-scope` to
remove: a caller who has met `+`/`-` list syntax elsewhere writes the marker and
is told it worked.

The cost arrives four commands later. The scope matches no file, so `check`
reports a dead scope with no rename and no deletion to explain it, which is a
fault; and by then the task can be `done`, where `amend` refuses because the
plan is settled. That leaves `edit` as the only route to a correction, and
`edit` does not declare the refusal `amend` does -- two verbs disagreeing about
whether a done task's plan may move, which is a disagreement to settle rather
than a door to use.

Two candidate answers, and choosing between them is the work: refuse a scope
whose first character is `+` or `-` at `amend` time, naming `--drop-scope` as
the way to remove; or accept it, strip it, and say so. What must not stay is a
silent acceptance whose only report is a fault four commands downstream.
