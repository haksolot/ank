import json
import pathlib
import unittest

from bench import score

TESTDATA = pathlib.Path(__file__).resolve().parents[1] / "testdata"


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


if __name__ == "__main__":
    unittest.main()
