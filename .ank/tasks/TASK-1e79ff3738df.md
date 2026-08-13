---
id: TASK-1e79ff3738df
type: task
slug: check-names-where-a-dead-scope-went-and-the-comm
title: check names where a dead scope went, and the command that repairs it
created: 2026-08-11T22:28:53Z
author: claude-code@sean-laptop
status: in_progress
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
version: 4
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

## Log
- 2026-08-13T04:08:50Z claude-agent-b — Read the four branches the proposal has to serve and found one the criterion does not resolve. check_scope_alive reports structural death (a scope matching no file) two ways: a signal for an open or in_progress task, a fault for an ADR or a finished task. The criterion says the repair is 'ank amend for a task', but amend refuses a done or closed task outright (code 7, 'its plan is settled') -- and those are exactly the tasks the fault branch covers. Naming ank amend there would name a command that refuses on the spot, which ADR-97be forbids in the same breath as the accepted-ADR case. Resolution: the walk runs on both branches, since both are the same structural condition (spec section 11, and check_scope_alive's own doc comment says so), which is what makes 'ank amend for a task' real -- an open task is the one task amend accepts. A done or closed task gets the rename named and no repair command, because there is none. Accepted or superseded ADR gets the supersession, worded as amend's own refusal already words it.
- 2026-08-13T04:22:28Z claude-agent-b — Implemented and falsified. git::rename_of is rev-list -1 HEAD -- <path> then diff-tree -M -r -z --name-status --no-commit-id on that commit; diff-tree was missing from the PLUMBING list and is added with the criterion ADR-9307 asks for. Deliberately no --full-history here, unlike ratification_at: there the target is one commit identified by subject and simplification would lose the anchor, here the target is the change itself, and --full-history would keep the merges that merely carried it -- a merge is a commit diff-tree prints nothing for, so asking for more history would answer less often. The walk is skipped for a glob carrying a wildcard: git has no answer for where src/** went, and the single-file entry is the common case. Finding gained a note field rather than folding the proposal into message, because review filters findings on the opening of message and a note carries no severity of its own. Rendered with the LAST and CLEAR glyphs already declared in section 4, so no character is added to the structure alphabet. Falsified by making the walk return nothing: the five tests that assert it fail, and outside_a_repository_the_rename_walk_is_skipped_without_a_word stays green, which is what it should do. The renamed-ADR test spends the proposed command instead of matching it -- split back into argv, handed to the binary, must exit 0 and leave the scope alive.
