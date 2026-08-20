---
id: LOG-b0c40c0f8db2
type: log
title: Three platforms, gix 0.86 against git 2.55, same 1024 entity blobs everywhere, warm run.
created: 2026-08-20T22:54:56Z
author: claude-code/opus-5
scope:
  - crates/ank-cli/tests/**
  - docs/**
  - spike/**
  - Cargo.toml
  - .github/workflows/**
about: TASK-a8054da67947
seq: 3
schema: 3
version: 1
---



                    linux    macos    windows CI   windows local
  spawn floor        0.7      4.2       14.7        29 to 40
  library open       0.3      0.3       25.2        53
  binary  open       0.9      4.4       16.8        37
  library blobs     38.5     25.5      160.8       424
  binary  blobs     54.1     85.1      453.8       990

Net saving on open plus blobs: about 17 ms on Linux, 64 ms on macOS, 285 ms on a Windows runner and 550 ms on the maintainer's Windows machine.

So the answer is unambiguous and it is not the one either of us expected: a git library is a Windows optimisation and very nearly nothing anywhere else. On Linux it saves seventeen milliseconds, which is noise. The spawn floor is what drives it, 0.7 ms against 30 to 40, a factor of fifty.

Note that gix is slower to open a repository than a rev-parse on Windows, 25 ms against 17 there and 53 against 37 locally, and faster on the other two. The open is paid once, so it matters less than it looks, but it is real and it eats a tenth of the saving on the platform where the saving exists.

The refs figures from CI are worthless and must not be quoted: a fresh clone carries no refs under refs/ank, so both readers were timed against zero refs. Only the local run has real ones, 95 of them, where the library read them in 3.4 ms against 56.

The conclusion an ADR would have to argue against: TASK-2ba2619b90e2 removes most of these blob reads, replacing them with a comparison of object names. After it, the library would be left winning tens of milliseconds on Windows and single digits elsewhere, for a large dependency, a workspace-wide serde unpin, and a wide rewrite. On these numbers I would not write that ADR. It is a decision for a human, and the number is now in the corpus rather than in an argument.
