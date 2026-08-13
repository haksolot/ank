---
id: TASK-b288bc31e2d7
type: task
slug: show-paints-the-entity-it-prints-and-a-pipe-stil
title: show paints the entity it prints, and a pipe still receives it byte for byte
created: 2026-08-09T02:58:15Z
author: seanl@sean-laptop
status: closed
scope:
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/style.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/paint.rs
  - crates/ank-cli/src/main.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: [TASK-bfe1cbd9ec42]
done_criteria: |
  Section 4 of docs/ank-spec-v1.1.md says, before the code moves, that show paints the entity it renders and adds, removes and moves no character, so the byte-for-byte guarantee ADR-01b6dd05f0db states holds for every reader that parses; and its palette table gains the elements that painting uses: the frontmatter fences and keys dim, the value of id yellow, the value of status the colour of its own marker, a markdown heading in the body bold, and the timestamp and author of a log entry dim. A single new method on Style in crates/ank-cli/src/style.rs performs the painting, so every escape byte is still written in style.rs and nowhere else, and human::show calls it in place of writing the text raw. The invariant is asserted rather than assumed: on a fixture carrying a block scalar, a sequence, a --- inside the body, a heading and a log entry with its em-dash, stripping the SGR sequences from the painted form yields the input byte for byte, and painting with PLAIN yields the input byte for byte. show_prints_the_entity_verbatim and show_on_an_adr_stays_verbatim_and_adds_nothing pass with no edit. A test drives show itself at COLOR and asserts its stripped output equals its PLAIN output. The pipe suite through the binary stays green with no edit, show being already in styled_surface, and that is verified rather than assumed.
criteria_by: creator
schema: 3
version: 3
---

## Why

`ank show` writes the entity with a single `write!(out, "{text}")`. On a real
task file -- YAML frontmatter, block scalars, a `## Log` and its timestamped
entries -- that is a uniform wall of text, and the only styled thing on screen
is the `BLOCKED BY` / `UNBLOCKS` sections show appends itself.

## What is not available, and why

Re-laying it out is not on the table, and not because it would be hard.

`ank show <id>` returns the entity byte for byte: ADR-01b6dd05f0db says so, and
skill/SKILL.md says so under a frozen revision hash. Nothing may be added,
removed or moved.

Laying it out only at a terminal is the option ADR-0c8ab846d262 examined and
rejected in writing: it gives one corpus two shapes, and turns "what did the
agent see" into a question with two answers.

What is left is colour, and colour is enough. Escapes carry no width, so the
painted form strips back to the byte sequence that exists today. A pipe -- which
is what every agent reads through -- is unchanged, and the guarantee holds
exactly where it was ever asserted. A human at a terminal gets hierarchy.

## Where the scanner lives

On Style, in style.rs, rather than in human.rs. Two reasons, and the second is
the load-bearing one: TASK-4601ed18d84e's criterion says every escape byte is
written in style.rs and nowhere else, and a scanner that emits escapes belongs
where that sentence can stay literally true; and the strip-equality invariant is
then a unit test of the module that owns it, rather than a test that has to
stand up a repository to reach show.

## The traps the scan has to survive

A block scalar (`constraint: |`) whose continuation lines are indented and are
not keys. A sequence (`scope:`) whose items begin with `  - `. A `---` inside
the body, which must not be read as a fence once the frontmatter region has
closed. A quoted title containing a colon. And the em-dash separating a log
entry's author from its message, which is multi-byte and must never be sliced
through -- so every cut point is found on an ASCII pattern or by searching for
the separator whole.
