# CI jobs and required checks

Four workflows run on this repository. Three of them run on every pull request
and on every push to `main`; the fourth runs on a tag.

## The required checks

The ruleset on `main` requires six checks by name, and a pull request cannot
merge until each of them is green:

    ubuntu-latest                  the three gates, on Linux
    macos-latest                   the three gates, on macOS
    windows-latest                 the three gates, on Windows
    version check / ubuntu-latest  release.yml's version check, against its fixtures
    msrv / ubuntu-latest           the workspace builds on the declared MSRV
    msrv is tight / ubuntu-latest  the minor below it does not

The three gates are the ones CONTRIBUTING.md asks you to run locally: `cargo fmt
--check`, `cargo test --workspace`, and `ank check` on the repository's own
corpus. The ruleset also allows the merge commit and nothing else, and has no
bypass actors; [Ratifying and archiving](ratifying.md#anchors-made-before-the-ref)
says which anchors that still protects.

## ci.yml

- **`changes`**, first, says whether the pull request touches anything outside
  `.ank/`. It diffs the merge commit against its base and writes `code=true`
  when any path outside `.ank/` changed, or on a push, where everything runs.
- **`test`**, a matrix over `ubuntu-latest`, `macos-latest` and
  `windows-latest`, and the three required checks named after them. It checks
  out the whole history, because `ank check` walks it, and runs the three gates.
  Before `ank check`, on every event, it fetches `+refs/ank/*:refs/ank/*` from
  `origin`, read-only: a ratification made since ADR-5e47007a75be is a ref
  there, `actions/checkout` does not fetch it, and a pull request from a fork
  verifies it the same way as one from a branch here. The Linux leg keeps the
  binary it built for `attest`.

  The suite runs under the `ci` profile of `.config/nextest.toml`, which runs
  every test to the end and bounds each one at 600 seconds, about twice the
  slowest test that passes (the bound and the measurement are written beside the
  setting). A line `SLOW [> 60.000s]` in the log is a test still running at that
  mark and is not a failure. A line `TIMEOUT` is a test nextest killed when it
  reached the bound: it counts as a failure, the leg is red, and the line names
  the test. Read it as a hang, a test waiting on something that will never come,
  rather than as a slow runner, and look at that test on that platform before
  rerunning the job.
- **`attest / ubuntu-latest`** runs on a push to `main` only, after the matrix.
  It anchors every task `check` reports as `done with no test proof` to this
  run, with `ank attest --proof test:<run-id> --detached`, and it is the only
  job in the file that holds `contents: write`. It turns red rather than
  skipping when the proof does not reach the remote. The recipe it follows is
  [Anchoring a run](ci.md#anchoring-a-run).
- **`version check / ubuntu-latest`** runs `.github/scripts/check-version-fixtures.sh`,
  the release's version check exercised against fixture trees on every pull
  request instead of only on a tag ([Releasing](releasing.md)). It has no local
  equivalent other than running the script:

      bash .github/scripts/check-version-fixtures.sh

- **`msrv`**, a three-platform matrix building on the declared minimum Rust
  version. Only its ubuntu leg is required, because the floor is one number for
  the workspace; the other two legs run and report, and they are what would
  catch a floor that differed per target.
- **`msrv is tight / ubuntu-latest`** requires the minor below the floor to fail.
  Both MSRV jobs are [The MSRV](msrv.md).

**A pull request that changes only `.ank/` runs the ubuntu leg whole and nothing
else.** The corpus is judged by `ank check`, which that leg runs, and by the
tests that read the live repository, which that leg runs too; the macOS and
Windows legs, both MSRV jobs and the version check have nothing to learn from a
corpus diff, and would cost it thirteen minutes. They are skipped by a condition
on the job, or on every step of a matrix job, never by a `paths:` filter on the
workflow: a job skipped by its condition reports success to the ruleset, where a
workflow filtered out leaves its required checks pending and the pull request
unmergeable. `docs/` counts as code: the suite replays the pages.

Pull request runs superseded by a new push are cancelled. Runs on `main` are
queued and never cancelled, because two `attest` jobs racing on
`refs/ank/proof/*` would turn an intermediate merge red while the head of `main`
is green.

## docs.yml

Builds this site with mdBook, pinned by version and by the digest of its release
archive, on every pull request, and checks that every internal link and fragment
resolves in the built site. A push to `main` also deploys it to GitHub Pages; a
pull request never does. It is not a required check, and a red build still
blocks nothing but the site.

## install.yml

Tests `install.sh` and `install.ps1` against the published release, on the
platforms each claims to install, including `macos-15-intel`, which no other
workflow carries. It asserts the two things neither installer may ever do:
unpack an archive whose hash does not match, and end in silence on a platform it
has no archive for. It reads the release page and writes nothing.

## release.yml

Runs on a pushed `v*` tag, and on `workflow_dispatch` as a rehearsal that builds
and publishes nothing. Its jobs are `version`, `build`, `npm-smoke`, `publish`
and `publish-npm`, and [Releasing](releasing.md) walks them in order.
