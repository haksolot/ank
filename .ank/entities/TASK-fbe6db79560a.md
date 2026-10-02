---
id: TASK-fbe6db79560a
type: task
slug: a-page-says-how-several-repositories-share-const
title: "A page says how several repositories share constraints: peers at both ends, the override, id@peer, and what never crosses"
created: 2026-09-27T21:37:42Z
author: claude-code/opus-5.5+plan
status: in_progress
scope:
  - docs/peers.md
  - docs/SUMMARY.md
  - crates/ank-cli/tests/doc_replay.rs
blocked_by: [TASK-eff5715bfd87]
done_criteria: |
  docs/peers.md exists, is linked from docs/SUMMARY.md, and is in the list of pages crates/ank-cli/tests/doc_replay.rs replays. It shows, in blocks the replay runs against two scratch corpora: a constraint in one corpus scoped <peer>:** served by ank context and ank scope inside the other once each declares the other; ank config --user peers.<identity>.<name> <path> restoring a binding whose committed path is wrong on this machine; ank config refusing a URL-shaped peer value; ank show <id>@<peer>; and ank claim refusing a task whose blocked_by names a peer task not yet done. It states in prose that claims and writes never cross a corpus, that an unreadable peer holds a blocked_by edge, and that a local glob never covers a peer path, citing ADR-a1de673043b4, ADR-da2819aef598, ADR-96fe1f9d619a and ADR-c23bef1cc93e. cargo test --workspace passes, doc replay included.
criteria_by: creator
verify: [cargo-test, fmt-check]
method: tdd
schema: 4
version: 3
---

Issues #500, #501 and #502 changed how several repositories share a corpus, and
not one page of docs/ says so: `peers` appears only as a generated row of
config-keys.md, and `<id>@<peer>`, the override, the URL refusal and cross-peer
blocked_by appear nowhere. Issue #500 itself was filed because the direction of
a declaration was written down nowhere; the answer now exists in `ank help
config`, and a person reading the site still cannot find it.

What the page has to teach, in the order someone meets it:

1. A binding across corpora needs a declaration at both ends: the governing
   corpus scopes `<peer>:**` and names the peer, the governed corpus declares the
   governing one to read it (TASK-2f5d6af5de36).
2. The committed path is a convention; a reader whose layout differs overrides it
   in their own corpora.yml (TASK-e95c729e5d43), and a URL is refused
   (TASK-c6d184d238e1, TASK-bcdc32d08947).
3. An entity of a peer is named `<id>@<peer>` (TASK-f4c4b3bafa2d), and a task can
   wait on one (TASK-08615a199a6a).
4. What never crosses: claims, writes, and a local glob (TASK-c666eb306102).

Every output shown is replayed, so the page is measured by the suite rather
than trusted; the replay harness builds scratch repositories, and two of them
declaring each other is the setup to give it.
