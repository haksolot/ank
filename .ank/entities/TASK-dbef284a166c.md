---
id: TASK-dbef284a166c
type: task
slug: every-ratification-is-verified-by-gpg-on-every-r
title: Every ratification is verified by gpg on every run, and the verdict cannot change
created: 2026-08-20T18:31:47Z
author: claude-code/opus-5
status: open
scope:
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/tests/**
blocked_by: []
done_criteria: |
  A signature verdict already computed for a commit is not computed again: check run twice on an unchanged corpus starts gpg on the second run for no ratification it verified on the first, measured through the binary. The verdict is keyed on the commit and on the content of .ank/allowed_signers, so declaring a key changes every key that depends on it. A cache that cannot be read or that does not match its key is recomputed and never trusted, and no cached state can turn a signature check into a pass it did not earn: the four outcomes SPEC-199de7ac4730 lists are reported from a cache exactly as they are from git. cargo test --workspace and ank check stay green.
criteria_by: creator
schema: 3
version: 1
---

Measured on 2026-08-20, after TASK-1b3d7b61dc8f batched the calls: the whole
corpus is verified in one `rev-list`, and that one process costs 4.5 seconds.
It is not process startup any more, it is gpg doing real work, forty-three
times, inside git.

Nothing else in this repository can remove it. Batching is done. A signature is
a cryptographic operation and it takes what it takes.

**But it never needs doing twice.** A ratification commit is immutable, so the
question "is this object signed by a declared key" has an answer that cannot
change while the commit, the allowed-signers file and the local keyring stay as
they are. Two of those three are in the repository and hashable; the third is
the one that moves.

`.ank/index.db` is where it belongs. §6 calls the index derived, disposable and
gitignored, which is exactly the standing a cache may have: losing it costs a
recomputation and never a wrong answer.

**The danger is the whole subject, and it is why this was refused once already
(TASK-1b3d7b61dc8f left it out deliberately).** A cache that lies about a
signature is worse than one that is slow, because the thing being cached is the
one anchor §8 says holds when everything else can be forged. Two rules follow,
and they are not negotiable:

- **A miss is a recomputation, never a pass.** Unreadable, unparseable, keyed on
  something that no longer matches: all of them mean ask git, and none of them
  means assume the last answer.
- **No cached state may produce an outcome the run did not earn.** §8 lists four
  and they stay four. In particular `unchecked` -- signature present, no local
  public key -- must not be cached into `verified`, and importing a key must be
  able to change the answer. Keying on the keyring is not possible; what is
  possible is that a stale `unchecked` is the safe direction, since it
  under-claims. Say so where the key is built, and give the reader a way to
  drop the cache.

The test that matters is not the timing one. It is that a corpus whose
`allowed_signers` changed, or whose ratification commit was replaced, reports
the new verdict and not the old.
