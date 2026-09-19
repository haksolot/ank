---
id: LOG-0330cdecc35c
type: log
title: "Measured the verb before touching it: status --json on this corpus of 1518 entities cost 3.7-6.0s,"
created: 2026-08-28T18:57:16Z
author: claude-code/opus-5+status-counts-cheap
scope:
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/src/index.rs
about: TASK-be17972988d9
seq: 0
schema: 4
version: 1
---

 a bare index-opening verb (show) cost 0.25s, so the whole difference was human::inspect parsing every file to print two integers.

Landed the memoised verdict in the index (schema 6, table 'verdict'). The key is files digest + refs/ank/* (name and object) + the plane's reading of each claim + default branch and its tip + config.yml + allowed_signers + the build version. Three components beyond the two ADR-f3d1dea65d84 names, and each for a finding that would otherwise go stale silently: the default branch is what drift and the completion refs are judged against, config.yml declares the verifiers and the budget, allowed_signers decides whether a ratification is checkable.

The clock is the input that cannot be hashed, and the check reads it twice. A claim lapsing is covered by putting the plane's own Claimed/Lapsed word in the key -- the plane uses claim::is_expired, the same predicate the check uses, so no tolerance is restated. An entity created in the future is not: its signal turns off a day before its own timestamp, and rather than copy that threshold here, a corpus holding such an entity gets no key at all and pays the full price.

Two second-order costs found and left alone, both outside this scope: context::plane costs 550-1000ms on this repository because refs/ank/* holds 645 refs and every record is read; repo::identity walks the whole history for the --json 'corpus' field. Filing them.

Also fixed in passing, in index.rs: scan() decoded every file to a String and held the whole corpus in memory to conclude nothing had changed (11MB per invocation of any verb here); wipe_in() dropped four of the six tables the schema creates, which is the defect its own comment describes.

New: 154-213ms for status --json over 1000 entities, unoptimised build, loaded machine. 2.0s before, optimised.
