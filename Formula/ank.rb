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
class Ank < Formula
  desc "Tasks and architecture decisions in your repo, behind one CLI agents can call"
  homepage "https://github.com/haksolot/ank"
  version "0.4.0"
  license "Apache-2.0"

  on_macos do
    on_arm do
      url "https://github.com/haksolot/ank/releases/download/v0.4.0/ank-0.4.0-aarch64-apple-darwin.tar.gz"
      sha256 "d5465e9fccebac6c9bffd7a8d7c401874477eb5fc8eebfc0ff477fcee8237fab"
    end

    on_intel do
      url "https://github.com/haksolot/ank/releases/download/v0.4.0/ank-0.4.0-x86_64-apple-darwin.tar.gz"
      sha256 "baa5d8f221973a5cf5f1972a8360ec94deed45bfb89a8dc48662889e0af40a7c"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/haksolot/ank/releases/download/v0.4.0/ank-0.4.0-x86_64-unknown-linux-musl.tar.gz"
      sha256 "54903683df786819b34f295fb88c11dc586f4908514f2800f673983535b1b298"
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
