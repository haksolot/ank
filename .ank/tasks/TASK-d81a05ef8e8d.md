---
id: TASK-d81a05ef8e8d
type: task
slug: ci-proves-the-msrv-is-sufficient-never-that-it-i
title: CI proves the MSRV is sufficient, never that it is tight
created: 2026-08-04T04:52:06Z
author: seanl@sean-laptop
status: in_progress
scope:
  - .github/workflows/ci.yml
blocked_by: []
done_criteria: |
  CI fails when the declared MSRV is higher than the tree actually needs, not only when it is lower. The check runs the toolchain one minor below rust-version with --ignore-rust-version and requires it to fail; a build that unexpectedly succeeds names the number to lower and the command that measured it. It runs on the platform where the floor is set, and a rustup release that has not shipped yet is an environment failure and not a passing check.
criteria_by: creator
schema: 2
version: 4
---

The msrv job builds on the exact toolchain rust-version names, which proves the number is sufficient. Nothing proves it is tight. The floor comes from one build script in libsqlite3-sys, so the day that crate stops calling cfg_select! -- or the day the dependency is replaced -- the declared 1.95 becomes higher than the tree needs, every job stays green, and the only signal is that nobody can build ank on a toolchain that would have worked.

That asymmetry is the same one TASK-daf25ab8a9b7 corrected in the other direction: an MSRV asserted and never run rots silently. Half the claim is now enforced and half is still an assertion.

Found while answering TASK-973e9dc3f9ce, whose criterion was to choose between two dependency versions and to make the manifests agree with the measurement. Making the measurement continuous is a different piece of work and is not in that criterion.

The design question belongs to whoever claims this. Requiring the previous minor to fail is a negative test, and negative tests are brittle in a way positive ones are not: a build that fails for an unrelated reason -- a network hiccup, a C compiler missing -- passes this check while proving nothing. The failure has to be attributed, not merely observed, and how tightly is the decision to record before implementing.

## Log
- 2026-08-04T18:13:22Z seanl@sean-laptop — Attribution decided before implementing, which the criterion asks for, and measured rather than reasoned. Walked 1.94 locally: the workspace fails with error[E0658] 'use of unstable library feature cfg_select' in the libsqlite3-sys 0.38.1 build script, exit 101 -- exactly what the manifest comment records, so the comment is not stale. Rejected two altitudes. Requiring only a non-zero exit is what the body calls brittle: a network hiccup or a missing C compiler passes the negative test while proving nothing. Requiring E0658 specifically is too tight in the other direction -- it breaks the day the floor moves for a different reason, which is precisely the event this job exists to notice, and it would go red with a message pointing at the wrong cause. Landing on two independent gates instead. A positive control: ank-core alone builds on the previous minor (verified, exit 0 in 13s, no C dependency), so if that fails the toolchain, the network or the registry is broken and the job reports an environment failure rather than a pass. Then the negative test, required to carry a rustc diagnostic code (error[E), which separates 'the compiler rejected this tree' from 'the environment fell over' -- cargo's own infrastructure failures and a missing cc produce 'error: failed to ...' with no code. Residual gap recorded on purpose: a floor set by something that emits no E-code, a lockfile version for instance, would be read as untight and the job would say lower the number when it should not. That fails loudly and gets investigated, which is the direction to fail in; a false pass is the defect being fixed.
- 2026-08-04T18:16:55Z seanl@sean-laptop — Falsified the job end to end rather than only unit-testing its branches, which for a negative test is the whole point. Extracted the decision step verbatim and drove it with real cargo results. Three outcomes exercised: the real tree on 1.94 fails with error[E0658] and the step passes naming the code; a simulated declared 1.96, where prev is 1.95 and genuinely does build the workspace (measured, exit 0, not stubbed), makes the step exit 1 with the number to lower and the command that measured it; a failure carrying no rustc diagnostic exits 9 as environment. So the check fails in the direction the criterion asks for and not only in the direction that was already covered. Discovered and out of this criterion: the existing msrv job hardcodes 1.95 in its name and in both commands, while the new job derives the number from the manifest. Bumping rust-version therefore leaves the sufficiency job building a toolchain the manifests no longer name, silently testing the wrong thing -- the same class of rot as this task, in the job next door. Filing it separately rather than widening a frozen criterion.
