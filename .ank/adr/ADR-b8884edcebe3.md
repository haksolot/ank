---
id: ADR-b8884edcebe3
type: adr
slug: git-plumbing-by-criterion
title: git stays a hard dependency, and the allowed plumbing is defined by a criterion
created: 2026-07-29T10:40:00Z
status: superseded
scope:
  - crates/ank-cli/**
constraint: |
  One claim mechanism only: the git refs refs/ank/claims/<id>. No fallback
  through file locks. The plumbing goes through the git binary and never
  through a library. A git command is usable only if its output is stable by
  contract across versions: update-ref, for-each-ref, symbolic-ref,
  rev-parse, merge-base, verify-commit, hash-object, cat-file. Never
  porcelain. Minimum git version 2.34, checked at startup. git missing, too
  old, a working directory outside a git repository, or an indeterminable
  default branch all exit with code 9 and the exact command to run.
supersedes: ADR-92b9cda9f6a9
ratified: a62aacef2111
schema: 1
version: 4
---

Replaces ADR-92b9cda9f6a9 without changing its substance: git stays a hard
dependency, the plumbing goes through the binary, porcelain stays forbidden, the
floor stays 2.34. Every justification in the replaced ADR holds as written and is
not repeated here.

What changes is the form of the restriction. The replaced ADR stated "uses
plumbing only" followed by a list of five commands, which made it closed.
ADR-bcf222a31525 needs three commands that are not in it — `for-each-ref` to
enumerate `refs/ank/*`, `symbolic-ref` to read `refs/remotes/origin/HEAD` and the
current branch, `merge-base` for reachability — and replacement is the only
legitimate route to add them, `constraint` being locked on an accepted ADR (§3).

The occasion is worth using to fix a defect in the list rather than merely
lengthening it: the closed list was already missing `for-each-ref` while the
specification had required `check` to prune orphan refs since revision c. Pruning
without enumeration is not implementable. The constraint was therefore in
contradiction with the specification before anyone wrote the line of code that
would have revealed it, and a closed list goes stale at every new need.

The criterion retained — the output is stable by contract across git versions —
is the property we were actually trying to capture. It is what excludes
porcelain, and it excludes it by its reason rather than by its name. The
enumeration stays in the constraint, because a criterion alone would leave every
agent to judge for itself, and judging is precisely what a constraint avoids; but
it is now the application of the criterion rather than its definition.

Code 9 gains the case of an indeterminable default branch, which belongs to the
same family as the other three: an environment to repair, not a failure of the
agent's work. `accept` cannot evaluate its branch precondition without an answer,
and guessing `main` would be the guess the tool refuses everywhere else.
