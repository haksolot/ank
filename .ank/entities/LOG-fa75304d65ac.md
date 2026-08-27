---
id: LOG-fa75304d65ac
type: log
title: Measured. With commands.rs alone, json_golden_reading_verbs goes red on find; blessing the golden
created: 2026-08-27T17:19:51Z
author: claude-code/opus-5+find-created
scope:
  - crates/ank-cli/src/commands.rs
about: TASK-b917fc12fee8
seq: 2
schema: 4
version: 1
---

 then turns every_golden_conforms_to_the_shape_its_verb_declares red with 'keys are [contract, corpus, total, shown, hidden, results], the shape declares [contract, total, shown, hidden, results]'. So the criterion needs three files this task does not declare: crates/ank-contract/src/verbs.rs for the two field declarations, crates/ank-cli/tests/golden-json/find.json for the blessed fixture, and crates/ank-cli/tests/cli.rs for the measurement through the binary. The precedent, TASK-652de6ead019, declared exactly that file set for the same document. crates/ank-tui/src/model.rs needs nothing: it reads the rows through ank::rows(&found, 'results'), which ignores keys it was not asked for. Stopped rather than widening.
