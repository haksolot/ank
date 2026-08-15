---
id: LOG-19fe7cc9950f
type: log
title: Three claims in the status section are false, not one, and the walk to check the verb count is what
created: 2026-08-04T05:32:35Z
author: seanl@sean-laptop
scope:
  - README.md
  - docs/getting-started.md
about: TASK-143c90665ed4
seq: 0
schema: 3
version: 1
---

 found the other two. Binaries ship: release v0.1.1 (2026-08-03) carries ank-0.1.1 for aarch64-apple-darwin, x86_64-pc-windows-msvc and x86_64-unknown-linux-musl, each with a .sha256, while the README still says there are none and sends the reader to cargo build --release. The skill is installable and installed -- npx skills add haksolot/ank is in SKILL.md and this session loaded it -- while the README lists it as missing before v1. And the verb count: ank help lists eighteen verbs plus init and help, not thirteen, split across an agent surface and a human one that ADR-c656cbcc33a9 withdrew. docs/getting-started.md repeats the binaries error, since I wrote it yesterday from the README rather than from the releases page.
