---
id: LOG-bfc81b7ee0f9
type: log
title: "First green-path run panicked through the binary at git.rs output(): 'porcelain forbidden"
created: 2026-10-20T12:44:28Z
author: claude-code/6e3f
scope:
  - crates/ank-cli/src/**
  - crates/ank-cli/tests/**
about: TASK-6e3f8f0049e9
seq: 3
schema: 4
version: 1
---

 (ADR-9307e5d214a7): [-c, gpg.ssh.allowedSignersFile=..., verify-tag, --raw, <tag>]', exit 101. verify-tag was not in PLUMBING. Admitted on ADR-9307's criterion, as ls-files was before it: git classifies verify-tag beside verify-commit (already admitted), ADR-5e47007a75be and the frozen criterion name it as the verifier, and what is parsed is its --raw stderr, gpg's [GNUPG:] status protocol or ssh-keygen's 'Good "git" signature [for <principal>] with <type> key <fp>' line, the same text git parses for %G?. Noted for whoever amends ADR-9307's list next.
