---
id: LOG-546ed7a59ca0
type: log
title: "Two flags on an existing verb, so ADR-2f8a is untouched: --verify (repeatable) and --body. Verifier"
created: 2026-07-31T22:41:35Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-bc214fd815b2
seq: 0
schema: 3
version: 1
---

 names are resolved against config.yml at creation, same doctrine as --blocked-by in the same function -- a name matching nothing would otherwise surface at done, far from its cause; the refusal is a 7 naming what is declared. --verify on an ADR is refused rather than dropped, because an ADR has no verify field and a flag silently ignored teaches the caller it worked. body_of emits the canonical shape directly, blank line after the frontmatter and a trailing newline, so creation cannot write a file the first rewrite would reformat. new now takes cfg, which meant threading it through dispatch.
