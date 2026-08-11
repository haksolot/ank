# ank

**The stupid coordination tool.** Tasks and architecture decisions in your repo,
behind one CLI any coding agent can call.

    npx @haksolot/ank --version
    npm install -g @haksolot/ank

The binary travels inside the package: nothing is downloaded at install time, so
this channel works where fetching a bare executable does not. It covers
`linux x64`, `darwin arm64` and `win32 x64`; on any other platform the wrapper
exits 9 and names `cargo install`. ank needs **git 2.34 or newer**.

This package carries the CLI. The skill an agent loads is a separate install, and
[the documentation](https://github.com/haksolot/ank) covers both, along with the
specification and the source.

GPL-3.0 — the copyleft covers the tool's code, not the format: your `.ank/`
files, and the third-party tools that read or write them, are not derivative
works.
