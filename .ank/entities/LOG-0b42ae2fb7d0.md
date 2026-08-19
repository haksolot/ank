---
id: LOG-0b42ae2fb7d0
type: log
title: "SPEC-b156a5571668 supersedes SPEC-80bff12ceae8, proposed, one paragraph apart: a body diff of the"
created: 2026-08-19T22:14:14Z
author: claude-code/opus-5
scope:
  - .ank/entities/SPEC-80bff12ceae8.md
about: TASK-659ebaa4f68e
seq: 0
schema: 3
version: 1
---

 two documents reports exactly one changed line, the binary-distribution paragraph, and everything else travels unchanged. Two things measured rather than assumed. The body of this task expects citing documents to need amend --reference afterwards, as every supersession before it did; nothing cites SPEC-80bff12ceae8 in a references field, and a grep over .ank/entities finds it only in its own file, in the successor's supersedes, in this task and in three log entries, so no re-pointing is owed and check answers ok with no unresolved reference. And the clause 'names no channel that does not ship' is read literally: the paragraph names none of Homebrew, Scoop, apt, winget or the AUR, not even as withdrawn, because the defect it repairs was a sentence naming winget and Arch as pending and a reader acts on a named channel. It says instead that no package manager ships ank and that none is named here, so the omission reads as deliberate, and it cites ADR-221aa5da440a for the measurement. The scope drops Formula/**, bucket/** and packaging/**, which no longer exist, and gains install.ps1.
