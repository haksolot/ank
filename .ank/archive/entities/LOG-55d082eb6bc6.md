---
id: LOG-55d082eb6bc6
type: log
title: "discrepancy: ADR-e1d7 states ank skills --install needs no network and attempts none; measured"
created: 2026-09-13T13:32:48Z
author: claude-code/b2c8
scope:
  - README.md
  - docs/getting-started.md
  - docs/agents.md
  - npm/ank/README.md
about: TASK-b2c8b4bc58df
seq: 3
schema: 4
version: 1
---

 2026-09-13, ank attempts none, but npx skills add <dir> with an empty npm cache (npm_config_cache pointed at a fresh scratch dir) downloaded the skills CLI, 7.9M under _npx, before installing. Behind a firewall with no cached skills CLI and no registry mirror the route would stop at npx. docs/agents.md states the measured fact (npx fetches the skills CLI the first time it has none cached); the README comment keeps the criterion's word offline, which holds for the skill files. For ank-drift or planning, not this task.
