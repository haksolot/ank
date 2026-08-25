---
id: LOG-1ce4cb8bc9fc
type: log
title: The reader's writing half runs the verb with --json, like its reads. ADR-8bd76e8d7c4e and
created: 2026-08-25T06:02:28Z
author: claude-code/opus-5+reader-acts
scope:
  - crates/ank-tui/**
about: TASK-b50b340c0bb1
seq: 0
schema: 4
version: 1
---

 SPEC-93531977642f both say the reader reaches the corpus 'only by running the CLI with --json', so an act that dropped the flag to get a human line back would be reading that sentence as decoration. A refusal reaches the screen the same way either way: the CLI renders error[N] and its hint on stderr whatever --json says, so passing stderr through unaltered gives the criterion its exit code and its way out with no second vocabulary. A success is the document's own fields, one per line, under a chrome header naming the command run -- the split lib.rs already declares, chrome ours and data the CLI's.
