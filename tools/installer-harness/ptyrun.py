# -*- coding: utf-8 -*-
"""Run a command on a pty and print what it wrote.

The cursor-position reports are answered, because .NET's Unix console asks and
blocks until a terminal replies. Without that a PowerShell run here costs a
timeout per read and echoes an uninitialised buffer.
"""

import fcntl
import importlib.util as ilu
import os
import select
import signal
import struct
import sys
import termios
import time

HERE = os.path.dirname(os.path.abspath(__file__))
spec = ilu.spec_from_file_location("anktrm", os.path.join(HERE, ".term.py"))
term_mod = ilu.module_from_spec(spec)
spec.loader.exec_module(term_mod)

cmd = sys.argv[1:]
rows, cols = 46, 118

pid, fd = os.forkpty()
if pid == 0:
    env = dict(os.environ)
    env.pop("CI", None)
    env["TERM"] = "xterm-256color"
    try:
        os.execvpe(cmd[0], cmd, env)
    finally:
        os._exit(127)

try:
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", rows, cols, 0, 0))
except OSError:
    pass

term = term_mod.Terminal(rows, cols)
out = []
deadline = time.time() + 90
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
    out.append(text)
    term.feed(text)
    reply = term.take_replies()
    if reply:
        os.write(fd, reply.encode())

_, status = os.waitpid(pid, 0)
os.close(fd)
sys.stdout.write("".join(out))
sys.exit(os.WEXITSTATUS(status) if os.WIFEXITED(status) else 1)
