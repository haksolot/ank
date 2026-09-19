---
id: LOG-5ee91f217ea8
type: log
title: "released: The criterion is not met and the last file that keeps it from being met is not in this"
created: 2026-08-28T23:47:33Z
author: claude-code/opus-5+child-temp-dir
scope:
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/human.rs
about: TASK-02350943e2b1
seq: 3
schema: 4
version: 1
---

 task's scope.

Landed: the harness change and the signers lifetime, PR #330, merged as fbde0e6 on main, green on all three platforms. Measured three times, TMPDIR pointed at an empty directory, cargo test --workspace green: 16 ank-edit-* and 20 ank-signers-* before, 0 ank-signers-* and 1 ank-edit-* after.

The one that remains is crates/ank-tui/tests/entity.rs, which runs 'ank new task' with a fake $EDITOR through the spawn helper in crates/ank-tui/tests/terminal/mod.rs and lets the child inherit whatever the machine calls temporary. Attributed by running every test target of the workspace into its own empty directory, not by reading: crates/ank-cli/tests/cli.rs was 15 of the 16 and 13 of the signers, and no other target left anything. Same defect, same one-line fix, another crate's file. Filed as TASK-ec85b1561855 and recorded as a blocker here (PR #331, 4fc9779), so the next holder claims it in the right order instead of measuring their way back to this.

Releasing rather than calling it done: I did not meet the criterion I froze, and softening it to 'no ank-signers-*' would retire a true sentence about the workspace to make one task look finished.

Two things the body got half right, both worth having before the next claim. The eight ank-signers-* are not all from human.rs's own unit test: one is, five more come from Temp fixtures reaching signers_for_git through signature_state -- often only because their allowed-signers line carries no trailing newline -- and the last is written by human::tests::this_repositorys_own_ratifications_are_signed and cli::tests::the_foundation_is_resolved_before_the_verb_runs alike, both checking this repository under gpg.format=ssh. That second writer is in crates/ank-cli/src/cli.rs, outside this scope, so no cleanup written in a test could have made the count deterministic: whichever finished last decided whether a file remained. The copy is therefore cleaned up where it is written, by a guard whose last holder removes it. The lifetime moved and nothing else: the name stays content-addressed or git::batched loses the memo TASK-1b3d7b61dc8f added, and it gains the process id and a count so no other process or thread can have it pulled out mid-verification.
