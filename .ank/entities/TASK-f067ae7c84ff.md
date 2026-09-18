---
id: TASK-f067ae7c84ff
type: task
slug: ank-init-writes-remote-origin-fetch-and-git-remo
title: ank init writes remote.origin.fetch, and git remote add origin then refuses the command status tells you to run
created: 2026-09-18T10:13:59Z
author: claude-code/opus-5+followups
status: open
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  Through the binary, in a scratch repository with no remote: after ank init, git remote add origin <url> succeeds, and git config --get-all remote.origin.fetch then carries the +refs/ank/* refspec ank needs together with the +refs/heads/* refspec a plain git fetch needs, so git fetch origin brings branches. ank status --remote keeps naming a command that works from the state ank init leaves. A test drives that whole sequence through the binary and asserts on the refspecs, not only on the exit code.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 1
---

Found while writing TASK-7eccd56c8c30 and reported there. `init.rs` adds
`remote.origin.fetch` with `git config --add` before any remote exists, which is
what makes the ank refspec survive a later `git remote add`. But git reads that
key as proof the remote is already configured, so `git remote add origin <url>`
exits 3 with "remote origin already exists" -- and that is verbatim the command
`ank status --remote` prints as the repair (status.rs:984).

`git remote set-url origin <url>` gets past it, and is the workaround an operator
finds on their own, but it leaves no `+refs/heads/*` fetch refspec behind, so a
plain `git fetch` brings no branches and the repository looks broken in a second,
quieter way. Neither route is written down anywhere, which is why the guide left
both unnamed.

The fix belongs in init.rs: either write the key in a form `git remote add`
tolerates, or have ank offer the one command that reaches the working state.
Whichever is chosen, status must name a command that works.
