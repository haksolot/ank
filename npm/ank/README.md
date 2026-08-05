# ank

**The stupid coordination tool.** Tasks and architecture decisions in your repo,
behind one CLI any coding agent can call.

    npx @haksolot/ank --version
    npm install -g @haksolot/ank

The binary travels inside the package: nothing is downloaded at install time, so
this channel works where fetching a bare executable does not. ank needs **git
2.34 or newer**, and checks at startup.

    ank init                      # creates .ank/, once
    ank context                   # what binds here, and what is free to take
    ank claim <id>                # takes the task, freezes its criterion by hash
    ank log "<what you learned>"  # renews the claim; working is what holds it
    ank done                      # runs the verifiers itself and writes the proof

Documentation, the specification and the source are at
<https://github.com/haksolot/ank>. GPL-3.0 — the copyleft covers the tool's
code, not the format: your `.ank/` files, and the third-party tools that read or
write them, are not derivative works.
