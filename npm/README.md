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

`npm/ank-*/bin/` is empty here and ignored. The binary lands in it during the
release run, from the same artefacts the GitHub release publishes: one build,
two channels, no second compilation that could disagree with the first.

## Versions

All four carry one version, and the wrapper pins the three platform packages to
it exactly. `release.yml` stamps it from the tag, so the copy committed here is
only ever the last released one.

## Publishing

`release.yml` does it on a `v*` tag, with `NPM_TOKEN` from the repository
secrets and `--access public`, which a scoped package needs on its first
publish. On `workflow_dispatch` the same job assembles the packages, installs
them from the tarballs on all three platforms, and holds one `initialize`
conversation with `npx ank mcp`, so the pipeline is proved before a tag depends
on it, and a tag is the one thing here that is awkward to take back.
