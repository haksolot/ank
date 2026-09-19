---
id: LOG-8cd950e0ba89
type: log
title: "Codepage measurement, held while the claim is live. Encoded through .NET and decoded back: cp437"
created: 2026-08-30T14:42:11Z
author: haksolot@vmi3223161
scope:
  - install.sh
  - install.ps1
  - .github/workflows/install.yml
about: TASK-54c95c5f2d18
seq: 1
schema: 4
version: 1
---

 U+2588 -> 0xDB -> U+2588 survives; cp437 U+2713 -> 0xFB -> U+221A, a square root; cp1252 U+2588 -> 0xA6 -> U+00A6, a broken bar; cp65001 both survive. No failure produces 0x3F, so a question-mark test passes every one of them. This is why Test-GlyphSurvives round-trips instead of looking for '?', and it reversed the old comment in install.ps1 claiming conhost cannot draw the block.
