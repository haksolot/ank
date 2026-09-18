---
id: ADR-894d4bfbf9bd
type: adr
slug: a-differential-context-is-anchored-on-the-lease
title: A differential context is anchored on the lease, and the cursor is what renewal already writes
created: 2026-09-15T09:20:20Z
author: claude-code/fable-5.1+distributed-review
status: accepted
scope:
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-contract/src/verbs.rs
constraint: |
  context --since answers what moved since the caller last worked on the task it holds: the entities whose file changed, and the claims and completions recorded, at or after the instant the held claim's expires minus its ttl names. That instant is the cursor and nothing else: no field is added to the claim record, no state is kept per reader, and no daemon is consulted. context --since is the holder's work on the task it holds and renews the lease, so the cursor moves when it is read. With no claim held it is refused with the command that takes one. What moved is named and never carried: an entity is listed by id, and its content is what show answers.
ratified: 8f33ee5e6178
verified:
  - by: haksolot@vmi3223161
    at: 2026-09-18T10:43:06Z
schema: 4
version: 2
---

SPEC-15a56aeedcfd defers differential context under the name `--since`, and it
prices the deferral exactly: per-agent seen-state, one cursor per reader,
persisted somewhere no revision holds. Two of the three parallel sessions of
2026-08-13 asked for it under the name "what moved since I last looked". The
trigger fired, the row stayed, and the reason it stayed was the cost of the
cursor.

## The cursor already exists

A claim record carries `claimed`, `expires` and `ttl`. `claimed` never moves.
`expires` moves on every renewal, and a renewal is what a verb the holder runs
against the task it holds performs (ADR-0bb7ea8991bc). `ttl` is recorded
because a renewal recomputes `expires` from it. So `expires - ttl` is the
instant of the holder's last work on the task, written by the one mechanism
that already writes on every such verb, in a ref no revision holds, per
identity and per corpus. That is the cursor the deferred row said had nowhere
to live.

Measured on ank 0.7.0 (b654d479), 2026-09-15: `ClaimRecord` in
`crates/ank-cli/src/claim.rs` is `#[serde(deny_unknown_fields)]`. A field
added to the record would be refused by every binary older than the one that
wrote it, on a ref that clones share, which is why the cursor is derived and
not stored: a new field is a schema change for the whole fleet, and this
decision needs none.

## What moved

Three things, all read from planes `context` already opens, so the git
process count does not grow (ADR-cc65f1388a71):

- an entity whose file's mtime, as the index records it (ADR-1556aaffe0c5),
  is at or after the cursor;
- a claim whose `claimed`, or a completion whose `completed`, is at or after
  the cursor, in this clone's plane and in the watcher's mirror of another
  clone's claims (ADR-4b45f344344f);
- and nothing else. A constraint accepted mid-work is already the
  `constraints` hash on the record, and `done` already warns on it.

The comparison is `at or after`, and the direction is chosen. The cursor is
this clone's clock in UTC; an entity's mtime is this clone's filesystem clock;
a record mirrored from another clone was stamped by another machine. A skew
between any two of these makes the answer either miss a change or repeat one,
and only the second is safe: a `--since` that omits what moved is the stale
context the flag exists to remove, where one that repeats a line costs a line.
The same direction ADR-1556aaffe0c5 chose for the index, where a stat may say
unchanged and never changed.

## Why it renews, and why it is refused without a claim

`context` today declares that it renews never: its positional is a path, so
no id is resolved from it. `context --since` is a different question, asked
about the task the caller holds, and it is the holder's work on that task in
the sense ADR-0bb7ea8991bc gives the phrase. If it did not renew, the cursor
would move only on `log` and `show`, and "since I last looked" would answer
since the last time the caller reported, which is the very gap that decision
closed. So it renews, and two consecutive calls answer differently, which is
what a cursor means.

Without a claim there is no lease, so there is no cursor, and a `--since`
with nothing to be since is refused rather than answered from a guess. The
moment without a claim is the moment the full `context` is the right read
anyway: an agent choosing work needs the whole perimeter, not a delta.

## What this does not touch

The daemon (ADR-24e21cb83793) answers no verb and holds no cursor, and this
decision keeps it so. `events.jsonl` says a corpus moved and names nothing;
`--since` names what moved and carries nothing. Both leave the content to
`show`. A reader that follows the stream gets a reason to run `context
--since`, and a reader without a watcher runs it at the top of every turn;
the answer is the same.

What this does not lift: the deferred row is about a cursor, and the cursor
lives only while a claim does. A "since" that spans two claims, or a reader
with no claim at all, is still deferred and still costs what the row said.

## The trigger

An architecture review of 2026-09-15 proposed event subscriptions pushed to
agents so that a claim taken elsewhere reaches them within seconds. An agent
driven by a model cannot be interrupted: it acts in turns, and what it reads
at the top of a turn is its whole subscription. The reactive half of that
proposal therefore reduces to this question, answered here from state the
corpus already keeps.
