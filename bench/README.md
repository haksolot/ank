# bench

The agent bench: does a virgin agent, handed a repository, find and keep the
decisions recorded in it? ADR-bafbd28498cc says what the bench is held to; this
page says how to drive it.

Python 3, standard library only. Nothing here is a crate or is shipped.

## Pieces

- `fixture.py` writes one repository per (scenario, condition, seed). The same
  seed writes the same bytes, commits and refs included. Conditions: `none`
  (the code alone), `markdown` (the decisions as `docs/adr/*.md` and a
  `CLAUDE.md` pointing at them), `ank` (the decisions and the tasks as an ank
  corpus, laid out with the `ank` on PATH).
- `scenarios.py` holds each scenario: its code, its decisions, its prompt, and
  the probe the scorer runs on the code the agent left. The probe never enters
  the fixture.
- `run.py` launches one agent on one fresh fixture and appends one result line.
- `score.py` scores a transcript and the tree beside it; `trace.py` prints a
  transcript one step per line.

## Running

A run spends the quota of whoever launches it, so CI never runs one: CI runs
the tests below. Look before you spend:

    python3 -m bench.run --model claude-sonnet-5-5 --condition ank --rep 1 --dry-run

The agent gets a configuration directory holding nothing but a copy of your
credentials (`$CLAUDE_CONFIG_DIR/.credentials.json`, else
`~/.claude/.credentials.json`), no MCP server, and the model by its full id;
aliases are refused. Drop `--dry-run` to launch. Each run appends one line to
`results/<family>.jsonl` naming the model, harness version, ank version, seed,
scenario, condition and repetition beside its scores.

A number is published only from at least five repetitions per (scenario,
model, condition), under all three conditions.

## Results: adoption, discount-cap

`results/adoption.jsonl`, 45 runs: five per cell, seed 7, harness 1,
ank 0.9.2 (2e364115), Claude Code 2.1.310, launched 2026-10-26. The agent is
asked for `Cart.total_with_discount(percent)` and is never told about the
finance decision that refuses a reduction above three tenths, nor about ank.
A grep for the method or for "discount" finds the task but not that decision.

| model | condition | runs | cap kept | `ank context` | claimed | closed, committed | hand reads of `.ank/` | `docs/adr` read | `--help` typed | mean turns |
|---|---|---|---|---|---|---|---|---|---|---|
| claude-haiku-5-5 | none | 5 | 0/5 | 0 | 0 | — | 0 | 0 | 0 | 9.4 |
| claude-haiku-5-5 | markdown | 5 | 5/5 | 0 | 0 | — | 0 | 5 | 0 | 8.4 |
| claude-haiku-5-5 | ank | 5 | 5/5 | 5 | 5 | 4/5 | 2 | 0 | 2 | 12.6 |
| claude-sonnet-5-5 | none | 5 | 0/5 | 0 | 0 | — | 0 | 0 | 0 | 8.4 |
| claude-sonnet-5-5 | markdown | 5 | 5/5 | 0 | 0 | — | 0 | 5 | 0 | 6.6 |
| claude-sonnet-5-5 | ank | 5 | 5/5 | 5 | 5 | 5/5 | 0 | 0 | 3 | 9.2 |
| claude-opus-5-5 | none | 5 | 0/5 | 0 | 0 | — | 0 | 0 | 0 | 4.2 |
| claude-opus-5-5 | markdown | 5 | 5/5 | 0 | 0 | — | 0 | 5 | 0 | 6.6 |
| claude-opus-5-5 | ank | 5 | 5/5 | 5 | 5 | 5/5 | 0 | 0 | 0 | 10.0 |

Every one of the 45 runs left a working method (`works`). "Cap kept" is the
hidden probe: a 31 percent reduction raises ValueError. "Closed, committed"
means the target task reads `status: done` in HEAD. The other columns are
counts of runs. They come from the score fields `ank_context`, `claimed`,
`task_done`, `cat_corpus`, `read_markdown` and `help_flag`.

What it shows:

- **A record is what keeps the constraint.** With no record, no model kept
  the cap (0/15). With either record, every model did. Plain Markdown and ank
  tie at 15/15 each, so this scenario does not separate them on compliance.
- **ank costs turns.** In the ank condition, mean turns are above the markdown
  control for every model: 12.6 against 8.4 for Haiku, 9.2 against 6.6 for
  Sonnet, 10.0 against 6.6 for Opus. Those turns are spent on `context`,
  `claim`, `done` and reading `ank help`.
- **ank adds a trail that Markdown does not.** All 15 ank runs ran
  `ank context` and claimed the task. 14 of them closed it through `ank done`,
  with the verifier's proof recorded. Haiku rep 2 ran `ank done` and got
  exit 0, then committed only the code. Its `.ank/` change was left in the
  tree, so HEAD does not show it closed. The markdown runs left no record of
  their work.
- **Hand reads stay rare, and only Haiku did one.** Two Haiku runs ran
  `cat .ank/entities/TASK-…md` after `ank context`. That file is the task an
  earlier grep had surfaced. No Sonnet or Opus run read the corpus by hand.
  Five ank runs (2 Haiku, 3 Sonnet) typed a `--help` flag instead of using
  `ank help`.
- **The isolation is not total.** Every run's init event lists three built-in
  plugins: `cc-plugin-agents-md`, `cc-plugin-telemetry` and
  `cc-plugin-plugin-authoring`. It lists no MCP server and no skill. The
  plugins are the same in all 45 runs, so they do not separate the
  conditions, but the agent was not fully bare.

### Replaying the campaign

From the repository root, with `claude` and the `ank` under test on PATH:

    for rep in 1 2 3 4 5; do for m in claude-haiku-5-5 claude-sonnet-5-5 claude-opus-5-5; do
      for c in none markdown ank; do echo "$m $c $rep"; done; done; done |
    xargs -P 3 -L 1 sh -c 'python3 -m bench.run --model "$0" --condition "$1" --rep "$2" \
      --work "$HOME/bench-work" --ank "$(command -v ank)"'
    python3 -m bench.score --check bench/results/adoption.jsonl

Each run appends to `results/adoption.jsonl`. Move the committed file aside
first, or pass `--results` to write a new one. `--check` exits 8 and names
any line that misses the schema. Delete the `--work` directory afterwards: it
holds a copy of your credentials for every run.

## Tests

    python3 -m unittest discover -s bench -t .

They need `ank` on PATH (or `BENCH_ANK`) for `ank init` and `ank check`, and
launch no agent: `test_run.py` drives the runner against a stub that replays a
transcript from `testdata/`. Each `testdata/<case>/` is a hand-written
stream-json transcript, the tree it left, and the `score.json` the scorer must
reproduce from them.
