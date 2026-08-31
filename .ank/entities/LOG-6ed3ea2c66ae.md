---
id: LOG-6ed3ea2c66ae
type: log
title: drift audit, re-measured and holds; no finding. The risk this constraint names is a cache whose key
created: 2026-08-31T03:11:37Z
author: claude-code/opus-5+drift
scope:
  - crates/ank-cli/**
about: ADR-f3d1dea65d84
seq: 0
schema: 4
version: 1
---

 is wrong making the verb lie, so the key was exercised on both halves. In a throwaway corpus: ank status --json and ank check --json both answered faults 0, signals 18; ank new task then changed the files and both answered 20; ank claim then changed refs/ank/* alone and both still answered 20. On this corpus ank status reports 0 faults and 428 signals and ank check reports the same, while status spawns 7 git processes against check's 18. No counter is absent, optional or null under --json.
