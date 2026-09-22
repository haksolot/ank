# The MCP server

A client that has no shell reaches ank through `ank mcp`, a verb of the one
executable every route installs (ADR-1ea31c2f3c5a). There is no second file to
fetch, sign or discover: what the CLI dispatches is what the surface serves,
because they are the same file. It speaks JSON-RPC over stdio and the client
spawns it, which means it is configured rather than started.

## Configuring a client

Three configurations, and each of them is pasted rather than derived.

**Claude Code**, one line, which writes the entry for you:

    claude mcp add ank -- ank mcp --repo /path/to/your/repo

or `.mcp.json` at the root of the repository, which is the form that travels
with the tree and reaches everyone who clones it:

    {
      "mcpServers": {
        "ank": {
          "command": "ank",
          "args": ["mcp", "--repo", "/path/to/your/repo"]
        }
      }
    }

**Claude Desktop**, in `claude_desktop_config.json` (`~/Library/Application
Support/Claude/` on macOS, `%APPDATA%\Claude\` on Windows), which has no project
directory of its own and so has nothing else to go on:

    {
      "mcpServers": {
        "ank": {
          "command": "ank",
          "args": ["mcp", "--repo", "/path/to/your/repo"]
        }
      }
    }

**Cursor**, in `.cursor/mcp.json` beside the repository, or `~/.cursor/mcp.json`
for every project at once:

    {
      "mcpServers": {
        "ank": {
          "command": "ank",
          "args": ["mcp", "--repo", "/path/to/your/repo"]
        }
      }
    }

**`command` is `ank` and `mcp` is the first argument, in all three.** If you
have a configuration written against `ank-mcp`, that is the one line to change:
releases up to 0.6.0 placed a second executable by that name and no route places
one any more, so a client still naming it gets `command not found` rather than a
wrong answer.

**`--repo` is written out in all three, and that is the point of showing them.**
A client spawns the server in whatever directory it happens to be in, and with
no `--repo` the server takes that directory. The failure mode is not an error:
it is a process quietly speaking for a corpus nobody meant, or for none. The
configuration is also where it is named rather than something a call may
override; a call that tries is refused, below. A path with no corpus under it is
refused before any client is listening, rather than after, so it reaches a
person rather than a log:

<!-- replay mcp dir=/tmp -->

    $ ank mcp --repo /tmp
    error[1]: no .ank/ found from /tmp
      -> ank init

## Several repositories, one server

**Several repositories do not need several servers.** `--repo` names the corpus
a call naming none of its own goes to; every other corpus that server may reach
is declared once, outside every repository, and the configuration above does
not change by a character. The declaration is written through the CLI, keyed on
the repository identity of the corpus and never on a path:

<!-- replay corpora dir=/srv/back
$ ank init
$ cd /srv/back && git init -q && ank init && ank new task --title "The back answers a query" --scope "**" --criteria "c" --no-verify && git add -A && git commit -q -m back
$ git -C /srv/back rev-list --max-parents=0 HEAD
bccc32d77d8a9a329f772f789dc5fb1054259d70
-->

    $ ank config --user corpora.bccc32d77d8a9a329f772f789dc5fb1054259d70 /srv/back
    corpora.bccc32d77d8a9a329f772f789dc5fb1054259d70 /srv/back

What that writes is `corpora.yml` (ADR-96174f1ac2b7), in the reader's
configuration directory ([Environment variables](environment.md#where-the-readers-configuration-lives)
says where that is):

<!-- replay corpora
$ cat "$XDG_CONFIG_HOME/ank/corpora.yml"
-->

    schema: 1
    corpora:
      bccc32d77d8a9a329f772f789dc5fb1054259d70: /srv/back

The identity is the root commit, which `ank status --json` prints under
`"corpus"`, so run that in the repository you want to declare and paste what it
gives you.

## What the surface is

Four properties of it are load-bearing, and none of them is visible from a tool
list.

**Every verb `COMMANDS` carries, generated from that table.** Not a curated
subset, under any protocol (ADR-fd98f4bc6dea). It is the same table `ank help
--json` is generated from, walked: the summary becomes the tool description, the
refusals and their exit codes are written into it so a client can read what a
call will refuse before making it, and the flags become the input schema. One
tool per verb, whatever the table carries, named `ank_<verb>` because a bare
`context` collides with every other server a client has loaded and `ank context`
is not a legal tool name. Positionals arrive as `arguments`, an array of strings,
exactly as they sit on the command line; flags arrive under their own names with
the leading dashes stripped. What a call gets back is the document `--json`
returns, with the exit code beside it. Nothing in the server names a verb, so
the two surfaces cannot disagree about what exists.

**One process may speak for several corpora, and never for a merged one.**
`--repo` is resolved once, at startup, and that corpus is where a call naming
none of its own goes, so a client that never passes the argument sees exactly
what it saw before the argument existed. Every tool also carries an optional
`corpus` argument (ADR-fd98f4bc6dea), whose value is the repository identity of
ADR-621a7fd96ce1 -- the root commit, never a path. One server, addressed at one
corpus at startup, answering out of another the reader declared:

<!-- replay corpora -->

    --> {"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ank_find","arguments":{"arguments":["--status","open"],"corpus":"bccc32d77d8a9a329f772f789dc5fb1054259d70"}}}
    <-- {"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"{\"contract\":1,\"corpus\":\"bccc32d77d8a9a329f772f789dc5fb1054259d70\",\"total\":1,\"shown\":1,\"hidden\":0,\"results\":[{\"id\":\"TASK-6a3615347674\",\"kind\":\"task\",\"status\":\"open\",\"state\":\"open\",\"title\":\"The back answers a query\",\"created\":\"2026-08-26T00:22:04Z\",\"archived\":false}]}"}],"isError":false,"exitCode":0}}

That permits multiplexing. It still forbids merging, and **telling those two
apart is the whole of the decision**, so it is worth being exact about which one
you are building. Every call becomes `ank --repo <one corpus> <verb> --json`,
one corpus at a time. There is no merged claim space, no claim held on a
client's behalf, and no arbitration across clones, because `refs/ank/*` is per
repository and cannot carry one -- the same ban federation gets
(ADR-a1de673043b4), carried into the multi-corpus clause in the same words.
Two claims taken through one server land in `refs/ank/claims` of two
repositories, and neither corpus carries a word about the other's task. So a
board over four repositories is one server and four corpora addressed on their
own, presented together by whatever sits above them; it is not four claim
spaces made into one, and a client that shows them as one list must not
arbitrate over that list. What a multi-corpus server does acquire is one
identity holding a lease in several corpora at once, and nothing beyond it.

The reachable set is **declared, and nothing is discovered**: the startup corpus
plus whatever `corpora.yml` declares. A caller cannot name a corpus by path, so
there is no spelling of "every corpus on this machine"; and an identity nobody
declared is refused by name, with nothing spawned and no falling back to the
corpus the client did not ask for:

<!-- replay corpora
>> {"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"ank_find","arguments":{"corpus":"0000000000000000000000000000000000000000"}}}
>> {"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"ank_find","arguments":{"corpus":"/srv/back"}}}
-->

    <-- {"jsonrpc":"2.0","id":4,"result":{"content":[{"type":"text","text":"error[9]: no corpus is declared under 0000000000000000000000000000000000000000, and this server reaches no corpus nobody declared\n  -> ank config --user corpora.0000000000000000000000000000000000000000 <path>"}],"isError":true,"exitCode":9,"stderr":"error[9]: no corpus is declared under 0000000000000000000000000000000000000000, and this server reaches no corpus nobody declared\n  -> ank config --user corpora.0000000000000000000000000000000000000000 <path>"}}
    <-- {"jsonrpc":"2.0","id":5,"result":{"content":[{"type":"text","text":"error[9]: '/srv/back' is not a repository identity\n  -> a corpus is named by its root commit, never a path, a remote or a slug: ank status --json prints it under \"corpus\""}],"isError":true,"exitCode":9,"stderr":"error[9]: '/srv/back' is not a repository identity\n  -> a corpus is named by its root commit, never a path, a remote or a slug: ank status --json prints it under \"corpus\""}}

Both are **9**, and 9 is the right code for both: what is missing is a
declaration in the reader's configuration, not anything in either corpus.

The three flags the server keeps for itself stay refused. A call that passes
`--repo`, `--json` or `--quiet` is turned away by name rather than being allowed
to contradict the process it is talking to, and `--repo` is turned away naming
the argument a caller reaches for instead:

<!-- replay corpora
>> {"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"ank_status","arguments":{"repo":"/srv/back"}}}
-->

    <-- {"jsonrpc":"2.0","id":6,"error":{"code":-32602,"message":"--repo belongs to the server: name a corpus with the corpus argument, by the identity ank status --json prints, never by a path"}}

Nothing is hidden by that and nothing is curated: every verb takes exactly the
arguments the table gives it, plus the one argument that says which corpus it
runs in.

**A refusal is the CLI's refusal, and it carries the CLI's exit code.** The
surface spawns `ank`; it does not link it. So a refusal on state is not
re-derived here, it is inherited, hint and all, and it comes back as a result
rather than as a protocol error, because the request was well formed and the
answer is no:

<!-- replay corpora
>> {"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ank_show","arguments":{"arguments":["TASK-9999"]}}}
-->

    <-- {"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"error[2]: entity not found: TASK-9999\n  -> ank find TASK-9999"}],"isError":true,"exitCode":2,"stderr":"error[2]: entity not found: TASK-9999\n  -> ank find TASK-9999"}}

`exitCode` is present on every call including a successful one, so a client that
branches on it never has to tell absence from zero; `stderr` is carried
separately for the reason warnings live there in the first place. The two error
channels stay apart: a JSON-RPC error means the *request* was wrong, a result
with `isError` means the *corpus* said no. A client that conflates them reports
its own bug as a state of your repository.

**No claim is taken that the CLI would not have taken in that clone.** Every
claim goes to `refs/ank/claims/<id>` in that repository -- the corpus the call
named, or the startup one where it named none -- arbitrated by the same
compare-and-swap against the same remote. The server holds no claim on a
client's behalf, renews none for anybody, and pools no clients under one
identity: one stdio server serves one client, so one process is one caller. It
writes under a typed process identity, `ank-mcp/<version>`, unless `$ANK_AGENT`
names one, so a deployment that already names its agents keeps naming them.

`accept` is a tool here like every other verb, because a generated surface
curates nothing out. Being reachable over a protocol changes nothing about it:
it still refuses off the default branch, with no way around it.

What a document, an exit code and a warning mean is the same here as on the
command line, and [the machine surface](integrating.md) is where it is said.
