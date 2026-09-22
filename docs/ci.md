# Running ank in CI

Read this part before the recipes, because the recipes are the easy half. The
whole integration surface is two things: **an exit code, and `--json`**. Learn
those and you can write the pipeline for a CI system nobody here has heard of.

`ank check` is the verb a pipeline runs, and it has three answers a pipeline
routes on: **0** for a healthy corpus, signals included; **8** for findings,
which is the failure a pipeline exists to catch; and **9** for the environment
rather than the corpus -- git too old, `sh` missing, the default branch
indeterminable. A pipeline that collapses 9 into "the check failed" sends
somebody to fix sound work. What the levels mean is [Reading ank
check](check.md).

`--json` is opt-in on every verb and is what you parse. It carries no colour and
no layout, and it stays byte-for-byte what your parser already reads.

That is the contract. Everything below is the CI system's own syntax around it.

## Check out the whole history

On GitHub that is one line. `ank check` walks history: it reads the ratification
commit that anchors a frozen constraint, and it asks where a scope that matches
nothing went, a rename or a deletion. `actions/checkout` fetches one commit by
default, and a corpus read from one commit does not report itself unreadable --
it reports itself unverified, at exit 0. Measured on this repository at
8310e75: a `--depth 1` clone answers
`check: ok — 440 tasks, 61 adr, 728 signal(s)` where the full clone of the
same commit answers 663, and 71 of those extra signals read
`ratified, but no ratification commit is reachable: the freeze cannot be verified`,
each noting that `the history here is shallow, so where it went cannot be
verified (git fetch --unshallow)`. Green, and every frozen constraint
unchecked, which is a worse failure than the red it replaces:

    - uses: actions/checkout@v5
      with:
        fetch-depth: 0

Every host spells the depth its own way, and the thing to carry across is the
property rather than the key: `check` needs the history that reaches the
ratification commits, and a shallow checkout is silent about what it could not
read.

## The recipes

**A bare shell**, which is the recipe the other two wrap:

    #!/bin/sh
    code=0
    ank check || code=$?
    case $code in
      0) ;;
      8) echo "ank check: findings, see above" >&2; exit 1 ;;
      9) echo "ank check: environment unavailable, not a corpus failure" >&2; exit 2 ;;
      *) echo "ank check: unexpected exit $code" >&2; exit 1 ;;
    esac

**GitHub Actions:**

    - name: ank check
      run: |
        code=0
        ank check || code=$?
        case $code in
          0) ;;
          8) exit 1 ;;
          9) echo "::notice::ank could not run: environment"; exit 2 ;;
          *) exit 1 ;;
        esac

**GitLab CI:**

    ank:check:
      script:
        - |
          code=0
          ank check || code=$?
          case $code in
            0) ;;
            8) exit 1 ;;
            9) echo "ank could not run: environment"; exit 2 ;;
            *) exit 1 ;;
          esac

Three vendors, one contract, and the third one is a shell script in a YAML file
like the other two.

`ank check || code=$?` and never a bare `ank check` followed by `case $?`: a
GitHub Actions `run:` block is `bash -e`, so the bare form aborts the step on
exit 8 and the routing you wrote is never reached. The `|| code=$?` form is what
survives `set -e`, which is why it is in all three rather than in the one that
needs it.

**There is no `--format github`, and there never will be.** Annotations,
folding markers and job summaries are one vendor's protocol, and putting them in
the binary would couple the tool to a company. A pipeline that wants annotations
pipes `--json` into whatever produces them, which is exactly the arrangement
that lets the fourth vendor work without anybody shipping a release for it.

## Anchoring a run

`done` records what ran on the machine that ran it. That is a local claim, and a
pipeline can anchor the same task to a run anybody can re-read
([Proof and verifiers](proof.md#what-a-proof-is-worth) says why that is worth
more):

    ank attest <id> --proof test:<run-id> --detached

`--detached` writes the proof to `refs/ank/proof/<id>` and touches no file, so
the pipeline produces **no commit**: it needs no write access to the branch and
cannot race the merge. It is still a write to the remote, though, and on GitHub
that is a permission: the job that attests raises it, to the one thing it
writes, and every other job keeps `contents: read`.

    permissions:
      contents: write

**Which ids.** `check` is the work list, and nothing diffs `.ank/` to build it.
A finished task with no external anchor carries the signal
`done with no test proof: nothing external anchors it`, so the ids are the
subjects of that finding and `--json` is what a pipeline reads them out of:

    ids=$(ank check --json | jq -r '
      .findings[]
      | select(.message | startswith("done with no test proof"))
      | .subject')

**Fetch the proofs already written first**, before that `check` runs.
`actions/checkout` fetches history and not `refs/ank/*`, so a job that skips
this reads a corpus in which nothing is anchored and re-attests every finished
task, on every push, forever. Measured on run 33285805350: 199 ids listed and
336 seconds spent pushing refs that were already there, against 16 genuinely
unproved in a clone that carries them.

    git fetch origin '+refs/ank/proof/*:refs/ank/proof/*'

One direction and read-only. A refspec matching nothing is not an error, so a
corpus with no proofs yet passes through untouched.

**Run it on the default branch and on a push, and nowhere else.** The signal is
gated on the task appearing done on the default branch, so on a feature branch
straight after `done` there is nothing to anchor and this job would build for
two minutes to be told so. A `pull_request` event runs on a merge commit no
branch carries, and a fork's token cannot push a ref at all, which would fail
the job for a reason about the event rather than about the corpus:

    if: >-
      github.event_name == 'push' &&
      github.ref == format('refs/heads/{0}', github.event.repository.default_branch)

The identity doing the attesting is typed like any other, and a pipeline is
`process:` ([Claims and identity](claims.md#one-identity-per-session)):

    env:
      ANK_AGENT: process:github-actions

The proof is a ref, so it has to reach the remote to be worth anything, and
because the ref is the whole of what this verb produces, a push that did not
land is a failure and not a warning: `attest --detached` exits **9** and names
the push to run. Nothing special is needed to notice it, which is the point:

    ank attest "$id" --proof "test:$RUN_ID" --detached

`--json` still reports `"pushed"`, so an integration that prefers to read the
flag reads the same fact. What it must not do is read the flag *instead* of the
code, because the two now say the same thing.

Attest on every run without worrying about the ref: it grows with facts and
never with runs. A fact is a proof type, the criteria hash it was attested
against and the identity attesting it, and a second run of the same fact
replaces the first, so `show` lists the latest run. A corpus whose proof refs
were written before that rule carries one entry per run, and `check` names each
such ref with the command that rewrites it, one ref by name:

    ank attest <id> --compact --detached

It keeps one entry per fact, adds nothing and pushes that one ref. Never push
`refs/ank/*` with a wildcard to do the same: from a worktree, that force-reverts
the attestations a pipeline wrote.

This repository's own pipeline does all of the above in its `attest` job, and
[CI jobs and required checks](ci-jobs.md) maps it.
