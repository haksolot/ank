---
id: TASK-7cb77bc870b1
type: task
slug: the-mcp-server-passes-worktree-through-and-a-cal
title: The MCP server passes --worktree through, and a caller writes a path into the address
created: 2026-09-20T17:55:46Z
author: claude-code/opus-5+308c
status: done
scope:
  - crates/ank-mcp/**
blocked_by: [TASK-308ce062f427]
done_criteria: |
  Over MCP, a tools/call carrying "worktree" is refused with a reason that names the flag, in the same shape the other withheld globals are refused in; the three already withheld keep the reasons they have. A test drives ank mcp over stdio for it.
criteria_by: creator
verify: [cargo-test, fmt-check]
proof:
  - type: test
    ref: local/17e3d8624833@12c606f
    tree: scope/a96fed90c1a3
    criteria: f8994afea5ad
    verifier: cargo-test@f14aeab36e1b
    via: verifier
  - type: test
    ref: local/e3b0c44298fc@12c606f
    tree: scope/a96fed90c1a3
    criteria: f8994afea5ad
    verifier: fmt-check@5ca6d10bcd55
    via: verifier
schema: 4
version: 3
---

Measured 2026-09-20 during TASK-308ce062f427, which the task body asked to decide this during. Over 'ank mcp --repo .': {"worktree":"/tmp"} on ank_find is accepted and the call runs -- 618 results came back, no comment. {"worktree":"/no/such/dir"} on ank_status comes back exitCode 1, 'error[1]: --worktree /no/such/dir is not a directory', so the value reaches the CLI and is interpreted there.

--worktree is a GLOBAL_FLAGS entry (crates/ank-contract/src/verbs.rs:510). It takes a path, and ADR-9e56318631f3 makes it the second half of an address: --repo says which corpus, --worktree says which tree that corpus is anchored to. tools::SERVER_FLAGS withholds --repo in so many words because 'a caller that could write a path into a flag would reach every corpus on the machine, which is exactly what turns a declared set into a merged one'. --worktree writes a caller's path into the same address and is not withheld. crates/ank-cli/src/done.rs:478 runs every declared verifier in repo.worktree, so the flag is not only a read.

Verdict recorded in TASK-308ce062f427's log: not intended. Left out of that diff because the criterion frozen there names three flags and a diagnosis widens into a new task rather than into itself.

After TASK-308ce062f427 lands, SERVER_FLAGS is a table of (flag, reason) and withheld() reads it, so this is a fourth row plus a stdio case in crates/ank-mcp/tests/withheld.rs. Worth checking at the same time whether the refusal should be the schema's too: --worktree is a global and input_schema already skips what client_flag hides, so a fourth row removes it from every advertised tool as well -- confirm that is wanted rather than assumed.
