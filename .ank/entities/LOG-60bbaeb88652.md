---
id: LOG-60bbaeb88652
type: log
title: "reproduced with ank 0.8.0 (298c01a) in a scratch repo: git init -b main; ank init -> exit 0, 5"
created: 2026-09-27T17:32:46Z
author: claude-code/opus-5.5+623d
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/**
  - crates/ank-cli/tests/init_refspec.rs
about: TASK-623d80886c2f
seq: 2
schema: 4
version: 1
---

 lines, none mention the refspec; git remote add origin <bare> -> exit 0; git config --get-all remote.origin.fetch -> 1 line, +refs/heads/*:refs/remotes/origin/* only; ank status: 0 lines matching refspec; ank check: exit 0, 0 lines matching refspec. Hypothesis: ensure_refspec returns Ok(false) for 'no URL' and 'already present' alike, so Report cannot tell them apart, and neither status nor check reads remote.origin.fetch at all (grep refspec in status.rs/human.rs: only a doc comment).
