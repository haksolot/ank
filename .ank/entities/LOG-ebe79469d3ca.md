---
id: LOG-ebe79469d3ca
type: log
title: Implemented and measured, git 2.47.3. ensure_refspec now asks one question, git config --get
created: 2026-09-18T13:37:05Z
author: claude-code/opus-5+origin-refspec
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/**
about: TASK-0878e19675f4
seq: 3
schema: 4
version: 1
---

 remote.origin.url, and writes remote.origin.fetch only when it answers; the deferred file, the includeIf and the git-version branch are gone, along with the two-question --get-regexp. Lab, ank init in a repository with no remote: .git/config carries no origin section and no includeIf, .git/ank-origin.config does not exist, git remote add upstream then git remote add origin both exit 0, and the next ank init prints 'refspec added' and leaves remote.origin.fetch carrying +refs/heads/* and +refs/ank/*, with a plain git fetch origin bringing a branch and an ank ref. Four callers moved with it: init_origin.rs gained a_remote_added_after_init_leaves_origin_free and its first test now expects no refspec until origin exists, cli.rs init_writes_the_same_refspec_this_suite_assumes sets the URL before the second init, init_at.rs does the same for a detached corpus, and golden-json/init.json was re-blessed because 'added' no longer carries remote.origin.fetch for a repository with no origin. What this costs is one extra ank init after the remote is added, which the guide already prescribes; what it buys is that git remote add origin <url>, the command status --remote prints, works from every state init can leave behind. cargo test --workspace exit 0, 368 tests; cargo fmt --check exit 0.
