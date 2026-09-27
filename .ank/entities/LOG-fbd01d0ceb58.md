---
id: LOG-fbd01d0ceb58
type: log
title: "cargo-test failed at ank-cli tests/tui.rs a_refusal_on_screen_is_the_one_the_cli_gave: it typed"
created: 2026-09-27T08:52:16Z
author: claude-tui-495
scope:
  - crates/ank-tui/src/view.rs
  - crates/ank-tui/src/lib.rs
  - crates/ank-tui/tests/**
  - crates/ank-cli/tests/tui.rs
about: TASK-0e544fa90566
seq: 4
schema: 4
version: 1
---

 ':LOG-000000000000', a line the reader stopped reading under ADR-559e, so nothing was refused, and it passed on 'no entity' out of the unread listing's 'no entity matches this filter', the text this task removes. Rewritten on a corpus whose config does not parse (find exit 1, 'duplicate field'), asserting error[1] and the CLI's bytes on screen. Without view.rs's fix it is red too: the suite's opened() waits for 'the corpus has not been read' to leave the header and a refused read never lifted it (30s timeout, frame shows the refusal under '(0 in the corpus)'); with it, green. Scope amended to carry crates/ank-cli/tests/tui.rs.
