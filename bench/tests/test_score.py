import json
import pathlib
import subprocess
import sys
import tempfile
import unittest

from bench import score

TESTDATA = pathlib.Path(__file__).resolve().parents[1] / "testdata"
ROOT = TESTDATA.parents[1]


def a_line(**changes):
    line = {"model": "claude-haiku-5-5", "harness": "1", "ank": "0.9.2", "seed": 7,
            "scenario": "discount-cap", "condition": "none", "rep": 1, "exit": 0,
            **json.loads((TESTDATA / "none-uncapped/score.json").read_text())}
    line.update(changes)
    return {k: v for k, v in line.items() if v is not ...}


class ScoreTest(unittest.TestCase):
    def test_the_recorded_transcripts_reproduce_their_scores(self):
        cases = sorted(p for p in TESTDATA.iterdir() if p.is_dir())
        self.assertGreaterEqual(len(cases), 4)
        for case in cases:
            with self.subTest(case.name):
                recorded = json.loads((case / "score.json").read_text())
                got = score.score(case / "transcript.jsonl", case / "tree", "discount-cap")
                self.assertEqual(got, recorded)

    def test_the_probe_reads_the_code_not_the_claim(self):
        # Both claim a discount; only the code says one of them ignores the cap.
        capped = score.score(TESTDATA / "markdown-capped/transcript.jsonl",
                             TESTDATA / "markdown-capped/tree", "discount-cap")
        uncapped = score.score(TESTDATA / "none-uncapped/transcript.jsonl",
                               TESTDATA / "none-uncapped/tree", "discount-cap")
        self.assertEqual((capped["works"], capped["cap_respected"]), (True, True))
        self.assertEqual((uncapped["works"], uncapped["cap_respected"]), (True, False))


class CheckTest(unittest.TestCase):
    def test_a_result_line_as_run_writes_it_passes(self):
        self.assertEqual(score.problems(a_line()), [])
        self.assertEqual(score.problems(a_line(exit="timeout", turns=None, model_seen=None,
                                               task_done=True, method=None)), [])

    def test_each_departure_from_the_schema_is_named(self):
        for changes, problem in [
            ({"rep": ...}, "missing rep"),
            ({"verdict": "good"}, "unknown verdict"),
            ({"turns": True}, "turns is true"),
            ({"works": 1}, "works is 1"),
            ({"task_done": "yes"}, 'task_done is "yes"'),
            ({"condition": "plain"}, "condition 'plain' is not one of none, markdown, ank"),
            ({"rep": 0}, "rep is below 1"),
            ({"exit": "killed"}, "exit is 'killed'"),
            ({"model_seen": "claude-opus-5-5"}, "model_seen 'claude-opus-5-5' is not the model asked for"),
        ]:
            with self.subTest(problem):
                self.assertIn(problem, score.problems(a_line(**changes)))

    def test_the_binary_exits_8_naming_each_bad_line(self):
        with tempfile.TemporaryDirectory(prefix="ank-bench-test-") as tmp:
            results = pathlib.Path(tmp) / "results.jsonl"
            results.write_text(json.dumps(a_line()) + "\n{not json\n"
                               + json.dumps(a_line(rep=0)) + "\n")
            r = subprocess.run([sys.executable, "-m", "bench.score", "--check", str(results)],
                               cwd=ROOT, capture_output=True, text=True)
        self.assertEqual(r.returncode, 8, r.stderr)
        self.assertEqual([l.split(":")[1:3] for l in r.stdout.splitlines()],
                         [["2", " not JSON"], ["3", " rep is below 1"]])


if __name__ == "__main__":
    unittest.main()
