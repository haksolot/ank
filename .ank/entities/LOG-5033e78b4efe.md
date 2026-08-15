---
id: LOG-5033e78b4efe
type: log
title: parse_entity normalises CRLF once, at the entry, so nothing downstream carries a line-ending case;
created: 2026-07-31T17:08:14Z
author: claude-code@ank
scope:
  - crates/ank-core/src/parse.rs
  - crates/ank-core/src/error.rs
  - crates/ank-core/src/lib.rs
  - crates/ank-core/tests/golden/**
  - crates/ank-core/tests/golden.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
  - .gitattributes
  - .github/workflows/ci.yml
  - README.md
about: TASK-aca0cb103980
seq: 0
schema: 3
version: 1
---

 the body is verbatim with respect to the normalised text, which is what normalised on first rewrite means. Error::CrlfLineEndings is the one diagnostic in ank-core carrying a command, because the cause is a git setting and a reader told only CRLF would edit a file git converts back on the next checkout. check tells the two deviations apart by asking whether dropping the carriage returns leaves the canonical form: if it does, signal and exit 0; if not, still a fault. Scope widened six times, each recorded in the body.
