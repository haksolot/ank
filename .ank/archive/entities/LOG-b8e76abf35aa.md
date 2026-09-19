---
id: LOG-b8e76abf35aa
type: log
title: Two defects fixed together, because the measurement cannot separate them and both are wrong on
created: 2026-08-18T22:38:51Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/git.rs
  - crates/ank-cli/src/human.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-01cc22478782
seq: 0
schema: 3
version: 1
---

 their own terms.

The one that is almost certainly the cause: .ank/allowed_signers is read by two parsers and is only valid to one of them. Its gpg <fingerprint> entries are ank's extension for the OpenPGP branch, where SPEC-199de7ac4730 says git never opens the file; ssh-keygen has no keytype for them and answers '.ank/allowed_signers:23: invalid key'. On this machine it says that twice and still verifies, so how much of the rest it reads is a property of its version. git is now handed only the lines it can parse. Filtered and never re-rendered: an entry may carry options between the principal and the key, parse_signers drops them, and rebuilding the file from what it returns would hand git a permission the reviewed file does not grant -- there is a test for exactly that line. When the file needs no filtering, the source itself is handed over and nothing is written, so the ordinary corpus verifies against the file under review.

The smaller one: the -c pair was built with p.display(), so on Windows an absolute path full of backslashes went into a git config value, where a backslash opens an escape sequence. Forward slashes now, guarded by cfg!(windows) because a backslash is an ordinary character in a POSIX filename.

Local green proves nothing here and the criterion says so: 291 tests pass in ank-cli, ank check is ok, and none of that runs the code path that failed. The measurement is the ci workflow on three runners over a HEAD that is an SSH-signed ratification, which is what this branch is for. If Windows is still U after this, the cause is neither of the two and the next step is a diagnostic run printing git's stderr rather than another guess.
