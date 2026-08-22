---
id: ADR-f7dc76886db2
type: adr
slug: an-entity-accounts-for-its-content-by-hash-and-t
title: An entity accounts for its content by hash, and the count is kept where it closes
created: 2026-08-22T20:17:11Z
author: claude-code/opus-5
status: accepted
scope:
  - crates/ank-cli/**
constraint: |
  A verb that changes an entity's content outside a status transition writes a log entry recording the fields it changed, the version it moved from and the version it moved to, the hash of the state it replaced, and the hash of the content it produced. The entry anchors nothing: no authority rests on it and no verb refuses on it, which is what ADR-ff294eff4d1a requires of the log. The hashes exist so that a reader handed a claim about a past state can check it, and so that check can compare the present one. Content is every field a transition does not write: status, proof, ratified and verified belong to a transition and version belongs to the store, and the rest is content. check compares the newest produced hash an entity's entries carry against the entity as it stands, and reports a signal naming both when they differ. An entity carrying at least one such entry is accounted for that way. The version count is kept for the kinds whose transitions their own fields evidence, and is not attempted for a task, whose claim and release leave no durable record naming a version. An entry carrying no produced hash is silent, and an entity carrying no entry is silent, because the corpus is not migrated by a rule it predates. Nothing here refuses an edit: what the trace buys is that the ordinary case becomes legible, not that the dishonest case becomes impossible.
supersedes: ADR-16813b3bcf37
ratified: 32efe0ddc23b
verified:
  - by: claude-code/opus-5
    at: 2026-08-22T20:39:47Z
schema: 4
version: 3
---

ADR-16813b3bcf37 asked an entity to account for the versions it carries, and
TASK-dfe5a1bb0857 delivered that arithmetic for an ADR and a spec while leaving
a task silent. This is the half that was left, and the answer is not a better
count.

**Counting cannot close for a task, and the reason is structural.** Five verbs
write a task file: the three content verbs, which each leave a machinery entry,
and `claim`, `release`, `done`, `close` and `attest`, which are transitions.
`done`, `close` and `attest` leave a log entry; `claim` and `release` leave
nothing durable that names a version, and a claim's own record is the ref
`release` deletes. So a task claimed and released five times carries ten
versions no reader can evidence afterwards, and a lapse loses one more.

**Putting a version on the records does not repair it.** That was the shape
TASK-cbc6963fd0ef was filed with, and it was worth trying on paper before
building: a version in the claim record and in the completion ref accounts for
the claim that is live and for the `done` that settled it, and loses every
version of every cycle in between, because the record that carried it was
deleted when the cycle ended. The arithmetic still does not close, and a rule
that fires on a task claimed twice is the volume section 11 names as what
teaches a reader to stop reading `check`.

**So the accounting stops counting and compares.** A machinery entry already
carries the hash of the state its write replaced; it now also carries the hash
of the *content* that write produced, and `check` compares the newest one
against the entity as it stands. Equal, and the entity has not moved since the
CLI last wrote it. Unequal, and it has.

**Content is what a transition never touches**, which is what makes the
comparison survive a claim. `status`, `proof`, `ratified` and `verified` are
written by transitions, and `version` is the store's; everything else --
title, slug, scope, blocked_by, done_criteria, criteria_by, verify, references,
constraint, see, supersedes, author, created and the body -- is content, and
only the three verbs that leave an entry write it.

**It is strictly stronger than the count it replaces**, and that is the
argument rather than a consolation. A hand edit that bumps the version is
caught by both. A hand edit that does not bump it -- which is the likelier
hand edit, since the field is machinery a human has no reason to touch -- is
invisible to a count and caught here. The count is kept where it works, for an
ADR and a spec, because it catches the one case a hash cannot: a version moved
and nothing else.

**The bootstrap is the same one, and no corpus is migrated.** An entry written
before this carries no produced hash and is silent; an entity carrying no
machinery entry at all is silent, as it already is. Nothing gains a field,
nothing is rewritten, and the thousand entries already written stay valid.

**A signal, never a fault**, on the terms ADR-16813b3bcf37 already sets: an
entity edited outside the CLI is legal, is what a human with an editor does,
and what the signal says is that it happened rather than that it was wrong.
