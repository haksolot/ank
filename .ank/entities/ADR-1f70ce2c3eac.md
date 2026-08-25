---
id: ADR-1f70ce2c3eac
type: adr
slug: what-a-status-means-is-one-table-and-each-surfac
title: What a status means is one table, and each surface paints it its own way
created: 2026-08-25T22:44:57Z
author: haksolot@vmi3223161
status: proposed
scope:
  - crates/ank-cli/src/style.rs
  - crates/ank-contract/**
constraint: |
  Every long flag keeps its form. Short forms are single-dash, single-letter mappings declared in the specification, section 4, before any code moves: one table, one letter per long flag where a letter is available, no bundling (-st is an error naming the two flags to type), and no verb abbreviation.
  
  Presentation splits in two, and only one half depends on who is reading. Structure -- tree connectors, gutters, the marker on a held task -- is text, emitted identically to every reader, at a terminal or into a pipe or a file, on every platform. --json carries neither structure nor colour: it is data, and it stays byte-for-byte what a caller's parser reads.
  
  What a status, a kind or a severity means is one table, held where every surface reads it and holding no escape sequence. Each surface paints that meaning its own way: the CLI in hand-written ANSI, emitted only when stdout is a terminal and NO_COLOR is unset, with no --color flag; the terminal reader through the library it draws with (ADR-0b55983421dd), honouring NO_COLOR the same way. Two renderers are allowed and a second table is not: a surface that decided for itself what done looks like would be the second place this rule exists to prevent.
supersedes: ADR-0c8ab846d262
schema: 4
version: 1
---

ADR-0c8ab846d262 is kept whole except in one clause, and the amendment is
narrower than it first appears.

## What was actually load-bearing

That decision said colour is "hand-written ANSI, no new dependency". Two things
were bundled in one sentence, and only one of them was the point.

The point is that **one place decides what a status looks like**.
`crates/ank-tui/src/frame.rs` emits no colour at all today and says why: the
palette lives in `ank-cli/src/style.rs`, this crate may not link `ank-cli`, and
a second palette would be a second place deciding. That reasoning is correct and
this decision keeps it. It is also what made a monochrome reader the only
available answer, since the crate could reach the palette by no other route.

"Hand-written ANSI, no new dependency" was the *implementation* that gave it, at
a time when the only surface was the CLI's own lines. It stops being available
the moment a second surface draws with a library, and holding to it would decide
by inertia what nobody chose.

## What moves, and what deliberately does not

The escape sequences do not move. What moves is the **meaning**: a table saying
that `done` is an accomplishment, `blocked` an impediment, a fault a fault --
roles, not colours, and no byte of ANSI anywhere in it.

`ank-cli` renders those roles in hand-written ANSI, exactly as it does now and
under the same two conditions, so nothing a caller reads changes by a byte.
`ank-tui` renders them through ratatui, which is what a library that owns the
screen has to be allowed to do. Neither knows how the other paints.

That is the smaller change and the better one. Putting escape sequences in a
shared crate would have made `ank-tui` carry ANSI it never emits, to satisfy a
sentence about how `ank-cli` writes -- coupling two surfaces through the one
thing they genuinely do differently, in order to share the one thing they must
not do differently.

## NO_COLOR reaches the reader too

Somebody who sets `NO_COLOR` means it, and a full-screen reader is the surface
where ignoring it is most visible. The reader is always at a terminal by
construction, so the terminal half of the CLI's condition is vacuous there; the
variable is not, and it is honoured.
