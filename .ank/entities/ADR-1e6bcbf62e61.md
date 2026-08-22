---
id: ADR-1e6bcbf62e61
type: adr
slug: an-identifier-written-in-prose-is-not-a-referenc
title: An identifier written in prose is not a reference, and one that names nothing is a signal
created: 2026-08-22T20:50:30Z
author: claude-code/opus-5
status: accepted
scope:
  - crates/ank-cli/**
constraint: |
  check reads entity identifiers out of the prose it stores -- a log entry's message, a task's done_criteria, an entity's body -- and reports, once for the corpus and never once per mention, those that name an entity the corpus does not hold. A signal and never a fault: the prose is not wrong, it points at nothing. An identifier that resolves is silent whatever its status, superseded included, because prose is where history is written and because a frozen criterion cannot be repaired. Nothing is refused at write time and nothing in prose confers, orders or is followed: an identifier in prose is not a reference, which is the rule ADR-c88f99e1c16e states and this one does not touch.
ratified: 583a92484162
verified:
  - by: claude-code/opus-5
    at: 2026-08-22T20:55:54Z
schema: 4
version: 2
---

Measured on this corpus on 2026-08-22, before deciding: 1087 distinct entity
identifiers are named in the prose of log entries, and 15 of them resolve to
nothing. That is 1.4 percent, which is what makes this affordable at all -- a
rule firing on a fifth of the corpus would be the volume section 11 names as
what teaches a reader to stop reading `check`.

**The corpus has already decided the neighbouring question, and this does not
overturn it.** ADR-c88f99e1c16e and the catalogue say that only the declared
field is read: a section number written in prose is not a reference, and no
finding pretends otherwise. That stands. A citation in prose confers nothing,
orders nothing and is followed by nobody. What is reported here is narrower: an
identifier shaped like one this corpus mints, naming an entity the corpus does
not hold. It is not treated as a reference; it is reported as a pointer to
nothing.

**The case that produced this is a closure reason.** `ank close --reason` took a
message naming `TASK-eb2c8b0bee45`, an identifier written before the task it
meant existed, and nothing refused it because no verb resolves an id inside a
message. The mistake is now permanent, since a closure cannot be corrected, and
it was found by a human reading rather than by the tool.

**The write is not where this belongs.** Refusing a message that names an
unresolvable id would make the tool a gatekeeper over prose, which
ADR-6b3f19e08a24 refuses in general, and would refuse the legitimate mention of
an entity that has since been deleted. The measurement says the same thing from
the other side: of the 15, roughly half are fixture identifiers quoted in prose
about tests, which will never resolve and are perfectly good writing.

**One finding for the corpus, not one per mention**, which is the volume rule
this catalogue already applies three times -- to entities predating `author`, to
actor values, and to orphan entries. The first few are named and the count
carries the rest.

**An identifier that resolves is silent, whatever its status.** A superseded
document named in prose is history, and history is what prose is for; and a
`done_criteria` naming one is frozen at claim and cannot be repaired at all, so
reporting it would be a finding nobody can clear -- the failure this corpus has
already named twice.
