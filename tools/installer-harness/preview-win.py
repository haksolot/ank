# -*- coding: utf-8 -*-
"""Record install.ps1 the way .preview.py records install.sh: driven on a pty,
against a release staged on this machine, and saved as the bytes it wrote.

What this is honest about. It runs under pwsh 7 on Linux, because that is the
PowerShell this machine has. The script's own decisions -- the shape, the
timeline, the dim-against-normal axis, which lines are marked and which are
dimmed -- are the ones a Windows console would take, and that is what the
recording shows. What differs underneath is the delivery: pwsh here turns
`Write-Host -ForegroundColor` and RawUI cursor moves into VT sequences, where
Windows PowerShell 5.1 in conhost sets console attributes instead and emits no
sequence at all. So this previews the design and not the transport, and the
transport is what the workflow's Windows job measures.
"""

import fcntl
import functools
import json
import os
import select
import signal
import struct
import sys
import termios
import threading
import time
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

import importlib.util as _ilu

_spec = _ilu.spec_from_file_location(
    "anktrm", os.path.join(os.path.dirname(os.path.abspath(__file__)), ".term.py"))
term_mod = _ilu.module_from_spec(_spec)
_spec.loader.exec_module(term_mod)

HERE = os.getcwd()
PWSH = ("/tmp/claude-1000/-home-haksolot-Projects-ank/"
        "1ce9a43d-5095-4fde-9ec4-5dd445b54ac9/scratchpad/pwsh/pwsh")


class Quiet(SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


srv = ThreadingHTTPServer(
    ("127.0.0.1", 0), functools.partial(Quiet, directory=os.path.join(HERE, ".stage"))
)
port = srv.server_address[1]
threading.Thread(target=srv.serve_forever, daemon=True).start()

name = sys.argv[1]
keys = [k.encode() + b"\n" for k in (sys.argv[2] if len(sys.argv) > 2 else "n,n").split(",")]
extra = {}
for kv in sys.argv[3:]:
    k, _, v = kv.partition("=")
    extra[k] = v

home = "/tmp/ank-win-preview"
os.system("rm -rf " + home)
os.makedirs(os.path.join(home, "AppData", "Local"))

script = os.environ.get("PREVIEW_SCRIPT", "install.ps1")
cmd = [PWSH, "-NoProfile", "-File", os.path.join(HERE, script),
       "-Version", "9.9.9-preview"]

# A window a Windows Terminal actually opens at, and tall enough that nothing
# scrolls off: the Windows PATH instructions are longer than the sh ones, and a
# preview whose logo has scrolled away previews nothing.
rows, cols = 46, 118
pid, fd = os.forkpty()
if pid == 0:
    env = dict(os.environ)
    env.pop("CI", None)
    env["TERM"] = "xterm-256color"
    env["ANK_BASE_URL"] = "http://127.0.0.1:%d" % port
    # What a Windows console would already have set. PROCESSOR_ARCHITECTURE is
    # how Resolve-Target decides, and LOCALAPPDATA is where the default install
    # directory comes from.
    env["PROCESSOR_ARCHITECTURE"] = "AMD64"
    env["LOCALAPPDATA"] = os.path.join(home, "AppData", "Local")
    env["HOME"] = home
    # No node, so the skills offer prints the command rather than running it.
    env["PATH"] = "/usr/bin:/bin"
    env.update(extra)
    try:
        os.execvpe(cmd[0], cmd, env)
    finally:
        os._exit(127)

try:
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
except OSError:
    pass

t0 = time.time()
chunks = []
pending = list(keys)
term = term_mod.Terminal(rows, cols)
deadline = t0 + 60
while True:
    left = deadline - time.time()
    if left <= 0:
        os.kill(pid, signal.SIGKILL)
        break
    try:
        ready, _, _ = select.select([fd], [], [], min(left, 0.2))
    except OSError:
        break
    if not ready:
        continue
    try:
        data = os.read(fd, 65536)
    except OSError:
        break
    if not data:
        break
    text = data.decode("utf-8", "replace")
    chunks.append([round(time.time() - t0, 4), text])
    # Answered before anything else: a cursor-position report the terminal owes
    # is a read the child is blocked on.
    term.feed(text)
    reply = term.take_replies()
    if reply:
        os.write(fd, reply.encode())
    for _ in range(text.count("[Y/n] ")):
        if pending:
            time.sleep(0.45)
            os.write(fd, pending.pop(0))

os.waitpid(pid, 0)
os.close(fd)
srv.shutdown()

out = {"rows": rows, "cols": cols, "chunks": chunks, "name": name}
with open(".capture-%s.json" % name, "w") as f:
    json.dump(out, f)
print("%s: %d chunks, %.2fs" % (name, len(chunks), chunks[-1][0] if chunks else 0))
