---
id: LOG-e1b4ceca75e5
type: log
title: "macOS hang, head 4dda9b9 (lock and group restored, 26 minutes in the nextest step): the new test,"
created: 2026-10-17T20:59:12Z
author: claude-code/opus-5.5+adea
scope:
  - crates/ank-tui/tests/terminal/mod.rs
  - crates/ank-cli/tests/tui.rs
  - crates/ank-tui/tests/**
  - .config/nextest.toml
about: TASK-adea229723df
seq: 9
schema: 4
version: 1
---

 not the removal. Mechanism measured on Linux: the barrier-aligned test panicked on a failed open and left the other threads at Barrier::wait for ever; under ulimit -n 60 (posix_openpt answering EMFILE) it was still running when timeout 60 killed it, against 1.3s normally. The window it held was 64 masters (8 threads x 8), under macOS's 511, so the table-full guess is not established; what failed on macOS is not known, only that any failure hung. Fix: pty::try_open answers io::Result (posix_openpt, fcntl, grantpt, unlockpt, ptsname_r each named with errno), pty::open panics with that message and never retries, and tui.rs's asserts now carry the errno; the test records failures instead of panicking, every thread reaches every barrier, a stop flag read after the start barrier ends all threads on one round, on a failure or a 120s deadline, and it asserts failures, opens done and crossings separately. Window sized off pty::ceiling(): /proc/sys/kernel/pty/max read 4096 here, kern.tty.ptmx_max on macOS; held per thread = clamp(ceiling/8/8, 1, 8), so 8 here and 7 at 511. Numbers on Linux: ulimit -n 60 now fails in 0.006s naming 'posix_openpt: Too many open files (os error 24)'. 8 threads x 4000: fixed 0 red of 10 (1.3s), ptsname(3) read 6 red of 10 (1-3 crossed per 32000). 8 x 8000, kept for Linux: ptsname(3) read 9 red of 10 (1-7 crossed per 64000), fixed 0 red of 20 (2.8s). Non-Linux: 8 x 1000. The pty module and test type-check for aarch64-apple-darwin in a scratch crate (sysctlbyname path); not run on macOS.
