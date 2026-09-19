---
id: LOG-a98af14f1a65
type: log
title: "released: Two more errors in my own criterion, both found by reading the suite before writing code,"
created: 2026-08-24T18:15:37Z
author: claude-code/opus-5-reading
scope:
  - crates/ank-cli/**
  - crates/ank-contract/**
about: TASK-e3370ef322d8
seq: 2
schema: 4
version: 1
---

 and both about the order the work has to land in rather than about what the verb does.

The contract version does not move. Its own doc says a document may gain a field within a version and may never lose, rename or retype one: a new verb adds a new document and breaks no caller that binds an existing one. It has been 1 since it existed and no verb added since moved it. Asking for a bump would spend the number on nothing.

And a verb cannot ship in the same landing that puts it in section 4. read_section_4_document in tests/skill.rs reads the ratified spec on purpose, in as many words: while a supersession is in flight the suite has to read the document the corpus is actually held to. So a proposed successor listing read leaves every_dispatched_verb_is_listed_in_section_4 red from the moment the verb dispatches until a human ratifies. The suite already carries the convention for this, NOT_YET_DISPATCHED, and scope, graph, status and edit each came through it in turn.

So this is two landings with a human act between them, and one task cannot hold both. Releasing to split it.
