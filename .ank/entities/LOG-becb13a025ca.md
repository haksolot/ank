---
id: LOG-becb13a025ca
type: log
title: "measured before writing any of it: in this repository 62 of 64 refs under refs/ank/ live in"
created: 2026-08-17T07:01:35Z
author: claude-code/2.1.233+integration-contract
scope:
  - viewer/**
about: TASK-34d27790dba9
seq: 0
schema: 3
version: 1
---

 .git/packed-refs and only 2 are loose ref files, and 62 of the 64 objects those refs point at are inside packfiles rather than loose. So the criterion's clause about reading loose refs and packed-refs alike is the easy half; the hard half it implies is that a claim ref yields a blob sha, and reading that blob from a browser means implementing enough of the git object database to inflate a loose object and to resolve one out of a packfile through its .idx, including OFS_DELTA and REF_DELTA chains. A loose-object-only viewer would display two claims out of sixty-four and look correct while being useless.
