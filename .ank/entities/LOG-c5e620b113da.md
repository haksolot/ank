---
id: LOG-c5e620b113da
type: log
title: "The fourth route is now runnable and was run as written. v0.1.3 published: npm view reports version"
created: 2026-08-08T17:47:45Z
author: seanl@sean-laptop
scope:
  - README.md
  - docs/getting-started.md
about: TASK-207e07504dcd
seq: 1
schema: 3
version: 1
---

 0.1.3 with the pi-package keyword, the published tarball carries package/skills/ank/SKILL.md, pi install npm:@haksolot/ank succeeds against the registry, and pi's loader resolves one skill named ank from what pi installed. One scare worth recording: the published skill's sha256 did not match the local file. It matches the git blob exactly. The difference is CRLF -- there is no .gitattributes rule for skill/, so a Windows checkout holds CRLF while the ubuntu publish job holds LF. The CI assertion compares tarball against the same runner's checkout, so it is right on every platform; only a cross-platform comparison like this one sees a difference, and it is not a defect.
