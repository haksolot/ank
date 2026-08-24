---
id: TASK-b3306bccf09d
type: task
slug: a-client-is-told-how-to-reach-the-protocol-surfa
title: A client is told how to reach the protocol surface, in a form it can paste
created: 2026-08-24T21:58:27Z
author: claude-code/opus-5+planning
status: done
scope:
  - docs/getting-started.md
  - docs/integrating.md
  - README.md
blocked_by: []
done_criteria: |
  docs/getting-started.md states that installing ank installs the protocol surface with it, and carries a working MCP client configuration for Claude Code, Claude Desktop and Cursor, each naming --repo explicitly so the corpus a process speaks for is never implicit. docs/integrating.md states what the surface is: every verb COMMANDS carries, generated from that table, one process for one corpus, a refusal carrying the CLI exit code as its reason, and no claim taken that the CLI would not have taken in that clone. Every entity id cited in the new prose resolves under ank check, and check reports no finding on the three files. cargo test is green.
criteria_by: creator
proof:
  - type: commit
    ref: 6f5dadfda46bf26111d400c2db7303a0a4e11f05
    criteria: 2f868a43d968
    via: submitted
schema: 4
version: 3
---

The third thing ADR-e39a44f80e0e requires, and the only one a reader ever sees.
A surface that ships in every archive and is documented nowhere is reachable
only by someone who already knew it existed.

**The configuration is paste-ready or it is not documentation.** An MCP client is
configured by a JSON block, and a reader who has to derive one from prose will
derive it wrong: the failure mode is a server started in the wrong working
directory, speaking for whatever corpus the client happened to launch in. `--repo`
is therefore written out in every example rather than left to default, which also
puts ADR-372b82af1ec7's "one process speaks for one corpus" in front of the
person for whom it has consequences.

**What integrating.md owes is different from what getting-started.md owes.** One
is a person wiring a client in ninety seconds. The other is a reader building
against the contract, who needs to know that the surface cannot drift from the
CLI because it is generated, that a refusal is the CLI's own refusal with the
CLI's own code, and that claims stay per repository so a deployment over several
repositories is several servers. That second reader is exactly who ADR-894defc26f3d
made this documentation for.

This task is deliberately not blocked: prose about a surface that exists can be
written while the pipeline that ships it is being changed.
