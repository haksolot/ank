---
id: LOG-2dc4d88c02a4
type: log
title: Test written red-first, and made to fail four different ways before it was allowed to pass.
created: 2026-08-31T08:56:00Z
author: claude-code/opus-5+licence
scope:
  - NOTICE
  - npm/ank/README.md
  - crates/ank-cli/tests/skill.rs
about: TASK-9b82a1edd42e
seq: 1
schema: 4
version: 1
---



the_relicensing_boundary_is_stated_the_same_way_everywhere in crates/ank-cli/tests/skill.rs walks the tree the way declared_licences() does (target, node_modules, .git and .ank excluded, non-UTF-8 files skipped by failing to decode rather than by an extension list) and reports every line naming an N.N.N release with the word GPL within two lines either side. A window and not a whole file, measured: crates/ank-cli/Cargo.toml declares version 0.7.0 eleven lines above its licence comment and is untouched, and so are install.sh, install.ps1, release.yml and docs/getting-started.md, which name 0.2.0 as an install example. On this tree the walk returns exactly three sites and no false positive.

Four measured reds, each with the rest of the fix in place:
1. NOTICE saying 0.2.0 -> FAILED, naming NOTICE:18 and the version.
2. NOTICE corrected, npm/ank/README.md still 'until 0.3.0' -> FAILED on the one-sided rule: a file that states the boundary must name both ends, because one release alone reads as inclusive or exclusive and the two documents were read the two different ways.
3. A file that does not exist yet, docs/licence-mutant.md with 'GPL-3.0-only up to and including 0.2.0' -> FAILED naming it. So the check is a walk and not a list of two paths.
4. NOTICE truncated to its Apache header -> FAILED: neither NOTICE nor npm/ank/README.md may fall silent, since an assertion over an empty walk passes.

The test file is inside its own walk: a probe line '// probe 0.9.9 GPL' appended after the constants FAILED at skill.rs:1085. The two constants sit in one window with the word GPL so that this file states the boundary whole and is held to the rule it enforces.

Green after: cargo test -p ank-cli --test skill, 25 passed 0 failed; cargo fmt --check clean; ank check 0 faults.
