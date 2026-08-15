---
id: LOG-f30b7a704ee7
type: log
title: Implemented in body_of, which now takes the entity kind so the refusal can name the flag form to
created: 2026-08-11T00:57:34Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
about: TASK-8e7c8e7724ee
schema: 3
version: 1
---

 retype. Three decisions settled. (1) Trailing newline: absorbed, not stored. The existing trim() runs on the piped text exactly as it ran on a flag value, so a heredoc and a flag produce the same file byte for byte; the integration test pins it by creating the same body through both channels and comparing. Storing the newline verbatim would have made the input channel visible in the corpus. (2) The same spelling on --criteria is left out: the flag is read in three places here plus claim, so giving it to new alone would be an inconsistency, and giving it to all of them is a surface question of its own rather than something that falls out for free. (3) Nothing to read is refused with code 9, as an unset EDITOR is, for the empty pipe and for a terminal alike: the prose channel the caller named is unavailable, and there is one fix for both. Noticed on the way, not fixed here: this very message could not be typed starting with two dashes, because there is no end-of-flags separator.
