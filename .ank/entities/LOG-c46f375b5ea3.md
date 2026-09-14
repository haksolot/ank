---
id: LOG-c46f375b5ea3
type: log
title: "The seam for TASK-97fd1992567a (ank archive): Store::move_to_archive(id) renames the file from"
created: 2026-09-14T12:19:34Z
author: claude-code/opus-5+cold-rebuild
scope:
  - crates/ank-cli/src/store.rs
  - crates/ank-cli/src/index.rs
  - crates/ank-cli/src/commands.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/src/entries.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-contract/**
  - crates/ank-cli/tests/cli.rs
about: TASK-da978b214eca
seq: 4
schema: 4
version: 1
---

 where the store reads it hot to Store::archive_path_of(id) and returns the new path, refusing NotFound when the hot corpus does not hold the id and Io/AlreadyExists when the archive already does; bytes are unchanged, so the digest the index records at first sight is the one check verifies. Store::archived_ids() lists the archive by file name with no parse, for check's resolutions. Readers: Store::resolve_with_archive / load_with_archive / load_prefix_with_archive (hot wins), Index::open_with_archive (walks archive/entities last, rows carry Row.archived, Index::archived_digests() for check). Store::ARCHIVE_DIR = "archive/entities". Default Store::resolve/load/read_path_of/list_ids and Index::open are unchanged and never see the archive, so no writing verb can land on an archived entity.
