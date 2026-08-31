---
id: LOG-8553c20beeca
type: log
title: Reproduced before the fix, through this worktree's target/debug/ank. Reader home holds corpora.yml
created: 2026-08-31T03:32:54Z
author: claude-code/opus-5+init
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/init_at.rs
about: TASK-0dd151b02854
seq: 0
schema: 4
version: 1
---

 at 'schema: 2'; source repo has one root commit; target is a fresh 'git init' holding only .git. Run: ank init --at <target>. Exit 1, message 'schema 2, this binary reads 1: the file is newer than this ank', corpora.yml left byte-identical -- the refusal is correct. The target, before: .git alone. After: .ank/config.yml, .ank/entities/, .ank/log/, AGENTS.md, .gitattributes, .gitignore, and 'fetch = +refs/ank/*:refs/ank/*' written into the target's own .git/config. Six artefacts and one foreign config mutation, with nothing declared pointing at any of them. Two more than LOG-1bf8280879af recorded: .gitattributes and the refspec.
