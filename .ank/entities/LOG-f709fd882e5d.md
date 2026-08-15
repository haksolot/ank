---
id: LOG-f709fd882e5d
type: log
title: "Read the ground before writing. The renewal write exists twice already: commands.rs log_write"
created: 2026-08-13T23:10:18Z
author: claude-code/12db
scope:
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/config.rs
  - crates/ank-cli/tests/cli.rs
  - docs/ank-spec-v1.1.md
about: TASK-12db5686c024
schema: 3
version: 1
---

 recomputes renewal_ttl + put inline, and claim::retake does the same three lines for a lapsed claim. So the single place this task asks for is not a new third copy but one extracted claim::renew(cwd, id, object, record, cap) that both of those call and the dispatcher hook calls too. Where the rule is tested: cli.rs::dispatch, after the verb returns, keyed on a new CommandSpec field -- the same argument the coordinates field already makes in its own doc comment, that a property of a verb declared beside the verb makes the compiler ask the question of every verb ever added, where a list beside the dispatch lets a new verb default to silence. context is Held (it is about the task in hand and nothing else), show/amend/attest/edit are Named (renew only when their <id> resolves to the held task), and claim/log/done/release are Never because each settles the lease itself. Two decisions the criterion does not spell out and I am taking: a lapsed claim is not renewed by a passive verb -- taking one back stays the re-acquisition log and done perform, or reading would silently retake a claim section 3 gives to the two verbs that write; and the renewal is silent, errors included, because show and edit are coordinates:false and must keep working outside a usable git.
