---
id: LOG-dcc8b0a44951
type: log
title: Measured on this Windows host. (1) A build with skill/ moved out of the tree, built with cargo
created: 2026-09-13T12:41:11Z
author: claude-code/544e
scope:
  - crates/ank-cli/build.rs
  - crates/ank-cli/src/**
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/skill.rs
about: TASK-544ec9655570
seq: 2
schema: 4
version: 1
---

 build -p ank-cli and run from a copy: 'ank skills' printed 1 line, 'this build carries no skills: there was no skill/ directory to read when it was built', exit 0; 'ank skills --install' the same line, exit 0; --version read 'ank 0.7.0 (45776d8, skill unknown)'. skill/ restored and touched so the next build reran build.rs. (2) The stub npx.cmd run through the binary with PATH reduced to it and stdin at NUL recorded arg:skills, arg:add, arg:<dir>, yes:1, written:yes, stdin:closed; the verb exited 42, the stub's code; the directory held ank, ank-diagnose, ank-drift, ank-loop, ank-plan, ank-tdd. Run directly with input piped in, the same stub recorded stdin:open, so the instrument discriminates. The POSIX stub's logic was run under sh with the same two inputs: stdin:closed then stdin:open. (3) Falsification: removing npm_config_yes turned the install test red on 'did not record yes:1'; appending --yes to the argv turned it red on the argv; translating npx's code to 1 turned it red on left Some(1) right Some(42).
