---
id: TASK-97437c25ddda
type: task
slug: ten-test-fixtures-inherit-the-developer-s-commit
title: Ten test fixtures inherit the developer's commit.gpgsign
created: 2026-08-09T17:52:06Z
author: claude-code@ank
status: done
scope:
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  No test in crates/ank-cli/tests/ depends on the git configuration of the machine running it. Demonstrated by running the full suite twice on a machine whose global git config sets commit.gpgsign true: once with the gpg agent locked, and once with GIT_CONFIG_GLOBAL and GIT_CONFIG_SYSTEM pointed at an empty file. Both are green and produce the same result. Fixtures that need a signature keep making their own through enable_signing. cargo test, cargo fmt --check and ank check stay green.
criteria_by: creator
proof:
  - type: test
    ref: "31355601158"
    criteria: 695c419c9595
schema: 2
version: 4
---

Found while executing TASK-2eefcdd80124, and confirmed on a clean worktree of main so it is not a regression of that work.

Ten integration tests seed their fixture with `r.git(&["commit", "-qm", "seed"])`, with no `-c commit.gpgsign=false`. The fixture sets user.email, user.name and core.autocrlf, and stops there, so `commit.gpgsign` is inherited from whatever the developer running the suite has in their global git config. On a machine that signs by default, every one of those tests depends on a gpg agent being unlocked.

Measured on 2026-08-09: the suite was green earlier in the session and later failed ten tests with `gpg: signing failed: Délai d'attente dépassé`, the pinentry having timed out. Re-running one of them on a clean worktree of main reproduced it, which is what rules out a regression. Running the whole suite with GIT_CONFIG_GLOBAL and GIT_CONFIG_SYSTEM pointed at an empty file turned it green again, which is what identifies the cause.

The repository already holds the principle. `enable_signing` exists precisely so the tests that need a signature make their own, and its doc comment says a signature configured from the developer's own global git config would make a test pass here and nowhere else. The seed commits were simply never brought under that rule.

Two directions, and the second looks better. Adding the flag to each seed commit fixes the ten and leaves the eleventh to be written wrong. Neutralising the environment once in the fixture -- so no test can inherit global git configuration at all -- fixes the class. `Repo::new` already sets three config keys; disabling signing there, or pointing the fixture at an isolated GIT_CONFIG_GLOBAL, is the same one-line habit applied where it holds.

Worth noting the failure mode is asymmetric. On CI, where nothing signs, these pass forever and the defect is invisible. It only bites a contributor whose machine signs by default -- which is to say, the maintainer's.

## Log
- 2026-08-10T04:27:11Z claude-code@ank — Neutralised the environment once rather than adding the flag to the two seed commits that lacked it. Every process the suite spawns is now built by one function, spawn, which points GIT_CONFIG_GLOBAL and GIT_CONFIG_SYSTEM at a file the suite writes itself; git_command and nk_command are its two faces. The binary gets it too, not only git: ank shells out to git on its own account.

The flag is gone from all twenty-four sites that carried it, so the isolation is load-bearing rather than doubled by a habit, and every test in the file exercises it. enable_signing is untouched: accept commits with -S, which outranks any configuration, so the fixtures that need a real signature still make their own.

Measured, not asserted. The defect reproduces deterministically with GIT_CONFIG_GLOBAL set to a config that signs: nine tests failed, all at the two flagless seeds, all with 'gpg failed to sign the data'. Both new tests bite. Removing the two env lines turns the isolation test red even under an empty ambient config -- that is, on CI, where the old defect was invisible -- because it asserts the origin git reports and not the value alone. Adding a raw Command::new elsewhere in the file turns the sweep red with 'a process is spawned outside spawn'. The sweep's reach is the literal form in this one file; skill.rs spawns the binary twice, for help and --version, and both answer before startup runs.

The full workspace suite is green in all three conditions and produces the same result: signing on with the key unusable, both config levels empty, and the machine's own config untouched. 385 tests.

One obstacle worth recording: restoring the file from a backup copy carried the backup's older mtime, so cargo saw nothing to rebuild and a whole run measured a stale binary. Touch the file after any restore.

Found on the way, and filed separately: with gpg.program pointing at a binary that cannot run, ank judges this repository's own ratifications Absent -- 'not signed' -- where the only correct answers are Trusted or Unchecked. A contributor on a full clone without GnuPG would be told the corpus is unratified.
- 2026-08-10T04:31:51Z claude-code@ank — done, proof test:31355601158
