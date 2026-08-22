---
id: TASK-49fce8b49d00
type: task
slug: init-at-creates-a-detached-corpus-and-declares-i
title: init --at creates a detached corpus and declares it, config --user edits the declaration
created: 2026-08-21T17:57:49Z
author: claude-code/opus-5
status: done
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/repo.rs
  - crates/ank-contract/src/verbs.rs
blocked_by: [TASK-88bff140d416]
done_criteria: |
  ank init --at <path> creates a corpus at that path, outside the working tree and never inside it, and writes the declaration keyed on this repository's identity, so that a verb run afterwards in the same tree resolves it with no flag and the working tree gains no file. ank config gains a --user scope that reads and writes the declaration with the discipline the repository file already has: a closed key set, an unknown key refused by name with the set it knows, comments and key order surviving a write byte for byte outside the line named, no default materialised, and a write that would produce an unparsable file refused with the file left as it was. A test drives the whole route through the binary: init --at, then claim, log and done with a commit proof on a task whose scope names a file of the code, asserting the refs land in the corpus repository and that git for-each-ref in the code repository lists no ank ref at all. SPEC-4eff92fd80ce lists both surfaces. cargo test is green, cargo fmt --check passes, and ank check reports no fault.
criteria_by: creator
proof:
  - type: commit
    ref: ece7611
    criteria: 2bc1c79fa9d7
    via: submitted
schema: 3
version: 4
---

The writing half. TASK-88bff140d416 makes a declaration resolvable; this makes
one without a text editor and a printed sha.

**One gesture, because the gesture is one thing.** Creating a corpus for a
repository whose tree must stay untouched is create, initialise, and declare,
and a design where the user performs three steps and looks up their own root
commit between the second and the third is a design nobody will run twice.
`--at` is what makes `init` the whole act.

**`ank config --user` exists for the reason `ank config` exists.** ADR-01b6dd05f0db
closed `.ank/` to agents and stopped at the configuration, leaving errors that
told their caller to open a file the same tool forbade them to open, and
SPEC-4eff92fd80ce records `ank config` as the verb those errors name instead.
The declaration map is outside `.ank/`, so nothing forbids opening it, and that
is precisely why the surgery discipline has to be repeated rather than assumed:
the file is small, hand-edited, and the one place a stale entry makes a corpus
vanish.

**The end-to-end test is the point of the task**, more than either surface.
Everything before this proves a piece; this proves the promise. A task claimed,
logged and finished, with the proof anchored, and `git for-each-ref refs/ank`
in the code repository printing nothing. That last assertion is the requirement
in its original words, leave no traces, turned into something that fails.

**What must not be built.** `--at` never writes into the working tree, not a
pointer, not a gitignore line, not a comment: the moment it does, the promise
is gone and the design is the committed pointer this plan refused. And `--at`
refuses a path inside the current work tree rather than accepting a detachment
that is not one.
