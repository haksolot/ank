# ank

**The stupid coordination tool.** Tasks and architecture decisions in your repo,
behind one CLI any coding agent can call.

    npx @haksolot/ank --version
    npm install -g @haksolot/ank

The binaries travel inside the package: nothing is downloaded at install time, so
this channel works where fetching a bare executable does not. It covers
`linux x64`, `darwin arm64` and `win32 x64`, and on anything else the wrapper
exits 9 rather than failing quietly. An Intel Mac is the common case, and it has
an official route this channel does not carry, since the release builds
`x86_64-apple-darwin` and `install.sh` knows it:

    curl -fsSL https://raw.githubusercontent.com/haksolot/ank/main/install.sh | sh

Reach for that before building from source. If you do build, it is
`cargo install --git https://github.com/haksolot/ank ank-cli`, with the `--git`:
nothing here publishes to crates.io, `ank-cli` is not a crate there, and the
crate named `ank` belongs to somebody else and installs something unrelated.
ank needs **git 2.34 or newer**.

Once ank is installed, it updates itself through whichever of the three routes
placed it: `ank update --check` reports the running and the latest version and
installs nothing, and `ank update` installs the latest, which from this package
means `npm install -g @haksolot/ank@<version>`.

One command is installed, and everything ank does is a verb of it. A client
with no shell reaches the same verbs over MCP by launching that command with
`mcp`, and every verb the CLI dispatches is a tool there, generated from the
same table:

    {
      "mcpServers": {
        "ank": {
          "command": "ank",
          "args": ["mcp", "--repo", "/path/to/your/repo"]
        }
      }
    }

The skills an agent loads come from the binary. `ank skills` lists the six it
carries, and `ank skills --install` writes them out of the executable and hands
the directory to `npx skills add` for the agent you run, with nothing cloned. It
is still a second command: installing the package places no skill for an agent,
though the package does carry the six `SKILL.md` for an installer that reads
them.

[The documentation](https://github.com/haksolot/ank) covers every route, along
with the specification and the source.

Apache-2.0. Your `.ank/` files, the third-party tools that read or write them,
and anything you build on top are yours. Ank was GPL-3.0-only up to and
including 0.3.0, the last release made under that licence; 0.4.0 is the first
released under Apache-2.0. The change is prospective: a release you already
received under GPL-3.0 stays available to you under GPL-3.0.
