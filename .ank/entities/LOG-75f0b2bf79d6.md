---
id: LOG-75f0b2bf79d6
type: log
title: "Scope amended to tests/golden-json/help.json: the config note in ank-contract is pinned there;"
created: 2026-09-27T19:20:34Z
author: claude-code/opus-5.5+e95c
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/peer_override.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
about: TASK-e95c729e5d43
seq: 7
schema: 4
version: 1
---

 blessed, and the diff is that one note and nothing else. peers.<wrong arity> stays 'unknown key' with the key set, which now lists peers.<identity>.<name>, so cli.rs's existing refusal test holds unchanged.
