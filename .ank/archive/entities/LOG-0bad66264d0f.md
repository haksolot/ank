---
id: LOG-0bad66264d0f
type: log
title: ank done runs under PATH=/tmp/claude-1000/-home-haksolot--herdr-worktrees-ank-task-791d-verifiers-mi
created: 2026-10-24T09:28:33Z
author: haksolot@vmi3223161
scope:
  - .ank/config.yml
about: TASK-791de95bdd2d
seq: 5
schema: 4
version: 1
---

se/1c761b02-a514-4992-913b-77f034ed1a2f/scratchpad/ankbin:/tmp/claude-1000/-home-haksolot--herdr-worktrees-ank-task-791d-verifiers-mise/1c761b02-a514-4992-913b-77f034ed1a2f/scratchpad/rustbin:/home/haksolot/.local/bin:/usr/local/bin:/usr/bin:/bin: <scratchpad>/ankbin holds a copy of ank, <scratchpad>/rustbin holds symlinks to the rustup proxies cargo, rustc, rustup, rustfmt, cargo-fmt, rustdoc and nothing else; sh resolves mise to /home/haksolot/.local/bin/mise and finds no cargo-nextest on PATH (command -v exit 127). Per LOG-de1b5eba3ce0, cargo itself still reaches ~/.cargo/bin/cargo-nextest through CARGO_HOME; TASK-0d767e9875a3 carries that.
