---
id: LOG-f5fb020afd55
type: log
title: "The specification had already settled the design question, at section 4 line 553: 'nothing calls"
created: 2026-08-04T17:52:04Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/human.rs
  - .github/workflows/ci.yml
about: TASK-0aaf0888c9f2
schema: 3
version: 1
---

 attest automatically. A CI provider appending its own run reference at the end of a pipeline is an integration, not a verb.' Found it while looking for where to record the new signal, after the reasoning was written rather than before -- so the detector shape was re-derived rather than read off, and the two agree. That deferral is the positive half of the same choice; this signal is the negative half, and section 4 now carries both next to each other. Also measured that no existing finding covers this case: commit is not weak, so the weak-proof signal stays silent by design on a commit:-only proof list, which is why the omission was invisible rather than merely unreported.
