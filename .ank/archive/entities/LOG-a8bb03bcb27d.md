---
id: LOG-a8bb03bcb27d
type: log
title: "Criterion split into clauses, tests in crates/ank-cli/tests/ratify_ref.rs: (1) --reanchor on a"
created: 2026-10-20T17:24:11Z
author: claude-code/4b25
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/**
  - crates/ank-contract/src/verbs.rs
about: TASK-4b25843e6151
seq: 1
schema: 4
version: 1
---

 ratify-commit anchor no ref carries writes tag+ref, same anchor hash, message 'reanchors: <commit>' and 'by: <actor>', supersedes line kept, signed, pushed; file gets anchor:, ratified unchanged, verified entry by the actor, nothing committed. (2) exit 6 when a ref already carries the entity, exit 6 when the commit's anchor differs from the file, exit 6 on an unratified document. (3) after it, check in a depth-1 clone judges the tag once (verify-tag) and says nothing unverifiable; show prints 're-anchored from commit <sha12>' and --json ratification.reanchored. Red measured: both tests exit 1, 'unknown flag --reanchor for accept'. Reading taken: the actor goes on 'by:', the original ratifier stays on the named commit; a commit unreachable (shallow) is refused at 7 with git fetch --unshallow, which the criterion does not fix.
