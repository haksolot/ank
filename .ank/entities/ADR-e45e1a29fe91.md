---
id: ADR-e45e1a29fe91
type: adr
slug: the-ank-directory-is-reached-only-through-the-cl
title: The .ank directory is reached only through the CLI, and the routes are the ones that dispatch
created: 2026-08-31T04:01:01Z
author: claude-code/opus-5+corpus
status: proposed
scope:
  - .ank/**
constraint: |
  Entities in .ank/ are read and written through the ank CLI only. Reading is ank show, ank find, ank context, ank scope, ank graph, ank status, ank review, ank check and the read form of ank log; writing is ank new, claim, log, done, release, attest, close, accept, amend, edit, config and read. No agent opens, greps, lists or edits a file under .ank/ directly. This constrains the agent, not the tool: a human with an editor keeps every power they had, and check remains what notices.
supersedes: ADR-01b6dd05f0db
schema: 4
version: 1
---

`.ank/` should be as opaque to an agent as `.git/` is. Nobody opens
`.git/refs/heads/main` to read a branch; they run `git rev-parse`. The reason is
not secrecy, it is that the tool knows things the file does not: the budget of
section 5, the truncation notice, the freeze anchored where the file's editor
cannot reach it, what is claimable and by whom.

That rule is ADR-01b6dd05f0db's and it is unchanged here. Only its two
enumerations move, because the binary has grown four routes since they were
written and an agent obeying the list obeyed a smaller CLI than the one it was
running.

## Measured, not read

In a throwaway corpus on ank 0.7.0 (50f4b39), 2026-08-31, counting the paths
`git status --porcelain -- .ank` reports after each verb, with `index.db`
ignored. Counting paths rather than hashing files is deliberate: a measurement
of this rule should not be the thing this rule forbids.

Wrote, each exiting 0:

    ank read <id>                     1 path
    ank amend <id> --scope <glob>     2 paths
    ank amend <id> --blocked-by <id>  2 paths
    ank edit <id> --title <t>         2 paths
    ank config default_branch main    1 path

Wrote nothing, each exiting 0: `ank scope`, `ank graph`, `ank status`,
`ank review`, `ank check`, `ank log <id>` in its read form, and the three the
predecessor already named -- `show`, `find`, `context`.

## What the two lists gain

**The writing side gains `amend`, `edit`, `config` and `read`.** The first three
are self-evidently writes. `read` is the interesting one: it is a reading act
that records the reading, so an agent told "reading is show, find and context"
and handed `ank read` had no way to place it. It is on the writing side because
the enumeration is about bytes.

`config` writes `.ank/config.yml`, which is not an entity. It is listed anyway,
because the sentence governs `.ank/` and not only the entities in it, and a
route that writes there and appears on neither list is exactly the gap this
successor exists to close.

**The reading side gains `scope`, `graph`, `status`, `review`, `check` and the
read form of `log`.** Every one was already the answer to a question an agent
would otherwise answer by opening a file: what constrains this path, what is
genuinely a root, who holds what, what awaits ratification, what is already
known to be wrong, and what previous holders tried.

## The enumeration is the part that goes stale, and it will again

This is a list of verbs inside a constraint, and the tree it describes moves. It
went stale exactly once per new route, silently, and what noticed was a drift
audit rather than anything mechanical. That is a real cost and it is accepted
here rather than engineered away: the alternative -- a constraint that says "the
verbs `ank help` prints" and names none -- is unreadable to the agent who most
needs it, which is the one meeting the rule for the first time in `ank context`.

The mitigation that costs nothing is a supersession like this one, filed by the
audit that finds it. The mitigation that would cost something -- `check`
comparing the two lists against the dispatch table -- is worth proposing on its
own and is not decided here.

## What the predecessor said that has since come true

Its closing paragraph named one act with no command: adding a `blocked_by` to a
task that already exists. `ank amend --blocked-by` does it today, measured above,
so the human-surface exception that paragraph carved out has closed. The rest of
its account stands: `ank new` writes a task that needs no hand finishing,
`ank attest` performs the one write section 3 permits after `done`,
`ank new adr --supersedes` writes the field nothing could write, and `accept`
performs the succession that field announces.

**This is still not the CLI becoming a gatekeeper.** ADR-6b3f19e08a24 says no
frozen field may rely on the CLI refusing a write, and nothing here changes it:
the freezes stay anchored by hash in artifacts the file's editor does not
control, and stay verifiable by anyone. What is constrained is the agent's
harness.

## What stands between this and a signature

ADR-3b6ba766a42e: `accept` refuses a supersession while any tracked file outside
`.ank/` still cites what it retires. Fourteen sites cite ADR-01b6dd05f0db today,
in eight files -- `CONTRIBUTING.md`, `crates/ank-cli/src/commands.rs`,
`crates/ank-cli/src/git.rs`, `crates/ank-cli/src/human.rs`,
`crates/ank-cli/src/paint.rs`, `crates/ank-cli/tests/cli.rs`,
`crates/ank-cli/tests/skill.rs` and `crates/ank-contract/src/verbs.rs`. Two of
them are inside string literals the binary prints, so re-pointing them changes
CLI output and the assertions over it.

Every one must name this document, or be dropped, before a ratification is
possible. The sweep is not performed here: the task that produced this document
is scoped to `.ank/entities/**` and those perimeters are held by other agents.

**Nothing is accepted by writing this.** It lands proposed and binds nobody. The
list an agent is served by `ank context` is still the predecessor's until a human
signs this one on the default branch.
