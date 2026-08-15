---
id: LOG-799cdbbc4749
type: log
title: accept now performs the succession, and turned out not to be performing the acceptance either.
created: 2026-08-01T01:51:10Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/human.rs
about: TASK-24b9456d8ec7
seq: 0
schema: 3
version: 1
---

 check_transition(Accepted) was called and the assignment that follows it never written, so accept wrote ratified onto an ADR still marked proposed. Both tests that existed asserted refusals and returned before the commit, so nothing ever ran the path -- the exact shape CLAUDE.md warns about, and the fix needed a test that signs for real. Fixture signs with SSH rather than GPG: ssh-keygen ships beside git on all three platforms and needs no agent, keyring or passphrase prompt. Everything that can refuse refuses before anything is written, because a half-performed succession is a corpus the ratification commit would then make authoritative and accept has no second pass. Target written first, then the superseder: between the two writes the corpus holds a target marked superseded whose superseder is still proposed, which check_adr calls clean in both directions; the reverse order leaves an accepted superseder over an unmarked target, which is precisely the fault. One commit for both paths -- two would leave a window in which history says both constraints bind. A target that is not accepted is a 7 and not the 6 of an illegal transition: nothing transitions, a prerequisite is unmet, and the hint names ank accept on the target when it is merely proposed.
