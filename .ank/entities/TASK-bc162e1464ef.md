---
id: TASK-bc162e1464ef
type: task
slug: the-threat-model-becomes-a-security-policy
title: The threat model becomes a security policy
created: 2026-08-05T18:35:37Z
author: seanl@sean-laptop
status: done
scope:
  - SECURITY.md
blocked_by: []
done_criteria: |
  SECURITY.md exists at the root and states the threat model as section 1
  and README.md already state it: ank protects against drift, not against an
  adversary, it is not a security boundary, and the roles in config.yml are
  advisory. It names the signed ratification commit as the single hard line of
  authority and states what that signature does not prove. It records that
  verifiers live in a repository-controlled file, so ank done on a PR from a
  fork executes nothing the repository did not accept. It names which version
  is supported and routes reports to GitHub private vulnerability reporting,
  which is enabled on the repository. No sentence in it contradicts
  docs/ank-spec-v1.1.md.
criteria_by: creator
proof:
  - type: commit
    ref: 55f2d7cad592869bae4c246b135d6f0284c8f934
    criteria: d1716893bbd0
schema: 3
version: 3
---

The project published its first package to a registry with v0.1.2, so it is
now something strangers install before they know anything about how it is
built. A security policy is what answers them, and here it cannot be
boilerplate: the specification takes a position and a generic policy would
contradict the source of truth.

Section 1 protects against drift, not against an adversary. README.md says in
as many words that ank is not a security boundary. Section 8 says the
signature proves access to an authorised key, not human intent -- an agent on
a developer machine with signing unlocked can produce a valid signed commit,
and the defence there is operational (a ratification key behind a passphrase
or hardware, distinct from the everyday key), never cryptographic. With no
signing configured at all, the hard line is gone and nothing replaces it;
check displays that rather than hiding it, because a verification that
degrades to success is not a verification.

One property is worth stating as the real one: verifiers are named in
config.yml and never inline, precisely because a task may arrive through a PR
from a fork. An inline command would be arbitrary code execution triggered by
ank done. Git had this problem with hooks and solved it the same way.

Reports go to GitHub private vulnerability reporting, which has to be enabled
on the repository before the file names it. A policy pointing at a channel
that does not exist is worse than no policy.
