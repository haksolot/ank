---
id: LOG-a2a4b106ab4b
type: log
title: "Falsification before any refactor, as the body asks, on a fixture of two git repositories: a code"
created: 2026-08-21T18:10:51Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/cli.rs
  - crates/ank-cli/src/claim.rs
  - crates/ank-cli/src/done.rs
  - crates/ank-cli/src/context.rs
  - crates/ank-cli/src/verify.rs
  - crates/ank-cli/tests/cli.rs
about: TASK-a12553192afa
seq: 0
schema: 3
version: 1
---

 tree holding src/main.rs and no .ank at all, and a corpus tree holding nothing but .ank. The corpus is addressed with --repo from inside the code tree, which is the detached case reachable today. All four questions ADR-9e56318631f3 sorts were driven through the binary, and the four fail in four different ways, three of them more precisely than the ADR predicted.

Scope glob: 'scope src/** matches no file yet: work not started, or a typo'. The glob was confronted against the corpus directory, where no src/ exists, while src/main.rs sits in the tree the caller is standing in. On a task this is a signal, and the wording sends the reader to look for a typo that is not there. On an ADR the same confrontation is a fault, so a detached corpus would report every constraint it carries as a fault it cannot clear.

context and scope on a path: sharper than expected, and it splits in two. An absolute path of the code is refused outright, error 1, 'does not name a path in this repository', which is the caller naming the file they are actually editing. A relative path is accepted and answered, because glob matching is lexical: src/main.rs matches src/** as text, with nothing ever asking the filesystem whether that file exists. So the failure is not that the verb breaks, it is that one form is refused and the other returns an answer the tool never verified. Two callers, one of them silently trusting.

Verifier working directory: verifiers.in-the-code.run = 'test -f src/main.rs', declared in the corpus, task claimed from the code tree. Result: FAILED, exit 5, on a file that ls shows present in the caller's tree. The verifier ran in the corpus.

commit: proof: exit 5, 'commit d996e9db not found in this repository', naming the HEAD of the code repository. The work is in the code, the proof is a commit of the code, and it cannot be recorded at all.

The fixture is disposable and lives in the scratchpad; what it establishes is that the four are independently observable through the binary before --worktree exists, which is what makes the assignment testable rather than argued.
