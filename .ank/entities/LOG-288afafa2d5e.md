---
id: LOG-288afafa2d5e
type: log
title: "Reading taken on 'byte-identical git invocations of a cycle': ank-daemon cannot link ank-cli"
created: 2026-10-23T17:15:26Z
author: claude-code/opus-5.5+w451a
scope:
  - crates/ank-daemon/**
  - crates/ank-cli/src/status.rs
  - crates/ank-cli/tests/**
  - docs/watch.md
about: TASK-451aaa32043f
seq: 1
schema: 4
version: 1
---

 (dependencies.rs forbids it), so the resolver of crates/ank-cli/src/git.rs (remote_for) is reused by handing it to the watcher through Address from cli.rs (scope +cli.rs), and it is asked ONCE per checkout at startup, after identity_of, never per cycle. Each cycle then runs 'config --get remote.<R>.url', fetch from R, for-each-ref refs/ank/watch/<R>/ -- with R=origin byte-identical to before. A None answer (no remote yet) keeps R=origin, so a remote added later is still picked up exactly as before. Cost: a watcher started before 'git remote add ank' mirrors origin until restarted; documented in docs/watch.md. status reads every refs/ank/watch/<remote>/claims mirror already (Namespaces::of ignores the remote segment), so no change to context.rs.
