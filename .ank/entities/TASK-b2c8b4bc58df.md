---
id: TASK-b2c8b4bc58df
type: task
slug: the-install-documentation-names-ank-skills-insta
title: The install documentation names ank skills --install on the npm route
created: 2026-09-13T09:21:28Z
author: claude-code/fable-5.1+planning
status: open
scope:
  - README.md
  - docs/getting-started.md
  - docs/agents.md
  - npm/ank/README.md
blocked_by: [TASK-544ec9655570]
done_criteria: |
  The README's install block reads npm install -g @haksolot/ank then ank skills --install, with the comment that the second installs the skills offline from the binary; the sentence that the skill is not the binary survives. docs/getting-started.md and docs/agents.md carry ank skills and ank skills --install with real output pasted from a run, and npx skills add haksolot/ank stays listed as the route for a machine with node and no ank. npm/ank/README.md says the skills an agent loads come from the binary. Every command written has been run before it is written down, and docs/agents.md's list of routes has one line per route with no prose between them.
criteria_by: creator
verify: [cargo-test, fmt-check]
schema: 4
version: 2
---

TASK-207e07504dcd set the shape of this section and its rule: a route written
down before it is run is a route that does not work. Run each one, paste what
came back, and keep `npx skills add haksolot/ank` as a route rather than
deleting it: it still serves whoever has node and no binary.
