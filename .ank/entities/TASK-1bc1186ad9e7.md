---
id: TASK-1bc1186ad9e7
type: task
slug: ank-mcp-refuses-valid-json-rpc-that-escapes-a-no
title: ank mcp refuses valid JSON-RPC that escapes a non-BMP character
created: 2026-08-31T07:52:05Z
author: claude-code/opus-5+drift2
status: in_progress
scope:
  - crates/ank-mcp/Cargo.toml
  - crates/ank-cli/tests/mcp.rs
  - crates/ank-mcp/src/**
  - crates/ank-mcp/tests/**
blocked_by: []
done_criteria: |
  ank mcp answers {"jsonrpc":"2.0","id":"\ud83d\ude00-alpha","method":"ping"} with a result whose id is the same string the client sent, never with -32700, and answers a request that repeats a key rather than failing to parse it; a test in crates/ank-cli/tests/mcp.rs drives the binary with both lines and fails on any -32700.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 4
---

Measured on 2026-08-31 by feeding lines to the binary and reading what came back.

`crates/ank-mcp/src/lib.rs:158` parses every inbound JSON-RPC message with
`serde_yaml::from_str`, on the argument the crate manifest states: "YAML 1.2 is
a superset of JSON, so a request on one line parses as flow YAML". Measured, it
is not. Eight requests were sent; two valid JSON documents were refused:

  sent  {"jsonrpc":"2.0","id":"😀-alpha","method":"ping"}
  got   {"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":
        "parse error: found invalid Unicode character escape code at line 1
        column 29, while parsing a quoted scalar at line 1 column 26"}}

  sent  {"jsonrpc":"2.0","id":3,"method":"ping","id":3}
  got   {"jsonrpc":"2.0","id":null,"error":{"code":-32700,"message":
        "parse error: duplicate entry with key \"id\""}}

The same request with the emoji as literal UTF-8 answers
`{"jsonrpc":"2.0","id":"<emoji>-alpha","result":{}}`. The difference is the
escape form, not the character: RFC 8259 encodes a non-BMP code point as a
surrogate pair, and YAML's `\u` escape is a single 16-bit unit with no pairing,
so serde_yaml rejects it. The five other cases passed -- `é`, `\t`, `\/`,
literal UTF-8, and a `<<` key.

This is not an exotic input. `json.dumps` in Python's standard library escapes
every non-ASCII character this way by default (`ensure_ascii=True`), so a client
written against the stdlib cannot send an emoji, a CJK character or an accented
name through any tool this surface offers: `ank_log`, `ank_new`, `ank_close
--reason`. ADR-fd98f4bc6dea requires the surface to refuse on state exactly as
the CLI does; the CLI accepts these strings and this surface does not, and it
answers `id: null`, so the client cannot even attribute the failure to its own
request.

The tree already holds the argument against this instrument: `ank-tui`'s manifest
replaced `serde_yaml` with `serde_json` for reading `--json` precisely because
the superset claim is about the grammar and not about the resolver. `ank-mcp`
was not brought along. `serde_json` is already in this workspace's lockfile.

Note that ank's own writer is not affected: `ank_contract::json::string`
(crates/ank-contract/src/json.rs:32) escapes nothing above U+001F, so the
documents the CLI emits and this crate parses back at corpora.rs:289 carry no
surrogate pairs. The defect is on the inbound path alone.
