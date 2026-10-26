import hashlib
import os
import pathlib
import shutil
import subprocess
import tempfile
import unittest

from bench import fixture


def ank_binary():
    ank = os.environ.get("BENCH_ANK") or shutil.which("ank")
    if not ank:
        raise AssertionError("no ank on PATH and no BENCH_ANK: the fixture needs ank init and ank check")
    return ank


def digest(repo):
    """Every byte of the fixture: the files, the commits and every ref."""
    h = hashlib.sha256()
    for p in sorted(repo.rglob("*")):
        rel = p.relative_to(repo).as_posix()
        if rel == ".git" or rel.startswith((".git/", ".ank/index.db")) or not p.is_file():
            continue
        h.update(rel.encode() + b"\0" + p.read_bytes() + b"\0")
    refs = subprocess.run(["git", "for-each-ref", "--format=%(refname) %(objectname)"],
                          cwd=repo, capture_output=True, text=True, check=True).stdout
    return h.update(refs.encode()) or h.hexdigest()


def tree(repo, path):
    return subprocess.run(["git", "rev-parse", f"HEAD:{path}"], cwd=repo,
                          capture_output=True, text=True, check=True).stdout.strip()


class FixtureTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.ank = ank_binary()
        cls.tmp = tempfile.TemporaryDirectory(prefix="ank-bench-test-")
        root = pathlib.Path(cls.tmp.name)
        cls.repos = {}
        for name, condition, seed in [("a", "ank", 7), ("b", "ank", 7), ("c", "ank", 8),
                                      ("md", "markdown", 7), ("none", "none", 7)]:
            cls.repos[name] = fixture.generate(root / name, "discount-cap", condition, seed, ank=cls.ank)

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_the_same_seed_writes_the_same_bytes_twice(self):
        a, b = self.repos["a"], self.repos["b"]
        self.assertEqual(digest(a), digest(b))
        self.assertEqual(tree(a, ""), tree(b, ""))

    def test_another_seed_writes_another_corpus(self):
        self.assertNotEqual(digest(self.repos["a"]), digest(self.repos["c"]))

    def test_ank_check_on_the_corpus_exits_zero(self):
        for name in ("a", "c"):
            r = subprocess.run([self.ank, "check"], cwd=self.repos[name], capture_output=True,
                               text=True, env=fixture.environment())
            self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
            self.assertIn("48 adr", r.stdout + r.stderr)

    def test_the_target_task_is_bound_by_the_finance_decision(self):
        r = subprocess.run([self.ank, "context", "src/shop/cart.py"], cwd=self.repos["a"],
                           capture_output=True, text=True, env=fixture.environment())
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn("Pricing rules agreed with finance", r.stdout)
        self.assertIn("Percentage discount on the cart total", r.stdout)

    def test_the_conditions_differ_by_the_record_and_nothing_else(self):
        repos = [self.repos[n] for n in ("a", "md", "none")]
        for path in ("src", "tests", "README.md"):
            self.assertEqual(len({tree(r, path) for r in repos}), 1, path)
        md, none = self.repos["md"], self.repos["none"]
        self.assertEqual(len(list((md / "docs/adr").glob("*.md"))), 48)
        self.assertTrue((md / "CLAUDE.md").is_file())
        self.assertFalse((md / ".ank").exists())
        self.assertEqual(sorted(p.name for p in none.iterdir() if p.name != ".git"),
                         [".gitignore", "README.md", "src", "tests"])

    def test_the_markdown_record_carries_the_same_decisions(self):
        text = "".join(p.read_text() for p in (self.repos["md"] / "docs/adr").glob("*.md"))
        for _, _, constraint in fixture.SCENARIOS["discount-cap"]["decisions"]:
            self.assertIn(constraint, text)


if __name__ == "__main__":
    unittest.main()
