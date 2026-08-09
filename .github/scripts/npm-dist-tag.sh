#!/usr/bin/env bash
#
# The npm dist-tag a version publishes under (§9 of the specification).
#
# `latest` for a release, `next` for a prerelease. Written here and nowhere else
# because two jobs read it -- the smoke rehearsal and the publish -- and a
# rehearsal passing different flags from the thing it rehearses is not a
# rehearsal. It is a function of the version and of nothing else: a flag chosen
# while pushing a tag is a flag that is eventually forgotten, and this one fails
# silently, since npm 10 applied `latest` to a prerelease without a word.
#
#   $ npm-dist-tag.sh 0.2.0        -> latest
#   $ npm-dist-tag.sh 0.2.0-rc1    -> next
#   $ npm-dist-tag.sh 1.2.3+build  -> latest
#
set -euo pipefail

version="${1:?usage: npm-dist-tag.sh <version>}"

# Build metadata is stripped first. `1.2.3+build-7` is a release, and a hyphen
# living in the metadata says nothing about the version in front of it --
# looking for the hyphen without stripping would call that release a candidate
# and hide it behind `next`.
core="${version%%+*}"

case "$core" in
  *-*) echo next ;;
  *) echo latest ;;
esac
