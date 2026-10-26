"""Runs one agent on one fixture and appends one scored result line.

    python3 -m bench.run --model claude-sonnet-5-5 --condition ank --rep 1 [--dry-run]

A run spends the quota of whoever launches it: it is never part of CI
(ADR-bafbd28498cc). `--dry-run` prints what would be launched and launches
nothing.

The agent is isolated: CLAUDE_CONFIG_DIR is a directory holding nothing but a
copy of the credentials (no hook, no skill, no memory, no setting),
--strict-mcp-config with an empty server list, --disable-slash-commands, and a
model named by its full id, never an alias that moves under the result.

A credentials-only directory is not enough on its own: claude syncs the
account's plugins and skills into it at start (the 2026-10-26 prototype's
transcripts list 13 plugins and 156 skills). The scorer reads what the init
event reports, so a run that was not isolated says so in its result line.
"""

import argparse
import json
import os
import pathlib
import re
import shlex
import shutil
import subprocess
import sys
import tempfile

from bench import HARNESS_VERSION, fixture, score
from bench.scenarios import CONDITIONS, SCENARIOS

TOOLS = ["Bash", "Read", "Edit", "Write", "Glob", "Grep"]
PINNED = re.compile(r"^claude-[a-z]+-\d+(-\d+)*$")
RESULT_KEYS = ("model", "harness", "ank", "seed", "scenario", "condition", "rep")


def default_credentials():
    base = os.environ.get("CLAUDE_CONFIG_DIR") or pathlib.Path.home() / ".claude"
    return pathlib.Path(base) / ".credentials.json"


def ank_version(ank):
    try:
        out = subprocess.run([ank, "--version"], capture_output=True, text=True).stdout
    except OSError:
        return None
    return out.strip().removeprefix("ank ") or None


def isolated_config(directory, credentials):
    """A configuration directory holding exactly one file: the credentials."""
    directory = pathlib.Path(directory)
    if directory.exists():
        shutil.rmtree(directory)
    directory.mkdir(parents=True)
    target = directory / ".credentials.json"
    shutil.copyfile(credentials, target)
    target.chmod(0o600)
    return directory


def command(claude, prompt, model):
    return [claude, "-p", prompt, "--model", model,
            "--strict-mcp-config", "--mcp-config", json.dumps({"mcpServers": {}}),
            "--disable-slash-commands",
            "--output-format", "stream-json", "--verbose",
            "--permission-mode", "acceptEdits", "--allowedTools", *TOOLS]


def environment(config):
    env = {k: v for k, v in os.environ.items()
           if not k.startswith(("ANK_", "CLAUDE_", "ANTHROPIC_"))}
    env["CLAUDE_CONFIG_DIR"] = str(config)
    return env


def result_line(*, model, ank, seed, scenario, condition, rep, **scores):
    return {"model": model, "harness": HARNESS_VERSION, "ank": ank, "seed": seed,
            "scenario": scenario, "condition": condition, "rep": rep, **scores}


def main(argv=None):
    p = argparse.ArgumentParser(prog="python3 -m bench.run", description=__doc__.split("\n")[0])
    p.add_argument("--model", required=True, help="a full model id, e.g. claude-sonnet-5-5")
    p.add_argument("--scenario", default="discount-cap", choices=sorted(SCENARIOS))
    p.add_argument("--condition", default="ank", choices=CONDITIONS)
    p.add_argument("--seed", type=int, default=7)
    p.add_argument("--rep", type=int, default=1)
    p.add_argument("--work", default=None, help="where runs are laid out (default: a new temp dir)")
    p.add_argument("--results", default=None, help="JSONL to append to (default bench/results/<family>.jsonl)")
    p.add_argument("--credentials", default=None)
    p.add_argument("--claude", default="claude")
    p.add_argument("--ank", default=shutil.which("ank") or "ank")
    p.add_argument("--timeout", type=int, default=1500)
    p.add_argument("--dry-run", action="store_true")
    a = p.parse_args(argv)

    if not PINNED.match(a.model):
        p.error(f"--model {a.model!r} is not a pinned model id (claude-<family>-<version>)")
    credentials = pathlib.Path(a.credentials) if a.credentials else default_credentials()
    if not credentials.is_file():
        print(f"no credentials at {credentials}: pass --credentials", file=sys.stderr)
        return 9
    scenario = SCENARIOS[a.scenario]
    if a.work is None:
        a.work = tempfile.mkdtemp(prefix="ank-bench-")
    name = f"{a.scenario}-{a.condition}-{a.model}-s{a.seed}-r{a.rep}"
    root = pathlib.Path(a.work) / name
    repo, config = root / "repo", isolated_config(root / "config", credentials)
    argv_ = command(a.claude, scenario["prompt"], a.model)

    if a.dry_run:
        print(f"config {config}: {', '.join(sorted(os.listdir(config)))}")
        print(f"cd {shlex.quote(str(repo))} && CLAUDE_CONFIG_DIR={shlex.quote(str(config))} "
              + shlex.join(argv_))
        return 0

    if repo.exists():
        shutil.rmtree(repo)
    fixture.generate(repo, a.scenario, a.condition, a.seed, ank=a.ank)
    transcript = root / "transcript.jsonl"
    with open(transcript, "wb") as out, open(root / "stderr.txt", "wb") as err:
        try:
            code = subprocess.run(argv_, cwd=repo, env=environment(config), stdout=out,
                                  stderr=err, timeout=a.timeout).returncode
        except subprocess.TimeoutExpired:
            code = "timeout"
    line = result_line(model=a.model, ank=ank_version(a.ank), seed=a.seed, scenario=a.scenario,
                       condition=a.condition, rep=a.rep, exit=code,
                       **score.score(transcript, repo, a.scenario))
    results = pathlib.Path(a.results) if a.results else (
        pathlib.Path(__file__).parent / "results" / f"{scenario['family']}.jsonl")
    results.parent.mkdir(parents=True, exist_ok=True)
    with open(results, "a", encoding="utf-8") as f:
        f.write(json.dumps(line) + "\n")
    print(json.dumps(line))
    return 0


if __name__ == "__main__":
    sys.exit(main())
