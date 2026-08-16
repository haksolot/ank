# typed: false
# frozen_string_literal: true

# Derived from the GitHub release by .github/scripts/brew-formula.sh, which
# .github/workflows/publish-brew.yml runs on every pull request and asserts this
# file is exactly what deriving it produces. A hand edit turns that job red
# instead of failing on somebody's machine -- in particular a sha256, which is
# read from the .sha256 asset the release publishes and is never typed.
#
# This repository is its own tap, and there is no satellite to keep in step
# (ADR-782a3556cf2d):
#
#   brew tap haksolot/ank https://github.com/haksolot/ank
#   brew install haksolot/ank/ank
#
# It is a tap and not homebrew-core because core's gate is notability -- of the
# order of 75 stars or 30 forks, with a track record -- and this repository has
# 1 star, 0 forks and three weeks of history. That is their documented door and
# not a judgement about the tool. The move costs a change of source and not a
# rewrite: the platform blocks below become one url and sha256 on the source
# tarball plus `system "cargo", "install", *std_cargo_args`, and the rest of
# this file stands.
#
# v0.2.0 published no archive for:
#   ank-0.2.0-x86_64-apple-darwin.tar.gz
# so this formula carries no block for it. That is a row of the release matrix
# and not a decision here: the first tag cut with that row restores the block,
# derived, with no edit to this file or to the script that writes it.
class Ank < Formula
  desc "Tasks and architecture decisions in your repo, behind one CLI agents can call"
  homepage "https://github.com/haksolot/ank"
  version "0.2.0"
  license "GPL-3.0-only"

  on_macos do
    on_arm do
      url "https://github.com/haksolot/ank/releases/download/v0.2.0/ank-0.2.0-aarch64-apple-darwin.tar.gz"
      sha256 "834f36627fc8325b0d3c46d2be62f39f6b6f53246ee754255c94952471297623"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/haksolot/ank/releases/download/v0.2.0/ank-0.2.0-x86_64-unknown-linux-musl.tar.gz"
      sha256 "9d45670ecf3c9472aa1f542a9d6a33b6f476040cb25e4a189417ca03db1aee01"
    end
  end

  # The archive wraps its contents in a directory named after itself and
  # Homebrew stages inside it, so these are the names as packaged by the
  # `package` step of release.yml. SKILL.md travels with the binary because a
  # channel shipping ank and not what it teaches has shipped half of it.
  def install
    bin.install "ank"
    doc.install "README.md"
    pkgshare.install "SKILL.md"
  end

  # `ank --version` prints "ank <version> (<commit>, skill <hash>)", so the
  # assertion is on the version token: pinning the whole line would make every
  # commit of a release a change to this file.
  test do
    assert_match(/\A\s*ank\s+#{Regexp.escape(version.to_s)}(\s|\z)/,
                 shell_output("#{bin}/ank --version"))
  end

  # For the move to core, and for anyone watching the tap: the newest tag is
  # what this formula tracks.
  livecheck do
    url :stable
    strategy :github_latest
  end
end
