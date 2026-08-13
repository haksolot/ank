---
id: TASK-03eaa26bddd1
type: task
slug: nothing-compares-a-ratified-adr-against-its-anch
title: Nothing compares a ratified ADR against its anchor
created: 2026-08-01T05:07:53Z
author: seanl@sean-laptop
status: done
scope:
  - crates/ank-cli/src/human.rs
  - docs/ank-spec-v1.1.md
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  check compares an accepted ADR's constraint and scope against its ratification anchor and reports a divergence, and an ADR reported as altered is no longer injected into context, with the warning the specification requires. What ratified holds is settled first, in writing, because the specification names the ratification commit and the code stores a hash of the constraint: a commit cannot contain its own identifier, so the two cannot both stand. Whichever wins, the seven ADR already anchored carry the settled form, and a test edits a ratified constraint and watches the finding appear.
criteria_by: creator
verify: [cargo-test, fmt-check, check-repo]
proof:
  - type: test
    ref: local/7c8d9a18a503@fd559a0
    tree: scope/4db60f169776
    criteria: 195b61e84aae
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: local/e3b0c44298fc@fd559a0
    tree: scope/4db60f169776
    criteria: 195b61e84aae
    verifier: fmt-check@5ca6d10bcd55
  - type: test
    ref: local/a9b241b045be@fd559a0
    tree: scope/4db60f169776
    criteria: 195b61e84aae
    verifier: check-repo@5734e9cf9d3d
schema: 3
version: 6
---

Found the moment the corpus first carried an anchor. TASK-313165e55d81 ratified
the seven bootstrap ADRs, and the criterion it froze asks for more than the
anchor: *the freeze is verified against the signed commit and not against the
file, so editing a constraint in place is visible as a divergence*. Nothing
verifies it. `check_adr` (human.rs:575) walks the succession chain both ways,
reports an accepted ADR with no anchor, and faults an empty constraint -- and
never calls `ratification_anchor`. The **altered** state of section 3 exists in
the specification and in no line of code. It could not be noticed before,
because with no ADR anchored there was nothing that could diverge.

Underneath sits a question this task has to settle before it can write the
comparison, and it is not a detail of naming.

The specification (line 221) documents `ratified` as *the signed ratification
commit*, and section 3 has `check` compare the current state against the hash
that commit records. That is the shape ADR-6b3f19e08a24 asks for: the anchor
lives in an artifact the file's editor does not control. The code instead
stores the hash of `constraint` + `scope` in `ratified`, and records no commit
identifier anywhere in the file. With no SHA to start from -- and `git log
--grep` being the porcelain ADR-b8884edcebe3 rules out -- there is no plumbing
path from an ADR to the commit that ratified it. The anchor can only ever be
compared against itself, and whoever edits a constraint can recompute it.

But the specification's literal form is not implementable as written: `accept`
produces one commit containing the ADR file, and a commit cannot contain its own
identifier. Writing the SHA into the file after the fact needs a second commit,
which section 12 rules out for the same reason it insists on one -- the window
in which the two disagree.

So one of the two moves, and the choice is a decision rather than a fix. Three
readings, and each has a real cost:

- **The hash stands, and the specification is wrong.** Honest about the threat
  model of section 1 -- drift, not an adversary -- and it does catch a
  constraint edited without its anchor, which is the accidental case. It gives
  up on the deliberate one, and ADR-6b3f19e08a24 is written in stronger terms
  than that.
- **`ratified` names the commit, written by a second commit.** Verifiable for
  real, at the cost of the atomicity section 12 argues for.
- **The anchor moves out of the file entirely**, into a git ref the way claims
  already live in `refs/ank/claims/<id>`. The precedent exists in this codebase
  and is the same argument -- state the file's editor does not control -- but it
  is a format decision and costs an ADR.

Whichever wins, the seven ADR ratified on 2026-08-01 carry whatever `accept`
wrote that day, and they have to end up carrying the settled form. Nothing is
pushed yet, so undoing those seven local commits is still the cheapest
migration; that stops being true the moment they leave this machine.

## Notes carried in the log

rev-list and cat-file are plumbing -- `git log` is the porcelain, not `rev-list` -- so ADR-b8884edcebe3 does not stand in the way, and its allowed list was already extended once in revision e. Verified by hand before choosing: rev-list --max-count=1 HEAD -- <path> finds 22e3663 for ADR-63b59c5c26f7, cat-file reads `constraint+scope: 986b639a642e` out of the message, and the signature is there.

That beats the git ref option for a reason worth writing down, because I proposed the ref first and was wrong: refs/ank/* does not travel with git push by default. Claims tolerate that -- they are local, ephemeral locks. A ratification is meant to be verifiable by anyone, on any clone, years later. Putting the proof in the one place that does not follow the repository is the opposite of what ADR-6b3f19e08a24 asks.

It also beats the second-commit option by needing no migration: the seven ADR ratified today already carry exactly this form, file and commit message agreeing on the same hash.

Consequence for the specification: line 221 documents `ratified` as the signed ratification commit, and that comment becomes wrong rather than aspirational. It has to say the field holds the constraint+scope hash, and describe how the commit is found.

One state has to stay distinct from `altered`: a shallow clone or a rewritten history finds no ratification commit at all. That is `cannot verify`, and reporting it as a divergence would be worse than not checking, because it would train people to ignore the finding.

What `ratified` still does is say the ADR claims to be ratified, which is what sends check looking for a commit at all. That is the whole of its remaining job, and the specification now says so instead of describing it as the commit itself.

rev-list --full-history, and not the default. Path simplification exists to explain a tree's final state and is free to drop a commit a merge made redundant; dropping the ratification commit would report a perfectly frozen ADR as unverifiable. The failure would appear only after a merge, which is the worst moment to discover it.

Unverifiable is a third state and not a soft Altered. A shallow clone, a rewritten history, a corpus moved between repositories -- none of them is a broken freeze, and a check that cries divergence over a shallow clone is a check people learn to ignore. Signal, not fault, and the message says cannot be verified.

git.rs carries a PLUMBING allow-list enforced by a debug_assert, which caught rev-list before any test did. ADR-b8884edcebe3 is not only written down, it runs.

Cost measured rather than assumed. Naive, check went from 0.9s to 4.3s: the question is asked once per ADR and again for every task that ADR bears on, which is hundreds of git spawns for an answer that cannot change under a running process. Memoised on (root, id) it is 14 spawns and 1.33s steady state, of which about 90ms is process start. The memo is safe for the reason it is fast: git history is immutable for the life of the invocation, and the test that alters a file between two freeze_state calls proves the key is the git question and not the file.

Verified on the real corpus through the rebuilt binary: the seven ADR ratified earlier today all read Intact, which is the end-to-end proof that accept and check agree on what the anchor is.
