---
id: LOG-117ed8f4b43d
type: log
title: "reproduced: ank mcp --repo . over stdio, three tools/call on ank_find. {\"json\":true} -> -32602"
created: 2026-09-20T17:45:52Z
author: claude-code/opus-5+308c
scope:
  - crates/ank-mcp/**
  - crates/ank-mcp/tests/**
about: TASK-308ce062f427
seq: 3
schema: 4
version: 1
---

 '--json belongs to the server: name a corpus with the corpus argument, by the identity ank status --json prints, never by a path'. {"quiet":true} -> the same sentence with --quiet. {"repo":"/somewhere/else"} -> the same sentence with --repo. Three flags, one message, and the message is --repo's: 'never by a path' says nothing about --json and 'name a corpus' says nothing about --quiet.
