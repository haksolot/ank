---
id: ADR-52bb0da2023a
type: adr
slug: an-entity-is-born-accounted-and-one-born-outside
title: An entity is born accounted, and one born outside the CLI is a fault
created: 2026-09-13T09:42:10Z
author: claude-code/fable-5.1+planning
status: proposed
scope:
  - crates/ank-cli/**
constraint: |
  A verb that creates an entity of any kind but log writes a log entry recording the creation, the version it produced and the hash of the content it produced, on the terms the edit record already sets: the entry anchors nothing, no authority rests on it and no verb refuses on it. A verb that changes an entity's content outside a status transition keeps writing the edit record, with the fields it changed, the versions it moved between, the hash of the state it replaced and the hash of the content it produced. Content is every field a transition does not write: status, proof, ratified and verified belong to a transition and version belongs to the store, and the rest is content. check compares the newest produced hash an entity's entries carry against the entity as it stands and reports a signal naming both when they differ; an edit outside the CLI stays a signal, because a human with an editor keeps every power they had. An entity of any kind but log whose created instant is later than the ratification of this decision and whose entries carry no creation record is a fault, and the fault names ank edit <id> as the command that accounts for it, saying that the id and the verifiers are the reader's to check. An entity created before that instant, or carrying at least one entry with a produced hash, is silent, because the corpus is not migrated by a rule it predates. A log entry is the record and carries none. The version count is kept for the kinds whose transitions their own fields evidence, and is not attempted for a task. Nothing here refuses an edit: what the record buys is that an entity nobody's verb wrote is visible, and red in CI, on every harness alike.
supersedes: ADR-f7dc76886db2
schema: 4
version: 1
---

## Context

ADR-f7dc76886db2 made a hand edit legible: a machinery entry carries the hash
of the content its write produced, and check compares the newest one against
the entity as it stands. It said, deliberately, that an entity carrying no entry
is silent. Creation was the one write that left no entry, so the one write the
comparison could not reach.

Measured on 2026-09-13 in a throwaway corpus on this tree: a task created by
`ank new`, amended once, then retitled with sed, is reported (`content is
49adc3a82e47 where the last write left b31ea7b33bd0`). A task written whole
with a heredoc into .ank/entities/, canonical form imitated, is reported as
nothing at all: the same two lines a legitimate entity gets. The rule that
binds the agent to the CLI (ADR-e45e1a29fe91) has no mechanical witness for the
act it most needs one for, and this repository has already had entities placed
by hand (LOG-3cc48d).

The harm is concrete. A hand-written task carries an id nobody generated
without coordination, a form nobody canonicalised, no author, and an empty
`verify:`, which is the shape ADR-443590981e41 exists to remove: `done` then
takes a typed proof and closes green on nothing run, the failure CLAUDE.md
records under TASK-54c95c5f2d18.

## The decision

Every entity is born with a record, and an entity born after this rule without
one is a fault. A fault, not a signal, because the CI job already fails on
check's exit 8 and main takes nothing but a merged pull request: a file no verb
wrote cannot reach the default branch, whatever harness the agent runs under.
That is the property asked for, and the corpus is the only layer every harness
shares.

The anchor is the entity's own `created` against the instant of this
decision's ratification, which check already reads. It costs no process and it
catches the drift this exists for: an agent that writes the file because the
file is nearer than the verb. An agent that also backdates `created` is
forging, and forging a record whose hash the canonical serializer has to
reproduce is strictly harder than running `ank new`; that is the inversion
ADR-6b3f19e08a24 asks for, verifiable rather than defended.

`ank edit <id>` is the road out, and it is the right one: the verb re-reads the
file, refuses what is not the format, writes it back canonical and leaves an
entry with the hash it produced. The CLI has passed over the entity, which is
what the rule wanted. What a verb cannot repair, an id nobody drew and a
`verify:` nobody filled, the fault says to check by hand.

## Rejected

- A harness hook or a permission rule shipped in the repository. It holds for
  one agent and for none of the others, and ank is agnostic by design; the
  corpus is the only place a rule reaches every agent.
- Making a hand edit a fault as well. A human with an editor keeps every power
  they had (ADR-e45e1a29fe91), and a fault a legitimate act raises, whose only
  exit is to repeat the act through a verb, teaches the human to stop reading
  check.
- Anchoring on the file's first appearance in git history. Stronger against a
  backdated `created`, and paid for with a `git log` over .ank/entities on
  every check, with no answer for an entity not yet committed.
- A schema bump. A hand-written file declares whatever schema it likes.
- A creation record for log entries. An entry is the record; a record of the
  record recurses, and an entry is written once (ADR-25f977377fa0).

## Consequences

`new` writes one more entity per creation, a log entry with `records: create`.
`create` joins `edit` in the records vocabulary check knows, and the successor
of SPEC-e258796162c4 that TASK-38dabf191c1a already drafts carries the word.
check gains one fault. The scratch measurement above becomes the regression
test: the heredoc entity is reported, and the entity `ank new` wrote is not.
