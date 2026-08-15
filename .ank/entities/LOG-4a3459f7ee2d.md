---
id: LOG-4a3459f7ee2d
type: log
title: Attribution decided before implementing, which the criterion asks for, and measured rather than
created: 2026-08-04T18:13:22Z
author: seanl@sean-laptop
scope:
  - .github/workflows/ci.yml
about: TASK-d81a05ef8e8d
seq: 0
schema: 3
version: 1
---

 reasoned. Walked 1.94 locally: the workspace fails with error[E0658] 'use of unstable library feature cfg_select' in the libsqlite3-sys 0.38.1 build script, exit 101 -- exactly what the manifest comment records, so the comment is not stale. Rejected two altitudes. Requiring only a non-zero exit is what the body calls brittle: a network hiccup or a missing C compiler passes the negative test while proving nothing. Requiring E0658 specifically is too tight in the other direction -- it breaks the day the floor moves for a different reason, which is precisely the event this job exists to notice, and it would go red with a message pointing at the wrong cause. Landing on two independent gates instead. A positive control: ank-core alone builds on the previous minor (verified, exit 0 in 13s, no C dependency), so if that fails the toolchain, the network or the registry is broken and the job reports an environment failure rather than a pass. Then the negative test, required to carry a rustc diagnostic code (error[E), which separates 'the compiler rejected this tree' from 'the environment fell over' -- cargo's own infrastructure failures and a missing cc produce 'error: failed to ...' with no code. Residual gap recorded on purpose: a floor set by something that emits no E-code, a lockfile version for instance, would be read as untight and the job would say lower the number when it should not. That fails loudly and gets investigated, which is the direction to fail in; a false pass is the defect being fixed.
