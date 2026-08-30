# -*- coding: utf-8 -*-
"""--no-welcome, at a real terminal. The one place ADR-5fbd99bf6fd5's "differs
in no outcome from an interactive run that declined everything" can be measured,
since without a terminal there is nothing to differ from.

Colour is behind that gate now as well as the logo, so the flag has to take both
away and the run has to still install."""

import fcntl
import functools
import os
import select
import signal
import struct
import sys
import termios
import threading
import time
from http.server import SimpleHTTPRequestHandler, ThreadingHTTPServer

HERE = os.getcwd()


class Quiet(SimpleHTTPRequestHandler):
    def log_message(self, *a):
        pass


srv = ThreadingHTTPServer(
    ("127.0.0.1", 0), functools.partial(Quiet, directory=os.path.join(HERE, ".stage"))
)
port = srv.server_address[1]
threading.Thread(target=srv.serve_forever, daemon=True).start()

home = "/tmp/ank-gate-home"
os.system("rm -rf " + home)
os.makedirs(home)

cmd = ["sh", "install.sh", "--no-welcome", "--version", "9.9.9-preview"]
pid, fd = os.forkpty()
if pid == 0:
    env = dict(os.environ)
    env.pop("CI", None)
    env["TERM"] = "xterm"
    env["LANG"] = env["LC_ALL"] = "C.UTF-8"
    env["HOME"] = home
    env["ANK_BASE_URL"] = "http://127.0.0.1:%d" % port
    env["PATH"] = "/usr/bin:/bin"
    try:
        os.execvpe(cmd[0], cmd, env)
    finally:
        os._exit(127)

try:
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 40, 100, 0, 0))
except OSError:
    pass

out = b""
deadline = time.time() + 30
while True:
    left = deadline - time.time()
    if left <= 0:
        os.kill(pid, signal.SIGKILL)
        break
    ready, _, _ = select.select([fd], [], [], left)
    if not ready:
        continue
    try:
        data = os.read(fd, 65536)
    except OSError:
        break
    if not data:
        break
    out += data

_, status = os.waitpid(pid, 0)
os.close(fd)
srv.shutdown()

text = out.decode("utf-8", "replace")
code = os.WEXITSTATUS(status) if os.WIFEXITED(status) else -1

fails = []
if code != 0:
    fails.append("exit %d" % code)
if "\x1b" in text:
    fails.append("an escape sequence reached a terminal that asked for no welcome")
for frame in ("####", "██"):
    if frame in text:
        fails.append("the logo was drawn: found %r" % frame)
if "✓" in text or "\n  - " in text:
    fails.append("a step marker was drawn")
if "Install them now?" in text or "Print the three prompts" in text:
    fails.append("it asked a question anyway")
if not os.access(os.path.join(home, ".local/bin/ank"), os.X_OK):
    fails.append("the binary is not in place")

sys.stdout.write(text)
print("-" * 60)
if fails:
    for f in fails:
        print("FAIL", f)
    sys.exit(1)
print("--no-welcome at a terminal: no escape, no logo, no marker, no question,")
print("and the binary is installed.")
