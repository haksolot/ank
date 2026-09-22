# Multi-agent work

Ank is built for several agents working one repository at once, each taking a
task the others can see is taken. This page is what handing the loop to an agent
involves, and what running several of them actually requires. The skills that
teach an agent the loop are installed by the routes in [Install](install.md#the-skills),
and claims and identities, which do the arbitrating, are [Claims and
identity](claims.md).

## What the agent is taught

`skill/SKILL.md` is the contract an agent loads, and the five siblings beside it
carry a policy each -- planning, drift audit, the autonomous loop, test-first
implementation, diagnosis -- loaded when the activity calls for them.

One convention it carries is worth knowing before you watch an agent follow it:
**`.ank/` is opaque to an agent, the way `.git/` is.** Reading goes through
`ank show`, `ank find` and `ank context`; writing goes through the verbs. The CLI
knows what the files do not: the context budget, the frozen criterion, who holds
which claim. A human with an editor keeps every power they had.

**What that costs a session is the frontmatter, not the page.** A skill's `name`
and `description` are always loaded, somewhere between fifty and eighty tokens
each; a body is read when its skill is invoked. [Install](install.md#claude-code-as-a-plugin)
shows the projection per skill.

### The methods a corpus uses

A task can name the sibling its work calls for with `ank new task --method tdd`,
and `ank context` prints that name beneath the criterion once the task is
claimed. A sibling that opens under a claim writes an entry saying so with
`ank log --method tdd`, titled with its name and kept apart from the work trace
(ADR-a8f9c603a0e7). The designation is read, never enforced, and `ank skills` is where it
is read: inside a corpus it prints the catalogue and then a second block. On a
corpus of three tasks -- one designating `tdd` whose holder loaded it, one
designating `diagnose` whose holder never did, and one designating nothing where
`tdd` was loaded anyway -- the block reads:

<!-- replay methods part
$ ank init
$ id=$(ank new task --title "Designates tdd" --scope "**" --criteria "c" --no-verify --method tdd | cut -d' ' -f2) && ANK_AGENT=a/1 ank claim $id && ANK_AGENT=a/1 ank log --method tdd
$ id=$(ank new task --title "Designates diagnose" --scope "**" --criteria "c" --no-verify --method diagnose | cut -d' ' -f2) && ANK_AGENT=b/1 ank claim $id
$ id=$(ank new task --title "Designates nothing" --scope "**" --criteria "c" --no-verify | cut -d' ' -f2) && ANK_AGENT=c/1 ank claim $id && ANK_AGENT=c/1 ank log --method tdd
$ ank skills
-->

    METHODS
    diagnose  designated 1  fired 0  undesignated 0
    drift     designated 0  fired 0  undesignated 0
    loop      designated 0  fired 0  undesignated 0
    plan      designated 0  fired 0  undesignated 0
    tdd       designated 1  fired 1  undesignated 1

`designated` counts the tasks naming the sibling, `fired` those of them carrying
its entry, and `undesignated` its entries on tasks naming none. `ank skills
--json` carries the same counts as integers. The numbers are read from the
corpus, printed to whoever asks, and sent nowhere; outside a corpus the verb
prints the catalogue alone.

## One agent, one working tree, one identity

The nominal case is a tree per agent, a clone or a `git worktree`, each on its
own branch cut fresh from the default one, and each session with an
`ANK_AGENT` of its own. `ank status` names the drift from the default branch,
and a stale base turns a green tree red elsewhere.

Several agents in one tree runs, and it is a degraded mode rather than the
design: two sessions sharing a tree and an identity share a claim instead of
arbitrating over it ([Claims and identity](claims.md#one-identity-per-session)
shows what that looks like). What arbitrates properly is a `git worktree` per
agent, because every worktree of a repository shares `refs/ank/`, so the
compare-and-swap settles them. Separate clones are arbitrated only when there is
a remote, which is what the push carries.

**Take the task that cannot collide, or take none.** `status` says what another
agent holds; `claim` names a live claim whose scope intersects yours and takes
the task anyway (ADR-052accd6e3b2), a fact to read and not an error to refuse;
`graph` shows what `blocked_by` orders. When nothing open is both unblocked and
clear, an idle session is cheaper than two agents rewriting one perimeter.

## Parallel work and integration

The section above says who works where. This one assembles the whole run:
several tasks, several agents, one change landing on the default branch.

**Parallelism is derived, not declared.** `blocked_by` is the only relation
between tasks, and it is the only thing that serializes work. Tasks whose
blockers are finished are ready together, and `ank context` computes that
mechanically: every open task in the perimeter, the ready ones first, ordered by
how many other tasks each would unblock. Do not serialize independent tasks
because they belong to the same change. If the order matters, that is a
`blocked_by`; if there is no `blocked_by`, the order is a fiction. `ank graph`
prints the DAG when you want the shape rather than the next move.

**One branch per task.** Each agent claims its task, works in its own tree on
its own branch, and finishes there. `ank done` proves the task in the working
tree it ran in: the verifiers ran against those files and the proof records
their hash. It does not prove the change merges, or that the combined system
works. Until the merge lands, the claim ref stands as a completion record, so no
other tree takes the task for free ([Claims and
identity](claims.md#when-done-the-claim-becomes-a-completion)).

**Integration is a task.** When several tasks form one change, the whole is
verified the way the parts were: an ordinary task, `blocked_by` each part, with
its own criterion and its own verifiers. This is the spec's model rather than a
workaround: `blocked_by` is a DAG with no rollup precisely because a parent
that completes when its children do is completion without proof, and the seam
between the parts is exactly where integration regressions live. The
integration task becomes ready when the last part finishes; whoever claims it
merges the branches, runs the combined verification, and records `done` like
any other task.

Where the branches meet is git's business, and both shapes are legitimate:

- **Independent tasks merge to the default branch directly.** Two tasks that
  share nothing need no ceremony between them, and no integration task either.
- **A multi-task change goes through an integration branch.** Branch it off the
  default branch, merge each task's branch into it, resolve conflicts there,
  and point the integration task's verification at the combined result. The
  default branch receives one verified change instead of three partial ones.

**What ank will not do.** No verb creates a worktree, names a branch, or merges
one. Tasks, claims, criteria, dependencies and proofs are ank's plane; branches,
worktrees, merges and history are git's, and git is already good at them. The
one place the planes touch is `accept`, which runs on the default branch and
nowhere else.
