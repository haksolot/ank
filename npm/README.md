# The npm channel

Four packages, and the shape is the one esbuild uses.

    @haksolot/ank                    the wrapper, and the only name anyone installs
    @haksolot/ank-linux-x64-musl     the binaries, one package per target
    @haksolot/ank-darwin-arm64
    @haksolot/ank-win32-x64

The wrapper declares the three as `optionalDependencies`, each carrying `os` and
`cpu`. npm installs the one that matches the machine and silently skips the
other two, so a `linux x64` install downloads one binary and not three.

**The binaries are inside the packages, and nothing is fetched at install time.**
That is the whole point of this channel rather than a detail of it. The driving
case is the corporate workstation whose firewall blocks downloading a bare
executable but lets the npm registry through; a `postinstall` script that
fetched the binary would die behind exactly the firewall this package exists to
cross. `bin/wrapper.js` is therefore a resolver, never a downloader: it finds
the platform package with `require.resolve` and executes what it finds.

**One executable, one package, one version.** Each platform package carries
`ank` and nothing else (ADR-1ea31c2f3c5a). The protocol surface is the verb
`ank mcp` and the watcher is `ank watch`, so there is no second file that could
arrive from a different build, or fail to arrive at all. The wrapper declares
one `bin` -- `bin/ank` is two lines, and `bin/wrapper.js` is the resolution,
the diagnostics and the exit code behind it.

**The Linux package declares no `libc`.** The build is `x86_64-unknown-linux-musl`
and statically linked, so it runs on a glibc distribution just as well; declaring
`"libc": ["musl"]` would make npm skip it on Debian and Ubuntu, which is most of
the installs. The target is in the package name because that is what was built,
not because it restricts where it runs.

**The wrapper forwards the exit code unchanged.** Section 4 of the specification
gives 4, 6, 8 and 9 distinct meanings that an agent branches on. A wrapper that
collapsed them into 0 and 1 would break every caller reading them, which is why
`bin/wrapper.js` exits with the child's status and reserves 9, the environment
code, for its own failures. `ank mcp` reaches its client through the same shim,
and `stdio` is inherited rather than captured, because a surface that speaks
JSON-RPC on stdin has nothing to say to a wrapper holding the pipe.

## What is not in git

Four paths are filled by `.github/scripts/npm-assemble.sh` during the release
run and ignored in the tree, and between them they are every file in a published
tarball that is not committed here:

    npm/ank-*/bin/ank, ank.exe   the executable, one per platform package
    npm/*/LICENSE                copied from the repository root into all four
    npm/ank/skills/<name>/       one SKILL.md per skill, from skill/ and
                                 skill/<dir>/, named by the frontmatter
    npm/*/*.tgz                  what npm pack produced

The binaries come from the same artefacts the GitHub release publishes: one
build, two channels, no second compilation that could disagree with the first.
The skills are copied rather than committed because `skill/SKILL.md` and its
siblings are the only copies that exist (ADR-8b3045cf11db): each is anchored
where it lives, `build.rs` hashing the contract's revision into `ank --version`
and `tests/skill.rs` holding every skill to its own, and a second copy beside
them would carry no such anchor and would drift with nothing turning red.

## Versions

All four carry one version. The assembly stamps it across the four
`package.json` and pins the three platform packages in the wrapper's
`optionalDependencies` exactly, from the tag `release.yml` passes it, so the
copy committed here is only ever the last released one.

## Publishing

`release.yml` does it on a `v*` tag, with `NPM_TOKEN` from the repository
secrets and `--access public`, which a scoped package needs on its first
publish. The workflow has two triggers, `push` on `v*` and `workflow_dispatch`,
and only `publish-npm` and `publish` are gated on the tag. The `npm-smoke` job
carries no such gate, so it runs on both: the pipeline is rehearsed on a
dispatch before a tag depends on it, and a tag is the one thing here that is
awkward to take back.

That job runs on `ubuntu-latest`, `macos-latest` and `windows-latest`, since a
wrapper is exactly the kind of code that is right on one platform and wrong on
another, and on each of the three it:

- takes `npm@latest`, because the npm inside a pinned node is the part that moves;
- assembles that runner's platform package from the artefact `build` uploaded;
- reads every skill out of the wrapper tarball and compares its SHA-256 with the
  file in `skill/`, walking the tree rather than a written list so a skill added
  later is checked by existing;
- derives the dist-tag on seven versions, a release resolving to `latest` and a
  prerelease to `next`, both branches of the rule exercised before either ships;
- runs the two `npm publish` commands `publish-npm` will run, with `--dry-run`,
  so the argument that was wrong when v0.1.2 shipped no packages is the argument
  under test;
- installs the two tarballs into a scratch project;
- compares `npx ank --version` with the artefact's own output rather than a
  literal;
- holds one `initialize` conversation with `npx ank mcp`, which is the only step
  that proves `stdio: inherit` still reaches the child;
- checks that an unknown verb still exits 2 and an unknown flag on `ank mcp`
  still exits 1 through the shim, because a wrapper is where exit codes go to
  die.
