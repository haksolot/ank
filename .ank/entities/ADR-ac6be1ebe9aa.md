---
id: ADR-ac6be1ebe9aa
type: adr
slug: the-reader-is-a-dashboard-the-listings-beside-th
title: "The reader is a dashboard: the listings beside the detail, the work in progress on the frame, and the keys always shown"
created: 2026-10-09T12:19:09Z
author: clausse@macbook-air-de-clausse
status: proposed
scope:
  - crates/ank-tui/**
constraint: |
  The terminal reader is a full-screen application drawn with ratatui over crossterm. Those two dependencies are spent here and nowhere else in the workspace, and what they buy is stated rather than assumed: several regions on one frame, a scrollbar over each, keystroke input in raw mode, terminal resize, and mouse events including the wheel. No FFI enters this tree for any of it, on any platform.
  
  The reader is a dashboard. Where the terminal is wide enough, its listings are drawn at once in a column of panels -- the work in progress, the tasks, the decisions, and what waits for a ratification -- beside a detail panel that shows the selected row. Exactly one panel has the focus; it is named by a character on its title and by the marker on its selected row, and colour only repeats what those characters say. A digit names a panel, Tab walks them, and opening a row moves the focus into the detail panel without taking the listings off the frame. Below a single declared width the same panels are reached one at a time with the same keys, and nothing collapses to a border with nothing inside it; where a row cannot afford its fields it drops them from the right, and the identifier and the marker are the two it never drops. One function composes a row, and a row is drawn the same way wherever it is drawn. A search narrows the focused listing as it is typed and is not a line to compose and submit.
  
  The work in progress is a panel of its own. Every live claim the reader can see is a row carrying the task, its holder, how long ago it was claimed, and when it expires or that it has lapsed, with the reader's own claims marked. Moving the cursor never spawns a verb that could renew a claim: the detail of a row is drawn from what the listings already read, and show runs when a person opens the row.
  
  The keys are always on the screen. The last row of every frame is a line of keys computed from the reader's own key table and filtered by what the focused panel offers; it always names the key that opens the full key list and the key that quits, and where the width cannot hold it whole it drops entries from the middle and keeps those two. Every entry of that line is a touch target that presses the key it names, and so is every panel title.
  
  Input is a keystroke and never a line. Every command that only moves the screen is one key, and no command requires a modifier chord. Every verb that writes is reached through a confirmation that shows the exact command line about to be run and does not run it until the person confirms: it must be impossible to reach a spawned write without passing through it, and a verb added to the reader is inside that rule on the day it arrives. Which verbs may be spawned at all stays a list written in the code, measured against the key table and never generated from it.
  
  A key is the verb it runs. Where the CLI declares a verb the reader binds that verb's own initial to it and navigation takes what is left. What the reader offers is read out of the contract's own verb table rather than transcribed beside it.
  
  What each listing shows first is chosen rather than inherited: the work that is alive, then what waits for a ratification, then what was created most recently. An identifier orders nothing. Log entries are read under the entity they annotate and are never rows of a listing.
  
  Structure is drawn with box-drawing glyphs and drops to ASCII on the terminal that declares it can render neither those nor colour; that probe is the terminal's own declaration and never NO_COLOR. The same corpus drawn with the paint and without it is identical character for character. Where a person is standing is carried by a character and never by colour. A scrollbar is characters or it is not drawn.
  
  What ADR-8bd76e8d7c4e fixed is untouched and this restates none of it: the reader reaches the corpus only by running the CLI with --json, it writes nothing the person at the keyboard did not ask for, it renews no claim on its own, and accept stays a signed human act it may drive and never perform. No browser reader, nothing under a viewer/ directory, no HTML page.
supersedes: ADR-559eebf5c6f5
schema: 4
version: 1
---

ADR-559eebf5c6f5 took the panels away, and its argument was sound for the
question it asked: lazygit shows planes of one repository that a person
compares by eye, ank shows a corpus a person walks and opens, so the
arrangement bought nothing. This supersedes it because the question was
incomplete, not because the answer was wrong for it.

## What one list cannot answer

There is a comparison on this screen after all, and it is the one a person
opens a coordination tool to make: who holds what, since when, and whether the
lease is still alive. On the one-list reader that answer is a screen reached in
its turn, and reaching it costs a `status` that the opening deliberately does
not ask (TASK-fff0a98511b2). So the person who runs several agents at once --
the audience this project names first -- opens the reader and sees a list of
entities ordered by recency, and has to know that `1` exists to learn whether
anybody is working.

That was a fair trade while `status` took twenty seconds. TASK-be17972988d9
fixed that: measured on 2026-10-09 with the release binary, `status --json`
answers in 1.0 s on this repository's own corpus (about 1500 entities) and in
0.36 s on a corpus of 87. The opening can draw from `find` first and add the
claims a moment later without either wait reaching the first frame.

## The key line

"No offer drawn at rest" kept the frame clean and made the reader opaque to
anyone who had not read its key list. The first thing a new user asked for,
before any feature, was to see how to quit. The cost of the line is one row,
and the rule that bought rows back -- that no frame collapses, that the region
is paid for before any band -- is kept: below the declared width the key line
shortens from the middle and the panels go back to one at a time.

## What does not move

Everything about writing is carried forward word for word: one key per command,
no chords, the confirmation that shows the exact command line, the verb list
written in the code, a key is its verb's initial. So is the order of what is
shown first, the folding of log entries under their entity, the ASCII fallback
and the rule that colour never carries a fact alone. ADR-8bd76e8d7c4e is not
touched.

One clause is new and it protects ADR-8bd76e8d7c4e rather than relaxing it: a
detail panel that followed the cursor by running `show` would renew the
reader's own claim every time the cursor crossed it (ADR-0bb7ea8991bc). The
detail is drawn from what the listings already read, and `show` waits for a
person to open the row, as it does today.

## What it costs

Chrome: a header row, a key row, and the borders of four panels on a wide
frame. The tests that pinned the old budget (`tests/chrome.rs` and its
siblings) are rewritten against this decision in the same change that draws
it, and a claim's start time has to reach `status --json`, where the claim
record already stores it as `claimed`.

Requested by a user of the reader, with the maintainer's agreement to a
redesign.
