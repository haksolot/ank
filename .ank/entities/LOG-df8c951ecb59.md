---
id: LOG-df8c951ecb59
type: log
title: drift audit 2026-08-31, re-measured and holds, the surgery included. A config.yml was written with
created: 2026-08-31T07:53:42Z
author: claude-code/opus-5+drift2
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/src/init.rs
  - crates/ank-cli/src/cli.rs
  - docs/**
about: ADR-e64dfaafd578
seq: 0
schema: 4
version: 1
---

 a top comment carrying trailing spaces, a blank line, 'context_budget:   8000' with three spaces, "claim_ttl_max: '2h'" single-quoted, 'claim_ttl_default: "45m"' double-quoted, and a trailing '# trailing comment' after a verifier's run. 'ank config context_budget 9000' produced a one-line diff: the value alone. Every comment, blank line, quoting style and the three-space alignment survived byte for byte. 'ank config nope 1' exits 1 with "unknown key 'nope'" and the list of nine keys, and leaves the file unchanged. Running without a parsed configuration holds both ways: on a file with an unknown field every other verb exits 1 refusing to parse it while 'ank config context_budget' answers 9000 at exit 0, and a write into it succeeds and warns 'still does not parse'. On genuinely malformed YAML the read falls back honestly -- 'ank config context_budget --json' answers {"key":"context_budget","value":"8000","source":"default"} where a readable line gives "source":"file", so the caller is told which it got.
