---
id: TASK-97437c25ddda
type: task
slug: ten-test-fixtures-inherit-the-developer-s-commit
title: Ten test fixtures inherit the developer's commit.gpgsign
created: 2026-08-09T17:52:06Z
author: claude-code@ank
status: open
scope:
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  No test in crates/ank-cli/tests/ depends on the git configuration of the machine running it. Demonstrated by running the full suite twice on a machine whose global git config sets commit.gpgsign true: once with the gpg agent locked, and once with GIT_CONFIG_GLOBAL and GIT_CONFIG_SYSTEM pointed at an empty file. Both are green and produce the same result. Fixtures that need a signature keep making their own through enable_signing. cargo test, cargo fmt --check and ank check stay green.
criteria_by: creator
schema: 2
version: 1
---

Found while executing TASK-2eefcdd80124, and confirmed on a clean worktree of main so it is not a regression of that work.

Ten integration tests seed their fixture with `r.git(&["commit", "-qm", "seed"])`, with no `-c commit.gpgsign=false`. The fixture sets user.email, user.name and core.autocrlf, and stops there, so `commit.gpgsign` is inherited from whatever the developer running the suite has in their global git config. On a machine that signs by default, every one of those tests depends on a gpg agent being unlocked.

Measured on 2026-08-09: the suite was green earlier in the session and later failed ten tests with `gpg: signing failed: Délai d'attente dépassé`, the pinentry having timed out. Re-running one of them on a clean worktree of main reproduced it, which is what rules out a regression. Running the whole suite with GIT_CONFIG_GLOBAL and GIT_CONFIG_SYSTEM pointed at an empty file turned it green again, which is what identifies the cause.

The repository already holds the principle. `enable_signing` exists precisely so the tests that need a signature make their own, and its doc comment says a signature configured from the developer's own global git config would make a test pass here and nowhere else. The seed commits were simply never brought under that rule.

Two directions, and the second looks better. Adding the flag to each seed commit fixes the ten and leaves the eleventh to be written wrong. Neutralising the environment once in the fixture -- so no test can inherit global git configuration at all -- fixes the class. `Repo::new` already sets three config keys; disabling signing there, or pointing the fixture at an isolated GIT_CONFIG_GLOBAL, is the same one-line habit applied where it holds.

Worth noting the failure mode is asymmetric. On CI, where nothing signs, these pass forever and the defect is invisible. It only bites a contributor whose machine signs by default -- which is to say, the maintainer's.
