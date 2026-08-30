# -*- coding: utf-8 -*-
"""The predicate the workflow uses to recognise the mark, asked of real output.

Written here and pasted into the workflow would be two spellings; instead this
reads the workflow's own text, lifts the predicate out of it, and runs it
against the recordings on disk. What passes here is what will run there.
"""

import json
import re
import sys

import yaml

doc = yaml.safe_load(open(".github/workflows/install.yml"))

OPEN = re.compile(r"^(\s*)cat > offer\.py <<'PY'$")
probe = None
for job in doc["jobs"].values():
    for step in job.get("steps", []):
        lines = step.get("run", "").split("\n")
        for i, line in enumerate(lines):
            m = OPEN.match(line)
            if not m:
                continue
            indent = len(m.group(1)) + 2
            body = []
            for rest in lines[i + 1:]:
                if rest.strip() == "PY":
                    break
                body.append(rest[indent:] if rest.startswith(" " * indent) else rest)
            probe = "\n".join(body)

if probe is None:
    sys.exit("could not find the probe in the workflow")

# Only the predicates are wanted, not the run that follows them -- and the
# environment the probe reads at import time belongs to the workflow, so it is
# supplied rather than demanded.
import os
os.environ.setdefault("STAGED_VERSION", "0.0.0-check")
head = probe[:probe.index("failures = []")]
ns = {}
exec(compile(head, "offer.py", "exec"), ns)
drew_the_mark = ns["drew_the_mark"]
drew_logo = ns["drew_logo"]

fails = []


def out_of(name):
    rec = json.load(open(".capture-%s.json" % name))
    return "".join(c[1] for c in rec["chunks"])


# The mark is in these, in one cell or the other.
for name in ("new", "ascii"):
    text = out_of(name)
    if not drew_logo(text):
        fails.append("%s: drew_logo says no logo" % name)
    if not drew_the_mark(text):
        fails.append("%s: drew_the_mark does not recognise the shape" % name)

# And not in a run that drew nothing. The staged non-terminal log is not kept,
# so a run with the welcome refused stands in for it.
plain = "ank v9.9.9-preview  x86_64-unknown-linux-musl\nchecksum ok  abc\n"
if drew_logo(plain) or drew_the_mark(plain):
    fails.append("a log with no logo in it was said to have one")

for f in fails:
    print("FAIL", f)
if fails:
    sys.exit(1)
print("the workflow's own predicate recognises the mark in both cells,")
print("and refuses a log that has none.")
