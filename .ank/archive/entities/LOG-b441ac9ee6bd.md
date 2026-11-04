---
id: LOG-b441ac9ee6bd
type: log
title: "Correction to LOG-9e65fb98ea70: the prototype's plugins were not left by an earlier run in a reused"
created: 2026-10-26T13:35:02Z
author: claude-code/opus-5.5+9ae9-bench-harness
scope:
  - bench/**
  - .github/workflows/ci.yml
about: TASK-9ae9b0959c9d
seq: 2
schema: 4
version: 1
---

 dir; its init events show plugins=13, skills=156 and an auto-memory path while the config started as credentials-only (plugins/synced, skills/synced): claude syncs the account's plugins into any config dir. --bare would stop it but also drops CLAUDE.md discovery and OAuth, so it cannot be used. The runner now passes --disable-slash-commands, and the scorer records claude_code version and the plugins/skills/mcp_servers counts from init, so a run that was not isolated says so in its own result line. Whether plugin hooks still fire under --disable-slash-commands is unmeasured: that needs a real run, which this task does not launch.
