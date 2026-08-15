---
id: LOG-e2c553b4bfa0
type: log
title: Asserted the criterion the way it is written -- by changing the declared value and observing CI
created: 2026-08-04T18:38:49Z
author: seanl@sean-laptop
scope:
  - .github/workflows/ci.yml
about: TASK-3b085be1bf6a
schema: 3
version: 1
---

 follow it, with no workflow edit -- on a throwaway branch that touched only the two manifests. Run 30939293330: msrv printed 'declared 1.96, in both manifests' and installed rustc 1.96.1, where the old job would have installed 1.95 regardless and stayed green. The same run doubled as the first real exercise of TASK-d81a05ef8e8d's job in the direction it was built for: msrv-tight derived 1.96, tested 1.95, found that 1.95 genuinely builds the whole workspace in 11.83s, and failed with 'the declared MSRV 1.96 is higher than this tree needs'. So one push proved both jobs, one by following and one by refusing. Branch deleted after reading, never merged, and the manifests on the real branch are untouched at 1.95.
