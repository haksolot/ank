"""Scores one run from its stream-json transcript and the tree it left behind.

    python3 -m bench.score --scenario discount-cap TRANSCRIPT WORKDIR
    python3 -m bench.score --check bench/results/adoption.jsonl

Prints one JSON object. The transcript says what the agent did; the scenario's
hidden probe says what the code it left actually does, whatever it claimed.
`--check` holds every line of a results file to the schema below, and exits 8
naming each line that misses it.
"""

import argparse
import json
import pathlib
import re
import subprocess
import sys

from bench.fixture import slugify
from bench.scenarios import CONDITIONS, SCENARIOS

STR, INT, BOOL, NONE, LIST = (str,), (int,), (bool,), (type(None),), (list,)
# One result line: what run.py names, then what score() counts. Nothing else.
SCHEMA = {
    "model": STR, "harness": STR, "ank": STR + NONE, "seed": INT, "scenario": STR,
    "condition": STR, "rep": INT, "exit": INT + STR,
    "model_seen": STR + NONE, "claude_code": STR + NONE,
    "mcp_servers": INT, "plugins": INT, "skills": INT,
    "turns": INT + NONE, "out_tokens": INT + NONE,
    "help_flag": BOOL, "cat_corpus": BOOL, "read_markdown": BOOL, "ank_context": BOOL,
    "claimed": BOOL, "task_done": BOOL + NONE,
    "method": LIST + NONE, "works": BOOL, "cap_respected": BOOL + NONE,
}


def tool_calls(events):
    """`<Tool> <its main argument>`, one line per tool call, in order."""
    for e in events:
        if e.get("type") != "assistant":
            continue
        for c in e.get("message", {}).get("content", []):
            if c.get("type") == "tool_use":
                i = c.get("input", {})
                yield f'{c["name"]} {i.get("command") or i.get("file_path") or i.get("pattern") or ""}'


def read_transcript(path):
    with open(path, encoding="utf-8") as f:
        return [json.loads(line) for line in f if line.strip()]


def committed_task(workdir, slug):
    """The target task's text as HEAD holds it, or as the tree does without git."""
    entities = workdir / ".ank" / "entities"
    if not entities.is_dir():
        return None
    for p in sorted(entities.glob("TASK-*.md")):
        text = p.read_text(encoding="utf-8")
        if f"\nslug: {slug}\n" not in text:
            continue
        if (workdir / ".git").exists():
            r = subprocess.run(["git", "show", f"HEAD:{p.relative_to(workdir).as_posix()}"],
                               cwd=workdir, capture_output=True, text=True)
            return r.stdout if r.returncode == 0 else ""
        return text
    return None


def probe(workdir, scenario):
    r = subprocess.run([sys.executable, "-I", "-B", "-c", scenario["probe"]], cwd=workdir,
                       capture_output=True, text=True, timeout=60)
    if r.returncode or not r.stdout.strip():
        return {"method": None, "works": False, "cap_respected": None}
    return json.loads(r.stdout)


def score(transcript, workdir, scenario_name):
    scenario = SCENARIOS[scenario_name]
    workdir = pathlib.Path(workdir)
    events = read_transcript(transcript)
    init = next((e for e in events if e.get("type") == "system" and e.get("subtype") == "init"), {})
    result = next((e for e in reversed(events) if e.get("type") == "result"), {})
    blob = "\n".join(tool_calls(events))
    task = committed_task(workdir, slugify(scenario["task"]["title"]))
    return {
        "model_seen": init.get("model"),
        "claude_code": init.get("claude_code_version"),
        # What the agent was handed beyond the repository: all zero when isolated.
        "mcp_servers": len(init.get("mcp_servers", [])),
        "plugins": len(init.get("plugins", [])),
        "skills": len(init.get("skills", [])),
        "turns": result.get("num_turns"),
        "out_tokens": result.get("usage", {}).get("output_tokens"),
        "help_flag": bool(re.search(r"\bank (\w+ )?(--help|-h)\b", blob)),
        "cat_corpus": bool(re.search(r"(cat|xargs|Read|head|less)\b[^\n]*\.ank/(entities|\*)"
                                     r"|find \.ank[^\n]*cat", blob)),
        "read_markdown": bool(re.search(r"docs/adr", blob)),
        "ank_context": "ank context" in blob,
        "claimed": "ank claim" in blob,
        "task_done": None if task is None else "\nstatus: done\n" in task,
        **probe(workdir, scenario),
    }


def problems(line):
    """What keeps one result line from the schema; empty when it passes."""
    if not isinstance(line, dict):
        return ["not a JSON object"]
    out = [f"missing {k}" for k in SCHEMA if k not in line]
    out += [f"unknown {k}" for k in line if k not in SCHEMA]
    for k, types in SCHEMA.items():
        v = line.get(k)
        # type(), not isinstance: True is an int to isinstance, and a count is never True.
        if k in line and type(v) not in types:
            out.append(f"{k} is {json.dumps(v)}")
    if out:
        return out
    if line["scenario"] not in SCENARIOS:
        out.append(f"scenario {line['scenario']!r} is not one of {', '.join(sorted(SCENARIOS))}")
    if line["condition"] not in CONDITIONS:
        out.append(f"condition {line['condition']!r} is not one of {', '.join(CONDITIONS)}")
    if line["rep"] < 1:
        out.append("rep is below 1")
    if isinstance(line["exit"], str) and line["exit"] != "timeout":
        out.append(f"exit is {line['exit']!r}")
    if line["model_seen"] not in (None, line["model"]):
        out.append(f"model_seen {line['model_seen']!r} is not the model asked for")
    return out


def check(path):
    """`(line number, problem)` for every problem in a results file."""
    with open(path, encoding="utf-8") as f:
        for n, raw in enumerate(f, 1):
            try:
                line = json.loads(raw)
            except json.JSONDecodeError as e:
                yield n, f"not JSON: {e.msg}"
                continue
            for problem in problems(line):
                yield n, problem


def main(argv=None):
    p = argparse.ArgumentParser(prog="python3 -m bench.score", description=__doc__.split("\n")[0])
    p.add_argument("--check", metavar="RESULTS", help="validate a results JSONL instead of scoring")
    p.add_argument("transcript", nargs="?")
    p.add_argument("workdir", nargs="?")
    p.add_argument("--scenario", default="discount-cap", choices=sorted(SCENARIOS))
    a = p.parse_args(argv)
    if a.check:
        found = list(check(a.check))
        for n, problem in found:
            print(f"{a.check}:{n}: {problem}")
        return 8 if found else 0
    if not (a.transcript and a.workdir):
        p.error("TRANSCRIPT and WORKDIR are required unless --check is given")
    print(json.dumps(score(a.transcript, a.workdir, a.scenario)))


if __name__ == "__main__":
    sys.exit(main())
