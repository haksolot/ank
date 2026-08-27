---
id: LOG-7887dc4f2b4c
type: log
title: The key list is an overlay drawn over everything below the header, and arrange() does not know it
created: 2026-08-26T22:41:16Z
author: claude-code/opus-5+overlay
scope:
  - crates/ank-tui/**
about: TASK-8a6578851244
seq: 1
schema: 4
version: 1
---

 exists. That is the whole of why Esc gives the frame back character for character: nothing moved to make room, so nothing has to move back. The note band still reserves its one row underneath, unchanged.

ONE ROW PER BINDING, GENERATED FROM THE LINE THE TRAILER ALREADY DRAWS. bindings::listing() now returns Vec<Item> -- text plus the binding a press runs -- built from the same lines() the trailer's screen_line/write_line/ratify_line come from, so there is still one list and the unit test that holds every binding to exactly one line still states it. What the trailer's lead and note become is a heading, and a heading is the one row of the overlay that runs nothing. A line of six entries joined by two spaces could not be a target; a row can.

A KEYLESS VERB IS REACHED THROUGH input::parse AND NOT AROUND IT. The writing half has no letter until TASK-1a415107fd56, so a press on the row reading 'claim' composes the verb the way a typed line does: parse(verb.name, focus) -> Command::Act -> propose. Tail::Message and Tail::Behind cannot be composed with nothing, so those rows seed the prompt instead. dependencies.rs is untouched by this -- there is still one 'Command::Act(' arm and one 'ank.act(' in src/.

THE OVERLAY'S GRAMMAR IS COMMANDS, NEVER LETTERS. App::over_list reads keys::typed and answers Move/Page/Top by scrolling and Help/Back by closing; everything else closes the list and goes on to do what it does. So Esc closes it because Esc is an alias of back, and there is no fifth key table.

THE PTY FLAKE THAT COST ME A RED WORKSPACE RUN, AND IT IS THE SUITE'S AND NOT THE READER'S. Live::frame() settles the screen -- it returns once two reads 200ms apart agree -- and a settled screen is NOT a screen that has answered the keystroke just sent to it. Under load the reader was still getting to the 'n' that scrolls, frame() returned the page that was already there, and the row located on it was pressed against a list that had since moved: the press landed on '? keys' instead of 'claim', which reopens the list at the top and looks exactly like a scroll that never happened. The fix is to synchronise on something the reader prints: the overlay's own count line, 'KEYS 20-38 of 40', waited for with until() after every page.

TWO MORE TRAPS IN terminal/mod.rs FOR WHOEVER WRITES THE NEXT SUITE. until(|t| t.contains("2 ENTITIES")) waits for nothing at all -- a panel title carries the panel's NUMBER and its name and is on the very first frame, before the reader has asked the CLI anything; wait for a row of the corpus ('TASK-') instead. And live.quit() over a waiting confirmation hangs for ever: quit() sends 'q', and 'q' over a confirmation dismisses the command rather than ending the session, so child.wait() never returns. Decline it first.
