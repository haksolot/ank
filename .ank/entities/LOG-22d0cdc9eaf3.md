---
id: LOG-22d0cdc9eaf3
type: log
title: "The reading half, and four cases in one order: --repo, a declaration keyed on the identity, the"
created: 2026-08-22T19:35:10Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/cli.rs
about: TASK-88bff140d416
seq: 1
schema: 4
version: 1
---

 walk, the refusal. Two things the criterion named needed a reading. The spec clause says SPEC-201041998d90 states the resolution order; that document is superseded twice over and its chain ends on SPEC-55162f52d40e, which is what I superseded -- the rule TASK-e2da6b0cc817 implemented, applied to the criterion that names it. And the git question: resolving a corpus now asks for the repository identity, which is a git process at startup, where ADR-9307e5d214a7 says git is required per verb and never at startup. It is compatible and the guard is the order: an empty map costs a file that is not there and asks git nothing, so only a reader who has declared something pays a spawn, and a directory that is no repository has no identity, matches nothing and falls through to the walk. No verb is refused for want of git. Measured by hand before the tests: the declared corpus answers from the tree root and from a subdirectory two deep, with the scope glob confronted against the tree and not against the corpus.
