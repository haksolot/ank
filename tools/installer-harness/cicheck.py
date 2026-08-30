# -*- coding: utf-8 -*-
"""The workflow is YAML, and the probe inside it is Python. Both are checked
here, because a syntax error in a heredoc is a red run that costs a push."""

import re
import sys

import yaml

doc = yaml.safe_load(open(".github/workflows/install.yml"))
print("yaml ok, %d jobs" % len(doc["jobs"]))

# The probe is written into offer.py by a heredoc. It is found by its own
# opening line rather than by a line number, so this keeps working when the
# step moves.
OPEN = re.compile(r"^(\s*)cat > offer\.py <<'PY'$")

checked = 0
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
            src = "\n".join(body)
            try:
                compile(src, "offer.py", "exec")
            except SyntaxError as e:
                sys.exit("offer.py line %s: %s\n  %s" % (e.lineno, e.msg, e.text))
            if "drew_logo" not in src:
                sys.exit("offer.py no longer knows how to recognise the logo")
            print("offer.py compiles (%d lines) and knows drew_logo" % len(body))
            checked += 1

if checked != 1:
    sys.exit("expected one embedded probe, found %d" % checked)
