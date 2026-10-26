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

## Tests

    python3 -m unittest discover -s bench -t .

They need `ank` on PATH (or `BENCH_ANK`) for `ank init` and `ank check`, and
launch no agent: `test_run.py` drives the runner against a stub that replays a
transcript from `testdata/`. Each `testdata/<case>/` is a hand-written
stream-json transcript, the tree it left, and the `score.json` the scorer must
reproduce from them.
