# -*- coding: utf-8 -*-
"""Enough of a terminal to answer a cursor-position report truthfully.

This exists because .NET's Unix console asks. `RawUI.CursorPosition` on Windows
is a console API call that returns; on Linux it writes ESC[6n and blocks until
the terminal answers. A driver that never answers makes every read cost a
timeout and hands .NET an uninitialised buffer to echo, which is the noise and
the sixteen seconds. Answering is what a terminal does, so the driver does it.

The answer has to be true, not merely prompt: Show-Logo reads the position once
and places every frame relative to it, so a wrong row puts the whole animation
somewhere the recording would show and a console would not.
"""

import re

CSI = re.compile(r"\x1b\[([0-9;?]*)([A-Za-z@])")
OSC = re.compile(r"\x1b\](?:[^\x07\x1b]*)(?:\x07|\x1b\\)")


class Terminal(object):
    def __init__(self, rows, cols):
        self.rows = rows
        self.cols = cols
        self.y = 0
        self.x = 0
        self.replies = []

    def _num(self, params, default=1):
        if params == "" or not params.isdigit():
            return default
        return int(params) or default

    def feed(self, text):
        """Advance the cursor over text, and queue any report it asks for."""
        i = 0
        while i < len(text):
            ch = text[i]

            if ch == "\x1b":
                m = OSC.match(text, i)
                if m:
                    i = m.end()
                    continue
                m = CSI.match(text, i)
                if m:
                    params, final = m.group(1), m.group(2)
                    self._csi(params, final)
                    i = m.end()
                    continue
                # A two-character escape, or one this does not know.
                i += 2
                continue

            if ch == "\n":
                self.y += 1
                if self.y >= self.rows:
                    self.y = self.rows - 1
            elif ch == "\r":
                self.x = 0
            elif ch == "\b":
                self.x = max(0, self.x - 1)
            elif ch == "\t":
                self.x = min(self.cols - 1, (self.x // 8 + 1) * 8)
            elif ch >= " ":
                self.x += 1
                if self.x >= self.cols:
                    self.x = 0
                    self.y = min(self.rows - 1, self.y + 1)
            i += 1

    def _csi(self, params, final):
        if final == "A":
            self.y = max(0, self.y - self._num(params))
        elif final == "B":
            self.y = min(self.rows - 1, self.y + self._num(params))
        elif final == "C":
            self.x = min(self.cols - 1, self.x + self._num(params))
        elif final == "D":
            self.x = max(0, self.x - self._num(params))
        elif final == "G":
            self.x = min(self.cols - 1, self._num(params) - 1)
        elif final in ("H", "f"):
            parts = params.split(";")
            row = int(parts[0]) if parts and parts[0].isdigit() else 1
            col = int(parts[1]) if len(parts) > 1 and parts[1].isdigit() else 1
            self.y = max(0, min(self.rows - 1, row - 1))
            self.x = max(0, min(self.cols - 1, col - 1))
        elif final == "n" and params == "6":
            # 1-based, which is what the report is defined in.
            self.replies.append("\x1b[%d;%dR" % (self.y + 1, self.x + 1))
        # K, J, m, h, l and the rest move nothing.

    def take_replies(self):
        out = "".join(self.replies)
        self.replies = []
        return out
