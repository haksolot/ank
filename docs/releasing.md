# Releasing

A release is a tag. `release.yml` publishes on a pushed `v*` tag and on nothing
else, because a release that could be produced from a branch would make "the
binary for v0.8.0" a question rather than an answer. Everything a release ships
-- the archive names, the npm packages, the release title -- derives from the
tag. What makes that safe is that the tree has to agree with the tag before
anything is built.

## Bump the version literals

Ten literals across seven files carry the version, and every one of them must
say the version you are about to tag:

    crates/ank-cli/Cargo.toml                          version = "<v>"
    crates/ank-core/Cargo.toml                         version = "<v>"
    npm/ank/package.json                               "version"
    npm/ank/package.json                               the three optionalDependencies pins
    npm/ank-linux-x64-musl/package.json                "version"
    npm/ank-darwin-arm64/package.json                  "version"
    npm/ank-win32-x64/package.json                     "version"
    .claude-plugin/plugin.json                         "version"

You do not have to remember that list. The check that gates the release prints
it, with the exact line that repairs each literal, when you hand it a version the
tree does not carry:

    $ bash .github/scripts/check-version.sh 0.8.1
    version 0.8.1 disagrees with 10 of 10 version literals
    ...
    repair every line above, then move the tag:
      crates/ank-cli/Cargo.toml: version = "0.8.1"
      crates/ank-core/Cargo.toml: version = "0.8.1"
      npm pkg set version=0.8.1 --prefix npm/ank
    ...

and answers `version 0.8.1 agrees with all 10 version literals` once they are
bumped. Run it before you tag. The bump lands like any change, by pull request,
and `Cargo.lock` follows the two manifests on the next build.

The same script runs on every pull request against fixture trees, as the
`version check` job, so a bump that touched six of the seven files is red on the
branch that made it and not on the tag ([CI jobs and required
checks](ci-jobs.md)).

## Rehearse with a dispatch

`release.yml` also runs on `workflow_dispatch`, and a dispatch builds the same
four targets and publishes nothing. Run one before tagging whenever the pipeline
itself changed:

    gh workflow run release.yml --ref main

The alternative is discovering that the release pipeline is broken at the moment
you use it, on a tag you then have to delete, and a tag is the one thing here
that is awkward to take back. On a dispatch the version job says there is no tag
to compare and skips the comparison, the four builds and the three npm smoke
tests run in full, and the two publish jobs do not start.

## Tag

With the literals bumped and merged, tag the merge commit on `main` and push the
tag. The tags so far are annotated, with the version as the first line of the
message and what the release adds under it:

    git switch main && git pull
    git tag -a v<version>
    git push origin v<version>

What the run then does, in order:

1. **version** compares the tag with every literal and refuses before anything
   is built. A check at the end would already have spent the matrix.
2. **build** runs the tests and packages one archive per target:
   `x86_64-unknown-linux-musl`, `aarch64-apple-darwin`, `x86_64-apple-darwin`
   on an Intel runner, and `x86_64-pc-windows-msvc`, each with a `.sha256`.
3. **npm smoke** installs the assembled packages from their tarballs on three
   platforms and checks the wrapper answers like the binary.
4. **publish** creates the GitHub release from the four archives, only once
   every build row passed. A release carrying three targets out of four would
   look complete.
5. **publish-npm** publishes the three platform packages and then the wrapper,
   only once all three smoke tests agree.

A version with a prerelease suffix, `v0.9.0-rc1`, publishes under the npm
dist-tag `next` and as a GitHub prerelease, so it is installable by whoever asks
for it by name and never becomes what a bare `npm install -g @haksolot/ank`
resolves to. Anything else publishes as `latest`.

## NPM_TOKEN

`publish-npm` authenticates with the repository secret `NPM_TOKEN`, and it is
the only credential outside the run that the pipeline uses: the GitHub release
is created under the workflow's own token. It has to be an npm token allowed to
publish every package under the `@haksolot` scope, and a missing or expired one
fails `publish-npm` alone -- the GitHub release still publishes, because the
release page is the primary channel and npm a convenience on top of it. Rotate
it under the repository's *Settings → Secrets and variables → Actions*, and
re-run the failed job rather than the tag.

## After the release

`ank update --check` against the new tag should report it as latest, and the
installers read the release page directly, so both routes serve it as soon as
the release exists. Nothing else is published: no package manager ships ank
([Install](install.md#the-binary) says why).
