---
id: LOG-4e95fc4864bd
type: log
title: Measured the cost clause instead of assuming it, with a git shim on PATH logging every subcommand.
created: 2026-08-25T06:03:43Z
author: claude-code/opus-5+refs-drift
scope:
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-6596aae0713c
seq: 0
schema: 4
version: 1
---

 Default 'ank status' makes ZERO network round trips: no ls-remote, no fetch, no push. 'ank status --remote' makes exactly one, 'ls-remote origin refs/ank/claims/*'. So the body's premise -- that status already reads the remote to name claims held elsewhere -- holds only under --remote, and is false on the default path the criterion also protects. The two clauses of the criterion are simultaneously satisfiable in exactly one place: name the refs drift on the --remote path, widening that single ls-remote pattern from refs/ank/claims/* to refs/ank/*, so the round trip count stays one and the default path stays network-free. Naming it unconditionally would add a round trip to the most-read verb, which the criterion forbids in the same sentence. Local half is free: git::ank_refs (for-each-ref refs/ank/) is already run three times per status, so a fourth is local and costs no network.
