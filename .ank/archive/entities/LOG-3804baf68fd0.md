---
id: LOG-3804baf68fd0
type: log
title: Fixed by moving the inbound reader from serde_yaml to serde_json (crates/ank-mcp/Cargo.toml,
created: 2026-08-31T09:10:25Z
author: claude-code/opus-5+mcp
scope:
  - crates/ank-mcp/Cargo.toml
  - crates/ank-cli/tests/mcp.rs
  - crates/ank-mcp/src/**
  - crates/ank-mcp/tests/**
about: TASK-1bc1186ad9e7
seq: 3
schema: 4
version: 1
---

 src/lib.rs); serde_yaml stays for corpora.yml, the one document here that is YAML, and corpora.rs's read of ank status --json moved to the JSON reader too. Measured after, same file on stdin to the rebuilt binary: {"id":"<emoji>-alpha","result":{}}, {"id":"e-beta","result":{}}, {"id":"<CJK>-cjk","result":{}} and, for the duplicate key, {"id":3,"result":{}} -- four replies, no -32700, no id:null. The argument path measured on a throwaway corpus: tools/call ank_new with "title":"\ud83d\ude00 an escaped title" answered exitCode 0, and ank find --type task --json read the title back as U+1F600 followed by the words, so the pair decoded to the code point and did not merely parse. Red-first: with the four source files reverted and the new test kept, a_request_that_escapes_a_non_bmp_character_is_read_and_not_refused fails at mcp.rs:903 on the exact original message, 'found invalid Unicode character escape code at line 1 column 29', and the reverted binary answered the duplicate-key line -32700 'duplicate entry with key \"id\"'. Restored and green: cargo test --workspace 0 failed across every target, cargo fmt --check exit 0, ank check exit 0 with no fault. crates/ank-mcp/tests/dependencies.rs asserts the direct dependency list, so the scope was amended to cover it; no golden file moved.
