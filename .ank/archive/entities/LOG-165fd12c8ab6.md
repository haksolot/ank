---
id: LOG-165fd12c8ab6
type: log
title: "Reproduced through the PATH binary on git 2.47.3, repo with one commit and no remote: after ank"
created: 2026-09-18T11:44:51Z
author: claude-code/opus-5+init-origin
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/tests/**
about: TASK-f067ae7c84ff
seq: 1
schema: 4
version: 1
---

 init, git config --get-all remote.origin.fetch = [+refs/ank/*:refs/ank/*]; ank status --remote warned 'no remote named origin ... (git remote add origin <url>)'; git remote add origin <bare> exited 3 'error: remote origin already exists.'; git remote set-url exited 0 and left fetch = [+refs/ank/*] only, a git fetch then brought 0 branches. Minimised without ank: git config --add remote.origin.fetch X; git remote add origin -> exit 3. A plain include.path of a file holding the key -> exit 3 too (included keys take the including file's local scope). includeIf hasconfig:remote.*.url:** -> remote add exit 0, fetch = [+refs/ank/*, +refs/heads/*]. Refuted edge: a second remote (upstream) added before origin activates the include and origin add exits 3 again; so init writes nothing when remotes exist but none is origin, and a re-run of init after origin exists adds it directly. Fix: deferred file .git/ank-origin.config under that includeIf when no remote has a URL and git >= 2.36; direct --add otherwise. Regression tests/init_origin.rs (3 tests) run with init.rs reverted: 2 red, the status-named command failing with 'remote origin already exists.'; green with the fix. Two shared tests read .git/config as text (cli.rs init_writes_the_same_refspec_this_suite_assumes, init_at.rs an_accepted_declaration...) and now ask git for the resolved key after setting a URL. cargo test --workspace --no-fail-fast: 0 failures.
