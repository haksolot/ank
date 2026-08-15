---
id: LOG-0cedd243da23
type: log
title: Extended the scope with context.rs and status.rs. The declared scope was narrower than the frozen
created: 2026-08-12T22:19:47Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/status.rs
  - docs/ank-spec-v1.1.md
about: TASK-65017ea098f2
seq: 2
schema: 3
version: 1
---

 criterion needs: it named cli.rs, repo.rs, git.rs, human.rs and tests/cli.rs, but removing the startup gate exposes every verb at once, and context.rs and status.rs are the two readers that call git with '?' -- context::coordination through ank_refs, status through current_branch and origin_head. Left alone they would go from a clean 'not inside a git repository -> git init' to a raw git failure, still code 9. The criterion's first list does not name them, so this is not required by it; ADR-9307's constraint is what requires it -- 'a verb that only reads or writes the corpus requires none of it and answers on the files alone' -- and context is the verb the loop opens with. The criterion is untouched. The other verbs it does name -- show, find, graph, scope, new, amend -- call no git at all, so removing the gate is the whole change for them.
