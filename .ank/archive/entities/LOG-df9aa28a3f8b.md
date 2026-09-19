---
id: LOG-df9aa28a3f8b
type: log
title: Chose the refusal over strip-and-say. Stripping would write a glob the caller did not type, and a
created: 2026-08-25T04:02:49Z
author: claude-code/opus-5+amend-plus
scope:
  - crates/ank-cli/src/human.rs
about: TASK-86babab0eb1b
seq: 3
schema: 4
version: 1
---

 leading + is a real filename in the wild (SvelteKit's +page.svelte), so the strip makes a legitimate path unwritable while claiming it worked. The refusal has an escape the normaliser already provides: normalize_path drops a leading ./ segment, so --scope "./+page.svelte" stores +page.svelte, and the hint can name it. The precedent is one screen above in amend itself, on --reference against a non-spec: a flag silently ignored teaches the caller it worked.
