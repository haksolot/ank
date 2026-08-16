---
id: LOG-c1ac1f673e1a
type: log
title: The slim Debian images throw the documentation away as they unpack, and the first run of the
created: 2026-08-16T17:33:09Z
author: claude-code/opus-5
scope:
  - packaging/deb/**
  - .github/workflows/publish-apt.yml
about: TASK-07e021f95aa3
seq: 1
schema: 3
version: 1
---

 install job reported that as a defect in the package. debian:stable-slim ships /etc/dpkg/dpkg.cfg.d/docker carrying path-exclude=/usr/share/doc/*, with copyright excepted -- which is exactly the shape of what was observed: copyright present, README.md and SKILL.md absent, and dpkg -L ank listing /usr/bin/ank alone. The package was carrying all three the whole time.

Two changes, and the second is the one that matters. The image's exclusion is now removed by name before installing, printed rather than silently rm'd, so the container behaves like a Debian install instead of like a container. And the assertion is made twice against two different things: dpkg-deb -c reads the package itself, which is the claim the criterion actually makes and which no dpkg configuration anywhere can change, and the filesystem check is the other half, that a real install lands them. Only the second existed on the first run, and it measured the environment rather than the artefact.

Everything else on that run was already true: five packages built from five releases, signed with the distribution key 307CF740...E0C1 out of Actions secrets, the negative control refusing an unverifiable repository and saying it was the signature, apt-get install serving 0.2.0, and ank --version answering "ank 0.2.0 (c86eeeb, skill 3f350ad26459)".
