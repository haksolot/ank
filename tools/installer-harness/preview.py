# -*- coding: utf-8 -*-
"""Record a whole install, from the logo to the last offer, off a release
staged on this machine.

Nothing is rendered here and nothing is described: the recording is the bytes
install.sh wrote to a pty, with the time it wrote them at. Whatever looks at
the file afterwards is looking at the installer's own output.
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

try:
    from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer
except ImportError:
    sys.exit("python3 required")

HERE = os.getcwd()
STAGE = os.path.join(HERE, ".stage")


class Quiet(SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


handler = functools.partial(Quiet, directory=STAGE)
srv = ThreadingHTTPServer(("127.0.0.1", 0), handler)
port = srv.server_address[1]
threading.Thread(target=srv.serve_forever, daemon=True).start()

name = sys.argv[1]
keys = [k.encode() + b"\n" for k in (sys.argv[2] if len(sys.argv) > 2 else "n,n").split(",")]
extra_env = {}
for kv in sys.argv[3:]:
    k, _, v = kv.partition("=")
    extra_env[k] = v

# A home of its own and no --dir, so the recording shows the path a person
# actually gets: the default install directory, under a HOME that is short
# enough to read and is thrown away afterwards.
home = "/tmp/ank-preview-home"
os.system("rm -rf " + home)
os.makedirs(home)

cmd = [
    "sh", os.path.join(HERE, os.environ.get("PREVIEW_SCRIPT", "install.sh")),
    "--version", "9.9.9-preview",
]

rows, cols = 34, 92
pid, fd = os.forkpty()
if pid == 0:
    env = dict(os.environ)
    env.pop("CI", None)
    env["TERM"] = "xterm-256color"
    env["ANK_BASE_URL"] = "http://127.0.0.1:%d" % port
    env["LANG"] = "C.UTF-8"
    env["LC_ALL"] = "C.UTF-8"
    env["HOME"] = home
    env["SHELL"] = "/bin/bash"
    # No node, so the skills offer takes the branch that prints the command
    # instead of running it. Nothing in this preview may reach npm.
    env["PATH"] = "/usr/bin:/bin"
    env.update(extra_env)
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
seen = 0
deadline = t0 + 40
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
    # An answer is typed when its question has finished arriving, and never on
    # a clock: keys sent early are echoed into the middle of the logo, and two
    # sent at once are read by one question.
    asked = text.count("[Y/n] ")
    for _ in range(asked):
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
