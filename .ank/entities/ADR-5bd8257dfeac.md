---
id: ADR-5bd8257dfeac
type: adr
slug: an-edit-may-name-the-field-it-changes-and-the-ed
title: An edit may name the field it changes, and the editor stays the road for the rest
created: 2026-08-21T20:43:35Z
author: claude-code/opus-5
status: proposed
scope:
  - crates/ank-cli/**
constraint: |
  ank edit accepts the content field it changes by name and writes only what is named. With no field named it opens EDITOR exactly as it did, which remains the road for an edit touching several fields or none in particular. A named edit is refused on the states the editor path is refused on and on no others: the surface changes how a change is expressed, never what may be changed. The frozen fields are compared against their anchors on both paths, and the result is parsed before it reaches the corpus on both paths.
schema: 3
version: 1
---

`edit` hands `$EDITOR` a scratch copy and reads back text. It therefore knows
that something changed, by comparing two strings, and never what. That is
enough to validate and not enough to record, and ADR-16813b3bcf37 asks for a
record that names fields.

**The information already exists at the call site.** `edit.rs` holds `original`
and `edited` and computes `base_version` before the editor runs. A diff of two
parsed entities would recover the field names without any new surface at all,
and that is the option this decision does not take, for two reasons. A diff
recovers what the caller meant only when the parse is unambiguous, and it
recovers nothing about intent: a caller who meant to fix a typo in the body and
also moved a scope line produces the same two-field diff as one who meant both.
A named edit is the caller saying which, and the record is then a statement
rather than an inference.

**The second reason is the one that prompted this.** `ank edit` requires a
human at a keyboard, or a scripted `$EDITOR`, and an agent that wants to change
one field today has to write a program that patches a temporary file and hand
it to the tool as an editor. That was done in this repository, on this
decision's own predecessor, and it works. But it means the one surface the
corpus offers for changing an entity is a surface no agent can use honestly and
plainly, which pushes exactly the wrong way: toward the direct file write that
`ADR-01b6dd05f0db` asks agents not to perform, and away from the paved road
whose whole argument is that a chaperoned edit strengthens the invariants.

**The editor does not leave**, and that is the half worth being explicit about.
An edit that rewrites a body around a new argument is prose work, and a flag
that takes a body on stdin is a worse tool for it than an editor. `edit.rs`
already calls itself the paved road rather than a gate; this widens the road, it
does not move it.

**Refusals do not fork.** A named edit meets `Freeze` exactly where the editor
path meets it, and an entity that refuses one refuses the other with the same
code and the same sentence. Two paths that could refuse differently would be
two surfaces, and §4 has one.
