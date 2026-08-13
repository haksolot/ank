---
id: TASK-aca0cb103980
type: task
slug: crlf-tolerance
title: CRLF tolerated on read, LF on write, with a dedicated diagnostic
created: 2026-07-28T00:22:06Z
status: done
scope:
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/error.rs
  - crates/ank-core/src/lib.rs
  - crates/ank-core/tests/golden/**
  - crates/ank-core/tests/golden.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - .gitattributes
  - .github/workflows/ci.yml
  - README.md
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
proof:
  - type: test
    ref: local/c0af7fb4d906@f8d64ff
    tree: scope/4f9181310af1
    criteria: f127e6564a14
    verifier: cargo-test@f14aeab36e1b
  - type: test
    ref: ci://haksolot/ank/runs/30650046088
  - type: commit
    ref: 22c7ad2
schema: 3
version: 8
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

Scope widened before claiming, criterion untouched, four additions and each one
forced by the criterion rather than convenient:

- `lib.rs` — the criterion needs `check` to tell a file that is merely CRLF from
  one that is genuinely non-canonical, so the normalisation `parse` performs has
  to be reachable from `ank-cli`. A re-export, no logic.
- `tests/golden.rs` — the declared scope covered `tests/golden/**`, the fixture
  directory, and not the harness that reads it. "The golden suite no longer
  requires byte-for-byte identity except on already canonical files" is a change
  to the harness by definition.
- `.gitattributes` — this is the sharp one. The suite is pinned `text eol=lf`,
  so a CRLF fixture committed today comes back LF on the next checkout and the
  test silently stops testing anything. The fixture needs `-text` to survive as
  the bytes it is. A golden that git normalises is not a golden.
- `.github/workflows/ci.yml` — its line-endings step greps the golden tree for
  `\r` and fails the build. That guard is right and stays; it gains an exception
  for the one file whose whole purpose is to carry a `\r`.

Two more added while working, same rule, criterion untouched:

- `crates/ank-cli/tests/cli.rs` — the criterion says "ank check ... exits 0",
  which names the binary and an exit code. The unit-level harness in `human.rs`
  calls `check()` directly and never becomes a process, so on its own it cannot
  answer the thing asked. CLAUDE.md is explicit, and the two defects it cites
  are exactly this shape.
- `README.md` — it stated that `valid/` must round-trip byte for byte, which
  stopped being true the moment a CRLF golden joined the suite. Leaving it would
  recreate, in the same file, the class of defect TASK-0da5af5afd5f just cleared
  out of it.
