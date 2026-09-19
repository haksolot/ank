---
id: LOG-81e031fd91bf
type: log
title: The parse_corpora() call site is worse than a misleading sentence, and measured rather than
created: 2026-08-31T02:58:56Z
author: claude-code/opus-5+corpora
scope:
  - crates/ank-cli/src/config.rs
about: TASK-409e4c7d8aac
seq: 1
schema: 4
version: 1
---

 inferred. parse_corpora is only ever used differentially -- refused when the write introduces a parse failure, never when the file already had one -- so a corpora.yml at schema 2 fails BOTH sides of that comparison and the guard never fires. Measured: 'ank config --user corpora.<40-hex> /some/where' onto a schema-2 file exits 0, prints the key, and writes the entry into it, leaving a file this binary has just said it cannot read (ank status on the same file, same second: exit 9). ank config --user runs before corpus resolution (cli.rs:1199) so declarations() never gets a say, and 'ank init --at' reaches the same surgery through declare_corpus(). Fixing only the message inside parse_corpora would therefore be untestable through the binary: nothing propagates it. The refusal has to be taken where the existing text is read, ahead of the surgery.
