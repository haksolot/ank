# -*- coding: utf-8 -*-
"""Write out the probe.ps1 the workflow writes, so it can be run here.

Taken from the workflow's here-string rather than written again: a second
spelling of a probe is a second thing that can be wrong on its own, which is
the mistake this whole round has been about.
"""

import re
import sys

import yaml

doc = yaml.safe_load(open(".github/workflows/install.yml"))

OPEN = re.compile(r"^(\s*)Set-Content -Path 'probe\.ps1' -Value @'$")

for job in doc["jobs"].values():
    for step in job.get("steps", []):
        lines = step.get("run", "").split("\n")
        for i, line in enumerate(lines):
            m = OPEN.match(line)
            if not m:
                continue
            indent = len(m.group(1))
            body = []
            for rest in lines[i + 1:]:
                if rest.strip() == "'@":
                    break
                body.append(rest[indent:] if rest.startswith(" " * indent) else rest)
            with open("probe.ps1", "w", encoding="utf-8") as f:
                f.write("\n".join(body))
            print("wrote probe.ps1 (%d lines)" % len(body))
            sys.exit(0)

sys.exit("could not find probe.ps1 in the workflow")
