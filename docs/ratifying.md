# Ratifying and archiving

Two acts decide what the corpus is, and both are a human's: `ank accept` makes a
proposed decision binding, and `ank archive` moves what is cold out of the hot
corpus. This page is the recipe for each on this repository, where `main` is
protected and takes no direct push.

## Ratifying a decision

Ratifying is one command, on whatever branch you are on:

    ank accept <id>...

For each id, `accept` writes an annotated tag object at `refs/ank/ratify/<id>`,
signed where the repository signs ([Signing keys](signing.md)), and rewrites the
entity file: `status: accepted`, `ratified:` with the hash, `anchor:` naming the
tag object, and a `verified` entry. It then pushes the ref to `origin` with
compare-and-swap. **It commits nothing**, and it carries no branch precondition
(ADR-5e47007a75be): the ref is a fact about the repository, not about a branch.

When the push does not land, `accept` exits 9 and keeps nothing: no ref, no
change to the file. A ratification readable by nobody else did not happen, and
the verb says so rather than warning (ADR-af533e7a3e03).

**The ref binds as soon as it reaches the remote.** `context` and `scope` serve
a decision as binding wherever a verified ref matches the constraint and scope
of the file they read, whatever its `status:` says, so a branch that still
carries the file as `proposed` is bound by it too. `check` names a file that
has not caught up, as a signal: `ratified by refs/ank/ratify/<id>, but
.ank/entities/<id>.md still reads proposed: the file has not caught up`.

**The file lands with the next change, by any merge method.** Commit it on the
branch you are working on, or on one of its own, and let it reach `main`
through a pull request like anything else. It does not need a pull request of
its own, a merge commit, or anybody to come back for it: the authority is the
ref, and the file only declares what the ref already makes true.

A clone reads the refs only once it has fetched them. `git clone` does not
fetch `refs/ank/*`, and the first `ank check` or `ank status` in a clone adds
the refspec and fetches them once; the line below does it by hand. A clone
without them falls back to the file and sees `proposed`: less, never wrong. CI fetches them before `check` ([CI jobs](ci-jobs.md)).

    git fetch origin '+refs/ank/*:refs/ank/*'

`accept` refuses a supersession while any tracked file outside `.ank/` still cites
the document it retires (ADR-3b6ba766a42e). Re-point those citations first, in
their own change: the refusal names every site with its line, and there is no
bypass.

### Without ank

The ratification is git plumbing, so a repository without the binary can make
one. Bring the file to the form `accept` writes, every field but `anchor:`
([the file format](format.md#what-lives-outside-the-files) has the fields, and
[the two hashes](format.md#the-two-hashes) how `ratified:` is computed), then:

    git hash-object -w .ank/entities/<id>.md          # the blob
    git tag -s -m 'ratify <id>' -m '<lines>' ratify/<id> <blob>
    git update-ref refs/ank/ratify/<id> "$(git rev-parse ratify/<id>)"
    git tag -d ratify/<id>                            # the object stays, the tag name goes
    git push --force-with-lease=refs/ank/ratify/<id>: origin refs/ank/ratify/<id>

and add `anchor: <tag>` to the file, the object name `git rev-parse` printed.
`<lines>` are `constraint+scope: <hash>` (or `body+scope: <hash>` for a spec)
and `by: <actor>`, and `supersedes: <id>` where one applies. In a repository
that does not sign, `tag -s` is `tag -a`. The temporary name under `refs/tags/`
lives between the second and the fourth command only, because a ratification is
not a release and must not look like one to a forge. The empty lease is the
compare-and-swap: the push is refused, `! [rejected] ... (stale info)`, when
the remote already holds a different ratification under that name.

### Anchors made before the ref

Before ADR-5e47007a75be, a ratification was a commit whose subject is
`ratify <id>`, and those anchors are never rewritten. `check` reaches a decision
by its ref first, and by walking `rev-list --full-history` for that subject only
when no ref carries it. **The merge-commit rule now protects these anchors and
nothing else**, and it stays because keeping it costs nothing:

- a **merge commit** keeps the subject, the SHA and the signature, and `ank check`
  verifies the ratification exactly as if it had been committed in place;
- a **squash** rewrites the subject to the pull request title, so the anchor is
  never found again. `check` reports the entity as unverifiable, which is a
  signal and exit 0: the corpus quietly stops being verifiable while CI stays
  green;
- a **rebase** keeps the subject, so the anchor is still found, but it replays the
  commit without its signature. In a corpus that signs, and this one does, `check`
  reports that as a fault (ADR-964be4d940b2 makes signing a regime the corpus is
  in, so an unsigned corpus survives a rebase and this one would not).

A squash or a rebase of a branch carrying no `ratify <id>` commit loses
nothing, and every ratification made by `accept` today is such a branch. Both
are still disabled on the repository and in its ruleset, which has no bypass
actors, for as long as a commit anchor lives in the history. Which key signs,
and how `check` judges the signature, is [Signing keys](signing.md).

**A commit anchor moves into a ref with one command:**

    ank accept <id> --reanchor

It writes the tag and the ref that `accept` writes today, over the same anchor:
the message carries the `constraint+scope:` (or `body+scope:`) hash the commit
recorded, any `supersedes:`, and a `reanchors:` line naming the commit it moves.
The tag is signed by whoever runs it, and the `by:` line and the `verified`
entry it adds to the file name that person, not the author of the first
ratification: re-anchoring is a new reading, and it says so. It pushes the ref
like `accept` does, and fails at exit 9 with nothing kept when the push does
not land. **Nothing in history is rewritten**: the `ratify <id>` commit stays
where it is, and nothing is committed; the file, now naming the tag under
`anchor:`, lands with the next change. From then on `check` judges the ref, so
the decision verifies in a shallow clone that never held the commit, and `ank
show <id>` says the tag was re-anchored from that commit.

It refuses, writing nothing:

- at exit 6, a document a ref already carries: `<id> is already carried by
  refs/ank/ratify/<id> (tag <sha>): there is no commit anchor left to move`;
- at exit 6, a document whose file has moved from the anchor its commit
  records: `<id> has moved from the anchor its commit <sha> records (ratified
  <hash>, now <hash>): re-anchoring would sign the edit`. The commit ratified
  the old text, and a new reading of it does not ratify the new one: the way
  forward it names is `ank new adr --supersedes <id>`;
- at exit 6, a document no commit ratified: `<id> is proposed, and no
  ratification of it has an anchor to move`, and the command it names is a
  plain `ank accept <id>`;
- at exit 7, a document whose `ratify <id>` commit this clone does not hold:
  `no ratify <id> commit is reachable from this clone, so there is nothing to
  re-anchor from`, which a shallow clone answers with `git fetch --unshallow`.

### Moving the scope of a ratified decision

When the code moves and the decision does not, the perimeter follows the file
without a new identifier (ADR-bb9ccdba9be7). Two commands, the second a human's:

    ank amend <id> --drop-scope "<old>" --scope "<new>"
    ank accept <id> --rescope

`amend` edits the scope of an accepted ADR or spec, and only its scope. The
file then no longer matches the anchor its ref carries, and until the rescope
`check` says so as a signal, `its scope was amended after its ratification,
and .ank/entities/<id>.md has not caught up with it: ank accept <id>
--rescope`. While it waits, `context` does not inject the constraint: the
scope it would bind is one nobody has signed yet. When `check` reports a scope
git records as renamed on an accepted document, the repair it names is these
two commands.

`accept --rescope` signs that scope, like `accept`, where the repository signs.
It writes a new tag in place of the one at `refs/ank/ratify/<id>`, with
compare-and-swap here and on `origin`, and rewrites `ratified:`, `anchor:` and
`verified` in the file. The message carries the new anchor, the previous one,
and the previous scope, one line per glob:

    ratify <id>

    constraint+scope: <new hash>
    previous: <previous hash>
    previous-scope: <glob>
    by: <actor>

`check` holds the chain to what it claims: the signature, the new anchor
against the file, and the previous anchor recomputed from the current
constraint (or body) with the previous scope. A tag whose previous anchor does
not recompute is a fault, `its rescope <tag> names the previous anchor <hash>,
but its constraint with the previous scope hashes to <hash>: the decision
changed under a rescope`. The identifier, the status and every citation stay.
Like `accept`, it commits nothing and fails at exit 9 with nothing kept when
the push does not land, including when `origin` holds another tag than the one
this clone replaced.

It refuses at exit 6, writing nothing:

- a document that is not accepted: `<id> is proposed, and only an accepted
  document has a scope to rescope`;
- a scope that did not change: `<id> declares the scope its ratification
  carries: there is nothing to rescope`;
- a constraint, or a spec's body, that changed since the ratification: `<id>'s
  constraint changed since its ratification: a rescope only moves the
  perimeter`. Changing the decision is `ank new adr --supersedes <id>`;
- a document no ref carries. An anchor that is still a `ratify <id>` commit is
  moved into a ref first, before the scope is amended, which `amend` asks for
  at exit 6 and `check`'s repair names in that order:

      ank accept <id> --reanchor
      ank amend <id> --drop-scope "<old>" --scope "<new>"
      ank accept <id> --rescope

## Archiving what is cold

`ank archive` moves what is cold into `.ank/archive/entities/`: superseded
documents, and every entry whose subject is cold, meaning a superseded document
or a task done on the default branch (ADR-467ce7e9cda1). `check` names the verb
in one signal when the hot corpus holds any of it. Like `accept`, it decides what
the corpus is, so a human runs it, and like `accept` it commits nothing: it
renames files and stops. Unlike `accept`, it writes no ref, so the move exists
once it lands, and the commit and its pull request are yours.

    git switch main && git pull
    git switch -c archive/<date>
    ank archive --dry-run            # read the list: this is what moves
    ank archive                      # the same list, moved
    ank check                        # green: references, blockers and scopes resolve into the archive
    git add -A .ank                  # git sees each file as a rename
    git commit -m "archive what is cold"
    git push -u origin archive/<date>
    gh pr create --fill --base main --head archive/<date>
    gh pr merge --merge

`git switch -c archive/<date>` puts you on the branch, so here `gh` resolves the
pull request from where you stand and the bare `gh pr merge` works.

Run it from a branch cut from the default branch and level with it: an entry is
cold when its task is done *on the default branch*, which is what the verb reads,
so a local default branch that is behind leaves entries hot that could have
moved. An archived file is never edited afterwards; `check` verifies it against
the digest it arrived with and reports a changed one as a fault.
