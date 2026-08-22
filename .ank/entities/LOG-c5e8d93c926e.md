---
id: LOG-c5e8d93c926e
type: log
title: Two defects found by driving it rather than by reading it. init --at wrote the declaration and
created: 2026-08-22T20:03:36Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/repo.rs
  - crates/ank-contract/src/verbs.rs
about: TASK-49fce8b49d00
seq: 1
schema: 4
version: 1
---

 never said so, which is the one thing it writes outside the directory the caller pointed at; it is in both report shapes now. And a declared corpus tripped the outside-the-git-repository warning, which exists for the checkout nested inside another repository -- an accident nobody chose. A declaration is the reader naming the corpus, exactly as --repo is, and section 6 already exempts --repo for that reason; left in, it would have fired forever on the one layout the declaration exists to make usable. Three sweeps caught the two new flags and each needed a decision rather than an entry. --at carries a path that must not be normalised against this repository, since being outside it is the whole of what it means, so the classification gained a third category with one member instead of a lie in one of the two. And --at selects a different act rather than a different report, so the same-exit-code sweep cannot hold for it: listed by name so a second one is a decision somebody writes down. The golden moved by exactly two flags and two notes, verified structurally rather than by reading a character diff of a one-line document; contract still 1, additive.
