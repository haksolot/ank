---
id: TASK-1dbb6e7843f1
type: task
slug: a-ratification-that-could-not-be-committed-leave
title: A ratification that could not be committed leaves nothing behind
created: 2026-08-18T18:34:43Z
author: claude-code/opus-5
status: done
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  When the ratification commit fails, accept leaves the entity byte-for-byte as it found it: status, ratified and verified unchanged, and the exit code and message name what failed. Tested through the binary with signing configured to a key accept cannot use, asserting the file is identical before and after and that a second accept -- once signing works -- still ratifies. cargo test --workspace and ank check stay green.
criteria_by: creator
proof:
  - type: commit
    ref: 30cb83076ae59fac2f01203ad0dcb045a49e48e6
    criteria: c49b57954805
    via: submitted
schema: 3
version: 3
---

Measured on 2026-08-18, ratifying ADR-768374fe6076 on this machine, where the
ratification key is protected by a passphrase nothing supplied.

`accept` exited 9 with git's message -- `Load key ".../gh_signing": incorrect
passphrase supplied to decrypt private key?`, `fatal: failed to write commit
object`. No commit was made. The entity, however, had already been written: it
came out `status: accepted` carrying `ratified: ab3b9ed512ae`, the correct
constraint+scope anchor for a ratification that does not exist. `verified` was
never written, so the entity did not even record who supposedly ran it.

`check` names the state exactly -- "ratified, but no ratification commit is
reachable: the freeze cannot be verified" -- and names it as a signal, which is
right for the bootstrap case that clause was written for and generous here: this
corpus was claiming a binding decision anchored to nothing.

What makes it more than a cosmetic ordering bug is that ank cannot then repair
it. `accept` over an existing anchor refuses with exit 7 and hints at
`ank new adr --supersedes`, which is correct for a decision genuinely ratified
and wrong here: it would succeed a ratification that never happened. The two
tests around this, `accept_ratifies_an_adr_accepted_by_hand_and_never_anchored`
and `accept_refuses_an_adr_that_already_carries_an_anchor`, are each right about
the case they cover, and the failed-commit state falls between them -- accepted,
anchored, and unreachable. It was repaired here through `ank edit`, which is a
route out but not one the failure names.

The fix is ordering: the commit is the act, so nothing about the entity is
written until git has taken it. That is the same argument ADR-af53d0b62a5c makes
for a write whose only product is a ref, and the same one the refusal above rests
on -- a refusal "that had rewritten either would be the laundering it exists to
prevent", in the words of the test. A failure should be held to what the refusal
already is.
