---
id: LOG-4f3c6f0bed03
type: log
title: Counted after, the same way as before and through git rather than by eye. The baseline in
created: 2026-08-31T05:07:35Z
author: claude-code/opus-5+citC
scope:
  - crates/ank-cli/tests/cli.rs
  - crates/ank-cli/tests/skill.rs
  - crates/ank-cli/tests/watch.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/Cargo.toml
  - crates/ank-contract/Cargo.toml
  - crates/ank-daemon/Cargo.toml
  - crates/ank-mcp/Cargo.toml
  - crates/ank-tui/Cargo.toml
  - NOTICE
about: TASK-435d812d66bd
seq: 2
schema: 4
version: 1
---

 LOG-3a9cb6f5a787 says 36 occurrences; that number is wrong and the correct one is 37 -- I summed the per-id counts by hand and dropped one. Recounted against origin/main with str.count over the ten files: ADR-a22cd3196529 12, ADR-9f03438f5422 12, ADR-01b6dd05f0db 7, ADR-ff294eff4d1a 4, SPEC-fe8bdb84faca 2, ADR-85e6bbb195b8 0, total 37 on 36 lines. After: 0 retired ids in the perimeter, 34 citations of the successors -- ADR-24e21cb83793 12, ADR-534c7a3e6cf8 11, ADR-e45e1a29fe91 7, ADR-67a4ac10c534 3, SPEC-d58b3a9e4e4d 1. ank check answers 'ok -- 364 tasks, 84 adr, 456 signal(s)' at exit 0, with no fault of any kind and no citation finding against the six. cargo fmt --check exits 0; cargo test --workspace exits 0 across 46 test-result blocks with no FAILED line.

Three of the 37 were dropped instead of re-pointed, each because the sentence is a claim about a particular document rather than about the rule it carries, and a re-point would have made the sentence false.

crates/ank-cli/tests/skill.rs, the NOT_YET_DISPATCHED doc comment: 'while SPEC-fe8bdb84faca was still proposed and the block this suite walks was its predecessor' is a dated observation, and SPEC-d58b3a9e4e4d did not exist when mcp and watch were declared. The file argues against pinning an id here in its own words sixty lines above -- 'the document is found by what it carries rather than by its id, so a supersession that replaces it keeps this test green' -- so the id went and 'the section 4 document of the day' took its place. The ADR-a22cd3196529 on the same line was re-pointed; only the SPEC id was dropped.

crates/ank-cli/tests/cli.rs, over two_branches_recording_an_entry_merge_with_no_conflict: 'the case ADR-ff294eff4d1a celebrated most and the one it did not cover'. ADR-67a4ac10c534 keeps that clause word for word but argues the opposite about it -- it says the case is gone rather than uncovered, because ADR-25f977377fa0 made an entry a file. Naming the successor would have attributed to it an omission it addresses in as many words. The sentence now names the decision by what it did, and ADR-25f977377fa0 in the next clause anchors it.

crates/ank-cli/tests/skill.rs, over declared_licences: 'TASK-47beb64fd204 swept the tree when ADR-9f03438f5422 relicensed it'. ADR-534c7a3e6cf8 relicenses nothing: it carries the licence verbatim and says the licence is its predecessor's and does not move, changing only the channel list. TASK-47beb64fd204 still anchors the sweep, so the id was dropped and the act named plainly.
