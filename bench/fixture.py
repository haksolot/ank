"""Seeded fixture repositories, one per (scenario, condition, seed).

    python3 -m bench.fixture --scenario discount-cap --condition ank --seed 7 DEST

The same seed writes the same bytes, git objects included: identifiers come
from the seed, every timestamp is fixed, git runs with no user configuration.
That rules out `ank new` and `ank accept`, which mint identifiers from the
clock, so the entity files and the ratification tags are written here in the
form those verbs write them, and `ank check` on the result is what holds this
module to that form. `ank init` is deterministic and does run: the fixture
carries what the ank under test teaches.

Conditions (ADR-bafbd28498cc): `none` is the code alone, `markdown` the same
decisions as docs/adr/*.md with a CLAUDE.md pointing at them, `ank` the
decisions and the tasks as an ank corpus. The code is identical in all three.
"""

import argparse
import hashlib
import os
import pathlib
import random
import shutil
import subprocess
import sys
import time

from bench.scenarios import CONDITIONS, SCENARIOS

# Typed actors, and a ratifier who is not the author, as a kept corpus has.
AUTHOR = "human:architect"
RATIFIER = "human:maintainer"
GIT_NAME = "Bench Fixture"
GIT_EMAIL = "bench@fixture.invalid"
EPOCH = 1767603600  # 2026-01-05T09:00:00Z, the clock every fixture reads
STEP = 600  # seconds between two entities: a corpus written over days, not in a burst
COMMITTED = EPOCH + 60 * 86400

MODULES = ["inventory", "shipping", "tax", "users", "payments", "catalog", "orders",
           "returns", "invoicing", "search", "reviews", "loyalty", "warehouse",
           "notifications"]
NOUNS = ["the batch job", "the nightly export", "the public API", "the admin screen",
         "the read model", "the event stream", "the retry queue", "the audit trail",
         "the cache layer", "the import pipeline", "the reconciliation report",
         "the partner feed"]
VERBS = ["must not block", "is rebuilt from", "is reconciled against", "never writes to",
         "is the only reader of", "publishes to", "is versioned with", "is validated before"]
WHY = ["An incident in the spring showed that", "Operations asked that",
       "After the migration we learned that", "Support tickets made it clear that",
       "The auditors require that", "Load tests revealed that"]


# -- the model: everything the seed decides, before any condition lays it out --

def plan(scenario, seed, adrs=44, tasks=330, closed=270):
    """The decisions and tasks of one fixture, independent of the condition."""
    rng = random.Random(seed)

    def para(m, n=6):
        return " ".join(f"{rng.choice(WHY)} {rng.choice(NOUNS)} of {m} {rng.choice(VERBS)} "
                        f"{rng.choice(NOUNS)}." for _ in range(n))

    def body(m, k):
        return "\n\n".join(para(m) for _ in range(k))

    decisions = []
    for _ in range(adrs):
        m = rng.choice(MODULES)
        decisions.append({
            "title": f"{m.capitalize()}: {rng.choice(NOUNS)} {rng.choice(VERBS)} {rng.choice(NOUNS)}",
            "scope": f"src/shop/{m}.py",
            "constraint": f"In src/shop/{m}.py, {rng.choice(NOUNS)} {rng.choice(VERBS)} "
                          f"{rng.choice(NOUNS)}.\nEvery change to {m} {rng.choice(VERBS)} "
                          f"{rng.choice(NOUNS)}.",
            "body": body(m, 3),
        })
    for title, scope, constraint in scenario["decisions"]:
        decisions.append({"title": title, "scope": scope, "constraint": constraint,
                          "body": body(pathlib.PurePosixPath(scope).stem, 3)})

    work = []
    for i in range(tasks):
        m = rng.choice(MODULES)
        verb = rng.choice(VERBS).replace("must not", "not").replace("is ", "be ")
        work.append({
            "title": f"{m.capitalize()}: make {rng.choice(NOUNS)} {verb} {rng.choice(NOUNS)}",
            "scope": [f"src/shop/{m}.py"],
            "criteria": para(m, 2),
            "body": body(m, 2),
            "closed": f"superseded by the {rng.choice(NOUNS)} rework" if i < closed else None,
        })
    t = scenario["task"]
    work.append({"title": t["title"], "scope": t["scope"], "criteria": t["criteria"],
                 "body": "", "closed": None, "target": True})
    return {"seed": seed, "decisions": decisions, "tasks": work}


# -- the ank format, as crates/ank-core/src/parse.rs emits it --

def scalar(s):
    plain = (s and "\n" not in s and ": " not in s and not s.endswith(":")
             and " #" not in s and s[0] not in "-?:#&*!|>'\"%@`[]{}, "
             and not s.endswith(" ")
             and s not in ("null", "~", "true", "false", "yes", "no"))
    # Rust's f64 parse, which Python's float matches but for the underscores
    # and the surrounding whitespace it alone accepts. A hash can be a float:
    # 58847e064135 is one, and canonical form quotes it.
    if plain and "_" not in s and s == s.strip():
        try:
            float(s)
            plain = False
        except ValueError:
            pass
    if plain:
        return s
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"').replace("\n", "\\n") + '"'


def block(key, value):
    lines = value.rstrip("\n").split("\n")
    return f"{key}: |\n" + "".join(f"  {line}\n" for line in lines)


def seq(key, items):
    return f"{key}:\n" + "".join(f"  - {scalar(i)}\n" for i in items)


def slugify(title):
    out = ""
    for c in title:
        if c.isascii() and c.isalnum():
            out += c.lower()
        elif out and not out.endswith("-"):
            out += "-"
    return out[:48].strip("-")


def normalize(text):
    lines = [line.rstrip() for line in text.replace("\r\n", "\n").split("\n")]
    while lines and lines[-1] == "":
        lines.pop()
    return "\n".join(lines)


def ratification_anchor(constraint, scope):
    buf = normalize(constraint) + "\n" + "".join(g.strip() + "\n" for g in scope)
    return hashlib.sha256(normalize(buf).encode()).hexdigest()[:12]


def entity_id(kind, seed, n):
    return f"{kind}-" + hashlib.sha256(f"{seed}:{kind}:{n}".encode()).hexdigest()[:12]


def stamp(n):
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime(EPOCH + STEP * n))


def head(id_, kind, title, created):
    return (f"---\nid: {id_}\ntype: {kind}\nslug: {scalar(slugify(title))}\ntitle: {scalar(title)}\n"
            f"created: {scalar(created)}\nauthor: {scalar(AUTHOR)}\n")


def adr_text(id_, d, created, anchor=None):
    out = (head(id_, "adr", d["title"], created) + "status: accepted\n" + seq("scope", [d["scope"]])
           + block("constraint", d["constraint"])
           + f"ratified: {scalar(ratification_anchor(d['constraint'], [d['scope']]))}\n")
    if anchor:
        out += f"anchor: {scalar(anchor)}\n"
    out += (f"verified:\n  - by: {scalar(RATIFIER)}\n    at: {scalar(created)}\n"
            "schema: 4\nversion: 2\n---\n")
    return out + "\n" + d["body"] + "\n"


def task_text(id_, t, created, verifier):
    status, version = ("closed", 2) if t["closed"] else ("open", 1)
    out = (head(id_, "task", t["title"], created) + f"status: {status}\n" + seq("scope", t["scope"])
           + "blocked_by: []\n" + block("done_criteria", t["criteria"])
           + f"criteria_by: creator\nverify: [{verifier}]\nschema: 4\nversion: {version}\n---\n")
    return out + ("\n" + t["body"] + "\n" if t["body"] else "")


def log_text(id_, about, t, created):
    return (f"---\nid: {id_}\ntype: log\ntitle: {scalar('closed: ' + t['closed'])}\n"
            f"created: {scalar(created)}\nauthor: {scalar(AUTHOR)}\n" + seq("scope", t["scope"])
            + f"about: {about}\nseq: 1\nschema: 4\nversion: 1\n---\n")


# -- laying a plan out --

def environment():
    """No user or system git configuration, no ank identity, a fixed clock."""
    env = {k: v for k, v in os.environ.items() if not k.startswith(("GIT_", "ANK_"))}
    date = f"@{COMMITTED} +0000"
    env.update(GIT_CONFIG_GLOBAL=os.devnull, GIT_CONFIG_NOSYSTEM="1",
               GIT_AUTHOR_NAME=GIT_NAME, GIT_AUTHOR_EMAIL=GIT_EMAIL, GIT_AUTHOR_DATE=date,
               GIT_COMMITTER_NAME=GIT_NAME, GIT_COMMITTER_EMAIL=GIT_EMAIL,
               GIT_COMMITTER_DATE=date, ANK_AGENT="process:bench-fixture")
    return env


def run(dest, *argv, stdin=None):
    r = subprocess.run(argv, cwd=dest, env=environment(), input=stdin,
                       capture_output=True, text=True)
    if r.returncode:
        raise RuntimeError(f"{' '.join(argv[:3])} exited {r.returncode}: {r.stderr.strip()}")
    return r.stdout.strip()


def write(dest, rel, text):
    p = dest / rel
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_bytes(text.encode())


def lay_code(dest, scenario):
    for rel, text in scenario["files"].items():
        write(dest, rel, text)
    for m in MODULES:
        write(dest, f"src/{scenario['package']}/{m}.py", f'"""The {m} module."""\n')


def lay_markdown(dest, model):
    for n, d in enumerate(model["decisions"], 1):
        write(dest, f"docs/adr/{n:04}-{slugify(d['title'])}.md",
              f"# {d['title']}\n\nStatus: accepted\nApplies to: `{d['scope']}`\n\n"
              f"## Decision\n\n{d['constraint']}\n\n## Context\n\n{d['body']}\n")
    write(dest, "CLAUDE.md",
          "# Architecture decisions\n\nThe decisions this repository is held to live in "
          "docs/adr/, one file each. An accepted decision binds the files it applies to: "
          "read the ones that cover a file before you change it.\n")


def lay_ank(dest, model, scenario, ank):
    run(dest, ank, "init")
    v = scenario["verifier"]
    config = dest / ".ank/config.yml"
    text = config.read_text()
    if "verifiers: {}\n" not in text:
        raise RuntimeError("ank init wrote a config this generator does not recognise")
    inputs = ", ".join(f'"{i}"' for i in v["inputs"])
    config.write_bytes(text.replace("verifiers: {}\n", (
        f"default_branch: main\nverifiers:\n  {v['name']}:\n    run: {v['run']}\n"
        f"    timeout: 2m\n    default: true\n    inputs: [{inputs}]\n")).encode())
    seed, clock = model["seed"], 0
    for n, d in enumerate(model["decisions"]):
        id_, created = entity_id("ADR", seed, n), stamp(clock)
        clock += 1
        # What `accept` does: the blob is the file without the anchor that names
        # the tag, the tag is annotated, and only refs/ank/ratify/<id> keeps it.
        blob = run(dest, "git", "hash-object", "-w", "--stdin", stdin=adr_text(id_, d, created))
        message = (f"ratify {id_}\n\nconstraint+scope: "
                   f"{ratification_anchor(d['constraint'], [d['scope']])}\nby: {RATIFIER}")
        run(dest, "git", "tag", "-a", "-m", message, f"ratify/{id_}", blob)
        tag = run(dest, "git", "rev-parse", f"refs/tags/ratify/{id_}")
        run(dest, "git", "update-ref", f"refs/ank/ratify/{id_}", tag)
        run(dest, "git", "tag", "-d", f"ratify/{id_}")
        write(dest, f".ank/entities/{id_}.md", adr_text(id_, d, created, anchor=tag))
    for n, t in enumerate(model["tasks"]):
        id_, created = entity_id("TASK", seed, n), stamp(clock)
        clock += 1
        write(dest, f".ank/entities/{id_}.md", task_text(id_, t, created, v["name"]))
        if t["closed"]:
            log = entity_id("LOG", seed, n)
            write(dest, f".ank/entities/{log}.md", log_text(log, id_, t, stamp(clock)))
            clock += 1


def generate(dest, scenario_name, condition, seed, ank="ank", **sizes):
    """Writes one fixture repository at `dest`, which must not exist."""
    if condition not in CONDITIONS:
        raise ValueError(f"condition {condition!r} is not one of {', '.join(CONDITIONS)}")
    scenario = SCENARIOS[scenario_name]
    dest = pathlib.Path(dest)
    dest.mkdir(parents=True)
    run(dest, "git", "init", "-q", "-b", "main")
    # The agent commits in this repository later, under the user's own git.
    run(dest, "git", "config", "commit.gpgsign", "false")
    run(dest, "git", "config", "tag.gpgsign", "false")
    lay_code(dest, scenario)
    run(dest, "git", "add", "-A")
    run(dest, "git", "commit", "-qm", scenario["package"])
    if condition == "none":
        return dest
    model = plan(scenario, seed, **sizes)
    if condition == "markdown":
        lay_markdown(dest, model)
    else:
        lay_ank(dest, model, scenario, ank)
    run(dest, "git", "add", "-A")
    run(dest, "git", "commit", "-qm", "the record")
    return dest


def main(argv=None):
    p = argparse.ArgumentParser(prog="python3 -m bench.fixture", description=__doc__.split("\n")[0])
    p.add_argument("dest")
    p.add_argument("--scenario", default="discount-cap", choices=sorted(SCENARIOS))
    p.add_argument("--condition", default="ank", choices=CONDITIONS)
    p.add_argument("--seed", type=int, default=7)
    p.add_argument("--ank", default=shutil.which("ank") or "ank")
    a = p.parse_args(argv)
    generate(a.dest, a.scenario, a.condition, a.seed, ank=a.ank)
    print(a.dest)


if __name__ == "__main__":
    sys.exit(main())
