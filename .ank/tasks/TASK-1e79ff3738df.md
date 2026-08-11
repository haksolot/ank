---
id: TASK-1e79ff3738df
type: task
slug: check-names-where-a-dead-scope-went-and-the-comm
title: check names where a dead scope went, and the command that repairs it
created: 2026-08-11T22:28:53Z
author: claude-code@sean-laptop
status: open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: [TASK-65017ea098f2]
done_criteria: |
  When a scope is dead and git recorded a rename of that path, ank check prints,
  under the existing finding, the new path and the commit that moved it, then the
  exact command that repairs the entity: ank amend for a task or an unratified ADR,
  and a supersession for an accepted ADR, whose scope amend refuses to touch.
  
  The walk uses rev-list and diff-tree and no porcelain. It runs only when a scope
  is already dead, and where there is no repository it is skipped in silence rather
  than warned about.
  
  A dead scope git cannot explain -- a deletion, or a move under the similarity
  threshold -- produces the finding exactly as it does today and no proposal, and
  no wording suggests the file was deleted.
  
  Asserted through the binary in crates/ank-cli/tests/cli.rs: a fixture repository
  where a file named literally in an ADR scope is renamed, and one where it is
  deleted.
criteria_by: creator
schema: 2
version: 1
---

Implements ADR-97beaf55e73a. Blocked on the lazy-git task because the skip-in-
silence clause needs a binary that can already run without a repository; doing it
first would mean writing that condition twice.

The detection exists and already goes red — `check_scope_alive` reports a scope
matching no file, as a fault for an ADR or a finished task. What is added is the
answer to the question the reader has at that moment, and nothing else. Do not
touch when the finding fires or what severity it carries.

343 of the 462 scope entries in this corpus name a single file with no wildcard,
so the case being handled is the common one, not an edge.

`git log` is porcelain and stays forbidden. `rev-list` for the last commit
touching the dead path, then `diff-tree -M --name-status` on that commit for an
`R` entry. Both are listed in ADR-9307e5d214a7's enumeration.

The ADR branch of the proposal is the part most likely to be got wrong. `amend`
refuses the scope of an accepted ADR, because that scope is hashed into the
ratification commit — so proposing `ank amend` there would name a command that
fails on the spot, which the error style forbids everywhere. Say supersession, and
say it in a way that does not read as an instruction to supersede lightly.

Rename detection is a similarity heuristic and answers sometimes. Absence of a
proposal must never read as evidence the file was deleted; the wording has to
leave the reader in the same position they are in today, no worse and no more
confident.

Cost: the walk runs per dead scope, and a healthy corpus has none, so the common
path pays nothing. Do not hoist it above the dead-scope test to make the code
tidier.
