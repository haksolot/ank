---
id: ADR-96fe1f9d619a
type: adr
slug: a-peer-is-declared-by-path-never-by-remote-url-a
title: A peer is declared by path, never by remote URL, and the reader's override is what makes it portable
created: 2026-09-27T17:05:30Z
author: claude-code/opus-5.5+c6d1
status: proposed
scope:
  - crates/ank-cli/src/repo.rs
  - crates/ank-cli/src/config.rs
  - docs/**
constraint: |
  peers.<name> in config.yml names a filesystem path, resolved against the declaring root, and never a remote URL: no verb clones, fetches or caches a peer, and a peer is read only from a checkout that already exists on the reader's disk. A layout that differs is answered by the reader's override of ADR-da2819aef598, never by a URL. ank config refuses a peer value shaped as a URL (a scheme followed by ://, or user@host:path) at the moment it is written, naming the override, because a URL on a peer's back-declaration otherwise makes a binding vanish with no warning.
schema: 4
version: 1
---

Raised by issue #500, studied under TASK-c6d184d238e1, whose log holds the four
experiments this rests on. The issue asks that `peers.<name>` accept a remote URL,
so that a committed `config.yml` works on a clone with any layout without every
reader writing an override. The study measured what a URL would have to buy, and
refuses it.

## What a URL would need, measured

A URL names nothing on disk. For a verb to read it, somebody has to clone it
somewhere, keep that clone fresh, and match it back to the reader. Each of the
three was tried, on git 2.55.0 against this repository's own remote (2402 files
under `.ank/`), 2026-09-27.

**A clone that is cheap is not a faithful corpus.** Four strategies, git
processes counted with `GIT_TRACE`:

| clone | `.git` bytes | processes | identity | `ank check` |
|---|---|---|---|---|
| full | 10 520 825 (+17.4 MB tree) | 9 | root, correct | green |
| `--depth 1` | 4 995 377 | 9 | **the tip**, wrong | - |
| `--filter=blob:none` | 7 715 664 | 20 | correct | - |
| blobless, sparse `.ank/` | 5 500 750 (+10.7 MB tree) | 22 | correct | **exit 8, 1262 faults** |

A shallow clone reports the tip as its root commit, so the ADR-621a7fd96ce1
identity every corpora.yml row is keyed on is wrong in it. A sparse clone of
`.ank/` alone makes every local scope dead, because a glob is confronted with a
filesystem that was deliberately not checked out. The only clone that is right is
the full one, about 28 MB here: a second checkout of the peer, which is what the
reader who writes an override already has.

**Freshness costs the network on every read, or it is not known.** `git
ls-remote origin HEAD` is 4 processes and a round trip; a fetch that brings
nothing is 6. ADR-64f32c74a0f9 reserves the network to a call that asked for it,
and ADR-24e21cb83793 lets the daemon fetch `refs/ank/*` and nothing else, never a
branch or a working tree, so neither may keep a peer's checkout current behind
the reader's back. A URL peer would therefore need its own verb (`ank peers
fetch`, say) and would be exactly as fresh as the last time somebody ran it, with
nothing in the tree saying how old that is. ADR-c23bef1cc93e makes that matter: a
`blocked_by` naming `<id>@<peer>` releases work on what the peer's checkout says,
and a stale clone answers about a tree nobody is looking at. A sibling checkout
has the same property, but it is the one its owner works in and fetches as a
matter of course; a clone under a cache directory is the one nobody looks at.

**The back-resolution cannot use a URL, and fails in silence.** A binding is
resolved twice (ADR-da2819aef598): the reader opens the peer through its own
declaration, and the peer resolves its scope `aa:src/**` back to the reader
through the peer's declarations (`Peer::binds`). Measured through the binary on
two fresh repositories: with paths both ways, `ank context` in the reader lists
the peer's ADR; with the peer's back-declaration set to `file://<reader>`,
`ank config` accepts the value without a word, and the ADR disappears from the
reader's context **with no warning at all**. A URL on the reader's own declaration
at least warns, "is not a corpus". Matching a URL to the reader would need the
reader's identity, and a URL does not carry one: `ls-remote` lists refs, never the
root commit. It would need a clone, per peer, per resolution.

**A URL is not portable either.** `https://github.com/haksolot/ank` and
`.../ank.git` reach the same repository; `git@github.com:haksolot/ank.git` and
`ssh://git@github.com/haksolot/ank` fail from the machine the study ran on, for
want of a key. One committed string works for the readers whose credentials match
it. A relative path depends on each reader's layout, a URL on each reader's
credentials, and neither is the property the issue wanted.

## Why the override is enough

What a URL adds over ADR-da2819aef598 is one thing: a reader whose layout differs
would not write one line in their own corpora.yml. Against that it brings a clone
location, a fetch verb, a freshness nobody can see, a second identity rule for the
back-resolution, and credentials in a committed file. The override needs no
network, is keyed on identity rather than a string, is read at both ends of a
binding, and points at the checkout the reader actually works in. The reader who
lacks the peer has to clone it once either way; with the override they clone it
where they want it and say where, and ank never owns a copy of anybody's code.

This refusal leans on ADR-da2819aef598. If that decision is not ratified, the
layout problem of #500 is open again and this document should be reread rather
than cited.

## What the constraint adds

Today `ank config peers.<name> <url>` is accepted, and on the back-declaration the
binding then vanishes silently: the failure this study found is live. Refusing the
value where it is written is the cheap end of that. Recognising a URL by its
shape is enough: a scheme followed by `://`, or `host:path` with an `@` before the
colon. A Windows drive letter (`C:\...`) is neither and stays a path. The refusal
names the override, so the caller who meant "the peer lives elsewhere" is sent
to the verb that says it.

## Rejected

- **Admit the URL, clone under a cache, fetch by an explicit verb.** Priced
  above: a second checkout ank owns, a freshness only a verb can restore, and a
  back-resolution that needs that clone to answer at all.
- **Admit the URL as a hint beside the path**, used only to name a
  `git clone` in the "not a corpus" warning. It would put credentials-shaped
  strings in `config.yml` for the sake of a message; the peer's name and the
  override already say what to do.
- **Resolve the back-declaration by comparing URLs to the reader's remotes.**
  Four spellings of one repository measured above, two of them unreachable here;
  a textual match is the label ADR-a1de673043b4 refused for scopes.
