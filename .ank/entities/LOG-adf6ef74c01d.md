---
id: LOG-adf6ef74c01d
type: log
title: Experiment 3, binding through the binary (ank at /home/haksolot/.local/bin/ank), two fresh repos a
created: 2026-09-27T17:04:49Z
author: claude-code/opus-5.5+c6d1
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/config.rs
  - docs/**
about: TASK-c6d184d238e1
seq: 3
schema: 4
version: 1
---

 and b, a declares peers.bb, b declares peers.aa, ADR-fca48444ef48 in b scoped 'aa:src/**'. Baseline with paths both ways: ank context src/x.rs in a lists 'ADR-fca48444ef48@bb' (1 hit). (i) b's back-declaration changed to file://<abs path of a>: ank config accepts it without a word, and ank context in a lists 0 hits for ADR-fca4 and prints NO warning -- the binding vanishes silently, because Peer::binds (repo.rs:512) joins the string to the peer root and same_corpus canonicalises a path that does not exist. (ii) a's own declaration changed to file://<abs path of b>: one warning 'peer bb at file://... is not a corpus, answered without it'. So the second resolution ADR-da2819aef598 names is exactly where a URL fails, and it fails with no trace. A URL there can only be matched against the reader by identity, which a URL does not carry: ls-remote returns refs, never the root commit.
