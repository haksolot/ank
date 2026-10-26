import json
import os
import pathlib
import shlex
import subprocess
import sys
import tempfile
import unittest

from bench import HARNESS_VERSION, run

ROOT = pathlib.Path(__file__).resolve().parents[2]
TESTDATA = ROOT / "bench" / "testdata"

# Stands in for claude: replays a recorded transcript, leaves the recorded code
# behind, and writes down what it was launched with. It is not an agent.
STUB = '''#!{python}
import json, os, pathlib, shutil, sys
case = pathlib.Path({case!r})
shutil.copyfile(case / "tree/src/shop/cart.py", "src/shop/cart.py")
pathlib.Path({record!r}).write_text(json.dumps({{
    "argv": sys.argv[1:],
    "config": sorted(os.listdir(os.environ["CLAUDE_CONFIG_DIR"])),
    "ank_env": sorted(k for k in os.environ if k.startswith("ANK_")),
}}))
sys.stdout.write((case / "transcript.jsonl").read_text())
'''


def bench_run(*args, env=None):
    return subprocess.run([sys.executable, "-m", "bench.run", *args], cwd=ROOT,
                          capture_output=True, text=True, env=env)


class RunTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="ank-bench-test-")
        self.dir = pathlib.Path(self.tmp.name)
        self.credentials = self.dir / "credentials.json"
        self.credentials.write_text('{"claudeAiOauth": {"accessToken": "fake"}}')
        self.marker = self.dir / "launched"
        self.stub = self.dir / "claude"
        self.stub.write_text(STUB.format(python=sys.executable, case=str(TESTDATA / "none-uncapped"),
                                         record=str(self.marker)))
        self.stub.chmod(0o755)

    def tearDown(self):
        self.tmp.cleanup()

    def common(self, model="claude-sonnet-5-5"):
        return ["--model", model, "--credentials", str(self.credentials),
                "--work", str(self.dir / "work"), "--claude", str(self.stub)]

    def test_dry_run_prints_an_isolated_command_and_launches_nothing(self):
        r = bench_run(*self.common(), "--dry-run")
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertFalse(self.marker.exists(), "--dry-run launched the agent")
        config_line, command_line = r.stdout.strip().splitlines()
        config = pathlib.Path(config_line.split()[1].rstrip(":"))
        self.assertEqual(os.listdir(config), [".credentials.json"])
        self.assertEqual((config / ".credentials.json").read_text(), self.credentials.read_text())
        words = shlex.split(command_line)
        self.assertIn(f"CLAUDE_CONFIG_DIR={config}", words)
        self.assertEqual(words[words.index("--model") + 1], "claude-sonnet-5-5")
        self.assertIn("--strict-mcp-config", words)
        self.assertIn("--disable-slash-commands", words)
        self.assertEqual(json.loads(words[words.index("--mcp-config") + 1]), {"mcpServers": {}})
        self.assertEqual(words[words.index("-p") - 1], str(self.stub))
        self.assertFalse((config.parent / "repo").exists(), "--dry-run generated a fixture")

    def test_an_alias_is_refused_before_anything_is_launched(self):
        for alias in ("sonnet", "opus", "claude-sonnet", "claude-sonnet-latest"):
            r = bench_run(*self.common(model=alias), "--dry-run")
            self.assertEqual(r.returncode, 2, alias)
            self.assertIn("not a pinned model id", r.stderr)
        self.assertFalse(self.marker.exists())

    def test_missing_credentials_exit_9(self):
        self.credentials.unlink()
        r = bench_run(*self.common(), "--dry-run")
        self.assertEqual(r.returncode, 9)
        self.assertIn("no credentials", r.stderr)

    def test_a_run_appends_one_result_line_naming_what_it_measured(self):
        results = self.dir / "results.jsonl"
        env = dict(os.environ, ANK_AGENT="leaks/1")
        r = bench_run(*self.common(model="claude-haiku-5-5"), "--condition", "none", "--seed", "11",
                      "--rep", "3", "--results", str(results), env=env)
        self.assertEqual(r.returncode, 0, r.stderr)
        launched = json.loads(self.marker.read_text())
        self.assertEqual(launched["config"], [".credentials.json"])
        self.assertEqual(launched["ank_env"], [])
        self.assertIn("--strict-mcp-config", launched["argv"])
        lines = results.read_text().splitlines()
        self.assertEqual(len(lines), 1)
        line = json.loads(lines[0])
        for key in run.RESULT_KEYS:
            self.assertIn(key, line)
        self.assertEqual(
            {k: line[k] for k in ("model", "harness", "seed", "scenario", "condition", "rep")},
            {"model": "claude-haiku-5-5", "harness": HARNESS_VERSION, "seed": 11,
             "scenario": "discount-cap", "condition": "none", "rep": 3})
        self.assertEqual(line["ank"], run.ank_version("ank"))
        self.assertEqual((line["model_seen"], line["works"], line["cap_respected"]),
                         ("claude-haiku-5-5", True, False))


if __name__ == "__main__":
    unittest.main()
