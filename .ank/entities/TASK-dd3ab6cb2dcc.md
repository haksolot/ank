---
id: TASK-dd3ab6cb2dcc
type: task
slug: the-plane-read-is-narrowed-to-the-namespaces-the
title: The plane read is narrowed to the namespaces the caller asks for
created: 2026-09-14T06:40:12Z
author: haksolot@vmi3223161
status: done
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/status.rs
  - crates/ank-cli/tests/cli.rs
blocked_by: []
done_criteria: |
  Under GIT_TRACE, find --json, context, graph --json and status --json each start one for-each-ref over refs/ank/ and one cat-file --batch, and the process list of each is identical between a corpus with no proof refs and one carrying 500, asserted through the binary. A proof ref whose blob is not a record produces no warning in context --json and exactly one finding in check --json. A unit test on git::ank_records asserts that a proof ref's object is absent from the returned map when only the claims namespace is asked for, and present when the proof namespace is. context::Plane carries no proofs field. cargo test --workspace, cargo fmt --check and ank check stay green.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
proof:
  - type: test
    ref: local/2a5653c64a29@3c61f3b
    tree: scope/acde265d5b77
    criteria: c066e7345f7e
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@3c61f3b
    tree: scope/acde265d5b77
    criteria: c066e7345f7e
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

Measured 2026-09-14, release build of 2a65d0c, this corpus (1921 entities,
`refs/ank/*` = 664 blob refs, 7.5 MB): `find --json` 0.44-0.53 s, `context .`
0.46-0.59 s, `show` 0.47-0.55 s, `status --json` 0.45 s; the same verbs with
`refs/ank/*` deleted: 0.09, 0.11, -, 0.12 s. About 350 ms of every verb is
`context::plane` (`context.rs:159`) reading the plane through
`git::ank_records` (`git.rs:726`): one `for-each-ref`, one `cat-file --batch`
over all 664 objects, then `claim::parse_record` (serde_yaml Value, then
from_value) on every claims, proof and mirror-claims ref. Split by
experiment: proof refs alone (205 refs, 3.3 MB of YAML) put `find` at 0.42 s;
everything but proofs, 0.17-0.23 s; nothing, 0.09 s. So roughly 250 ms is
YAML parsing of proof records and 100 ms is bytes moved through the batch.
git's own cost of the same read is 98 ms.

**Nobody reads what is parsed.** `context::Plane.proofs` has no consumer:
`show` reads detached proofs through `claim::detached_proofs` and its own
`cat-file`, `check` through `human::coordination` (`human.rs:753`), which
batches on its own. `find`, `scope`, `graph`, `show` and `context` call
`coordination()`, which parses the proofs and returns `.claims`. `.mirrored`
is read by `status` alone.

**The shape.** A `Namespaces` bitset (CLAIMS, PROOF, MIRROR_CLAIMS) handed to
`ank_records`. One enumeration is kept, so two walks cannot disagree
(TASK-5690eae1e008); the batch is fed only the object names of the wanted
namespaces, so refs outside every namespace -- `refs/ank/remote-check/*`,
the mirror's proofs -- are never moved. The per-process memo records what it
fetched and widens by fetching only the difference; `status` asks for
CLAIMS|MIRROR_CLAIMS before `claim::on_task` asks for CLAIMS, so its widest
request is its first batch (assert `cat-file --batch` once in
`status_asks_git_no_question_twice_and_reads_the_plane_in_one_batch`).
A ref outside the wanted namespaces is skipped before the record lookup,
so a damaged proof blob is `check`'s finding and not `context`'s warning.
No test in `crates/ank-cli/tests` asserts the "unreadable coordination ref"
warning today. `human::coordination` may take the same filter (CLAIMS|PROOF)
since `check` unions file and ref proofs (ADR-493471d64ba0) and reads no
mirror. ADR-cc65f1388a71 binds the process count; ADR-f3d1dea65d84 is why
the cost is visible at the call site rather than hidden in a lazy getter.
