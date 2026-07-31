---
id: TASK-aca0cb103980
type: task
slug: crlf-tolerance
title: CRLF tolerated on read, LF on write, with a dedicated diagnostic
created: 2026-07-28T00:22:06Z
status: open
scope:
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/error.rs
  - crates/ank-core/tests/golden/**
  - crates/ank-cli/src/human.rs
blocked_by: []
done_criteria: |
  parse_entity accepts a file in CRLF and its serialisation yields LF; a valid
  golden in CRLF covers it, and the golden suite no longer requires
  byte-for-byte identity except on already canonical files.
  Error::CrlfLineEndings exists and its message names the line endings and the
  command git config core.autocrlf input; a test asserts that this diagnostic,
  and not "missing frontmatter", is the one reported for a CRLF file.
  ank check reports the CRLF form as a signal, distinct from the fault it
  raises for a non-canonical form, and exits 0 when that is the only
  deviation.
  cargo test is green.
criteria_by: claimer
verify: [cargo-test]
schema: 1
version: 3
---

Found while checking a fresh clone: `core.autocrlf=true` without a
`.gitattributes` made all 15 entities and the whole golden suite unreadable, with
the "missing frontmatter" diagnostic sending the reader down the wrong path. The
`.gitattributes` placed at the root is the underlying fix; this is the safety
net, for earlier clones, archives and third-party tools.

Order imposed by ADR-63b59c5c26f7 (specification, then goldens, then code): §3
"Canonical form and round-trip" is already written, and the goldens come before
the parser.

Not blocking for the CLI foundation: `.gitattributes` is enough to make the
current tree healthy, which is why this task blocks neither TASK-244a842bc0cc nor
TASK-c8637488773c.

Amended by TASK-a7b8c9d0e1f2, which retired `examples/check_repo.rs` in favour
of the real `ank check`. The scope named that file and the criterion named that
command; both now name `crates/ank-cli/src/human.rs` and `ank check`. The task
was unclaimed, so the criterion is amended with no freeze to lift (§3), and the
amendment is a rename of the referent rather than a change of the rule — the
behaviour asked for is the same one, from the command that now carries it.
