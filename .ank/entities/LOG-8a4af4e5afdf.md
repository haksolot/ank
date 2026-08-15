---
id: LOG-8a4af4e5afdf
type: log
title: Every throwaway repository now turns signing off in its own config at creation -- commit.gpgsign
created: 2026-08-10T04:59:28Z
author: claude-code@ank
scope:
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/init.rs
about: TASK-40a972e98a9a
seq: 0
schema: 3
version: 1
---

 and tag.gpgsign both -- and all nine per-call -c commit.gpgsign=false repetitions are gone with it. Eight fixture creation points: claim.rs, context.rs, done.rs, git.rs (three: the Temp, the worktree fixture, the nested inner repo), human.rs, init.rs, plus the stamp fixture in tests/cli.rs, which is the one repository in that file that commits without being a Repo.
