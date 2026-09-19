---
id: LOG-642bdf091de3
type: log
title: Fixed at one choke point. ank_core::normalize_path does the lexical work -- separators unified to
created: 2026-08-03T06:18:40Z
author: seanl@sean-laptop
scope:
  - crates/ank-cli/**
  - crates/ank-core/**
  - docs/ank-spec-v1.1.md
about: TASK-df4c39031583
seq: 0
schema: 3
version: 1
---

 /, repeated and trailing ones collapsed, . dropped, .. resolved by popping, None when the path is absolute or climbs above the root -- and context::perimeter is the single place the four path-taking verbs read their argument. Normalising inside in_perimeter would have been cheaper and wrong twice: a refusal cannot be expressed through a bool, and scope echoes the perimeter it drew, which has to be the one it used. Found while wiring it: a third private copy of the matching, inside review's live-constraint loop, which is why review and context disagreed about docs backslash in the first place. It now calls in_perimeter like the others, so there is one implementation rather than three. .. is resolved rather than refused, since docs/../docs is docs and answering about a different perimeter was the defect. Case stays sensitive: folding it on Windows alone would give one corpus two meanings depending on the machine reading it. Two mistakes in my own verification, both caught by assertions I had written rather than by reading. Measuring the fix by hand, context reported 8 for every path form and looked fixed -- it was in execution mode because I held a claim, so it was ignoring the path entirely; the fixture holds no claim, and the test comment says why. Then the fixture itself failed the discriminator twice: review lists only accepted ADRs, so a proposed one made both perimeters answer zero, and check reports corpus-wide totals, so without a finding present on one side only it answered identically for every path. Both now real: the ADR is accepted, and a second ADR carries a dead scope under docs and nowhere else. Mutation: passing the raw path through the choke point fails the test on ank context docs backslash.
