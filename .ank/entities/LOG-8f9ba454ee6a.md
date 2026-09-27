---
id: LOG-8f9ba454ee6a
type: log
title: "Green through the binary: 5 URL shapes (https, ssh, file, git schemes; scp git@github.com:x.git) x"
created: 2026-09-27T19:59:32Z
author: claude-code/opus-5.5+bcdc
scope:
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/peer_url_refusal.rs
about: TASK-bcdc32d08947
seq: 3
schema: 4
version: 1
---

 2 peer names refused by 'ank config peers.<name>' = 10 refusals, and the same 5 by 'ank config --user peers.<id>.bb' = 5, every one exit 1 with config.yml / corpora.yml byte-identical (fs::read before == after) and stderr naming 'ank config --user peers.<id>.<name> <path>' with the declaring corpus's real root commit filled in. 5 paths (../b, sub/dir/b, absolute, C:/src/b, C:\src\b) accepted by both writers and read back verbatim = 10 accepts. Refusal is taken before the file is read; the identity git call runs only on the refusal path. In a repo with no commit the hint falls back to <identity> (checked by hand, exit 1).
