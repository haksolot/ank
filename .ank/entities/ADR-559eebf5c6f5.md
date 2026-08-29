---
id: ADR-559eebf5c6f5
type: adr
slug: the-reader-is-one-list-and-one-document-and-what
title: The reader is one list and one document, and what it shows first is what moved last
created: 2026-08-27T16:32:47Z
author: haksolot@vmi3223161
status: accepted
scope:
  - crates/ank-tui/**
constraint: |
  The terminal reader is a full-screen application drawn with ratatui over crossterm. Those two dependencies are spent here and nowhere else in the workspace, and what they buy is stated rather than assumed: one full-screen region, a scrollbar over it, keystroke input in raw mode, terminal resize, and mouse events including the wheel. No FFI enters this tree for any of it, on any platform.
  
  The reader is a list and a document, and never both at once. Opening a row replaces the list with the document it names; leaving that document gives the list back with the same row selected. There is no panel to choose between and so no focus to arbitrate: one row is selected, one function composes a row, and a row is drawn the same way wherever it is drawn. What was a panel is a screen reached in its turn or a section of the one list, never a rule drawn around emptiness. A search narrows the list as it is typed and is not a line to compose and submit.
  
  Input is a keystroke and no longer a line. Every command that only moves the screen is one key. Every verb that writes is reached through a confirmation that shows the exact command line about to be run and does not run it until the person confirms: it must be impossible to reach a spawned write without passing through it, and a verb added to the reader is inside that rule on the day it arrives. Which verbs may be spawned at all stays a list written in the code, measured against the key table and never generated from it.
  
  The reader answers a person who is not at a desk. One screen is what every width gets, so no arrangement reflows and nothing collapses to a border with nothing inside it; where a row cannot afford its fields it drops them from the right, and the identifier and the marker are the two it never drops. Touching the row already selected opens it. Every action a screen offers is reachable by touch through permanently visible targets carved out of the header the reader draws anyway, so they cost no row; the kind in force is one of them. No command requires a modifier chord, and no offer is drawn at rest.
  
  A key is the verb it runs. Where the CLI declares a verb the reader binds that verb's own initial to it and navigation takes what is left. What the reader offers is read out of the contract's own verb table rather than transcribed beside it.
  
  What the reader shows first is chosen rather than inherited: the work that is alive, then what waits for a ratification, then what was created most recently. An identifier orders nothing. Log entries are read under the entity they annotate and are never rows of the list.
  
  Structure is drawn with box-drawing glyphs and drops to ASCII on the terminal that declares it can render neither those nor colour; that probe is the terminal's own declaration and never NO_COLOR. The same corpus drawn with the paint and without it is identical character for character. Where a person is standing is carried by the marker on the row and never by colour. A scrollbar is characters or it is not drawn.
  
  What ADR-8bd76e8d7c4e fixed is untouched and this restates none of it: the reader reaches the corpus only by running the CLI with --json, it writes nothing the person at the keyboard did not ask for, it renews no claim on its own, and accept stays a signed human act it may drive and never perform. No browser reader, nothing under a viewer/ directory, no HTML page.
supersedes: ADR-c07e2694f0e1
ratified: 46d23695e444
verified:
  - by: haksolot@vmi3223161
    at: 2026-08-29T20:31:10Z
schema: 4
version: 3
---

ADR-c07e2694f0e1 is one day old and this supersedes it, which needs saying
plainly. Six of its eight clauses are carried forward, three of them word for
word. What changes is the shape of the screen and the order of what is on it,
and neither was wrong when it was written: they were inherited.

## The inheritance, and why it ends

ADR-0b55983421dd said it outright: "lazygit is the reference, and it was the
reference before any of this was written -- the intention simply never reached
the corpus". ADR-c07e2694f0e1 then spent its whole argument on what the panels
cost and none on whether there should be panels, because the panels were not up
for decision. They are now.

lazygit shows four planes of one repository that a person compares by eye: the
files, the branches, the commits, the diff. Comparison is what the arrangement
buys. ank shows a corpus of entities that a person walks and opens. Nothing on
that screen is compared with anything else on it, so the arrangement buys
nothing and is paid for anyway -- in rows, in borders, and in the four separate
answers the code now gives to "how is a row drawn".

## What four panels actually cost, measured

Three row schemas with no shared composer: claims is cursor, held marker, short
identifier, holder, expiry, title; entities is cursor, row number, short
identifier, status, title, and a trailing `[held]` word for the same fact the
claims panel draws as a marker; the queue has no status column at all. Only
entities prints row numbers, but a bare number at the prompt selects a row on
any focused listing, so two panels ask for a number they do not show.

Focus is a border weight, and every overlay draws the focused border
unconditionally, so a modal looks exactly like the panel behind it. That is the
inconsistency the owner saw, and it is not a rendering bug: it is what happens
when four things can be focused and one glyph has to say which.

With one region none of that has anywhere to live. That is the argument.

## Why the order was noise

`crates/ank-cli/src/index.rs:709` orders by identifier, and identifiers are
`KIND-<hex>`. On this corpus the first row is `ADR-01b6` because `01b6` sorts
first. It is not creation order, not recency, not relevance; it is a hash, shown
in the position a person reads as "most important".

The fix is cheap because the data is already there. The index stores `created`
at `index.rs:106` and `SELECT_ROW` already selects it; it is simply never
serialised into `find --json`. Logs already carry an `about` column with an
index on it, so folding them under the entity they annotate is a query that
exists. What was missing was the decision about what order means, and that is
this clause.

Seventy per cent of this corpus is log entries. A list that opens on them is a
list that has answered a question nobody asked.

## The target that costs no row

ADR-c07e2694f0e1 removed a band of touch targets because it cost four rows of
twenty-four, and it replaced it with `?` in the header plus second-touch-to-
open. That pattern is right and this extends it rather than reopening it:
`help_rect` carves `?` out of the header that is drawn anyway. The kind in force
has to be written somewhere regardless -- a person must be able to see what they
are looking at -- so making that cell a target adds nothing to the frame. It is
information made touchable, which is the opposite of an offer drawn at rest.

That is what answers "on a phone you cannot even change the type you are
searching", without buying back the tax.
