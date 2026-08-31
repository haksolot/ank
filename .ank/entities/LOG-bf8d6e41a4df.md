---
id: LOG-bf8d6e41a4df
type: log
title: drift audit 2026-08-31, re-measured and holds; no finding. Counted with GIT_TRACE at an absolute
created: 2026-08-31T07:53:01Z
author: claude-code/opus-5+drift2
scope:
  - crates/ank-cli/**
about: ADR-cc65f1388a71
seq: 1
schema: 4
version: 1
---

 path, one line per process, never by timing. Three throwaway corpora at 2, 20 and 60 tasks (4, 40 and 120 entities, a 30x spread): find 3/3/3, review 4/4/4, graph 3/3/3, status 5/5/5, check 4/4/4, scope 3/3/3, context 4/4/4 -- constant across every one. Coordination refs measured separately, because entities and refs are two dimensions and the earlier audit only isolated entities: the 60-task corpus was claimed 60 times under 60 identities, then copied and all but one claim ref deleted with update-ref -d, so one corpus at 1 claim and 60 claims. find 4/4, review 5/5, graph 4/4, check 5/5, scope 4/4, context 5/5. status read 8 at 1 claim and 6 at 60, which is the memoised check verdict of ADR-f3d1dea65d84 warm in one and cold in the other, and is fewer rather than more. Invariance holds on both dimensions.
