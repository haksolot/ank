# Ratifying and archiving

Two acts decide what the corpus is, and both are a human's: `ank accept` makes a
proposed decision binding, and `ank archive` moves what is cold out of the hot
corpus. Both land on the default branch by pull request, like every other
change. This page is the recipe for each on this repository, where `main` is
protected and takes no direct push.

## Ratifying a decision

`ank accept` is the one act ank commits for, and it runs on the default branch
only, with no flag around it (ADR-6d8736c04cfa). A constraint ratified on a
feature branch would bind on that branch alone, which is a constraint of variable
geometry and a ratification hash that depends on where it is read.

That rule is about where you stand when you sign. It is not a licence to push to
`main`. `accept` writes a commit and stops; it never pushes. So the ratification
commit reaches `main` through a pull request like every other change, and CI sees
it before it lands:

    git switch main && git pull
    ank accept <id>                  # the gate is satisfied here
    git branch ratify/<id>           # branch first, at the ratification commit
    git reset --hard origin/main     # local main back where it was
    git push -u origin ratify/<id>
    gh pr create --fill --base main --head ratify/<id>
    gh pr merge ratify/<id> --merge  # a merge commit, and nothing else
    git switch main && git pull

Branch before resetting. The commit is then held by a ref, and a botched ordering
is a reflog recovery rather than a lost signature.

**Both of the last two `gh` lines name the branch, and neither naming is
decoration.** Nothing in this sequence ever switches to `ratify/<id>`: `git
branch` creates it without moving, and `git push -u origin ratify/<id>` pushes a
branch you are not standing on. So the shell is still on `main` when `gh` runs,
and `gh` resolves a pull request from the current branch.

A bare `gh pr create --fill` therefore reads `main` as the head, finds it is also
the base, and refuses with "head branch is the same as base branch". A bare `gh
pr merge --merge` looks for the pull request whose head is `main`, finds none,
and exits 1 with `no pull requests found for branch "main"` -- with the
ratification sitting unmerged on a branch, which is the worse of the two because
it looks like the sequence ran. Naming the branch in both is what makes this work
from where it leaves you.

**Merge with a merge commit, never a squash and never a rebase.** This is
load-bearing and not a matter of taste. A ratification is located by the *subject*
of its commit, `ratify <id>`, walked with `rev-list --full-history` and no path
restriction ([the file format](format.md#what-lives-outside-the-files) has the
walk), so any strategy that preserves the commit preserves the anchor:

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

Both are disabled on the repository and in its ruleset, and the ruleset has no
bypass actors: a maintainer cannot merge around it either. If the repository ever
requires branches to be up to date before merging, set the update method to merge
for the same reason. Which key signs, and how `check` judges the signature, is
[Signing keys](signing.md).

`accept` refuses a supersession while any tracked file outside `.ank/` still cites
the document it retires (ADR-3b6ba766a42e). Re-point those citations first, in
their own change: the refusal names every site with its line, and there is no
bypass.

## Archiving what is cold

`ank archive` moves what is cold into `.ank/archive/entities/`: superseded
documents, and every entry whose subject is cold, meaning a superseded document
or a task done on the default branch (ADR-467ce7e9cda1). `check` names the verb
in one signal when the hot corpus holds any of it. Like `accept`, it decides what
the corpus is, so a human runs it and the result lands by pull request. Unlike
`accept`, it commits nothing: it renames files and stops, and the commit is
yours.

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
