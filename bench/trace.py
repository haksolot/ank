"""Prints a stream-json transcript as one line per step: python3 -m bench.trace LOG"""

import json
import sys


def clip(text, n):
    return text[:n].replace("\n", " | ")


def main(argv=None):
    path = (argv or sys.argv[1:])[0]
    with open(path, encoding="utf-8") as f:
        for line in f:
            e = json.loads(line)
            kind = e.get("type")
            if kind == "system" and e.get("subtype") == "init":
                print("MODEL", e.get("model"), "mcp", e.get("mcp_servers"))
            elif kind == "assistant":
                for c in e["message"]["content"]:
                    if c["type"] == "text":
                        print("TEXT:", clip(c["text"], 600))
                    elif c["type"] == "tool_use":
                        i = c["input"]
                        arg = i.get("command") or i.get("file_path") or i.get("pattern") or json.dumps(i)
                        print("TOOL", c["name"], ":", clip(arg, 400))
            elif kind == "user":
                for c in e["message"].get("content", []):
                    if isinstance(c, dict) and c.get("type") == "tool_result":
                        t = c["content"] if isinstance(c["content"], str) else " ".join(
                            x.get("text", "") for x in c["content"])
                        print("   ->", clip(t, 300))
            elif kind == "result":
                print("RESULT", e.get("num_turns"), "turns", e.get("total_cost_usd"))


if __name__ == "__main__":
    sys.exit(main())
