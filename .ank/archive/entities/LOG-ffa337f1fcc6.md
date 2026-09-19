---
id: LOG-ffa337f1fcc6
type: log
title: Measured, not read. TMPDIR set to an empty directory, cargo test --workspace green (exit 0), then
created: 2026-08-28T18:25:41Z
author: claude-code/opus-5+tests-leave-nothing
scope:
  - crates/ank-cli/src/editor.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-daemon/src/stream.rs
  - crates/ank-tui/src/stream.rs
about: TASK-ac2ff41162c6
seq: 0
schema: 4
version: 1
---

 the directory listed.

Before: 16 ank-edit-*, 19 ank-signers-*, 3 ank-daemon-stream-*, 6 ank-tui-stream-*, one ank-it-* root.
After: 16 ank-edit-*, 20 ank-signers-*, one ank-it-* root. The two stream families are gone, and so are ank-git-*, ank-nogit-* and ank-common-*, which git.rs was leaving on any run that panicked.

Splitting the run tells where each family is born. cargo test --workspace --lib --bins, into its own empty directory, leaves only the nine stream directories and eight ank-signers-*. Everything else appears only once tests/ runs. So the task body's premise is half right: the stream families do come from cfg(test) blocks under src/ and are fixed here, but ank-edit-* comes from no unit test at all -- it is the ank binary itself, spawned by crates/ank-cli/tests/cli.rs, keeping the edited text on the failure paths as editor::kept promises the caller it will. ank-signers-* is written by signers_for_git in crates/ank-cli/src/human.rs, eight times from that file's own unit test and a dozen more from the same binary under the integration suite.

Neither of those two is reachable from this task's scope, and neither should be fixed the way this one was. Moving the editor's scratch file under a swept root would delete a user's kept text the next time any ank process ran, which is the opposite of what that path exists for. The fix for both is one line in the integration harness -- point the child's TMPDIR at the suite's own ank-it-* root -- plus removing the copy human.rs's unit test leaves. Filed as a follow-up rather than reached for.

The helper: three copies, the compact form crates/ank-tui/tests/terminal/mod.rs already carries, all on the ank-it- prefix so one sweep collects every root any of them left. In ank-cli it sits at module level in git.rs and is pub(crate), because a dozen other modules of that binary build fixtures on disk and one of them has to hold the copy; holding it where the rest can name it is the difference between one copy and a dozen.
