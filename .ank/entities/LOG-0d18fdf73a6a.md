---
id: LOG-0d18fdf73a6a
type: log
title: "Measured on this corpus rather than argued: the binary built from this branch reports 57 signals"
created: 2026-08-14T19:23:02Z
author: claude-code/2946
scope:
  - crates/ank-core/src/model.rs
  - crates/ank-cli/src/human.rs
  - docs/ank-spec-v1.1.md
about: TASK-29461c1f508d
seq: 1
schema: 3
version: 1
---

 against 169 tasks, exactly what the binary built from main reports, and the same four tasks by name under 'done with no test proof'. Nothing fires retroactively on the twenty-one completions anchored by references typed before the field existed. Two holes the change opened and closed on the way. Prune could downgrade a proof: maintain_proofs retires refs/ank/proof/<id> once the file carries the same type and reference, so an attested entry copied into the file as 'submitted' would have deleted the only record of the route and brought the signal back on a task nobody touched -- attest now records 'attested' when the ref already carries that exact entry, which is the same 'Ank validates what it can' the commit check already performs, and prune additionally refuses to retire a ref whose anchor the file has not kept. And the hint had stopped correcting: a plain 'ank attest' writes via submitted, so the command the finding named no longer cleared the finding, and it now names --detached.
