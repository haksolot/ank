# Environment variables

Every variable the binary reads, and what it changes. None of them is required:
with the environment empty but for `PATH`, every verb works, and the variables
below adjust who is acting, where a file is found, and how output looks.

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

A few `ANK_` names appear in the source and in nothing above. `ANK_COMMIT`,
`ANK_SKILL` and `ANK_RELEASED_SCHEMA` are read by the build, not at run time:
they are what `ank --version` and the schema warning print. The `ANK_INDEX_`
names and `ANK_TRACE_READS` exist for the test suite to observe the index and
the reads, and a release may change or remove any of them without notice.
