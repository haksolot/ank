---
id: LOG-da22cd2b10db
type: log
title: "CI found two the local run could not. The dependency test ran cargo tree --target all --offline:"
created: 2026-08-26T01:39:51Z
author: claude-code/opus-5+ratatui-engine
scope:
  - crates/ank-tui/**
about: TASK-4fa385c1772d
seq: 3
schema: 4
version: 1
---

 --target all needs the manifest of every package on every target, --offline forbids fetching one, and a host build downloads only what it compiles. Before ratatui the two happened to agree; ratatui reaches bumpalo through a wasm32-only edge of time, no runner ever downloads it, and the test went red on all three asking for the network it is forbidden to use. The graph is read out of Cargo.lock now, which records every target and every optional dependency, is checked in, and is broader than --target all rather than narrower -- the safe direction for a rule that says a name must not appear. And macOS refused TIOCSWINSZ on the pseudo-terminal master with -1: there the master is a cloning device answering only the three ioctls grantpt, unlockpt and ptsname are built from, so the window is set through the slave, which is a tty on both and is where a window size belongs. The child's three slave descriptors are now opened before the size is stated, so no moment exists in which the terminal has hung up.
