---
id: LOG-ea858bf13aa6
type: log
title: "self-review. Criterion axis: each clause points at a test in tests/schema.rs -- schema 1 without"
created: 2026-10-24T14:42:51Z
author: claude-code/opus-5.5+w1091
scope:
  - crates/ank-core/src/config.rs
  - crates/ank-cli/src/config.rs
  - docs/config-keys.md
  - crates/ank-cli/tests/schema.rs
  - crates/ank-contract/src/verbs.rs
  - crates/ank-cli/tests/golden-json/help.json
  - crates/ank-core/src/reference.rs
  - crates/ank-cli/tests/cli.rs
  - crates/ank-core/tests/reference_pages.rs
about: TASK-109153abe8e0
seq: 10
schema: 4
version: 1
---

 key (a_schema_1_config_without_requires_ank_is_read_as_today), schema 2 with 1.2.3 (a_schema_2_config_with_requires_ank_is_read), key at schema 1 (requires_ank_at_schema_1_is_refused_naming_the_key_and_schema_2), 1.2/v1.2.3/1.2.3-rc1/word (a_requires_ank_not_of_the_form_..., read and write), schema 3 (a_schema_3_config_is_refused_as_newer_than_this_ank), write+schema bump+read back+unset keeps 2 (ank_config_writes_requires_ank_with_schema_2_and_unset_leaves_the_schema), page lists key (the_config_reference_lists_requires_ank); mutation 'never bump schema' reddens the write test. Hunks beyond the criterion: corpora.yml gets its own CORPORA_SCHEMA=1, required so raising config's max to 2 does not silently admit corpora.yml schema 2; the page note does not claim init writes the key, that is TASK-96fa's. Constraint axis over the touched paths: ADR-756e followed (exact form via update::Version::parse, the rule update applies to tags; reader accepts 1..=2; key declares 2); ADR-2b62 page regenerated from the table; ADR-6fd6 help.json changes only the keys note text, no shape; ADR-63b5 no spec enumerates config keys, nothing owed. Suite 1684/1684 green, fmt clean.
