<!-- The section between the BEGIN and END markers is generated from crates/ank-contract/src/env.rs; the rest is written by hand.
     Regenerate: cargo run -q -p ank-contract --bin environment -- docs/environment.md -->

# Environment variables

Every variable the binary reads, and what it changes. None of them is required:
with the environment empty but for `PATH`, every verb works, and the variables
below adjust who is acting, where a file is found, and how output looks.

<!-- BEGIN environment -->
<!-- Generated from crates/ank-contract/src/env.rs; do not edit between the markers. -->

## Every variable

| Variable | Read by | What it changes |
|---|---|---|
| `ANK_AGENT` | every verb, and `ank mcp` | the identity this session acts as: who holds a claim, who wrote an entity, who ran `done`. Unset or blank, `<user>@<hostname>`, and `ank mcp` writes under `ank-mcp/<version>` |
| `USERNAME` | every verb, with `ANK_AGENT` unset | the `<user>` of the fallback identity; the first of `USERNAME`, `USER`, `LOGNAME` set and not blank wins, and none gives `unknown` |
| `USER` | every verb, with `ANK_AGENT` unset | the `<user>` of the fallback identity, when `USERNAME` gives none |
| `LOGNAME` | every verb, with `ANK_AGENT` unset | the `<user>` of the fallback identity, when `USERNAME` and `USER` give none |
| `COMPUTERNAME` | every verb, with `ANK_AGENT` unset | the `<hostname>` of the fallback identity, cut at its first dot and lowercased; the first of `COMPUTERNAME`, `HOSTNAME` set wins, and none asks the `hostname` program, then says `localhost` |
| `HOSTNAME` | every verb, with `ANK_AGENT` unset | the `<hostname>` of the fallback identity, when `COMPUTERNAME` gives none |
| `ANK_UPDATE_REPOSITORY` | `ank update` | the repository release tags are read from, in place of `https://github.com/haksolot/ank`; empty counts as unset |
| `NO_COLOR` | every verb at a terminal, and `ank tui` | set and not empty, takes the colour and nothing else; the empty value is not an opt-out |
| `TERM` | every verb at a terminal, and `ank tui` | `dumb` takes the colour, as `NO_COLOR=1` does, and draws the structure of `ank tui` in ASCII; on Windows, set at all, it says the console renders escape sequences |
| `WT_SESSION` | every verb at a Windows terminal | set, says the console renders escape sequences; with none of `WT_SESSION`, `TERM`, `TERM_PROGRAM`, `ConEmuANSI`, `ANSICON` set, the output is plain |
| `TERM_PROGRAM` | every verb at a Windows terminal | set, says the console renders escape sequences |
| `ConEmuANSI` | every verb at a Windows terminal | set, says the console renders escape sequences |
| `ANSICON` | every verb at a Windows terminal | set, says the console renders escape sequences |
| `EDITOR` | `ank edit` | the editor `ank edit <id>` opens when given no field to change; unset or blank, the verb refuses at exit 9 |
| `APPDATA` | `ank config --user`, `--repo`, `ank mcp`, `ank watch`, `ank tui` | on Windows, the reader's configuration directory is `%APPDATA%\ank`; unset, a verb that needs it refuses at exit 9 |
| `XDG_CONFIG_HOME` | `ank config --user`, `--repo`, `ank mcp`, `ank watch`, `ank tui` | elsewhere than Windows, the reader's configuration directory is `$XDG_CONFIG_HOME/ank`; empty counts as unset |
| `HOME` | `ank config --user`, `--repo`, `ank mcp`, `ank watch`, `ank tui` | with `XDG_CONFIG_HOME` unset, the reader's configuration directory is `$HOME/.config/ank`; neither set, a verb that needs it refuses at exit 9 |
| `PATH` | `ank done`, `ank skills --install`, `ank update` | where `sh` and `git`, `npx`, and `npm`, `powershell`, `pwsh` and `curl` are looked for; a program found on none of its directories is refused by name |
| `PATHEXT` | `ank skills --install`, `ank update`, on Windows | the extensions tried on `PATH`, in its order, of `.COM`, `.EXE`, `.BAT`, `.CMD`; unset, those four |

The test suite sets these to observe the binary. They are not an interface, and a release may change or remove any of them without notice:

| Variable | Read by | What it changes |
|---|---|---|
| `ANK_INDEX_BUSY_MS` | every verb that opens the index | how long, in milliseconds, a connection waits on an index another process holds locked; unset, five seconds |
| `ANK_INDEX_STEPS` | every verb that writes the index | a file the SQLite steps a refresh executed are written to |
| `ANK_INDEX_REFRESHED` | every verb that opens the index | a file every refresh appends what it hashed and reindexed to |
| `ANK_TRACE_READS` | every verb that reads the corpus | an absolute path every entity parse and every index opening appends a line to |
<!-- END environment -->

## `ANK_AGENT`

The identity this session acts as: who holds a claim, who wrote an entity, who
ran `done`. Unset, it falls back to `<user>@<hostname>`, which carries no actor
type and makes two sessions on one machine one agent. `ank status` says which
one is in force and where it came from:

<!-- replay env ANK_AGENT=claude-code/opus-5+docs
$ ank init
-->

    $ ank status
    branch main
    warning: no default branch, so completion refs are neither pruned nor judged (ank config default_branch <name>)
    identity claude-code/opus-5+docs (ANK_AGENT)
    no claim
    elsewhere no claim by another agent
    perimeter the whole repository, 0 constraint(s)
    queue 0 proposal(s), 0 finished elsewhere
    corpus 0 fault(s), 2 signal(s)

    > ank context

How to write one, and why a concurrent session needs its own, is [Claims and
identity](claims.md#one-identity-per-session). `ank mcp` writes under
`ank-mcp/<version>` unless this variable names another identity.

## `ANK_UPDATE_REPOSITORY`

Where releases are read from. `ank update` is the only verb that reads it,
because it is the only verb that asks the network anything (ADR-64f32c74a0f9).
It replaces `https://github.com/haksolot/ank` in the `git ls-remote --tags
--refs` the check makes, so a mirror, an internal clone or a fixture all serve.
Against a bare clone tagged `v0.9.0` and `v0.7.0`:

<!-- replay mirror dir=/srv/ank-mirror.git
$ git init -q --bare /srv/ank-mirror.git
$ r=/srv/ank-mirror.git && c=$(git -C $r commit-tree -m release "$(git -C $r hash-object -t tree -w --stdin </dev/null)") && git -C $r tag v999.0.0 $c && git -C $r tag v0.0.1 $c
-->

    $ ANK_UPDATE_REPOSITORY=/srv/ank-mirror.git ank update --check --json
    {"contract":1,"current":"0.8.0","latest":"0.9.0","newer":true}

A repository carrying no tag it can parse answers `"latest":null` and
`"newer":false`, and still exits 0. What `update` does with the answer is
[Install](install.md#updating).

## `NO_COLOR` and `TERM`

**`NO_COLOR` takes the colour and nothing else.** Colour is emitted only when
stdout is a terminal, so a pipe, a file and `--json` are plain already and the
variable changes nothing for a program reading them. It matters when a person
has the terminal: through a pseudo-terminal `ank status` came back 538 bytes
carrying 22 escape sequences, and `NO_COLOR=1` 448 bytes carrying none. The
empty value is deliberately not an opt-out -- `NO_COLOR=` is how a shell spells
"unset this for the child" -- and it measured 538 bytes and 22 sequences, exactly
as unset did. `TERM=dumb` is read the same way as `NO_COLOR=1`.

## `EDITOR`

What `ank edit <id>` opens when it is given no field to change. It is read by
that verb alone, and its absence is an environment to repair rather than a
failure of the work:

<!-- replay editor
$ ank init
$ ank new task --title "t" --scope "**" --no-verify
created TASK-b7004333d81f t
-->

    $ env -u EDITOR ank edit TASK-b700
    error[9]: EDITOR is not set, and there is no editor to open
      -> EDITOR=vi ank edit TASK-b7004333d81f

## Where the reader's configuration lives

Three files belong to the person running ank rather than to any repository:
`corpora.yml`, the corpora [the MCP server](mcp.md#several-repositories-one-server)
may reach; `watch.yml`, what [the watcher](watch.md) keeps warm; and
`events.jsonl`, the stream the watcher writes. They sit in one directory:
`%APPDATA%\ank` on Windows, and elsewhere `$XDG_CONFIG_HOME/ank`, falling back
to `$HOME/.config/ank`. `ank watch --where` prints the path it resolved: with
`XDG_CONFIG_HOME=/srv/cfg` it printed `/srv/cfg/ank/watch.yml`, and with that
variable unset and `HOME=/home/me`, `/home/me/.config/ank/watch.yml`.

With none of the three set, a verb that needs the directory refuses at exit 9
and names the variable to set.

## Variables that are not an interface

Three `ANK_` names appear in the source and in no table above. `ANK_COMMIT`,
`ANK_SKILL` and `ANK_RELEASED_SCHEMA` are read by the build, not at run time:
they are what `ank --version` and the schema warning print. The `ANK_INDEX_`
names and `ANK_TRACE_READS` are read at run time, which is why the table lists
them, but they exist for the test suite to observe the index and the reads, and
a release may change or remove any of them without notice.
