---
id: LOG-20e6bad90d06
type: log
title: Settled where colour is allowed to exist in this crate, and put the whole of it in
created: 2026-08-26T06:37:14Z
author: claude-code/opus-5+paints-table
scope:
  - crates/ank-tui/**
about: TASK-6cd41d23b7d1
seq: 1
schema: 4
version: 1
---

 crates/ank-tui/src/paint.rs so TASK-dd9747e5e305 has one rule to obey and no palette to think about.

ONE RENDER, NAMED Ink::role. It is a total match on ank_contract::meaning::Role and carries no string at all: a name reaches a colour only through meaning::role_of_status / role_of_marker / role_of_kind, so there is nowhere left for a second opinion about 'done' to be written. The registers are ank-cli's on purpose -- blue/cyan/green/dim/magenta/yellow/red/yellow -- because ADR-1f70ce2c3eac lets each surface paint a role its own way but not disagree about which things are alike, and both surfaces draw into the same terminal palette. MADE MECHANICAL rather than asserted: paint.rs's own suite walks src/ and fails if any file but paint.rs names Color::, Modifier::, .fg(, .bg( or add_modifier, AND fails if paint.rs's shipped code contains any name MEANINGS declares. Comments and #[cfg(test)] are cut before the scan, which is the same exemption tests/dependencies.rs gives by reading only src/. That pair is the criterion's grep, run by cargo.

HOW A ROW CARRIES A COLOUR: paint::Composed. A line is still built as the characters it always was, with (from,to,Role) marks recorded beside it in CHARACTERS not bytes. That was the load-bearing choice: fit(), pad(), wrap() and every window count in this reader are arithmetic on chars, and a list of styled spans would have made each of them a second implementation. rows_of() is UNTOUCHED and still reads symbols only, so bb43's property -- the focused panel is distinguishable without colour -- is still a thing the suite can state. Composed::fitted clips a mark to the cut so the '~' announcing it is never the tail of a painted identifier.

THE RULE, and it is the one dd97 has to keep: THIS READER PAINTS THE ROWS IT COMPOSES AND NOTHING ELSE. A field carries a meaning -- the status column of a listing, the identifier a row is addressed by, the state in the body panel's title -- and a field is something this crate put there. A document's body, a refusal the CLI wrote, and the fields of an answer are drawn as they arrived. Not squeamishness: 'done', 'accepted' and 'proposed' are ordinary English and an ADR's prose is full of them, so a reader that painted every occurrence would be telling its person that a sentence is a state. view.rs asserts exactly that with a body containing the word 'closed', a status no row of the fixture carries.

SEAM FOR TASK-dd9747e5e305, and it cost nothing. App::arrange is UNTOUCHED: no rectangle moved, no panel added, no band. Colour is per line inside App::panel, which means the one-column branch inherits it for free and a tap resolves against the same rectangles it always did. The ONE thing to watch: rows reach the screen through painted(&[Composed], self.ink) and the three chrome bands through paragraph(&[String]). A one-column layout that built its rows as Strings and rendered them with paragraph() would be the single place in this crate that draws an unpainted listing, and nothing would fail -- so build rows as Composed. bb43's two traps are untouched and still true: arrange() sizes from counts (page() calls arrange(), so the recursion is a stack overflow), and the borders stay ASCII.

WHERE NO_COLOR IS READ: Ink::detect(), once, in App::new, held on the App -- ank-cli's opted_out to the letter (non-empty NO_COLOR, or TERM=dumb). The terminal half of the CLI's condition is absent because 'tui' already refuses without one. App::inked(ink) exists for the suite so no test depends on the environment of the machine running it, and app() in view.rs's tests is explicitly PLAIN for the same reason.

THE TRAP THAT WOULD HAVE MADE THE WHOLE TEST VACUOUS. crossterm 0.29 reads NO_COLOR ITSELF and silently drops the colour half of an SGR -- and drops nothing else: an attribute like Dim goes out on the wire whatever the variable says. So a reader that had left NO_COLOR to its dependency would have passed a naive pty assertion. That is why Role::Retired is DIM rather than a colour and why tests/colour.rs closes a task in its corpus before every drive: 'closed' is Retired, dim is the one register crossterm does not suppress, and its absence under NO_COLOR is therefore this crate's decision rather than crossterm's manners. Verified non-vacuous by disabling Ink::detect's NO_COLOR branch and watching the suite go red. Second crossterm fact worth knowing: it spells every NAMED colour in the extended form, so ratatui's Color::Yellow reaches the wire as ESC[38;5;3m, not ESC[33m -- a byte-level assertion written against 30-37 finds nothing.

MEASURED THROUGH THE BINARY as CLAUDE.md demands: crates/ank-tui/tests/colour.rs drives the built ank on a pty at 110x30 through all four panels, once with NO_COLOR=1 and once with it removed from the child's environment, and asserts on the RAW BYTES -- no SGR sets anything in the first, something does in the second, what it sets is only an attribute or one of the terminal's own sixteen foregrounds and never a background, and the two GRIDS are identical character for character. That last one is the criterion's 'stays readable' stated the strongest way there is: anything colour carried alone would be a difference between those two frames. tests/terminal/mod.rs gained Screen.raw, Live::painting and Live::raw; Live::open still sets NO_COLOR=1, so panels.rs and confirmation.rs are unaffected.

NOTHING OUTSIDE crates/ank-tui/** CHANGED. crates/ank-cli/tests/tui.rs needed NO edit and I want that on the record, because it was the expected cost: its emulator applies a CSI ending in 'm' as 'no character moves', so the grid it byte-slices for KIND-hhhh ids is the same grid it always was.
