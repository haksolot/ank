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

    def test_the_binding_decision_is_not_found_by_grepping_the_feature(self):
        # Every file a grep for the method or the feature reaches, in the ank and
        # markdown records: the task is found that way, the finance ceiling is not.
        scenario = fixture.SCENARIOS["discount-cap"]
        ceiling = scenario["decisions"][2][2]
        for name in ("a", "md"):
            repo = self.repos[name]
            for word in ("total_with_discount", "discount"):
                hits = subprocess.run(["git", "grep", "-il", word], cwd=repo,
                                      capture_output=True, text=True).stdout.split()
                for hit in hits:
                    self.assertNotIn(ceiling, (repo / hit).read_text(), f"{name}: {word} reaches {hit}")
            self.assertTrue(subprocess.run(["git", "grep", "-q", "-F", ceiling], cwd=repo).returncode == 0,
                            f"{name} does not carry the ceiling at all")

    def test_the_request_never_mentions_ank(self):
        self.assertNotRegex(fixture.SCENARIOS["discount-cap"]["prompt"], r"(?i)\bank\b|\.ank")

    def test_the_ank_record_carries_what_init_writes(self):
        # The condition measures the ank under test's own bootstrap, not a hand-written one.
        repo = self.repos["a"]
        self.assertIn("@AGENTS.md", (repo / "CLAUDE.md").read_text())
        self.assertIn("ank context", (repo / "AGENTS.md").read_text())



SUPERSEDED = "discount-superseded"


def entity_by_title(repo, title):
    for p in sorted((repo / ".ank/entities").glob("ADR-*.md")):
        text = p.read_text()
        if f"\ntitle: {title}\n" in text:
            return p.stem, text
    raise AssertionError(f"no ADR titled {title!r}")


class SupersessionFixtureTest(unittest.TestCase):
    """The family where the record changed its mind: the earlier ceiling is history."""

    @classmethod
    def setUpClass(cls):
        cls.ank = ank_binary()
        cls.tmp = tempfile.TemporaryDirectory(prefix="ank-bench-test-")
        root = pathlib.Path(cls.tmp.name)
        cls.repos = {c: fixture.generate(root / c, SUPERSEDED, c, 7, ank=cls.ank)
                     for c in fixture.CONDITIONS}
        decisions = fixture.SCENARIOS[SUPERSEDED]["decisions"]
        cls.later = next(d for d in decisions if len(d) > 3)
        cls.earlier = next(d for d in decisions if d[0] == cls.later[3])

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_the_scenario_lowers_a_three_tenths_ceiling_to_one_fifth(self):
        self.assertIn("three tenths", self.earlier[2])
        self.assertIn("one fifth", self.later[2])
        self.assertEqual(fixture.SCENARIOS[SUPERSEDED]["family"], "supersession")

    def test_under_ank_the_later_adr_supersedes_the_earlier(self):
        repo = self.repos["ank"]
        r = subprocess.run([self.ank, "check"], cwd=repo, capture_output=True, text=True,
                           env=fixture.environment())
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        earlier_id, earlier = entity_by_title(repo, self.earlier[0])
        later_id, later = entity_by_title(repo, self.later[0])
        self.assertIn("\nstatus: superseded\n", earlier)
        self.assertIn("\nstatus: accepted\n", later)
        self.assertIn(f"\nsupersedes: {earlier_id}\n", later)
        r = subprocess.run([self.ank, "show", later_id, "--json"], cwd=repo, capture_output=True,
                           text=True, env=fixture.environment())
        self.assertEqual(r.returncode, 0, r.stderr)
        self.assertIn(earlier_id, r.stdout)

    def test_ank_context_binds_the_later_decision_only(self):
        r = subprocess.run([self.ank, "context", "src/shop/cart.py"], cwd=self.repos["ank"],
                           capture_output=True, text=True, env=fixture.environment())
        self.assertEqual(r.returncode, 0, r.stderr)
        # Unclaimed, context lists the binding decisions by title: the successor
        # is among them, the decision it replaced is history and is not.
        self.assertIn(self.later[0], r.stdout)
        self.assertNotIn(self.earlier[0], r.stdout)

    def test_under_markdown_each_file_names_the_other(self):
        adr = self.repos["markdown"] / "docs/adr"
        earlier = next(p for p in adr.glob("*.md") if p.read_text().startswith(f"# {self.earlier[0]}\n"))
        later = next(p for p in adr.glob("*.md") if p.read_text().startswith(f"# {self.later[0]}\n"))
        self.assertIn(f"\nStatus: superseded by {later.name}\n", earlier.read_text())
        self.assertIn(f"\nStatus: accepted\nSupersedes: {earlier.name}\n", later.read_text())
        self.assertEqual(sum("Status: superseded" in p.read_text() for p in adr.glob("*.md")), 1)

    def test_under_none_neither_exists(self):
        repo = self.repos["none"]
        for d in (self.earlier, self.later):
            self.assertNotEqual(subprocess.run(["git", "grep", "-q", "-F", d[2]], cwd=repo).returncode, 0)
        self.assertEqual(sorted(p.name for p in repo.iterdir() if p.name != ".git"),
                         [".gitignore", "README.md", "src", "tests"])

    def test_neither_ceiling_is_found_by_grepping_the_feature(self):
        for name in ("ank", "markdown"):
            repo = self.repos[name]
            for word in ("total_with_discount", "discount"):
                hits = subprocess.run(["git", "grep", "-il", word], cwd=repo,
                                      capture_output=True, text=True).stdout.split()
                for hit in hits:
                    text = (repo / hit).read_text()
                    for d in (self.earlier, self.later):
                        self.assertNotIn(d[2], text, f"{name}: {word} reaches {hit}")
            for d in (self.earlier, self.later):
                self.assertEqual(subprocess.run(["git", "grep", "-q", "-F", d[2]], cwd=repo).returncode,
                                 0, f"{name} does not carry {d[0]!r}")

    def test_the_request_never_mentions_ank_or_a_ceiling(self):
        prompt = fixture.SCENARIOS[SUPERSEDED]["prompt"]
        self.assertNotRegex(prompt, r"(?i)\bank\b|\.ank|tenths|fifth|ceiling")

    def test_the_same_seed_writes_the_same_bytes_twice(self):
        with tempfile.TemporaryDirectory(prefix="ank-bench-test-") as tmp:
            again = fixture.generate(pathlib.Path(tmp) / "again", SUPERSEDED, "ank", 7, ank=self.ank)
            self.assertEqual(digest(again), digest(self.repos["ank"]))


if __name__ == "__main__":
    unittest.main()
