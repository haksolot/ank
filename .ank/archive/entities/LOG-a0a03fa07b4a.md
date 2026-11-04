---
id: LOG-a0a03fa07b4a
type: log
title: "Full ank-cli suite: 889 passed, 2 failed, both expected knock-ons in files outside the scope."
created: 2026-11-03T14:57:39Z
author: claude-code/opus-5.5+task-64f9-out-of-scope
scope:
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/done_out_of_scope.rs
  - crates/ank-cli/tests/main.rs
  - crates/ank-cli/src/verify.rs
about: TASK-64f941335624
seq: 5
schema: 4
version: 1
---

 claims_remote.rs pins done at 14 git processes on a typed-proof close and now sees 15 (the diff-tree --merge-base, the +1 measured earlier). docs/integrating.md:181 quotes done --json, which now carries outside_scope:[]. Asking the orchestrator to widen to both.
