---
id: LOG-1afc1b09f95b
type: log
title: The chrome is a header and the panels, and what the band of targets bought is now one cell of the
created: 2026-08-27T02:53:58Z
author: claude-code/opus-5+chrome
scope:
  - crates/ank-tui/**
about: TASK-9a402a54886f
seq: 2
schema: 4
version: 1
---

 header.

WHAT THE FRAME SPENDS, at 80x24 with no claim held and the queue never asked for: three rows of header and one of note band. Four, against ADR-c07e2694f0e1's measured thirteen. The trailer's two rows and the target band's two are gone; the note band keeps its single row for the reason it always had one minus the one that died with the trailer -- the panels' bottom border no longer moves under a reader every time a command reports something.

THE TWO TRAPS THE TASK RECORDED WERE NOT REDISCOVERED. arrange() still sizes from counts: the two bands it measures, note_lines and targets, read the width and nothing else and neither reaches page(). And the Length/Max ranking is untouched -- the panels are Max in both arrangements, the two bands are Length, and the confirmation's argv still wraps whole at 20x24 (the unit test that caught it is green).

Panels.keys is gone from the struct rather than left as a zero-height rectangle: a band nothing renders into is a field the next reader has to work out. Panels.actions stays and is zero rows except while a command is waiting -- targets() returns empty when self.pending is None, so drawing the band and hitting it are one arithmetic and there is no rectangle that is empty for the eye and occupied for the finger.

THE OFFER HAD TO GO SOMEWHERE OR THE ADR WOULD HAVE BEEN BROKEN BY THE REMOVAL. The clause asks for 'one permanently visible target that opens the key list', and there was none: TASK-8a6578851244 made ? open an overlay and made every line of it a target, but nothing on the frame opened it by touch, and the band I was deleting was the whole of the phone's way in. So the header's first row is padded and carries [?] on its right edge, three cells, and App::pointed resolves a press there to the key the table binds Command::Help to -- read through bindings::of_command, not written as a character here, because a bracketed letter in view.rs would be a sixth parallel key table one character long. It is resolved AFTER the pending and prompt checks, so a touch under a confirmation still dismisses it: the modal rule is not something a new target gets an exception from.

A SECOND PRESS ON THE SELECTED ROW OPENS IT, and it is the second and never the first because the first is what makes a row the selected one -- a tap that moved the cursor and opened what it landed on is a screen where nobody can look before they choose. It is also the panel already focused: a touch that crosses panels is a person changing where they are.

WHERE ratify_line WENT. It was the trailer's third line and the trailer is forbidden. It is not a key table, though -- it is a sentence about the document somebody has open -- so it moved into the row note_lines already keeps blank, behind the unresolved-scope sentence that was already using that slot the same way. Zero rows, and a person reading a proposal is still told what accept costs.

MEASURED THROUGH THE BINARY as CLAUDE.md and the criterion both demand: crates/ank-tui/tests/chrome.rs drives ank on a pseudo-terminal at 80x24, 40x24, 80x40, 40x12 and 120x40. A row is a panel's own content when its first character is a box edge read out of view::BOXES -- the header's rule is a horizontal and never a corner, the note band is a sentence or a blank -- and the desk frame has four such rows against a budget of five. The second test says the same thing in the shape it would break in: the last panel closes on the window's second-last row at every one of the five, the last row is the blank note band, and the only bracketed thing outside the panels is [?]. Asked outside the panels on purpose: a listing marks a held claim with [held], which is a field of a row and not a target.

WHAT I CHANGED IN TESTS I DO NOT OWN. crates/ank-cli/tests/tui.rs asserted the trailer was on the last row and counted rows with lines(). Screen::text joins the grid with newlines, so a blank last row makes the string end in one and lines() drops the empty piece -- a frame reads one row short of its window. Three assertions there now use split, and the resize test reads the bottom off the last panel's border instead of off a trailer. tests/panels.rs and tests/overlay.rs took the same change for the same reason. bindings::screen_line and write_line are deleted: nothing rendered them once the trailer went, and two suites were the only thing keeping them compiling.
