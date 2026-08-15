---
id: LOG-1ebeccd6fecf
type: log
title: "CI run 30839289286 green on all six jobs, three platforms plus the MSRV 1.95 walk: sh -c and the cp"
created: 2026-08-03T18:03:25Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
about: TASK-7ed19b16895e
schema: 3
version: 1
---

 editor behave the same on Windows and macOS as on Linux, which is what a verb that spawns a shell needed proving. Then found a defect in my own test rather than in the code: the unit test for an unset EDITOR called std::env::set_var, which is unsound while another thread reads the environment -- and the harness is threaded, with a neighbouring test calling std::env::temp_dir, which reads TMPDIR. Split the decision from the reading: editor_from(Option<&str>) is what the unit test exercises, and the absence itself is tested through the binary in a process of its own, which is where it belonged anyway.
