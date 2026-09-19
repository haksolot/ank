---
id: ADR-92b9cda9f6a9
type: adr
slug: git-is-a-hard-dependency
title: git is a hard dependency, and there is no mode without it
created: 2026-07-28T00:09:51Z
status: superseded
scope:
  - crates/ank-cli/**
constraint: |
  One claim mechanism only: the git refs refs/ank/claims/<id>. No fallback
  through file locks. The plumbing goes through the git binary and never
  through a library, and uses plumbing only (update-ref, rev-parse,
  verify-commit, hash-object, cat-file), never porcelain. Minimum git version
  2.34, checked at startup. git missing, too old, or a working directory
  outside a git repository all exit with code 9 and the exact command to run.
schema: 1
version: 5
---

A fallback through file locks would save only the claim — the one piece that is
useless on its own. Without git, Ank loses ratification (the signed commit, §8),
proofs of type `commit`, the recovery that stands in for a trash can and an undo
(§12), and the synchronisation refspec (§7). We would therefore maintain two
coordination mechanisms for a degraded mode in which nobody can work.

The principle "degrade, do not fail" (§2) is not weakened, it is made precise:
degradation covers services and the network — no remote, no daemon — never the
substrate. A level 0 with no remote stays fully functional, because a local git
ref update is already the atomic primitive the claim needs.

Code 9 is the right code rather than 1: an environment without git is not a
failure of the agent's task, it is an environment to repair.

Choosing the binary over a library (`gix`) does not follow from this decision, it
has its own and stronger reason: `accept` and `check` rest on signing. Producing
a signed commit and verifying it against `allowed_signers` is three lines with
`git commit -S` and `git verify-commit`, and a cryptographic project with a
library — for a result at best equivalent and at worst subtly different from what
the user will check by hand.

Restricting to plumbing is what makes the choice sustainable: porcelain has no
stability contract across versions, and parsing it would recreate exactly the
debt that resorting to the binary avoids. The 2.34 floor is the version that
introduces SSH signing and `gpg.ssh.allowedSignersFile`: below it, ratification
cannot work, and discovering that at the first `accept` would be late.
