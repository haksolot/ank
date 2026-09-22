# Claims and identity

A claim is how one agent takes a task and every other agent sees that it is
taken. It is a git ref rather than a field, it lasts as long as somebody is
working, and it is held by an identity the caller declares. This page is the
whole lifecycle, from `claim` to `release` or `done`.

## What a claim does

<!-- replay claims ANK_AGENT=human:marie
$ mkdir -p src/auth && echo 'export function createSession(id) { return store.put(id) }' > src/auth/session.ts && echo '# auth service' > README.md
$ ank init && ank config default_branch main
$ ank new task --title "Migrate auth to opaque sessions" --scope "src/auth/**" --criteria "The auth tests pass and no reference to jwt.verify remains in src/auth/" --no-verify
created TASK-820d259af6a7 Migrate auth to opaque sessions
$ ank new task --title "Say in the README what a session is now" --scope "README.md" --criteria "The README describes opaque sessions and names no JWT" --no-verify
created TASK-51c2a0f6d418 Say in the README what a session is now
$ git add -A && git commit -q -m "the service"
-->

    $ ank claim 820d
    claimed TASK-820d259af6a7 migrate-auth-to-opaque-sessions -> HEAD

Three things happened. The task moved to `in_progress`. A claim ref appeared at
`refs/ank/claims/TASK-820d259af6a7`, which is what arbitrates two people or two
agents reaching for the same task: git's compare-and-swap, one winner. And the
`done_criteria` was frozen: its hash went into the claim record, where the
file's editor cannot reach it.

**The freeze is the rule worth internalising.** Editing the criterion to make the
work fit does not unblock anything; `done` compares against the recorded hash
and refuses, and `check` reports the divergence. If the criterion is genuinely
wrong, hand the task back and say so. A subtask you discover is a new task with a
`blocked_by`, never a softened criterion.

`claim` also sets HEAD, so the commands that follow need no id. One claim at a
time, per identity.

## One identity per session

Claiming a second task while you already hold one is refused:

<!-- replay claims -->

    $ ank claim 51c2
    error[7]: human:marie holds a live claim on TASK-820d259af6a7 (expires in 30m)
      -> ank release --reason "<why>"   (a second session on this machine sets its own ANK_AGENT)

If you meant it, the first way out is the one to take: finish the task you hold
or hand it back. If the refusal surprises you, it is almost certainly two
terminals. `$ANK_AGENT` names the session, and unset it falls back to
`<user>@<hostname>`, so two sessions on one machine are the same agent as far as
the refs can tell. Give every concurrent session an identity of its own:

<!-- replay claims -->

    $ ANK_AGENT=human:marie-2 ank claim 51c2
    claimed TASK-51c2a0f6d418 say-in-the-readme-what-a-session-is-now -> HEAD

**Two sessions sharing an identity are not arbitrated, they are merged.**
Claiming the task the identity already holds is granted again at exit 0, so a
second session with no `ANK_AGENT` of its own silently starts work the first one
is already doing:

<!-- replay claims -->

    $ ank claim 820d
    claimed TASK-820d259af6a7 migrate-auth-to-opaque-sessions -> HEAD

So the failure mode is not a message nobody sees. It is two sessions writing the
same perimeter under one name, which the refs cannot tell apart and no reader
can untangle afterwards. That is why the refusal above names the identity rather
than calling you its holder: under a shared identity, the session being refused
may have claimed nothing at all.

**Write the identity typed.** An identity that goes into an entity says what
kind of actor it is (ADR-3877fef1d662): `human:<id>` is a person,
`<producer>/<version>` an agent -- `claude-code/opus-5`, and a suffix after `+`
for one session of it -- and `process:<id>` something automated, which is the
form a pipeline uses. The convention is what lets `check` say that an entity was
written by an agent and read by no human; the fallback `<user>@<hostname>`
carries no type and so answers that question with nothing. It is a signal and
not a wall -- anyone can type `human:` in front of a model -- and what it buys is
that the ordinary case is legible.

Identity is declared, never proved, and it is deliberately not bound to the
session: a PID or a TTY in it would mean losing your claim to a restarted
terminal. `$ANK_AGENT` records who was working rather than restricting who may.
Nothing in ank refuses on identity; the refusals are on state, and the one hard
authority line is the signed ratification commit `ank accept` produces.

## The lease, and what renews it

A claim lasts 30 minutes by default and is renewed by working, not by reporting
(ADR-0bb7ea8991bc). Three verbs move the lease, measured by reading the expiry
out of `ank status --json` before and after each call:

- `ank context`, in every form -- bare, with a path, with `--json`.
- `ank show <id>`, when `<id>` is the task this identity holds. `ank show` over
  any other entity leaves the lease alone, so it is the *subject* that renews
  and not the verb.
- `ank log "<message>"`, the appending form. `ank log <id>`, which reads, does
  not.

`find`, `status`, `scope`, `graph`, `check`, `review` and `help` leave the expiry
untouched. So there is no heartbeat command to remember: writing down what you
learned is what keeps the lock. A tool that shows a corpus to somebody must not
put `context` or `show` on a timer, and [the machine surface](integrating.md)
says what to poll instead.

An expired claim is not a live one. If a build ran long and the lease ran out,
the task stays `in_progress` and you re-acquire it silently, provided nobody
took it over; and nothing stands between anybody and a task whose lease ran out,
yours or another's. A task that is `in_progress` in its file with no ref behind
it is simply one whose claim expired.

## Handing a task back

<!-- replay claims -->

    $ ank release --reason "the criterion names a test that does not exist yet"
    released TASK-820d259af6a7 -> open

The reason is recorded in the task's log, where the next holder reads it with
`ank log <id>` before repeating what you tried. Never let a claim lapse in
silence: an expired lease tells the next agent nothing about why.

## When done, the claim becomes a completion

`done` does not delete the claim ref. It turns it into a completion record naming
the commit and the branch the task was finished on, because `status: done` lives
in the file, therefore on your branch alone until the merge, and during all that
time the task would look free to everyone else. Anyone who tries to claim it
meanwhile is refused with the commit and the branch named, and every other tree
answers `finished on another branch (commit …, branch …), not merged here yet`.
`check` prunes the record once the default branch says the task is done:
[Reading ank check](check.md) shows both states.

## Where claims travel

**Coordination between clones needs a remote named `origin`, and nothing
else.** Whether claims travel is not configured: a repository with an `origin`
pushes every claim to it as a compare-and-swap, and one without keeps them as
local refs. GitHub is not required. A bare repository reachable over `file://`
or `ssh` on the same network is a remote named `origin` like any other: `claim`
reads it first, so the second clone to take a task is refused with code 4 and
the holder named, and the push settles a race the read misses. `ank init` adds
the refspec that makes `refs/ank/*` travel, since hosts do not fetch
non-standard refs on their own.

Without a remote, `git worktree`s of a single clone are still arbitrated,
because they share `refs/ank/`. **Two clones with no common origin are not
arbitrated: both claims of one task succeed, both agents work, and nothing
reports it**, not `status`, not `check`, not later.

A push the remote refuses leaves the claim standing in your clone and says so on
stderr: the claim holds there only, and another clone can take the same task. A
contributor working from a fork is in exactly that position, and
[CONTRIBUTING.md](https://github.com/haksolot/ank/blob/main/CONTRIBUTING.md#working-from-a-fork)
says how coordination happens instead.
