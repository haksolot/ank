---
id: TASK-83d6eefdb36e
type: task
slug: claims-only-arbitrate-within-one-clone-level-1-i
title: "Claims only arbitrate within one clone: level 1 is unimplemented"
created: 2026-08-06T23:24:02Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/**
  - docs/**
blocked_by: []
done_criteria: |
  The spec section 7 level-1 description and the code agree: either the push of refs/ank/* on claim, renewal and done exists in crates/ank-cli/, or section 7 states that level 1 is unimplemented and names what a second clone can therefore do.
criteria_by: creator
proof:
  - type: test
    ref: "31360794469"
    criteria: b396da77f5b7
schema: 3
version: 4
---

Section 7 describes level 1 as "the same ref, pushed": the claim is pushed on
acquisition, on every TTL renewal through log, and again when done transforms it
into a completion ref, with an offline claim marked unsynchronised and warned
about. None of that exists. `ank init` adds a fetch refspec only
(init.rs:29,151-162), and there is no push, no fetch, no ls-remote anywhere in
crates/ - git.rs has no such primitive.

The consequence is exact, and worth stating in those terms rather than as a
generic gap:

- Two working trees of the same clone ARE arbitrated. `refs/ank/` is shared by
  every worktree of a repository, so the CAS in claim.rs:403-418 settles them.
  That is the setup the nominal execution model of section 7 recommends, and it
  works today.
- Two separate clones are NOT arbitrated. Each has its own refs/ank/claims/<id>
  namespace and neither ever learns of the other. Both agents claim the same
  task, both succeed, both work. Nothing detects it, not even later: check
  prunes on the default branch and would simply see two agents having done the
  same thing.

So "one piece of work, one holder" - the property ADR-bcf222a31525 rejected
per-branch claims to protect - holds per clone, not per repository, and nothing
says so out loud. An operator reading section 7 has every reason to believe
clones are covered, because section 7 lists clones first in the nominal model
("clones or git worktree").

Two honest exits, and this task deliberately does not pick:

1. Implement level 1. The primitive is already the right one - a non
   fast-forward push fails server-side, atomically, on every host - so the work
   is plumbing plus the offline degradation path, not design. Cost: a network
   round trip on claim, on every log, and on done.
2. Say in section 7 that only level 0 ships, that claims arbitrate within a
   clone including its worktrees, and that two clones are not covered. Then the
   nominal model sentence needs amending too, since it offers clones as an
   equal option.

Whichever is chosen, a criterion about the binary is tested through the binary:
if the push lands, the test drives two clones of one repository and asserts that
the second claim is refused with code 4.

## Notes carried in the log

Section 7 gains a section before the level list -- what ships, and what a second clone can therefore do -- naming the two cases in the terms this task used: worktrees of one clone are arbitrated by the shared refs/ank/, separate clones are not, both agents succeed and nothing detects it later either, because check prunes on the default branch where two agents having done the same work looks like two agents having done their work. Level 1 and level 2 headings now carry 'not implemented'.

Three other places said or implied the opposite and are corrected, because leaving them would have moved the lie rather than removed it. The nominal execution model offered 'clones or git worktree' as equals and now tells the reader to use worktrees while level 1 is unimplemented, with the reason. 'Why version coexists with git's CAS' claimed git's CAS protects between clones; it protects between working trees sharing a refs/ank/, and between clones once level 1 ships. The deferral table justified deferring level 2 by level 1 being enough, and now lists level 1 as deferred in its own right.

A sentence saying 'not implemented' goes stale in silence, so it gets a tripwire. Every git verb the tool may run passes the PLUMBING list, and a remote-aware claim needs push, fetch or ls-remote to appear there first; a negative test in git.rs asserts none of the three is allowed and names section 7 in the failure. Measured: adding push to the list turns it red with the message that names the file and what the same change owes it. It fails the day the feature lands, not while it is absent.

No code behaviour changed. 386 tests green, fmt clean, check exit 0.
