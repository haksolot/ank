---
id: TASK-659ebaa4f68e
type: task
slug: the-specification-names-three-install-modes
title: The specification names three install modes
created: 2026-08-19T16:22:04Z
author: claude-code/5
status: done
scope:
  - .ank/entities/SPEC-80bff12ceae8.md
blocked_by: [TASK-6de3f29911bd]
done_criteria: |
  The successor of SPEC-80bff12ceae8, created with ank new spec --supersedes and never by editing, arrives proposed with its references re-declared. Its binary-distribution paragraph names npm, curl | sh and the PowerShell one-liner as the channels that ship, names no channel that does not ship, and cites ADR-221aa5da440a. ank check is green with no unresolved reference.
criteria_by: creator
proof:
  - type: commit
    ref: 0873ac1d601f60c369a457034a7a93fbb4993254
    criteria: e3ae54fdf1eb
    via: submitted
schema: 3
version: 3
---

The same move as TASK-13f9162ed61a and last for the same reason: the
specification describes what is, so revising it before the teardown lands
would make it lie in the other direction. A reader acts on a named channel.

SPEC-80bff12ceae8 is accepted, so its body is anchored by the ratification
commit and any in-place edit is reported altered by check. The distribution
paragraph currently names five shipping channels plus winget and Arch as
pending; everything else in the document (help surface, init, token economy,
npm packaging, dist-tag derivation) stays true and travels into the successor
unchanged. Expect citing documents to need amend --reference afterwards, as
every supersession before this one did.
