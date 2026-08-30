# -*- coding: utf-8 -*-
"""Run a workflow step's script here, as the workflow would run it.

A step is a spelling of its own, and a spelling of its own can be wrong on its
own -- which is what happened to the sh geometry assertion. So the step is not
reimplemented to be checked; it is taken out of the file and executed.
"""

import os
import subprocess
import sys

import yaml

PWSH = ("/tmp/claude-1000/-home-haksolot-Projects-ank/"
        "1ce9a43d-5095-4fde-9ec4-5dd445b54ac9/scratchpad/pwsh/pwsh")

wanted = sys.argv[1]
doc = yaml.safe_load(open(".github/workflows/install.yml"))

script = None
for job in doc["jobs"].values():
    for step in job.get("steps", []):
        if step.get("name") == wanted:
            script = step["run"]

if script is None:
    sys.exit("no step named %r" % wanted)

path = ".step.ps1"
with open(path, "w", encoding="utf-8") as f:
    f.write(script)

code = subprocess.call([PWSH, "-NoProfile", "-File", path])
os.remove(path)
sys.exit(code)
