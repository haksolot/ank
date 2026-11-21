---
id: LOG-0bf3f255e41f
type: log
title: Measured on this repository (2 839 commits), release builds, GIT_TRACE at an absolute path. Before,
created: 2026-11-21T20:07:35Z
author: claude-code/fable-5.2+8137
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/status.rs
about: TASK-813700f66d63
seq: 3
schema: 4
version: 1
---

 1dbd58ff: show 440-520 ms with 1 rev-list --full-history per run, find <word> 400-600 ms with 1. After: show run 1 walks once (1 walk, 6 git), runs 2 and 3 start 0 walks and 5 git, 180-260 ms; find, context and scope start 0 walks; check starts 1 walk on each of 3 runs and the show after it 0. Output bytes of show and find equal the 1dbd58ff build's (sha1 cbf49bae, 50baea96). One process replaces the walk: rev-parse --verify HEAD^{commit}. Mutation: with the key ignored in the SELECT, status::a_head_that_moved_is_walked_again_and_the_answer_follows_it goes red. Clause 4 reading taken: an unborn HEAD is tested through the binary; an index that cannot be written is tested at index.rs, where an absent file is neither created nor answered from, since a read-only index refuses the verb before the walk is reached. Clause 6 is held by the existing index_per_schema tests, which read the schema number from the source. A decision with no ratify commit still pays the per-path fallback search on every read, as before: not in this criterion.
