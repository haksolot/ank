---
id: LOG-c4b663faa715
type: log
title: "edit ships and NOT_YET_DISPATCHED is empty: every verb section 4 specifies now answers. Four"
created: 2026-08-03T17:57:31Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
about: TASK-7ed19b16895e
schema: 3
version: 1
---

 mutations, four caught -- the no-op guard, the frozen-field guard, the temp copy (pointing the editor at the real file in .ank/ broke five tests at once), and a guessed editor fallback. The fallback mutation is worth recording: it made the suite launch a real vim and hang rather than fail, which is the shape a test takes when the mutation removes a refusal that was keeping a process from starting. Killed vim, the test then failed as expected. Dogfooded on the real corpus: 'ank edit TASK-7ed1' with a weakened criterion was refused code 6 naming 'ank release --reason', and the criterion on disk is unmoved -- the verb refuses its own author. Clippy adds no warning over the 6 already on main.
