---
id: TASK-eff5715bfd87
type: task
slug: the-quickstart-and-the-claims-page-say-what-init
title: The quickstart and the claims page say what init does with no origin, and a replay holds them to it
created: 2026-09-27T21:37:41Z
author: claude-code/opus-5.5+plan
status: in_progress
scope:
  - docs/quickstart.md
  - docs/claims.md
  - crates/ank-core/src/config.rs
  - docs/config-keys.md
  - crates/ank-cli/tests/doc_replay.rs
blocked_by: []
done_criteria: |
  docs/quickstart.md shows, in a block the doc replay runs, ank init in a repository with no origin printing the line that says the refs/ank/* refspec was not added and names ank init, and ank init run again after git remote add origin printing that the refspec was added; its prose no longer says init prints five effects and changes nothing on a re-run. docs/claims.md says the same case in one place and names the warning status and the signal check give while origin lacks the refspec. The peers.<name> row of docs/config-keys.md, regenerated from crates/ank-core/src/config.rs, says the value is a path and never a URL and names the per-reader override. cargo test --workspace passes, doc replay included, and the replay fails if the no-origin block's output is changed by one word.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 2
---

Measured on main after #510, with the binary built from the tree (the installed
ank 0.8.0 predates the change, so reading it would have said the page was
right):

    created .ank/entities
    ...
    pointer added to AGENTS.md
    refspec +refs/ank/*:refs/ank/* not added: no remote named origin yet, run this once it exists (ank init)

and, after `git remote add origin <url>`, a second `ank init` prints
`refspec added: +refs/ank/*:refs/ank/*`. docs/quickstart.md still says "the
last line is the one a repository with no origin does not print ... init
reports five effects instead of six ... re-running changes nothing either way".
Three statements, all three wrong now.

The replay did not catch it because the page's replayed block runs in a
repository that has an origin. That is the hole to close, and why the criterion
asks for the no-origin case in a replayed block rather than in prose: a
sentence the suite does not run is the sentence that drifted here
(ADR-2b62b9a1fe67).

config-keys.md is generated: edit the description in crates/ank-core/src/config.rs
and regenerate with the command in the file's header, never the page itself.
