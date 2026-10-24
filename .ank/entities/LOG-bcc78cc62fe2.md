---
id: LOG-bcc78cc62fe2
type: log
title: "Self-review. Against the criterion: mise installed by a step identical to ci.yml's 'mise' step"
created: 2026-10-24T10:02:45Z
author: claude-code/opus-5.5+w1fc7
scope:
  - .github/workflows/docs.yml
about: TASK-1fc7acf345db
seq: 3
schema: 4
version: 1
---

 (diffed: only the if: env.RUN guard and the tool installed differ), MISE_VERSION 2026.10.17 pinned, sha256 per runner OS read from stdin, no third-party action; mdBook from 'mise install --locked github:rust-lang/mdBook', build runs 'mise x -- mdbook build'; grep finds no MDBOOK_VERSION/MDBOOK_SHA256 left; file list compared in LOG-fbe3c46211f3. Nothing outside the criterion in the diff. Against the constraints (ank context on docs.yml): ADR-33970fcdb6e8 still holds (site built from docs/ on every PR, published from main only, deploy job untouched); ADR-221a/8b30/1ea3/e1d7 untouched by a workflow that feeds no channel. Noted, not acted on: the link check runs the runner image's python3, which mise.toml does not declare; ADR-9c69bd47ce23's 'every other tool reaches a workflow through mise' could be read to cover it, but the criterion names mdBook only, so it stays out of this diff.
