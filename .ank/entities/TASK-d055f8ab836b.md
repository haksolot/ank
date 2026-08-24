---
id: TASK-d055f8ab836b
type: task
slug: section-4-gains-read-and-the-suite-declares-the
title: Section 4 gains read, and the suite declares the gap until it ships
created: 2026-08-24T18:16:12Z
author: claude-code/opus-5-reading
status: done
scope:
  - crates/ank-cli/tests/skill.rs
blocked_by: []
done_criteria: |
  A successor to the spec that carries section 4 exists, created with new spec --supersedes, carrying the predecessor's document whole with read added to the Commands block in the group and position ank help will print it in, and a passage stating what the verb records and what it refuses. NOT_YET_DISPATCHED declares read with this corpus's task id for the implementation, so every_verb_section_4_lists_ships_or_is_declared_unimplemented stays green while the gap exists. cargo test is green, cargo fmt --check passes, and ank check reports no fault. The successor lands proposed: ratifying it is a human act and is not part of this criterion.
criteria_by: creator
proof:
  - type: commit
    ref: e64a74c1da6e58bc00faa746f4eb245e3f8ab31c
    criteria: a582e523ab1c
    via: submitted
schema: 4
version: 3
---

`read` cannot ship in the landing that puts it in section 4, and the suite says
so in as many words. `read_section_4_document` in `tests/skill.rs` reads the
**ratified** spec on purpose: while a supersession is in flight two documents
carry the Commands block, and the one the corpus is held to is the accepted one.
So a proposed successor listing `read` leaves
`every_dispatched_verb_is_listed_in_section_4` red from the moment the verb
dispatches until a human signs the ratification.

The convention for exactly this gap already exists. `NOT_YET_DISPATCHED` holds a
verb section 4 lists and the binary does not yet answer to, and its own comment
records that `scope`, `graph`, `status` and `edit` each came through it in turn,
each turning the suite red until the declaration was removed in the commit that
implemented it.

So this is the first of two landings, and the human act sits between them: this
one writes the surface down and declares the gap, ratification makes it binding,
and TASK-e3370ef322d8 ships the verb and removes the declaration.

**The successor carries the predecessor whole.** A spec's ratification anchor
covers its body as well as its scope, so `amend` refuses the change and a
succession is the only road. What that costs here is a document reproduced with
one verb added, which is the price the anchor buys.
