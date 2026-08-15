---
id: LOG-7fa3bcb76a3f
type: log
title: "Decision on the path, which the body left open: root .gitignore with the line '.ank/index.db', not"
created: 2026-08-04T17:14:03Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-c1783c841710
schema: 3
version: 1
---

 .ank/.gitignore with 'index.db'. Three reasons. (1) Symmetry: init already writes a root .gitattributes for the same class of reason, and a user asking what init did to their repo finds every effect at root plus .ank/. (2) ADR-01b6 makes .ank/ opaque to agents and a PreToolUse hook enforces it -- an ignore rule living inside .ank/ could not be read by the agent debugging why index.db is tracked. At root it is plain to everyone. (3) It reuses ensure_line unchanged, and it is the line this repository already carries by hand. Checked that a stray file at the top of .ank/ would have been inert anyway: index::scan and store::list_ids both read only tasks/ and adr/, so the choice was not forced by the tool. Also confirmed the pattern stays correct under 'ank init <subdir>': a path-bearing gitignore pattern is anchored to the directory holding the file.
