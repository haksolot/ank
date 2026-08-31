# ank

**The stupid coordination tool.** Tasks and architecture decisions in your repo,
behind one CLI any coding agent can call.

    npx @haksolot/ank --version
    npm install -g @haksolot/ank

The binaries travel inside the package: nothing is downloaded at install time, so
this channel works where fetching a bare executable does not. It covers
`linux x64`, `darwin arm64` and `win32 x64`; on any other platform the wrapper
exits 9 and names `cargo install`. ank needs **git 2.34 or newer**.

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

The skill an agent loads is a separate install, and
[the documentation](https://github.com/haksolot/ank) covers both, along with the
specification and the source.

Apache-2.0. Your `.ank/` files, the third-party tools that read or write them,
and anything you build on top are yours. Ank was GPL-3.0-only up to and
including 0.3.0, the last release made under that licence; 0.4.0 is the first
released under Apache-2.0. The change is prospective: a release you already
received under GPL-3.0 stays available to you under GPL-3.0.
