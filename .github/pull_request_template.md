<!--
Keep this short. The task carries the reasoning, the commit carries the change,
and this template only asks for what neither of them says.
-->

## Which task does this close

<!--
The id, from `ank find --status open`. If there is none, say why: a typo fix
does not need one, a behaviour change does -- and a change with no task is a
change with no frozen criterion to measure it against.
-->

TASK-

## What it does, and why this way

<!-- One paragraph. If a ratified ADR constrains this perimeter, name it. -->

## The three gates

Run locally, on your platform. CI runs the same lines on Linux, macOS and
Windows.

- [ ] `cargo fmt --check`
- [ ] `cargo test --workspace`
- [ ] `cargo run -q --bin ank -- check` (exit 8 means findings, and findings are
      a failure; signals exit 0)

## If this touches the format

<!-- Delete this section if it does not. -->

- [ ] `docs/ank-spec-v1.1.md` first -- no field exists in the code without
      existing there
- [ ] then `crates/ank-core/tests/golden/`
- [ ] then the code

## Before you open it

- [ ] The behaviour a `done_criteria` describes is tested **through the binary**,
      not only through the function meant to produce it
- [ ] `status:` was not edited by hand -- `ank done` runs the verifiers and
      writes the proof
- [ ] The MSRV number was not edited to make a build pass -- the floor is
      measured by walking toolchains, and a human runs the walk
- [ ] English everywhere, no emojis

Working from a fork? Your claim is local and invisible upstream ([why][fork]).
Say here which task you took.

[fork]: ../blob/main/CONTRIBUTING.md#working-from-a-fork
