---
id: TASK-d90e94afca08
type: task
slug: the-ratification-queue-is-worked-from-the-reader
title: The ratification queue is worked from the reader, and the signature stays the person's
created: 2026-08-24T22:01:53Z
author: claude-code/opus-5+planning
status: open
scope:
  - crates/ank-tui/**
blocked_by: [TASK-b50b340c0bb1]
done_criteria: |
  The reader shows the ratification queue and drives ank accept on one selected proposed document at a time. It supplies no key, answers no passphrase prompt, and accepts nothing beyond the single document under an explicit keystroke. Where the checkout is not on the default branch, or the identity in effect may not ratify, it shows the CLI's refusal and the command that resolves it. A test drives the built binary against a temporary corpus and shows that a document accepted through the interface is verifiable exactly as a shell accept's is, and that with the keystroke withheld nothing in the queue changes state. cargo test is green and cargo fmt --check passes.
criteria_by: creator
schema: 4
version: 1
---

Ratification is the act this project guards hardest, and it is also the one whose
current friction is worst: the queue is a list, and accepting from it means
copying an id into a command in another window.

**Driving is not performing, and the criterion is written to keep them apart.**
ADR-8bd76e8d7c4e allows the reader to drive `accept` and forbids it performing
one unattended, so the test that matters is the negative: with no keystroke,
nothing moves. A screen that batched the queue, or that remembered a passphrase
to spare a person typing it, would have turned a human act into an automated one
while looking like a convenience.

**One at a time, deliberately.** A proposed document binds nobody until somebody
reads it, and a queue that accepts in bulk is a queue nobody reads. The reader's
job here is to put the body in front of the person, then get out of the way of a
signature it must never hold.
