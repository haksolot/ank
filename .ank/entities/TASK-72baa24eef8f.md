---
id: TASK-72baa24eef8f
type: task
slug: the-pi-gallery-card-carries-the-logo
title: The pi gallery card carries the logo
created: 2026-08-08T18:43:03Z
author: seanl@sean-laptop
status: in_progress
scope:
  - assets/**
  - npm/ank/package.json
blocked_by: []
done_criteria: |
  npm/ank/package.json carries a pi.image URL, and the image it names is a raster
  format pi accepts -- PNG, JPEG, GIF or WebP, never SVG. The file lives under
  assets/ in this repository and is reachable anonymously over HTTPS at the URL
  written in the manifest, established by fetching that exact URL with no
  credentials and checking the bytes it returns are the image, not an HTML error
  page. The image reads correctly on both a light and a dark card background, or
  the manifest commits to one and the image carries its own. skill/SKILL.md is not
  touched and ank --version still prints skill 605f771e1955. cargo test and
  ank check stay green.
criteria_by: creator
schema: 2
version: 2
---

The pi gallery renders a card per package, and ank's is currently text alone.
pi.image is the field that changes that: a URL to a static preview in PNG, JPEG,
GIF or WebP. pi.video, MP4 only, sits beside it and takes precedence when both
are set.

SVG is not on that list, and SVG is all this repository has. assets/ank.svg and
assets/ank-dark.svg are 324 bytes each: a 24 by 24 viewBox, shape-rendering
crispEdges, and four path elements that are nothing but axis-aligned rectangles
on an integer grid. That shape is the opportunity. There is no SVG rasteriser on
the maintainer's machine -- the convert on PATH under Windows is the filesystem
utility, not ImageMagick -- and pulling one in for a single asset is the kind of
dependency the project refuses by default. Reading those rectangles and writing a
PNG with node's built-in zlib is a short, exact job with nothing added, and
nearest-neighbour scaling of a crispEdges glyph is not a compromise but the
intended look.

Three decisions the implementer owns, none of them settled here.

Whether the generator is committed. A binary asset nobody can regenerate is a
mystery in a repository that otherwise explains itself; a script under
.github/scripts/ beside npm-assemble.sh is the precedent. Against it: this asset
changes when the logo changes, which is approximately never.

Which URL the manifest names. A raw.githubusercontent.com path on main updates
the card whenever main does, and breaks it whenever main breaks. The same path on
a tag is stable and goes stale. Either is defensible; say which and why.

What the card actually shows. The source glyph is 24 pixels of monochrome mark.
Scaled up alone it is a small dark shape on whatever the gallery paints behind
it, and the two SVG variants exist precisely because that background is not
knowable. A card with its own ground, its own padding and room for the tagline is
a different image from the favicon, and probably the right one.

Two things that are true regardless. The gallery reads the registry, so nothing
appears until a release publishes a package carrying the field -- this lands with
the next tag, not at merge. And ADR-e3cb36646d77 binds npm/**: it forbids a second
copy of the skill, not an image, so an asset referenced by URL is untouched by it.

This is not TASK-ab3592c8c586. That one is a recording of the loop running, for
the README. This is a still, for the integration surfaces. They meet in one place
worth remembering: if a recording is ever produced as MP4 and set as pi.video,
pi will show it instead of this image.
