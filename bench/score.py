"""Scores one run from its stream-json transcript and the tree it left behind.

    python3 -m bench.score --scenario discount-cap TRANSCRIPT WORKDIR

Prints one JSON object. The transcript says what the agent did; the scenario's
hidden probe says what the code it left actually does, whatever it claimed.
"""

import argparse
import json
import pathlib
import re
import subprocess
import sys

from bench.fixture import slugify
from bench.scenarios import SCENARIOS


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


def main(argv=None):
    p = argparse.ArgumentParser(prog="python3 -m bench.score", description=__doc__.split("\n")[0])
    p.add_argument("transcript")
    p.add_argument("workdir")
    p.add_argument("--scenario", default="discount-cap", choices=sorted(SCENARIOS))
    a = p.parse_args(argv)
    print(json.dumps(score(a.transcript, a.workdir, a.scenario)))


if __name__ == "__main__":
    sys.exit(main())
