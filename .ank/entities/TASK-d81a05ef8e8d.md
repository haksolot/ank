---
id: TASK-d81a05ef8e8d
type: task
slug: ci-proves-the-msrv-is-sufficient-never-that-it-i
title: CI proves the MSRV is sufficient, never that it is tight
created: 2026-08-04T04:52:06Z
author: seanl@sean-laptop
status: done
scope:
  - .github/workflows/ci.yml
blocked_by: []
done_criteria: |
  CI fails when the declared MSRV is higher than the tree actually needs, not only when it is lower. The check runs the toolchain one minor below rust-version with --ignore-rust-version and requires it to fail; a build that unexpectedly succeeds names the number to lower and the command that measured it. It runs on the platform where the floor is set, and a rustup release that has not shipped yet is an environment failure and not a passing check.
criteria_by: creator
proof:
  - type: commit
    ref: a45301a
    criteria: d837d5971fdc
  - type: test
    ref: "30938913342"
    criteria: d837d5971fdc
schema: 3
version: 6
---

The msrv job builds on the exact toolchain rust-version names, which proves the number is sufficient. Nothing proves it is tight. The floor comes from one build script in libsqlite3-sys, so the day that crate stops calling cfg_select! -- or the day the dependency is replaced -- the declared 1.95 becomes higher than the tree needs, every job stays green, and the only signal is that nobody can build ank on a toolchain that would have worked.

That asymmetry is the same one TASK-daf25ab8a9b7 corrected in the other direction: an MSRV asserted and never run rots silently. Half the claim is now enforced and half is still an assertion.

Found while answering TASK-973e9dc3f9ce, whose criterion was to choose between two dependency versions and to make the manifests agree with the measurement. Making the measurement continuous is a different piece of work and is not in that criterion.

The design question belongs to whoever claims this. Requiring the previous minor to fail is a negative test, and negative tests are brittle in a way positive ones are not: a build that fails for an unrelated reason -- a network hiccup, a C compiler missing -- passes this check while proving nothing. The failure has to be attributed, not merely observed, and how tightly is the decision to record before implementing.
