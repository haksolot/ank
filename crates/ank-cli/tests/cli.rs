//! The binary, run as a process (§4).
//!
//! Everything else in this crate is tested inside the modules, where it is
//! cheaper and sharper. What cannot be tested there is what only exists once
//! the process exists: dispatch actually reaching a verb, and the exit code
//! carrying the semantics of §4 all the way out to the shell.
//!
//! The distinction is not academic here. Two real defects had already slipped
//! through green unit tests — a lock whose release failed under concurrency,
//! and a `--repo` resolution that dispatch never reached because it rejected
//! the verb first. `claim` is the third of the family: the module was complete
//! and tested, and no invocation of the binary could reach a line of it,
//! because the arm did not exist while six module headers said it did.
//!
//! An integration test is also the only place `CARGO_BIN_EXE_ank` is defined,
//! and `ank-cli` has no library target — so there is no unit test that could
//! spawn the binary even if we wanted one.

use std::ffi::OsStr;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// git's global and system configuration, for every process this suite spawns.
///
/// `Repo::new` declares `user.email`, `user.name` and `core.autocrlf` on the
/// repository it makes, and for a long time that was the whole of the fixture's
/// control over git: everything else came from the `~/.gitconfig` of whoever
/// happened to run the suite. `commit.gpgsign` is the one that bites. On a
/// machine that signs by default, nine tests here depended on a gpg agent
/// staying unlocked, and all nine failed with `gpg: signing failed` the moment
/// pinentry timed out (TASK-97437c25ddda). On CI, where nothing signs, they
/// passed forever — so the defect was invisible exactly where it was watched.
///
/// This file replaces both levels, so a fixture can no longer inherit what it
/// did not declare. It writes `commit.gpgsign = false` positively rather than
/// letting the default speak, because a value written down is a value a test
/// can assert, and
/// `every_spawned_process_reads_the_suites_git_config_and_no_other` asserts it.
///
/// `enable_signing` is untouched and still works: `accept` commits with `-S`,
/// which outranks any configuration, so the fixtures that need a real signature
/// go on making their own.
fn isolated_git_config() -> &'static Path {
    static PATH: OnceLock<PathBuf> = OnceLock::new();
    PATH.get_or_init(|| {
        let p = std::env::temp_dir().join(format!("ank-cli-it-gitconfig-{}", std::process::id()));
        std::fs::write(&p, "[commit]\n\tgpgsign = false\n").unwrap();
        p
    })
    .as_path()
}

/// The one door. Every process this suite spawns is built here, so the
/// isolation above cannot be forgotten at a call site.
///
/// A gate repeated at each site is one more chance to miss one, which is what
/// TASK-97437c25ddda was: twenty-four seed commits carried
/// `-c commit.gpgsign=false`, two did not, and nothing said so. Those two are
/// what nine tests reached through, so the flag is now gone from all
/// twenty-four: the isolation is load-bearing rather than doubled by a habit,
/// and every test in this file exercises it.
/// `nothing_spawns_a_process_outside_the_one_door` keeps this the only door.
fn spawn(program: impl AsRef<OsStr>) -> Command {
    let mut c = Command::new(program);
    let config = isolated_git_config();
    c.env("GIT_CONFIG_GLOBAL", config)
        .env("GIT_CONFIG_SYSTEM", config);
    c
}

/// git, run in `dir`.
fn git_command(dir: &Path) -> Command {
    let mut c = spawn("git");
    c.current_dir(dir);
    c
}

/// The binary under test. It shells out to git itself, so it needs the same
/// environment as the fixture that set the repository up.
fn ank_command() -> Command {
    spawn(ANK)
}

/// A real repository with a real `.ank/`: `startup` resolves the repository,
/// checks git and loads the config before any verb runs, so a fixture missing
/// any of the three would test the failure path instead.
struct Repo(PathBuf);

impl Repo {
    fn new() -> Repo {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "ank-cli-it-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(p.join(".ank/entities")).unwrap();
        let r = Repo(p);
        r.git(&["init", "-q", "-b", "main"]);
        r.git(&["config", "user.email", "test@ank.local"]);
        r.git(&["config", "user.name", "Test"]);
        r.git(&["config", "core.autocrlf", "false"]);
        // Belt and braces, and deliberately so (TASK-40a972e98a9a). The
        // environment above already puts the machine's configuration out of
        // reach; this says the same thing in the repository's own config, where
        // a reader looking at the fixture can see it. `tag.gpgsign` because
        // nothing tags today and the first thing that does should not have to
        // rediscover any of this.
        r.git(&["config", "commit.gpgsign", "false"]);
        r.git(&["config", "tag.gpgsign", "false"]);
        std::fs::write(
            r.0.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        r
    }

    fn git(&self, args: &[&str]) -> String {
        let out = git_command(&self.0)
            .args(args)
            .output()
            .expect("git must be installed: it is a hard dependency");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    }

    /// Declares verifiers and makes a commit, which `done` needs on both
    /// counts: something to run, and a HEAD for the completion record to name.
    fn with_verifiers(self, verifiers: &str) -> Repo {
        std::fs::write(
            self.0.join(".ank/config.yml"),
            format!("schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n{verifiers}"),
        )
        .unwrap();
        std::fs::write(self.0.join("seed.txt"), "x").unwrap();
        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", "seed"]);
        self
    }

    fn head(&self) -> String {
        self.git(&["rev-parse", "HEAD"])
    }

    /// Reads the ref with git and not through `claim::read`: what has to be
    /// true is the state of the repository, not the module's agreement with
    /// itself.
    fn claim_ref(&self, id: &str) -> Option<String> {
        let out = git_command(&self.0)
            .args(["cat-file", "-p", &format!("refs/ank/claims/{id}")])
            .output()
            .unwrap();
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).to_string())
    }

    /// The same read, performed from another checkout of this repository.
    ///
    /// `refs/ank/` is shared by every worktree, so this is not a different
    /// answer — it is the same one, asked from where the question actually
    /// arises: another agent, on another branch, wanting to know what the plane
    /// says about a task.
    fn claim_ref_at(&self, at: &Path, id: &str) -> Option<String> {
        let out = git_command(at)
            .args(["cat-file", "-p", &format!("refs/ank/claims/{id}")])
            .output()
            .unwrap();
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).to_string())
    }

    /// Pushes a claim's expiry into the past, leaving everything else in the
    /// record untouched.
    fn expire_claim(&self, id: &str) {
        // Long past any TTL plus its tolerance, and stable whatever the
        // machine's clock reads.
        self.set_expiry(id, "2020-01-01T00:00:00Z");
    }

    /// The same surgery the other way: an expiry far enough ahead that the
    /// claim reads live again.
    ///
    /// It exists because `claim` now refuses a second live claim under one
    /// identity (TASK-a548c95261a5), and one identity holding two live claims
    /// is still a state the corpus can be in — a ref written by hand, a claim
    /// taken by an earlier binary, a lapse revived. Ank is not a gatekeeper
    /// (ADR-6b3fa9ba3a05), so the fixtures that assert what the tool says about
    /// that state cannot be built by claiming twice any more. Claim, expire,
    /// claim, revive: both claims are taken by the binary and only the clock is
    /// forged, which is what `expire_claim` was already doing.
    fn revive_claim(&self, id: &str) {
        self.set_expiry(id, "2099-01-01T00:00:00Z");
    }

    /// Rewrites a claim record's expiry and leaves every other byte of it
    /// alone.
    ///
    /// Forged rather than waited for: expiry is judged with a two-minute
    /// clock-drift tolerance on top of the TTL, so the shortest honest wait for
    /// a lapsed claim is over two minutes of wall clock in a suite that runs in
    /// one. The resulting ref is byte-identical to one that lapsed on its own —
    /// the record carries the expiry, and nothing else records the passage of
    /// time.
    fn set_expiry(&self, id: &str, when: &str) {
        let record = self.claim_ref(id).expect("there is a claim to date");
        let rewritten: String = record
            .lines()
            .map(|l| {
                if l.starts_with("expires: ") {
                    format!("expires: {when}")
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n";

        let mut child = git_command(&self.0)
            .args(["hash-object", "-w", "--stdin"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(rewritten.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success(), "hash-object: {}", stderr(&out));
        let blob = String::from_utf8_lossy(&out.stdout).trim().to_string();
        self.git(&["update-ref", &format!("refs/ank/claims/{id}"), &blob]);
    }

    fn seed_task(&self, id: &str, criteria: Option<&str>) {
        self.seed_task_with(id, criteria, &[]);
    }

    /// Seeds a task declaring `schema`, whatever this binary supports.
    ///
    /// Written by hand and not by the binary, necessarily: a schema past
    /// `SCHEMA_VERSION` is one no build of this tool can write, since a writer
    /// writes the schema it knows. It is what a corpus touched by a newer
    /// release looks like from here, and the only way to obtain one is to
    /// forge it.
    fn seed_task_at_schema(&self, id: &str, schema: u32) {
        std::fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\nslug: newer\ntitle: Written by a newer ank\n\
                 created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
                 blocked_by: []\nschema: {schema}\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
    }

    fn seed_task_with(&self, id: &str, criteria: Option<&str>, verify: &[&str]) {
        let criteria = match criteria {
            Some(c) => format!("done_criteria: |\n  {c}\ncriteria_by: creator\n"),
            None => String::new(),
        };
        let verify = if verify.is_empty() {
            String::new()
        } else {
            format!("verify: [{}]\n", verify.join(", "))
        };
        std::fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\nslug: example\ntitle: Example task\n\
                 created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
                 blocked_by: []\n{criteria}{verify}schema: 1\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
    }

    /// Seeds a task where the **previous** layout put it, `.ank/tasks/<ID>.md`,
    /// with a title of its own so that a fixture holding both copies can say
    /// which one answered.
    ///
    /// Written by hand and not by the binary, deliberately: no writer produces
    /// this layout any more, so a corpus in it can only come from before the
    /// move — which is exactly the corpus under test.
    fn seed_task_legacy(&self, id: &str, title: &str) {
        let dir = self.0.join(".ank/tasks");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\nslug: example\ntitle: {title}\n\
                 created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
                 blocked_by: []\ndone_criteria: |\n  A verifiable criterion.\n\
                 criteria_by: creator\nschema: 1\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
    }

    fn seed_adr_legacy(&self, id: &str, constraint: &str) {
        let dir = self.0.join(".ank/adr");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: adr\nslug: example\ntitle: A decision\n\
                 created: 2026-07-20T00:00:00Z\nstatus: proposed\nscope:\n  - src/**\n\
                 constraint: |\n  {constraint}\nschema: 1\nversion: 1\n---\n\nWhy.\n"
            ),
        )
        .unwrap();
    }

    fn legacy_task_path(&self, id: &str) -> PathBuf {
        self.0.join(".ank/tasks").join(format!("{id}.md"))
    }

    fn flat_task_path(&self, id: &str) -> PathBuf {
        self.0.join(".ank/entities").join(format!("{id}.md"))
    }

    /// The same seed with a title of its own, for a fixture that has to say
    /// which task a line is about. Every seeded task is `Example task`
    /// otherwise, which is exactly the assertion a title test cannot make.
    fn seed_task_titled(&self, id: &str, title: &str) {
        self.seed_task(id, Some("A verifiable criterion."));
        let text = self
            .task_text(id)
            .replace("title: Example task", &format!("title: {title}"));
        std::fs::write(self.0.join(".ank/entities").join(format!("{id}.md")), text).unwrap();
    }

    /// The same seed with a scope of its own, for a fixture that needs an
    /// entity outside the perimeter under test.
    fn seed_task_scoped(&self, id: &str, scope: &str) {
        self.seed_task(id, Some("A verifiable criterion."));
        let text = self
            .task_text(id)
            .replace("  - src/**", &format!("  - {scope}"));
        std::fs::write(self.0.join(".ank/entities").join(format!("{id}.md")), text).unwrap();
    }

    /// Adds blockers to a seeded task. Written into the file rather than through
    /// `amend`, so that a graph fixture does not depend on the verb that edits
    /// the field it is drawing.
    fn blocked(&self, id: &str, blockers: &[&str]) {
        let list = blockers.join(", ");
        let text = self
            .task_text(id)
            .replace("blocked_by: []", &format!("blocked_by: [{list}]"));
        std::fs::write(self.0.join(".ank/entities").join(format!("{id}.md")), text).unwrap();
    }

    /// Seeds a task whose log is still a `## Log` section of its body, which is
    /// what every entity written before schema 3 looks like.
    fn seed_task_with_body_log(&self, id: &str, entry: &str) {
        self.seed_task(id, Some("A verifiable criterion."));
        let text = format!(
            "{}\n## Log\n- 2026-07-26T14:02Z marie@laptop \u{2014} {entry}\n",
            self.task_text(id)
        );
        std::fs::write(self.flat_task_path(id), text).unwrap();
    }

    /// The log file of an entity, empty when there is none. Since schema 3 the
    /// log is not in the entity file, so an assertion about a log line reads
    /// this and an assertion about a field reads `task_text`.
    fn log_text(&self, id: &str) -> String {
        std::fs::read_to_string(self.0.join(".ank/log").join(format!("{id}.md")))
            .unwrap_or_default()
    }

    fn task_text(&self, id: &str) -> String {
        std::fs::read_to_string(self.0.join(".ank/entities").join(format!("{id}.md"))).unwrap()
    }

    fn adr_text(&self, id: &str) -> String {
        std::fs::read_to_string(self.0.join(".ank/entities").join(format!("{id}.md"))).unwrap()
    }

    fn seed_adr(&self, id: &str, constraint: &str, scope: &str) {
        std::fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: adr\nslug: example\ntitle: A decision\n\
                 created: 2026-07-20T00:00:00Z\nstatus: proposed\nscope:\n  - {scope}\n\
                 constraint: |\n  {constraint}\nschema: 1\nversion: 1\n---\n\nWhy.\n"
            ),
        )
        .unwrap();
    }

    /// `accept` signs for real, and a signature configured from the developer's
    /// own global git config would make this test pass here and nowhere else.
    /// SSH because `ssh-keygen` ships beside git on all three platforms and
    /// needs no agent, no keyring and no passphrase prompt.
    fn enable_signing(&self) {
        let key = self.0.join("signing-key");
        let out = spawn("ssh-keygen")
            .args(["-t", "ed25519", "-N", "", "-C", "ank test", "-q", "-f"])
            .arg(&key)
            .output()
            .expect("ssh-keygen ships with git on all three platforms");
        assert!(
            out.status.success(),
            "ssh-keygen: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        self.git(&["config", "gpg.format", "ssh"]);
        self.git(&[
            "config",
            "user.signingkey",
            key.with_extension("pub").to_str().unwrap(),
        ]);
    }

    /// `--repo` rather than a working directory, so that the flag stays on a
    /// path a real invocation takes.
    fn ank(&self, agent: &str, args: &[&str]) -> Output {
        self.ank_at(agent, args, &self.0)
    }

    /// The same invocation, pointed at another checkout of the same
    /// repository. A worktree shares `refs/ank/` with the checkout that made
    /// it, which is why a question about a ref can never be answered from a
    /// working tree.
    fn ank_at(&self, agent: &str, args: &[&str], repo: &Path) -> Output {
        ank_command()
            .args(args)
            .arg("--repo")
            .arg(repo)
            .env("ANK_AGENT", agent)
            .current_dir(std::env::temp_dir())
            .output()
            .expect("the binary must have been built")
    }

    /// A bare origin and a second clone of this repository, both carrying the
    /// corpus this one seeded.
    ///
    /// Two clones and not two worktrees, because that is the whole distinction
    /// level 1 exists for: worktrees share `refs/ank/` and were always
    /// arbitrated, clones do not and are arbitrated only by the push (§7).
    /// Returned as a plain path rather than a `Repo`, so nothing about it can
    /// accidentally be seeded twice.
    fn cloned(&self) -> (PathBuf, PathBuf) {
        let origin = self.0.with_extension("origin.git");
        let other = self.0.with_extension("other");
        for p in [&origin, &other] {
            let _ = std::fs::remove_dir_all(p);
        }
        let out = git_command(&self.0)
            .args(["init", "-q", "--bare", "-b", "main"])
            .arg(&origin)
            .output()
            .unwrap();
        assert!(out.status.success(), "init --bare: {}", stderr(&out));

        self.git(&["add", "-A"]);
        self.git(&["commit", "-qm", "corpus"]);
        self.git(&["remote", "add", "origin", origin.to_str().unwrap()]);
        self.git(&["push", "-q", "origin", "main"]);

        let out = git_command(&self.0)
            .arg("clone")
            .arg("-q")
            .arg(&origin)
            .arg(&other)
            .output()
            .unwrap();
        assert!(out.status.success(), "clone: {}", stderr(&out));
        for args in [
            ["config", "user.email", "test@ank.local"],
            ["config", "user.name", "Test"],
            ["config", "commit.gpgsign", "false"],
        ] {
            let out = git_command(&other).args(args).output().unwrap();
            assert!(out.status.success(), "{args:?}: {}", stderr(&out));
        }
        (origin, other)
    }

    /// The same invocation, with `text` on stdin.
    ///
    /// Piped rather than redirected from a file, because a pipe is what the
    /// caller `--body -` exists for actually has: a heredoc, or the output of
    /// another command. `Command::output` would close stdin outright, which is
    /// the empty case and not this one.
    fn ank_stdin(&self, agent: &str, args: &[&str], text: &str) -> Output {
        let mut child = ank_command()
            .args(args)
            .arg("--repo")
            .arg(&self.0)
            .env("ANK_AGENT", agent)
            .current_dir(std::env::temp_dir())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("the binary must have been built");
        child
            .stdin
            .take()
            .expect("stdin was piped")
            .write_all(text.as_bytes())
            .expect("the child reads its stdin");
        child.wait_with_output().expect("the process must exit")
    }

    /// The same invocation with extra environment variables set.
    ///
    /// What the color rule of §4 reads, besides the terminal itself, is the
    /// environment — so the environment has to be under the test's control
    /// rather than inherited from whoever is running the suite. A developer
    /// with `NO_COLOR` exported would otherwise be testing a different rule
    /// from CI, and both would pass.
    fn ank_env(&self, agent: &str, args: &[&str], env: &[(&str, Option<&str>)]) -> Output {
        let mut c = ank_command();
        c.args(args)
            .arg("--repo")
            .arg(&self.0)
            .env("ANK_AGENT", agent)
            .current_dir(std::env::temp_dir());
        for (key, value) in env {
            match value {
                Some(v) => c.env(key, v),
                None => c.env_remove(key),
            };
        }
        c.output().expect("the binary must have been built")
    }

    /// The same invocation with `$EDITOR` under the test's control. `None`
    /// removes it, which is the environment failure §4 specifies — and removing
    /// it rather than trusting its absence matters, because the developer
    /// running this suite very probably has one exported.
    fn ank_edit(&self, agent: &str, args: &[&str], editor: Option<&str>) -> Output {
        let mut c = ank_command();
        c.args(args)
            .arg("--repo")
            .arg(&self.0)
            .env("ANK_AGENT", agent)
            .current_dir(std::env::temp_dir());
        match editor {
            Some(e) => c.env("EDITOR", e),
            None => c.env_remove("EDITOR"),
        };
        c.output().expect("the binary must have been built")
    }

    fn config_text(&self) -> String {
        std::fs::read_to_string(self.0.join(".ank/config.yml")).unwrap()
    }

    fn set_config(&self, text: &str) {
        std::fs::write(self.0.join(".ank/config.yml"), text).unwrap();
    }

    /// A non-interactive editor that saves `text`.
    ///
    /// `cp` is the exact shape of one: `edit` appends the file to open as the
    /// last word of the command line, so `EDITOR="cp <replacement>"` runs
    /// `cp <replacement> <scratch>` and the editor has saved. It ships beside
    /// git on all three platforms, which is the reason `enable_signing` reaches
    /// for `ssh-keygen`.
    fn editor_saving(&self, text: &str) -> String {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let p = self
            .0
            .join(format!("saved-{}.md", SEQ.fetch_add(1, Ordering::Relaxed)));
        std::fs::write(&p, text).unwrap();
        format!("cp '{}'", p.display())
    }
}

/// An editor that fills a `new` template in, rather than replacing it.
///
/// It has to transform: the id is minted by `new` before the editor runs and is
/// refused if it comes back changed, so the wholesale `cp` that serves `edit`
/// cannot serve here.
///
/// A shell function, and not `sed -i`, because `-i` is where the three
/// platforms part company: it takes a mandatory backup suffix on the BSD sed
/// macOS ships, so `sed -i -e ...` consumes `-e` as the suffix there and the
/// suite would pass on two runners and fail on the third. Redirecting into a
/// second file and moving it is POSIX everywhere. The path arrives as `$1`
/// because `sh -c 'f; f <path>'` binds it — which is also what makes this read
/// like an editor rather than a fixture.
fn editor_filling(title: &str, scope: &str, body: &str) -> String {
    format!(
        r#"f() {{ sed -e "s|^title:.*|title: {title}|" -e "s|^scope: .*|scope: [\"{scope}\"]|" "$1" > "$1.new"; printf "%s\n" "" "{body}" >> "$1.new"; mv "$1.new" "$1"; }}; f"#
    )
}

/// The same, plus one arbitrary substitution — for the tests that need to move
/// a field the template does not invite anyone to touch.
fn editor_filling_and(title: &str, scope: &str, extra: &str) -> String {
    format!(
        r#"f() {{ sed -e "s|^title:.*|title: {title}|" -e "s|^scope: .*|scope: [\"{scope}\"]|" -e "{extra}" "$1" > "$1.new"; mv "$1.new" "$1"; }}; f"#
    )
}

impl Drop for Repo {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn code(out: &Output) -> i32 {
    out.status
        .code()
        .expect("the process must exit, not signal")
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

const ID: &str = "TASK-000000000001";

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

// ---------------------------------------------------------------------------
// The fixture's own environment (TASK-97437c25ddda)
// ---------------------------------------------------------------------------

/// The isolation, asserted rather than assumed.
///
/// Without this test the property is invisible exactly where it is watched: CI
/// signs nothing, so every fixture here would go on passing if the two
/// variables stopped being set, and only a maintainer with `commit.gpgsign` in
/// their own `~/.gitconfig` would ever find out — which is how the defect
/// survived in the first place.
///
/// The assertion is on the origin git reports, not on the value alone. A
/// `false` read from the developer's own file would satisfy a test that asked
/// only what `commit.gpgsign` is, and would prove nothing about where it came
/// from.
#[test]
fn every_spawned_process_reads_the_suites_git_config_and_no_other() {
    let r = Repo::new();
    let expected = isolated_git_config().to_string_lossy().replace('\\', "/");

    for level in ["--global", "--system"] {
        let out = git_command(&r.0)
            .args(["config", "--list", "--show-origin", level])
            .output()
            .expect("git must be installed: it is a hard dependency");
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        let lines: Vec<&str> = text.lines().collect();

        assert_eq!(
            lines.len(),
            1,
            "git read something other than the one file this suite wrote as {level}: {text}"
        );
        let (origin, value) = lines[0]
            .split_once('\t')
            .unwrap_or_else(|| panic!("git config --show-origin: {}", lines[0]));
        // Windows: git quotes an origin holding backslashes, and escapes them.
        let origin = origin
            .strip_prefix("file:")
            .unwrap_or_else(|| panic!("the origin is not a file: {origin}"))
            .trim_matches('"')
            .replace("\\\\", "\\")
            .replace('\\', "/");
        assert_eq!(
            origin, expected,
            "{level} is a file this suite did not write"
        );
        assert_eq!(
            value, "commit.gpgsign=false",
            "the fixture no longer says what it needs said"
        );
    }
}

/// The next spawn site, caught before it is written.
///
/// Nothing in the language stops a test from reaching for `Command` directly:
/// `spawn` is a convention, and an unenforced convention is the reason this
/// task exists. So the convention is swept. The needle is assembled rather than
/// written out, or the sweep would match itself and could never fail.
///
/// Its reach is the literal form, and only in this file. A spawn written some
/// other way — through an alias, or by a helper this file does not hold — is
/// not caught. The other file in this directory, `skill.rs`, spawns the binary
/// twice, for `help` and `--version`; both answer before `startup` runs and
/// neither reads git at all, which is why one file is the whole of it.
#[test]
fn nothing_spawns_a_process_outside_the_one_door() {
    let needle = format!("Command::{}(", "new");
    let doors = include_str!("cli.rs").matches(needle.as_str()).count();
    assert_eq!(
        doors, 1,
        "a process is spawned outside `spawn`, so it inherits the git \
         configuration of the machine running the suite: route it through \
         `git_command`, `ank_command` or `spawn`"
    );
}

/// The freeze stops being declared and becomes verifiable. Through the binary
/// on purpose: "check reports a divergence" is a statement about the process,
/// and the anchor it compares against lives in git rather than in the corpus.
#[test]
fn an_edited_constraint_is_reported_altered_and_stops_being_injected() {
    const ADR: &str = "ADR-0000000000ab";
    let r = Repo::new();
    r.enable_signing();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    r.seed_task(ID, Some("A verifiable criterion."));
    // A scope matching nothing is a fault on an ADR, and it would exit 8 before
    // the comparison under test ever ran.
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // Ratified and untouched: the comparison must be silent, or the finding
    // below would prove nothing.
    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        !stdout(&out).contains("altered"),
        "an intact freeze reports nothing: {}",
        stdout(&out)
    );

    // The constraint moves in the file. The anchor recorded in the signed
    // commit cannot follow it without another signature.
    let edited = r.adr_text(ADR).replace("Do not do X.", "Do not do Y.");
    std::fs::write(r.0.join(".ank/entities").join(format!("{ADR}.md")), edited).unwrap();

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 8, "a divergence is a fault: {}", stderr(&out));
    assert!(
        stdout(&out).contains("altered since ratification"),
        "{}",
        stdout(&out)
    );

    // And it stops binding. Reporting it while still injecting it would leave
    // whoever edited the file rewriting the rule every agent works under.
    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("altered since ratification"),
        "the warning section 3 requires: {}",
        stdout(&out)
    );
    assert!(
        !stdout(&out).contains("Do not do Y."),
        "the altered constraint is withheld: {}",
        stdout(&out)
    );
}

/// Declares the key `enable_signing` generated, so the signature is judged at
/// all. Without the file there is no allowlist and §8 puts the corpus in
/// advisory mode, where every verdict is `None` — which is the shape of silence
/// this test exists to refuse.
fn declare_signing_key(r: &Repo) {
    let pub_key = std::fs::read_to_string(r.0.join("signing-key.pub")).unwrap();
    std::fs::write(
        r.0.join(".ank/allowed_signers"),
        format!("test@ank.local {}", pub_key.trim()),
    )
    .unwrap();
}

/// The incident of TASK-1ea38a17d854, reproduced at the only altitude that
/// would have caught it. GitHub rebased a ratification commit while merging a
/// pull request: same tree, same message, same anchor, signature gone. Anyone
/// who can push a branch can produce that commit, and the anchor it carries is
/// then worth nothing.
///
/// Through the binary, and that is the whole point. `signature_state` already
/// answered `Absent` in a unit test, and `check` already turned `Absent` into a
/// fault in another — and the binary measured on the real corpus still exited
/// 0, because the binary in hand was older than the check itself. Nothing
/// asserted that a process invocation runs the wiring end to end, so nothing
/// noticed that one wasn't.
///
/// The negative is only worth what the positive above it is worth: a real
/// `accept` runs first and must be silent, or an exit 8 here would prove
/// nothing but a broken fixture.
#[test]
fn a_ratification_commit_stripped_of_its_signature_is_a_fault_through_the_binary() {
    const ADR: &str = "ADR-0000000000cd";
    let r = Repo::new();
    r.enable_signing();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    // A scope matching nothing is a fault of its own, and it would exit 8
    // before the signature was ever consulted.
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    declare_signing_key(&r);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let ratification = r.head();

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(
        code(&out),
        0,
        "a real ratification must be silent, or the fault below proves nothing: {}{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(!stdout(&out).contains("not signed"), "{}", stdout(&out));

    // The rewrite. `--amend` keeps the message and the tree and drops the
    // signature, which is exactly what the merge did to f770c98.
    r.git(&["commit", "--amend", "--no-edit", "-q"]);
    assert_ne!(
        r.head(),
        ratification,
        "the commit must have been rewritten"
    );

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(
        code(&out),
        8,
        "an unsigned ratification is a fault: {}{}",
        stdout(&out),
        stderr(&out)
    );
    let said = stdout(&out);
    assert!(
        said.contains(ADR) && said.contains("not signed"),
        "the finding names the ADR and says what is wrong: {said}"
    );

    // And it is that finding and no other. The anchor still agrees with the
    // file and the commit is still reachable, so a report crying divergence or
    // unverifiability here would be describing a different repository.
    assert!(
        !said.contains("altered since ratification") && !said.contains("is reachable"),
        "only the signature is gone: {said}"
    );
}

/// `ank status` answers where am I, in one command (TASK-15336a0012d5).
///
/// The four scenarios the task names, in order: orientation with no claim,
/// execution with one, the ratification queue and an unmerged completion seen
/// from the default branch, and the degraded case with no default branch at all.
///
/// Through the binary, and the degraded case is the reason: `status` is what an
/// agent runs when it does not know where it is, so every one of its inputs can
/// be missing — no claim, no remote, no default branch, no commit — and each has
/// to be a line rather than an error. That is a property of the process, not of
/// the functions it composes.
#[test]
fn status_answers_where_am_i_in_every_state() {
    const ADR: &str = "ADR-00000000f001";
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    r.seed_task(ID, Some("A verifiable criterion."));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    // 1. Orientation: no claim, and the whole repository as the perimeter.
    let out = r.ank("claude-code@ank", &["status"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);
    assert!(said.starts_with("branch main (default main)"), "{said}");
    assert!(said.contains("no claim"), "{said}");
    assert!(said.contains("the whole repository"), "{said}");
    // A proposed ADR is the ratification queue.
    assert!(said.contains("queue 1 proposal(s)"), "{said}");
    assert!(said.contains("corpus 0 fault(s)"), "{said}");
    // Ends with the command to run next (§4), and it is the one this state
    // calls for rather than a fixed string.
    assert!(
        said.trim_end().ends_with("> ank context"),
        "orientation points at context: {said}"
    );

    // 2. Execution: the claim, its expiry, and the perimeter it implies.
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    let out = r.ank("claude-code@ank", &["status"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);
    assert!(said.contains(&format!("claim {ID}")), "{said}");
    assert!(
        said.contains("expires 20"),
        "the expiry is what makes a claim actionable: {said}"
    );
    assert!(
        said.contains(&format!("the scope of {ID}")),
        "under a claim the perimeter is the task's own scope: {said}"
    );
    assert!(
        said.trim_end().ends_with("> ank done"),
        "holding a claim points at done: {said}"
    );

    // The claim of another agent is not this agent's claim.
    let out = r.ank("codex@host-9", &["status"]);
    assert!(stdout(&out).contains("no claim"), "{}", stdout(&out));

    // 3. Accepted, so the queue empties and the constraint starts counting.
    r.enable_signing();
    assert_eq!(
        code(&r.ank("claude-code@ank", &["release", "--reason", "x"])),
        0
    );
    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(said.contains("queue 0 proposal(s)"), "{said}");
    assert!(said.contains("1 constraint(s)"), "{said}");

    // 4. Degraded: no default branch and no remote to infer one from. It is a
    // warning and a full report, never a refusal.
    let bare = Repo::new();
    std::fs::write(
        bare.0.join(".ank/config.yml"),
        "schema: 1\nclaim_ttl_max: 2h\n",
    )
    .unwrap();
    bare.seed_task(ID, Some("A verifiable criterion."));
    std::fs::create_dir_all(bare.0.join("src")).unwrap();
    std::fs::write(bare.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    bare.git(&["add", "-A"]);
    bare.git(&["commit", "-qm", "seed"]);
    let out = bare.ank("claude-code@ank", &["status"]);
    assert_eq!(
        code(&out),
        0,
        "a missing default branch degrades, it does not refuse: {}",
        stderr(&out)
    );
    let said = stdout(&out);
    assert!(said.contains("warning: no default branch"), "{said}");
    assert!(
        said.contains("default_branch"),
        "the warning carries the fix: {said}"
    );
    assert!(
        said.contains("corpus "),
        "and the rest of the report still arrives: {said}"
    );

    // `--json` is available on every command without exception (§4).
    let out = r.ank("claude-code@ank", &["status", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let j = stdout(&out);
    assert!(j.contains("\"branch\":\"main\""), "{j}");
    assert!(j.contains("\"claim\":null"), "{j}");
    assert!(j.contains("\"queue\":0"), "{j}");
}

/// `status` names the identity in effect and where it came from
/// (TASK-6f4ff66894d2).
///
/// Through the binary, and the environment is the whole reason: the subject is
/// what `$ANK_AGENT` resolved to in the process that ran. The suite runs its
/// cases in parallel threads of one process, so removing the variable inside the
/// test would remove it from every other case at once — which is exactly why
/// `identity.rs`'s own unit test asserts the *shape* of the fallback and cannot
/// assert that it was taken.
///
/// The last block is the defect itself rather than a string: a claim taken under
/// a declared identity is invisible to the same session with the variable
/// forgotten, and the refusal that follows talks about claims. The identity line
/// is the only place the true fact is said.
#[test]
fn status_names_the_identity_in_effect_and_its_source() {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    // Set: the value is the caller's, and the source names the variable it came
    // from. Above the claim, because it is the claim lines it explains.
    let said = stdout(&r.ank("claude-code/6f4f", &["status"]));
    assert!(
        said.contains("identity claude-code/6f4f (ANK_AGENT)"),
        "{said}"
    );
    assert!(
        said.find("identity ").unwrap() < said.find("no claim").unwrap(),
        "the identity introduces the claim it decides: {said}"
    );

    // Unset: the fallback, said as one. The value cannot be asserted literally —
    // it is the machine running the suite — so what is asserted is the shape of
    // the trap, `<user>@<hostname>`, and the fact that it is named as a
    // fallback rather than printed bare.
    let out = r.ank_env("claude-code/6f4f", &["status"], &[("ANK_AGENT", None)]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);
    let line = said
        .lines()
        .find(|l| l.starts_with("identity "))
        .unwrap_or_else(|| panic!("no identity line: {said}"))
        .to_string();
    assert!(
        line.ends_with(" (fallback, ANK_AGENT unset)"),
        "the fallback is named as one: {line}"
    );
    let value = &line["identity ".len()..line.find(" (").unwrap()];
    assert!(value.contains('@'), "user-at-host shape: {line}");
    assert!(!value.contains("claude-code/6f4f"), "{line}");

    // A variable set to nothing is a variable unset, and it is the likelier
    // accident of the two: the prefix was typed and the value was not.
    let said = stdout(&r.ank_env(
        "claude-code/6f4f",
        &["status"],
        &[("ANK_AGENT", Some("  "))],
    ));
    assert!(said.contains("(fallback, ANK_AGENT unset)"), "{said}");

    // `--json` separates the value from its source, as `config` already does:
    // a script decides on the token and never on the parenthesis.
    let j = stdout(&r.ank("claude-code/6f4f", &["status", "--json"]));
    assert!(
        j.contains("\"identity\":{\"value\":\"claude-code/6f4f\",\"source\":\"env\"}"),
        "{j}"
    );
    let j = stdout(&r.ank_env(
        "claude-code/6f4f",
        &["status", "--json"],
        &[("ANK_AGENT", None)],
    ));
    assert!(j.contains("\"source\":\"fallback\""), "{j}");

    // The trap, end to end. The claim is this session's; with the variable
    // forgotten the same session is another agent, `no claim` is the honest
    // answer, and nothing but the identity line says why.
    assert_eq!(code(&r.ank("claude-code/6f4f", &["claim", ID])), 0);
    let said = stdout(&r.ank_env("claude-code/6f4f", &["status"], &[("ANK_AGENT", None)]));
    assert!(said.contains("no claim"), "{said}");
    assert!(said.contains("elsewhere 1 claim(s)"), "{said}");
    assert!(said.contains("(fallback, ANK_AGENT unset)"), "{said}");
}

/// `ank graph` draws the `blocked_by` DAG (TASK-253e897d3330).
///
/// §5's ordering already walks these edges to count what a task unblocks; this
/// makes the same structure visible to a reader. Through the binary, because the
/// criterion is about what the process prints.
///
/// The chain is built so that every clause has something to be wrong about: a
/// root, two levels under it, a diamond, and a task whose only blocker sits
/// outside the perimeter.
#[test]
fn graph_draws_the_dag_and_names_the_perimeter() {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    std::fs::create_dir_all(r.0.join("docs")).unwrap();
    std::fs::write(r.0.join("docs/guide.md"), "# guide\n").unwrap();

    // root <- mid <- leaf, and `side` also blocked by root: a diamond only in
    // the sense that `leaf` is reachable twice once `side` blocks it too.
    let root = "TASK-000000000a01";
    let mid = "TASK-000000000a02";
    let leaf = "TASK-000000000a03";
    let outer = "TASK-000000000a04";
    let held = "TASK-000000000a05";
    for id in [root, mid, leaf, held] {
        r.seed_task(id, Some("A verifiable criterion."));
    }
    r.blocked(mid, &[root]);
    r.blocked(leaf, &[mid]);
    // Scoped to docs/, so it is outside the src/ perimeter below.
    r.seed_task_scoped(outer, "docs/**");
    r.blocked(held, &[outer]);

    let out = r.ank("claude-code@ank", &["graph", "src"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);

    // Names the perimeter it drew (§4).
    assert_eq!(
        said.lines().next().unwrap_or_default(),
        "src",
        "the perimeter is named: {said}"
    );

    // The shape: each level one connector deeper than what blocks it.
    //
    // Measured over the drawing alphabet of §4 rather than over whitespace. The
    // claim is unchanged — a01 flush left, a02 under it, a03 under that — but
    // `trim_start` answered it by counting spaces, and a row now begins with a
    // connector. It would have read zero for every indented line and gone on
    // passing for the wrong reason.
    let depth = |needle: &str| -> usize {
        let line = said
            .lines()
            .find(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("{needle} missing from:\n{said}"));
        let drawing = line
            .chars()
            .take_while(|c| matches!(c, ' ' | '│' | '├' | '└' | '─'))
            .count();
        // Reported in the old unit so the numbers below still say what they
        // said: four columns per level, two before.
        drawing / 2
    };
    assert_eq!(depth("000000000a01"), 0, "the root is flush left: {said}");
    assert_eq!(depth("000000000a02"), 2, "{said}");
    assert_eq!(depth("000000000a03"), 4, "{said}");

    // The connectors themselves, not only their width: a level drawn with the
    // right number of blanks and no glyph would satisfy every count above.
    let row = |needle: &str| -> &str {
        said.lines()
            .find(|l| l.contains(needle))
            .unwrap_or_else(|| panic!("{needle} missing from:\n{said}"))
    };
    assert!(
        row("000000000a02").starts_with("└── "),
        "the only child of the root is drawn as the last one: {said}"
    );
    assert!(
        row("000000000a03").starts_with("    └── "),
        "and its child continues under a cleared gutter: {said}"
    );

    // A blocker outside the perimeter is never silently dropped: drawing this
    // one flush left with no mark would say nothing is stopping it.
    let held_line = said
        .lines()
        .find(|l| l.contains("000000000a05"))
        .unwrap_or_default();
    assert!(
        held_line.contains("outside"),
        "a task held by something outside the perimeter says so: {said}"
    );
    assert!(
        !said.contains("000000000a04"),
        "and the outside task itself is not drawn: {said}"
    );

    // Says so explicitly when the perimeter holds no task (§4). `LICENSE` and
    // not `docs/nowhere`: the latter is *inside* `docs/**`, which is the whole
    // point of `overlaps_dir`, and the fixture would have been testing nothing.
    let out = r.ank("claude-code@ank", &["graph", "LICENSE"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("no task in this perimeter"),
        "{}",
        stdout(&out)
    );

    // `--json` for the raw edges (§4) — including the edge leaving the
    // perimeter, which the drawing can only mark and not show.
    let out = r.ank("claude-code@ank", &["graph", "src", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let j = stdout(&out);
    assert!(j.contains("\"path\":\"src\""), "{j}");
    assert!(
        j.contains(&format!("{{\"task\":\"{mid}\",\"blocked_by\":\"{root}\"}}")),
        "{j}"
    );
    assert!(
        j.contains(&format!(
            "{{\"task\":\"{held}\",\"blocked_by\":\"{outer}\"}}"
        )),
        "the raw edges include the one leaving the perimeter: {j}"
    );
}

/// A cycle is a corpus fault `check` reports, and `graph` still has to draw the
/// repository that has one rather than hang on it.
///
/// **Two cycles, because they take different paths through the code and only
/// one exercises the guard.** A cycle with no way in has no root, so nothing
/// recurses into it and it is collected flat at the end. A cycle reachable from
/// a root is walked, and only there does the visited-path check stand between
/// the drawing and a stack overflow. Removing that check left the first case
/// passing, which is how this test was found to be weaker than it looked — by a
/// mutation run, not by reading it.
#[test]
fn graph_terminates_on_a_cycle_and_says_where_it_is() {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    // Closed on itself and unreachable: no root, so nothing recurses into it.
    let a = "TASK-000000000b01";
    let b = "TASK-000000000b02";
    // A root leading into a cycle: this is the one that recurses, and the only
    // one the visited-path guard stands in front of.
    let root = "TASK-000000000b03";
    let mid = "TASK-000000000b04";
    let back = "TASK-000000000b05";
    for id in [a, b, root, mid, back] {
        r.seed_task(id, Some("A verifiable criterion."));
    }
    r.blocked(a, &[b]);
    r.blocked(b, &[a]);
    r.blocked(mid, &[root, back]);
    r.blocked(back, &[mid]);

    let out = r.ank("claude-code@ank", &["graph", "src"]);
    assert_eq!(
        code(&out),
        0,
        "a cycle is check's fault to report, not graph's: {}",
        stderr(&out)
    );
    let said = stdout(&out);

    // Reachable: walked, and stopped at the repetition rather than at the stack.
    assert!(
        said.contains("(cycle)"),
        "the walk names where it turned back: {said}"
    );
    assert!(
        said.contains("000000000b04") && said.contains("000000000b05"),
        "{said}"
    );

    // Unreachable: under no root, so it would vanish under a header with
    // nothing beneath it. Drawn, and named as the reason.
    assert!(said.contains("under no root"), "{said}");
    assert!(
        said.contains("000000000b01") && said.contains("000000000b02"),
        "{said}"
    );
}

/// `ank show` on a task says what it unblocks, not only what blocks it
/// (TASK-2415ddb92df8).
///
/// The same derivation `graph` draws, narrowed to one entity. Through the
/// binary, because the criterion is about what the process prints — and because
/// the entity has to still come out whole above the sections, which is a
/// property of the bytes on the pipe and of nothing else.
///
/// The fixture gives every clause something to be wrong about: a blocker that
/// is already `done`, two tasks waiting on the same one, a blocker naming an
/// entity the corpus does not hold, and a leaf with neither direction.
#[test]
fn show_lists_what_a_task_unblocks_alongside_its_blockers() {
    let r = Repo::new();
    let root = "TASK-000000000c01";
    let mid = "TASK-000000000c02";
    let leaf = "TASK-000000000c03";
    let side = "TASK-000000000c04";
    let ghost = "TASK-0000000000ff";
    for id in [root, mid, leaf, side] {
        r.seed_task(id, Some("A verifiable criterion."));
    }
    // One finished in each direction. A `done` entity keeps its line and
    // carries its status, which is what makes the status on each line worth
    // printing — and it is the difference between this list and §5's count,
    // which drops what is no longer held up because it is ordering work.
    for id in [root, side] {
        let done = r.task_text(id).replace("status: open", "status: done");
        std::fs::write(r.0.join(".ank/entities").join(format!("{id}.md")), done).unwrap();
    }
    r.blocked(mid, &[root, ghost]);
    r.blocked(leaf, &[mid]);
    r.blocked(side, &[mid]);

    let before = r.task_text(mid);
    let out = r.ank("claude-code@ank", &["show", mid]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);

    // The entity first, whole and unreformatted. `show` is still the one
    // command that does not summarise; the sections are appended under it.
    assert!(
        said.starts_with(&before),
        "the entity is no longer verbatim:\n{said}"
    );
    let derived = &said[before.len()..];

    // One line each, with status, in both directions.
    assert!(derived.contains("BLOCKED BY (2)"), "{said}");
    assert!(
        derived.contains(&format!("{root}  [done]")),
        "a blocker keeps its line and its status once done: {said}"
    );
    assert!(
        derived.contains(&format!("{ghost}  (no such entity)")),
        "a dangling blocker is named, never dropped: {said}"
    );
    assert!(derived.contains("UNBLOCKS (2)"), "{said}");
    let at = |needle: &str| derived.find(needle).unwrap_or_else(|| panic!("{said}"));
    assert!(
        derived.contains(&format!("{side}  [done]")),
        "a task that waited and is now done keeps its line: the list is the edge \
         set, not the count of what is still held up: {said}"
    );
    assert!(
        at(&format!("{leaf}  [open]")) < at(&format!("{side}  [done]")),
        "the derived direction is ordered by id, like every other listing: {said}"
    );
    // The two directions are not one list: `mid` is blocked by `root` and
    // unblocks `leaf`, and neither may appear under the other heading.
    let (blockers, unblocks) = derived.split_at(at("UNBLOCKS"));
    assert!(!blockers.contains(leaf), "{said}");
    assert!(!unblocks.contains(root), "{said}");

    // A leaf has neither direction, and both headings still print: an absent
    // heading and a heading with nothing under it are not the same answer.
    let out = r.ank("claude-code@ank", &["show", leaf]);
    let said = stdout(&out);
    assert!(said.contains("BLOCKED BY (1)"), "{said}");
    assert!(said.contains("UNBLOCKS (0)"), "{said}");

    // Derived at read time and stored nowhere: a task that did not exist at the
    // first read appears at the second, and the file `show` printed from is
    // byte for byte what it was.
    let late = "TASK-000000000c05";
    r.seed_task(late, Some("A verifiable criterion."));
    r.blocked(late, &[mid]);
    let out = r.ank("claude-code@ank", &["show", mid]);
    let said = stdout(&out);
    assert!(
        said.contains("UNBLOCKS (3)") && said.contains(&format!("{late}  [open]")),
        "the reverse edge is derived, not stored: {said}"
    );
    assert_eq!(
        r.task_text(mid),
        before,
        "show wrote the derivation into the entity it was asked to print"
    );

    // `--json` carries the same two lists, and the null of an unresolved one.
    let out = r.ank("claude-code@ank", &["show", mid, "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let j = stdout(&out);
    assert!(
        j.contains(&format!(
            "{{\"id\":\"{root}\",\"short\":\"{root}\",\"status\":\"done\""
        )),
        "{j}"
    );
    assert!(
        j.contains(&format!(
            "{{\"id\":\"{ghost}\",\"short\":\"{ghost}\",\"status\":null,\"title\":null}}"
        )),
        "{j}"
    );
    assert!(j.contains("\"unblocks\":[{"), "{j}");
}

/// One directory, one answer, however it is typed (TASK-df4c39031583).
///
/// Reported from a Windows shell against the real corpus. `docs` answered five
/// live constraints, `docs\` answered **four**, `.\docs\` answered zero. The
/// zeros were survivable because they were obvious; the four was not. It is what
/// tab-completion produces on Windows, it looks like a correct answer, and it
/// silently dropped a rule that binds — because the argument reached glob
/// matching verbatim, where a `**` glob still matched a backslash as an ordinary
/// character and a glob naming a segment did not.
///
/// Through the binary and across all four path-taking verbs, because the defect
/// was never in the matcher: it was in what each verb handed to it, and three of
/// them had their own copy of the handing.
///
/// **No claim is held in this fixture, and that is load-bearing.** `context`
/// with a claim is in execution mode and ignores the path entirely (§5), so a
/// comparison made while holding one would pass whatever the code did. I made
/// that exact mistake measuring the fix by hand.
#[test]
fn a_directory_resolves_the_same_however_the_path_is_written() {
    const ADR: &str = "ADR-00000000dddd";
    let r = Repo::new();
    r.enable_signing();
    std::fs::create_dir_all(r.0.join("docs")).unwrap();
    std::fs::write(r.0.join("docs/guide.md"), "# guide\n").unwrap();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    // One entity inside the perimeter, one outside: an answer that ignored the
    // path would match both and look like success.
    r.seed_adr(ADR, "Do not do X.", "docs/**");
    r.seed_task(ID, Some("A verifiable criterion."));
    // A dead scope under `docs/` and nowhere else. `check` reports corpus-wide
    // totals whatever the perimeter, so without a finding that exists on one
    // side and not the other it answers identically for every path — and the
    // comparison below would be measuring nothing. The assertion caught that too.
    r.seed_adr("ADR-00000000eeee", "Do not do Y.", "docs/missing/**");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    // Ratified, or `review` lists no live constraint at all and the two
    // perimeters answer identically for a reason that has nothing to do with
    // the path. The assertion below catches exactly that, and did.
    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let forms = [
        "docs",
        "docs/",
        "docs\\",
        "./docs",
        ".\\docs\\",
        "docs/../docs",
        "docs//",
    ];
    for verb in ["context", "review", "check", "scope"] {
        let base = r.ank("claude-code@ank", &[verb, "docs"]);
        let expected = stdout(&base);
        let expected_code = code(&base);
        for form in forms {
            let got = r.ank("claude-code@ank", &[verb, form]);
            assert_eq!(
                code(&got),
                expected_code,
                "`ank {verb} {form}` exits differently from `ank {verb} docs`: {}",
                stderr(&got)
            );
            assert_eq!(
                stdout(&got),
                expected,
                "`ank {verb} {form}` must answer as `ank {verb} docs` does"
            );
        }
        // And the perimeter is real: the ADR lives under docs/, the task under
        // src/, so the two paths must not give the same answer.
        let elsewhere = stdout(&r.ank("claude-code@ank", &[verb, "src"]));
        assert_ne!(
            elsewhere, expected,
            "`ank {verb}` gives one answer for two different perimeters, so \
             comparing path forms proves nothing"
        );
    }

    // `scope` echoes the perimeter it drew, and it has to echo the one it used.
    let out = r.ank("claude-code@ank", &["scope", ".\\docs\\"]);
    assert_eq!(
        stdout(&out).lines().next().unwrap_or_default(),
        "docs",
        "the echoed path is the normalised one: {}",
        stdout(&out)
    );

    // A path that leaves the repository has no answer, and saying nothing would
    // be the silently-partial set all over again.
    for outside in ["/etc/passwd", "..", "../sibling", "docs/../../elsewhere"] {
        let out = r.ank("claude-code@ank", &["review", outside]);
        assert_eq!(code(&out), 1, "{outside} must be refused: {}", stdout(&out));
        let err = stderr(&out);
        assert!(err.starts_with("error[1]:"), "{err}");
        assert!(
            err.contains("ank review"),
            "a refusal always says what to run next: {err}"
        );
    }
}

/// `ank scope <path>` answers what covers a path (TASK-e717ee625c5c).
///
/// The `check-ignore` of ank: a glob that matches nothing is otherwise only
/// discovered through `check`, after the entity is written and already
/// invisible. Through the binary, because the criterion is about what the
/// process prints and because dispatch reaching a new verb is the thing that
/// has silently failed here before (TASK-45d18f45de2c).
#[test]
fn scope_says_what_covers_a_path_and_says_when_nothing_does() {
    const ADR: &str = "ADR-00000000c0de";
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    r.seed_task(ID, Some("A verifiable criterion."));

    let out = r.ank("claude-code@ank", &["scope", "src/main.rs"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);

    // Grouped by type, one line each, carrying the status.
    assert!(said.contains("ADR (1)"), "{said}");
    assert!(said.contains("TASKS (1)"), "{said}");
    assert!(said.contains("[proposed] A decision"), "{said}");
    assert!(said.contains("[open] Example task"), "{said}");
    assert!(
        said.lines()
            .next()
            .unwrap_or_default()
            .contains("src/main.rs"),
        "it names the perimeter it drew: {said}"
    );

    // The same resolution `context` binds with. A directory the globs reach
    // under is covered, which is what `overlaps_dir` means and what an agent
    // asking about a package rather than a file needs.
    let out = r.ank("claude-code@ank", &["scope", "src"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("ADR (1)"), "{}", stdout(&out));

    // Nothing matching is said, never left to an empty answer: silence reads as
    // "nothing constrains this", which is the same sentence as "ank could not
    // tell", and only one of the two is safe to act on.
    let out = r.ank("claude-code@ank", &["scope", "docs/whitepaper.md"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("nothing covers this path"),
        "{}",
        stdout(&out)
    );

    // It answers about a path, not about the filesystem: a file that does not
    // exist yet resolves like any other, which is the point of asking before
    // writing the entity.
    let out = r.ank("claude-code@ank", &["scope", "src/not_written_yet.rs"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("ADR (1)"), "{}", stdout(&out));

    // `--json` is available on every command without exception (§4).
    let out = r.ank("claude-code@ank", &["scope", "src/main.rs", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let j = stdout(&out);
    assert!(j.contains("\"path\":\"src/main.rs\""), "{j}");
    assert!(j.contains("\"total\":2"), "{j}");
    assert!(j.contains("\"adr\":[{"), "{j}");
    assert!(j.contains("\"tasks\":[{"), "{j}");

    // A missing path is a refusal carrying the command to run next, not a
    // silent listing of the whole corpus.
    let out = r.ank("claude-code@ank", &["scope"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("ank scope <path>"),
        "{}",
        stderr(&out)
    );
}

/// The stamp follows the commit, including a commit that changes no file
/// (TASK-0b26c8b5bfc5).
///
/// The first `build.rs` emitted no `rerun-if-changed`, so Cargo watched the
/// package instead and a commit touching no file in it never reran the script.
/// Measured on the build that shipped `--version`: `HEAD 3110392`, stamp
/// `aeb1841`. A diagnostic that is sometimes wrong is one people learn to
/// re-verify by hand.
///
/// **Why a fixture crate and not `ank` itself.** The claim is about when Cargo
/// reruns the build script, and that needs a real `cargo build` around a real
/// commit — but rebuilding `ank` from inside its own test suite relinks
/// `target/debug/ank.exe` while other tests are executing it, which is the
/// Windows relink trap the development guide documents, and a fresh target
/// directory would rebuild `libsqlite3-sys` on every CI job of all three
/// platforms. So the fixture is a trivial crate driving **the real
/// `build.rs`** — the file under test, not a copy of its logic — through a real
/// cargo build and a real git commit. What that leaves unexercised is ank's own
/// linking, which the claim does not depend on. The literal measurement on
/// `ank --version` was taken by hand and recorded in the task log.
#[test]
fn the_stamp_follows_a_commit_that_changes_no_file() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("build.rs");

    let dir = std::env::temp_dir().join(format!("ank-stamp-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::copy(&script, dir.join("build.rs")).unwrap();
    // An empty `[workspace]` so the fixture is not adopted by any workspace it
    // happens to sit under, and the one build-dependency the real script has:
    // it hashes SKILL.md with `ank_core::freeze_hash_short` (TASK-ecda4070354f),
    // and the fixture drives the real script rather than a copy of its logic, so
    // it has to build like the real one. By path, and the builds below are
    // `--offline`, so this still needs no registry beyond the cache the
    // workspace build has already warmed.
    let core = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ank-core")
        .to_string_lossy()
        .replace('\\', "/");
    std::fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[workspace]\n\n[package]\nname = \"stamp\"\nversion = \"0.0.0\"\n\
             edition = \"2021\"\n\n[build-dependencies]\n\
             ank-core = {{ path = \"{core}\" }}\n"
        ),
    )
    .unwrap();
    std::fs::write(
        dir.join("src/main.rs"),
        "fn main() { println!(\"{}\", env!(\"ANK_COMMIT\")); }\n",
    )
    .unwrap();

    let git = |args: &[&str]| {
        let out = git_command(&dir)
            .args(args)
            .output()
            .expect("git must be installed");
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    git(&["init", "-q", "-b", "main"]);
    git(&["config", "user.email", "test@ank.local"]);
    git(&["config", "user.name", "Test"]);
    // The one fixture in this file that commits without being a `Repo`
    // (TASK-40a972e98a9a).
    git(&["config", "commit.gpgsign", "false"]);
    git(&["config", "tag.gpgsign", "false"]);
    git(&["add", "-A"]);
    git(&["commit", "-qm", "seed"]);

    let build_and_read = || {
        let out = spawn(&cargo)
            .current_dir(&dir)
            .args(["run", "-q", "--offline"])
            .env("CARGO_TARGET_DIR", dir.join("target"))
            .output()
            .expect("cargo must be on PATH");
        assert!(
            out.status.success(),
            "cargo run: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };

    let first = build_and_read();
    assert_eq!(
        first,
        git(&["rev-parse", "--short", "HEAD"]),
        "the stamp must name the commit it was built at"
    );

    // The case the old script missed: the commit moves, no file does.
    git(&[
        "commit",
        "-q",
        "--allow-empty",
        "-m",
        "nothing changes but the commit",
    ]);
    let moved = git(&["rev-parse", "--short", "HEAD"]);
    assert_ne!(moved, first, "the fixture must actually have moved");

    let second = build_and_read();
    assert_eq!(
        second, moved,
        "a rebuild after a commit that touches no file must name the new HEAD, \
         not the one before it"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

/// The binary can say what it is, and can say it where nothing else works
/// (TASK-548c518cb705).
///
/// TASK-1ea38a17d854 cost a full investigation to conclude that the binary in
/// hand predated the feature it was being measured for. The answer was a file
/// timestamp. One command should have said it.
///
/// The second half is the half that matters and the one a unit test cannot
/// reach: `--version` has to answer *before* the repository is resolved, git is
/// checked and the config is loaded. A version flag that needs a healthy
/// environment is mute in the one situation it exists for, and only a process
/// launched somewhere hostile proves it is not.
#[test]
fn the_version_answers_before_anything_can_stop_it() {
    let r = Repo::new();

    let out = r.ank("claude-code@ank", &["--version"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out).trim().to_string();
    assert!(
        said.starts_with(&format!("ank {}", env!("CARGO_PKG_VERSION"))),
        "the crate version, so a build can be placed against a release: {said}"
    );
    assert!(
        said.contains('(') && said.ends_with(')'),
        "and the commit it was built from: {said}"
    );
    // Empty parentheses would satisfy the shape and answer nothing. The commit
    // is the first value inside them; the skill revision beside it is
    // TASK-ecda4070354f's and is asserted against the file itself in
    // tests/skill.rs, which is the only place that comparison means anything.
    let inside = said
        .rsplit_once('(')
        .and_then(|(_, c)| c.strip_suffix(')'))
        .unwrap_or("");
    let commit = inside.split(',').next().unwrap_or("").trim();
    assert!(
        commit.len() >= 4,
        "a sha or the word `unknown`, never a blank: {said}"
    );
    assert!(
        inside.contains("skill "),
        "and the skill revision it was built alongside: {said}"
    );

    // Nowhere in particular: no `.ank/`, no git repository, no config. This is
    // where a stale or unidentified binary is actually met.
    let nowhere = std::env::temp_dir().join("ank-cli-it-nowhere");
    std::fs::create_dir_all(&nowhere).unwrap();
    let bare = ank_command()
        .arg("--version")
        .current_dir(&nowhere)
        .output()
        .expect("the binary must have been built");
    assert_eq!(
        code(&bare),
        0,
        "outside any repository it must still answer: {}",
        stderr(&bare)
    );
    assert_eq!(
        stdout(&bare).trim(),
        said,
        "and answer the same thing, since it describes the binary and not the place"
    );
    let _ = std::fs::remove_dir_all(&nowhere);

    // Discoverable, or it answers a question nobody knows how to ask.
    let out = r.ank("claude-code@ank", &["help"]);
    assert!(
        stdout(&out).contains("--version"),
        "help must name it: {}",
        stdout(&out)
    );
}

/// Git refusing to answer is an answer, and it used to be silence
/// (TASK-c92b7cc10f13).
///
/// `signature_state` ended in `.ok()?`, so any failure of the git invocation
/// became `None` — the one verdict `check_adr` says nothing about. An ADR whose
/// signature could not be read was indistinguishable from one in a corpus that
/// declares no key at all, which is the degradation to success ADR-6b3f refuses
/// and the `Unchecked` counter already exists to prevent one level down.
///
/// The lever is a `gpg.format` git rejects, one of the causes the task named.
/// It is surgical: `rev-list --full-history`, `cat-file`, `rev-parse` and
/// `for-each-ref` all still answer, so the corpus is read normally and the
/// ratification commit is still reached — only the signature read exits
/// non-zero. That is exactly the state under test, and no other.
///
/// A signal and not a fault, deliberately: a broken environment is not a forged
/// ratification, and exiting 8 over one would fail the `check-repo` verifier of
/// every task on a machine missing nothing but a working gpg config.
#[test]
fn a_signature_git_cannot_read_is_reported_rather_than_passed_over() {
    const ADR: &str = "ADR-0000000000ef";
    let r = Repo::new();
    r.enable_signing();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    declare_signing_key(&r);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // Readable and trusted: the silence below has to be broken by the config
    // change and by nothing else.
    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        !stdout(&out).contains("could not be read"),
        "{}",
        stdout(&out)
    );

    r.git(&["config", "gpg.format", "bogus"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let said = stdout(&out);
    assert!(
        said.contains("could not be read"),
        "a signature git refuses to answer about must be reported: {said}"
    );
    assert!(
        said.contains("neither verified nor refused"),
        "and it must say what that leaves undecided: {said}"
    );
    assert!(
        said.contains("gpg.format"),
        "carrying git's own reason, or the reader cannot act on it: {said}"
    );

    // Not a fault, and not confused with the missing-key case either.
    assert_eq!(
        code(&out),
        0,
        "a broken environment is not a forgery: {said}"
    );
    assert!(
        !said.contains("not signed") && !said.contains("no public key"),
        "an unread signature is not an absent one: {said}"
    );
}

/// A claim is not an orphan just because this checkout is too old to have
/// heard of the task (TASK-52fbffbfdf65).
///
/// Observed three times while diagnosing TASK-1ea38a17d854, each time on a
/// claim being held: a `check` run from a detached worktree at an earlier
/// commit printed `pruned refs/ank/claims/<id>` and deleted it. `refs/ank/` is
/// shared by every worktree, so the older checkout reached into the coordination
/// plane of the current one. A lost claim is a task two agents can hold at once.
///
/// Through the binary, and with the ref read by git afterwards rather than
/// through `claim::read`: what has to be true is the state of the repository,
/// not the module's agreement with itself.
#[test]
fn a_check_from_an_older_checkout_leaves_a_live_claim_alone() {
    let r = Repo::new();
    // An ADR-less corpus still needs its scope to exist, or `check` exits 8 on
    // a dead scope before any of this is reached.
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    let before_the_task = r.head();

    // The task arrives after that commit, which is the whole point: the older
    // checkout below cannot see it.
    r.seed_task(ID, Some("A verifiable criterion."));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "add the task"]);

    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(r.claim_ref(ID).is_some(), "the claim must exist to be lost");

    let older = r.0.with_extension("older-checkout");
    r.git(&[
        "worktree",
        "add",
        "--detach",
        older.to_str().unwrap(),
        &before_the_task,
    ]);

    // Another agent, running `check` from that checkout. It sees no tasks at
    // all, and must still not touch a ref it cannot judge.
    let out = r.ank_at("codex@host-9", &["check"], &older);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        !stdout(&out).contains("pruned"),
        "nothing to prune from a checkout that cannot see the corpus: {}",
        stdout(&out)
    );
    assert!(
        r.claim_ref(ID).is_some(),
        "the claim survives a check run from a checkout older than the task"
    );

    r.git(&["worktree", "remove", "--force", older.to_str().unwrap()]);

    // The other half, or the fix would be "never prune". A ref whose task the
    // default branch really has lost is still collected, and the live one is
    // still left alone in the same pass.
    let gone = "TASK-00000000dead";
    r.seed_task(gone, Some("A verifiable criterion."));
    let out = r.ank("codex@host-9", &["claim", gone]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    std::fs::remove_file(r.0.join(".ank/entities").join(format!("{gone}.md"))).unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "drop it"]);

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        r.claim_ref(gone).is_none(),
        "a real orphan is still collected: {}",
        stdout(&out)
    );
    assert!(r.claim_ref(ID).is_some(), "and the live claim still is not");
}

#[test]
fn claiming_through_the_binary_takes_the_ref_and_moves_the_task() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let record = r
        .claim_ref(ID)
        .expect("the ref must exist, read with git and not through the module");
    assert!(record.contains("state: claim"), "{record}");
    assert!(record.contains("claude-code@ank"), "{record}");
    assert!(record.contains("expires:"), "{record}");

    let text = r.task_text(ID);
    assert!(text.contains("status: in_progress"), "{text}");
    assert!(
        !text.contains("claude-code@ank") && !text.contains("expires"),
        "the claim must not reach the file (ADR-4e7c25b1f639):\n{text}"
    );

    // A prefix resolves, the way an agent actually types it.
    let out = r.ank("claude-code@ank", &["claim", "0000"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
}

/// Two live claims whose scopes meet are named at pickup, and the task is taken
/// anyway (ADR-052accd6e3b2).
///
/// Through the binary because the criterion is about the process: what has to
/// be true is that the lines appear **and** the exit code is 0. A refusal here
/// would turn a coarse glob into a mutex — one held task scoped
/// `crates/ank-cli/tests/**` made five of seven candidates unworkable in a real
/// session — and would push agents to declare narrower scopes than the truth to
/// get past it.
///
/// The defect this exists to have prevented: two agents on two disjoint tasks
/// each appended a block of tests to the end of `crates/ank-cli/tests/cli.rs`,
/// twice, and nothing warned either of them. The claims were on disjoint tasks,
/// correctly; the edits were on one file, and the claim plane has nothing to
/// say about files.
#[test]
fn a_claim_names_the_live_claims_whose_scope_it_overlaps_and_takes_the_task() {
    let r = Repo::new();
    let tests = "TASK-a00000000001";
    let cli = "TASK-b00000000001";
    let docs = "TASK-c00000000001";
    let after = "TASK-d00000000001";
    r.seed_task_scoped(tests, "crates/ank-cli/tests/**");
    r.seed_task_scoped(cli, "crates/ank-cli/**");
    r.seed_task_scoped(docs, "docs/**");
    r.seed_task_scoped(after, "crates/ank-cli/**");

    assert_eq!(code(&r.ank("mia@laptop", &["claim", tests])), 0);

    // Globs that overlap on a file: named, and named with the ground rather
    // than with the fact of an overlap.
    let out = r.ank("bob@laptop", &["claim", cli]);
    assert_eq!(
        code(&out),
        0,
        "an overlap is a signal and never a refusal: {}",
        stderr(&out)
    );
    let said = stdout(&out);
    assert!(said.contains("mia@laptop"), "the holder is named: {said}");
    assert!(said.contains(tests), "the task is named: {said}");
    assert!(
        said.contains("crates/ank-cli/tests/**"),
        "a line saying only that two scopes overlap leaves the reader where \
         they started: {said}"
    );
    assert!(
        r.claim_ref(cli).is_some(),
        "and the task is taken regardless: {said}"
    );

    // Globs that do not overlap: nothing at all. A signal that fires on
    // everything is one agents learn to skip.
    let out = r.ank("carol@laptop", &["claim", docs]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);
    assert!(
        !said.contains("mia@laptop") && !said.contains("bob@laptop"),
        "docs/** meets neither crates glob: {said}"
    );

    // A lapsed claim is not a live one. Getting this wrong makes the signal
    // fire on abandoned work forever, which is the same reading `claim`'s own
    // refusal already applies.
    r.expire_claim(tests);
    r.expire_claim(cli);
    let out = r.ank("dave@laptop", &["claim", after]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);
    assert!(
        !said.contains(tests) && !said.contains(cli),
        "the same overlap, against two lapsed claims, must produce nothing: \
         {said}"
    );
}

#[test]
fn the_exit_code_of_a_refusal_reaches_the_process() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("codex@host-9", &["claim", ID])), 0);

    // Held by somebody else: code 4, and the message names the holder.
    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 4, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.starts_with("error[4]:"), "{err}");
    assert!(err.contains("codex@host-9"), "{err}");
    assert!(
        err.contains("->"),
        "a refusal always says what to run next: {err}"
    );

    // No criterion: code 7, and the message carries the command that sets one.
    let other = "TASK-00000000ffff";
    r.seed_task(other, None);
    let out = r.ank("claude-code@ank", &["claim", other]);
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(stderr(&out).contains("--criteria"), "{}", stderr(&out));
    assert!(
        r.claim_ref(other).is_none(),
        "a refusal must leave no ref behind"
    );

    // An unknown id is code 2, which comes from the store and must survive the
    // trip out just the same.
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", "abcd"])), 2);
}

#[test]
fn every_verb_of_the_surface_answers() {
    // This test used to assert the opposite, verb by verb, as each module was
    // still a stub. The last of them landed with the human surface, so what it
    // guards now is that none regresses to `not_implemented` — a rejection is a
    // code 1 carrying that exact phrase, and no verb may produce it.
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task(ID, Some("A criterion."));
    for argv in [
        vec!["context"],
        vec!["find", "criterion"],
        vec!["show", ID],
        vec!["check"],
        vec!["review"],
        vec!["claim", ID],
        vec!["log", "working"],
        vec!["release", "--reason", "handing it over"],
        vec!["attest", ID, "--proof", "assertion:x"],
        vec!["close", ID, "--reason", "not needed after all"],
        vec!["help"],
    ] {
        let out = r.ank("claude-code@ank", &argv);
        let err = stderr(&out);
        assert!(
            !err.contains("not implemented yet"),
            "{argv:?} still answers not_implemented: {err}"
        );
    }
}

#[test]
fn the_foundation_is_crossed_before_the_verb_and_names_its_own_failures() {
    // A --repo pointing nowhere is named as such, never disguised as a broken
    // environment -- that ordering is why `startup` resolves the repository
    // before checking git.
    let out = ank_command()
        .args(["claim", ID, "--repo"])
        .arg(std::env::temp_dir().join("ank-does-not-exist"))
        .output()
        .unwrap();
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).contains(".ank"), "{}", stderr(&out));

    // And a parse error never reaches the foundation at all.
    let out = ank_command()
        .args(["claim", "--tll", "30m"])
        .output()
        .unwrap();
    assert_eq!(code(&out), 1);
    assert!(stderr(&out).contains("--tll"), "{}", stderr(&out));
}

#[test]
fn json_is_available_on_the_verb_that_exists() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let out = r.ank("claude-code@ank", &["claim", ID, "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.trim().starts_with('{'), "{text}");
    assert!(text.contains(ID), "{text}");
    assert!(text.contains("expires"), "{text}");
}

#[test]
fn context_exits_zero_in_both_modes_and_switches_on_the_claim() {
    // "exits 0 with an explicit message" is a statement about the process, so
    // it is the process that answers it.
    let r = Repo::new();
    r.seed_task(ID, Some("Auth integration tests pass."));

    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("TASKS (1)"), "{text}");
    assert!(text.contains("> ank claim"), "{text}");
    assert!(
        !text.contains("DONE_CRITERIA"),
        "orientation stays broad: {text}"
    );

    // Claiming flips the same command to execution, with no extra argument.
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("DONE_CRITERIA"), "{text}");
    assert!(text.contains("Auth integration tests pass."), "{text}");
    assert!(
        !text.contains("> ank claim"),
        "no other task to offer: {text}"
    );

    // A path argument is ignored while a claim is held, and said so.
    let out = r.ank("claude-code@ank", &["context", "src/"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(code(&out), 0);
    assert!(text.contains("execution context"), "{text}");

    // Another agent, no claim of its own: orientation again, and the task is
    // shown as held rather than offered.
    let out = r.ank("codex@host-9", &["context"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("[claimed:claude-code@ank]"), "{text}");
    assert!(text.contains("no ready tasks in scope"), "{text}");
}

#[test]
fn an_empty_corpus_is_a_clean_stop_and_not_a_breakdown() {
    // An agent in a loop needs a signal it can act on; an empty output reads
    // as a failure and triggers pointless retries.
    let r = Repo::new();
    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("no ready tasks in scope"), "{text}");
    assert!(!text.trim().is_empty());
}

#[test]
fn context_json_reaches_the_process_intact() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));
    let out = r.ank("claude-code@ank", &["context", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.trim_start().starts_with('{'), "{text}");
    assert!(text.contains("\"mode\":\"orientation\""), "{text}");
    assert!(text.contains("\"ready\":1"), "{text}");
}

/// A `.ank/` corpus with no repository anywhere above it.
///
/// `GIT_CEILING_DIRECTORIES` rather than trusting the machine. Git's walk for
/// `.git` would otherwise leave the temporary directory and could well reach the
/// developer's own repository — `TMPDIR` inside a checkout is unusual and not
/// forbidden — and a test whose entire subject is "there is no repository here"
/// must not depend on where temp happens to live. The ceiling stops the walk at
/// the fixture's parent, on every machine, so the state under test is
/// constructed rather than hoped for.
///
/// The `.ank/` walk needs no such treatment: `discover` stops at the first
/// `.ank/` going up, and the fixture puts one at its own root.
struct Bare(PathBuf);

impl Bare {
    fn new() -> Bare {
        static SEQ: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "ank-cli-bare-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(p.join(".ank/entities")).unwrap();
        std::fs::create_dir_all(p.join("src")).unwrap();
        std::fs::write(p.join("src/main.rs"), "fn main() {}\n").unwrap();
        std::fs::write(
            p.join(".ank/config.yml"),
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n",
        )
        .unwrap();
        std::fs::write(
            p.join(".ank/entities").join(format!("{ID}.md")),
            format!(
                "---\nid: {ID}\ntype: task\nslug: example\ntitle: Example task\n\
                 created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
                 blocked_by: []\ndone_criteria: |\n  A verifiable criterion.\n\
                 criteria_by: creator\nschema: 1\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
        Bare(p)
    }

    /// Run from inside the corpus and without `--repo`, because the criterion is
    /// about a caller standing in a directory, and `--repo` short-circuits the
    /// very walk being tested.
    fn ank(&self, args: &[&str]) -> Output {
        ank_command()
            .args(args)
            .env("ANK_AGENT", "claude-code@ank")
            .env("GIT_CEILING_DIRECTORIES", self.0.parent().unwrap())
            .current_dir(&self.0)
            .output()
            .expect("the binary must have been built")
    }
}

impl Drop for Bare {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Reading a corpus needs a parser; coordinating needs an arbiter.
///
/// The gate used to stand in front of the dispatch rather than in front of the
/// operation: `startup` called `git::ensure_usable` for every verb but `help`,
/// `config` and `--version`, so `show`, `find`, `graph`, `scope`, `new`, `amend`
/// and the whole formal half of `check` refused outside a repository although
/// none of them touches a ref, a commit or a branch (ADR-9307e5d214a7).
///
/// Both halves are asserted here, and the second is the one that keeps this
/// honest: degrading the coordinating verbs too would make the first half pass
/// while removing the property the coordination plane exists for. A `claim` with
/// no arbiter would succeed and guarantee nothing.
#[test]
fn outside_a_repository_the_readers_answer_and_the_coordinators_refuse() {
    let b = Bare::new();

    // Code 9 is the environment, and the criterion is that it never appears for
    // want of a repository. 8 stays legitimate: it means findings.
    for args in [
        vec!["show", ID],
        vec!["find", "--status", "open"],
        vec!["graph"],
        vec!["scope", "src/main.rs"],
        vec!["check"],
        vec!["context"],
        vec!["status"],
        vec![
            "new", "task", "--title", "T", "--scope", "src/**", "-c", "C.",
        ],
        vec!["amend", ID, "--scope", "docs/**"],
    ] {
        let out = b.ank(&args);
        assert!(
            code(&out) == 0 || code(&out) == 8,
            "ank {} exited {} outside a repository: {}",
            args.join(" "),
            code(&out),
            stderr(&out)
        );
    }

    // And the arbiter is still required where an arbiter is the point. Each one
    // names the command to run, which is what separates a refusal from a wall.
    for args in [
        vec!["claim", ID],
        vec!["log", "a message"],
        vec!["done", "--proof", "test:1"],
        vec!["release", "--reason", "why"],
        vec!["close", ID, "--reason", "why"],
        vec!["accept", "ADR-000000000001"],
        vec!["attest", ID, "--proof", "test:1"],
        vec!["init"],
    ] {
        let out = b.ank(&args);
        let said = format!("{}{}", stdout(&out), stderr(&out));
        assert_eq!(
            code(&out),
            9,
            "ank {} did not refuse outside a repository: {said}",
            args.join(" ")
        );
        assert!(
            said.contains("git init"),
            "ank {} refused without naming the command: {said}",
            args.join(" ")
        );
    }
}

/// `check` grows two halves, and the absent one says so exactly once.
///
/// A check that silently examines less than it did is how a corpus passes a gate
/// that stopped looking, so the line is not optional. Exactly one, because the
/// consequences of the missing half are many — no claim refs, no default branch,
/// no signatures, no pruning — and reporting each would turn one state into a
/// wall of findings. The maintenance arm distinguishes "never asked" from
/// "asked and indeterminable" for precisely that reason.
#[test]
fn check_outside_a_repository_skips_the_coordination_half_and_says_so_once() {
    let b = Bare::new();
    let out = b.ank(&["check"]);
    let said = stdout(&out);

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let lines: Vec<&str> = said
        .lines()
        .filter(|l| l.contains("coordination"))
        .collect();
    assert_eq!(
        lines.len(),
        1,
        "the coordination half must report its absence once and once only: {said}"
    );
    assert!(lines[0].contains("skipped"), "{said}");
    assert!(
        lines[0].starts_with("signal:"),
        "a corpus outside a repository is not a sick corpus, so the exit code \
         must keep meaning what it means: {said}"
    );
    // The formal half really ran, rather than the verb having answered nothing
    // and reported the skip.
    assert!(said.contains("1 tasks"), "{said}");

    // `--json` carries it too. A pipeline is the caller most likely to be
    // outside a repository, and the one least able to notice a line it never
    // sees.
    let json = stdout(&b.ank(&["check", "--json"]));
    assert!(json.trim_start().starts_with('{'), "{json}");
    assert!(json.contains("coordination"), "{json}");
    assert!(json.contains("skipped"), "{json}");
}

/// Inside a repository, nothing observable moves.
///
/// This is the regression guard, and the task it belongs to is meant to change
/// what happens outside a repository and nothing whatsoever inside one. The
/// byte-for-byte comparison against the previous build was made on this
/// repository's own corpus — `check`, `status`, `review` and `graph`, all
/// identical, coordination findings included — and what a test can hold from
/// here is the property that comparison proved: the coordination half runs, and
/// never announces itself as skipped.
#[test]
fn inside_a_repository_the_coordination_half_still_runs() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("codex@host-9", &["claim", ID])), 0);

    let said = stdout(&r.ank("claude-code@ank", &["check"]));
    assert!(
        !said.contains("skipped"),
        "the coordination half announced itself absent inside a repository: {said}"
    );

    // The plane was read rather than merely not refused: the claim another agent
    // holds reaches a reader that had to enumerate the refs to know about it.
    let ctx = stdout(&r.ank("claude-code@ank", &["context"]));
    assert!(ctx.contains("[claimed:codex@host-9]"), "{ctx}");

    let status = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(status.contains("branch main"), "{status}");
    assert!(!status.contains("no git repository"), "{status}");
}

/// A perimeter holding one proposal, with a budget as the only variable.
fn proposed_fixture(budget: &str, with_proposal: bool) -> Repo {
    let r = Repo::new();
    r.set_config(&format!(
        "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\ncontext_budget: {budget}\n"
    ));
    r.seed_task(ID, Some("A verifiable criterion."));
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    if with_proposal {
        r.seed_adr(
            "ADR-0000000000ab",
            "Prefer idempotent migrations.",
            "src/**",
        );
    }
    r
}

/// The PROPOSED header counts the perimeter, never what survived the budget.
///
/// The defect this pins was visible at the root of this repository, on a corpus
/// holding exactly one proposed ADR:
///
///     PROPOSED (0, non-binding)
///       +1 more
///
/// The header asserting there were none, the next line asserting one had been
/// cut, and nothing telling the reader which of the two to believe
/// (TASK-058469991999). PROPOSED is the one section truncation can empty: the
/// cutting order takes proposals down to zero while it stops at one task and at
/// one constraint, so a header counting survivors read `(0)` there and nowhere
/// else.
///
/// Through the binary, because the number is produced by the render and the
/// render is only reached by the cutting loop, which is reached by the
/// configured budget. A unit test on a hand-built view can assert the header
/// without ever going through the loop that produces the contradiction.
#[test]
fn the_proposed_header_counts_the_perimeter_and_never_contradicts_its_notice() {
    // Room for everything. The header reads 1 and no notice is printed, which
    // is what makes the tight case below a statement about truncation rather
    // than about a header that prints a constant.
    let roomy = proposed_fixture("8000", true);
    let text = stdout(&roomy.ank("claude-code@ank", &["context"]));
    assert!(text.contains("PROPOSED (1, non-binding)"), "{text}");
    assert!(text.contains("ADR-0000"), "{text}");
    assert!(!text.contains("not shown"), "nothing was cut: {text}");

    // Tight enough that the proposal itself is cut away.
    let tight = proposed_fixture("100", true);
    let text = stdout(&tight.ank("claude-code@ank", &["context"]));
    assert!(
        !text.contains("ADR-0000"),
        "the budget was not tight enough to cut the proposal, so this test \
         asserts nothing: {text}"
    );
    assert!(
        text.contains("PROPOSED (1, non-binding)"),
        "the header counted the survivors instead of the perimeter: {text}"
    );

    // The invariant read back out of the output rather than asserted as a
    // wording: the header equals what was printed plus what was cut. A fix that
    // changes either number in isolation fails here.
    let header: usize = text
        .lines()
        .find_map(|l| l.trim().strip_prefix("PROPOSED ("))
        .and_then(|rest| rest.split(',').next())
        .expect("the section is present")
        .parse()
        .expect("the header leads with a count");
    let notice = text
        .lines()
        .find(|l| l.contains("not shown"))
        .unwrap_or_else(|| panic!("the section was emptied and said nothing: {text}"));
    let cut: usize = notice
        .trim()
        .strip_prefix('+')
        .and_then(|rest| rest.split_whitespace().next())
        .expect("the notice leads with a count")
        .parse()
        .expect("the notice leads with a count");
    let printed = text
        .lines()
        .filter(|l| l.trim_start().starts_with("ADR-"))
        .count();
    assert_eq!(
        header,
        printed + cut,
        "the header disagrees with its own notice: {text}"
    );

    // The command the notice names answers rather than refuses. Naming a
    // command that fails on the spot is what the error style forbids
    // everywhere, and a truncation notice is not exempt from it.
    let named: Vec<&str> = notice
        .split_once("ank ")
        .expect("the notice names the command that recovers what it cut")
        .1
        .split_whitespace()
        .collect();
    let out = tight.ank("claude-code@ank", &named);
    assert_eq!(code(&out), 0, "ank {}: {}", named.join(" "), stderr(&out));
    assert!(stdout(&out).contains("ADR-0000"), "{}", stdout(&out));

    // A perimeter holding no proposal prints no section, at either budget, and
    // in particular no notice counting what was never there.
    for budget in ["8000", "100"] {
        let none = proposed_fixture(budget, false);
        let text = stdout(&none.ank("claude-code@ank", &["context"]));
        assert!(!text.contains("PROPOSED"), "at budget {budget}: {text}");
    }
}

#[test]
fn a_done_through_the_binary_leaves_a_completion_ref_naming_commit_and_branch() {
    // Read back with git rather than through the module: what has to be true
    // is the state of the repository, not the module's agreement with itself.
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task_with(ID, Some("A criterion."), &["ok"]);
    let head = r.head();

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    let out = r.ank("claude-code@ank", &["done"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert!(text.contains("proof recorded:"), "{text}");
    // The progress line moved to standard error, where it does not sit ahead
    // of the JSON document `--json` puts on stdout (TASK-2eefcdd80124). It is
    // still said, which is asserted here rather than only in the test that owns
    // the rule -- this is the test that watches a whole `done` from outside.
    assert!(
        !text.contains("running:"),
        "progress reached stdout: {text}"
    );
    assert!(
        stderr(&out).contains("running: ok ... ok"),
        "the progress line was dropped rather than moved: {}",
        stderr(&out)
    );

    let record = r
        .claim_ref(ID)
        .expect("the ref survives the done, it is not deleted");
    assert!(record.contains("state: completed"), "{record}");
    assert!(record.contains(&head), "the HEAD commit is named: {record}");
    assert!(
        record.contains("branch: main"),
        "the branch is named: {record}"
    );
    assert!(
        !record.contains("expires"),
        "a completion carries no TTL: {record}"
    );

    let file = r.task_text(ID);
    assert!(file.contains("status: done"), "{file}");
    assert!(
        file.contains("type: test"),
        "one proof entry per verifier: {file}"
    );
}

// ---------------------------------------------------------------------------
// Short forms (TASK-f3e92656b5df, ADR-962c25797569)
// ---------------------------------------------------------------------------

/// The short form is the long form, and the output proves it byte for byte.
///
/// Comparing the two invocations against each other rather than against an
/// expected listing is what makes this a test of the parser: it cannot pass
/// because `find` was taught to print something, only because both spellings
/// reached the same flag with the same value.
#[test]
fn a_short_flag_is_the_long_flag_and_takes_its_value_both_ways() {
    let r = Repo::new();
    r.seed_task("TASK-000000000f01", Some("A criterion."));
    r.seed_task("TASK-000000000f02", Some("A criterion."));
    seed_done(
        &r,
        "TASK-000000000f03",
        "  - type: commit\n    ref: abc1234\n",
    );

    let long = r.ank(
        "claude-code@ank",
        &["find", "criterion", "--status", "open"],
    );
    assert_eq!(code(&long), 0, "{}", stderr(&long));
    let expected = stdout(&long);
    assert!(
        expected.contains("TASK-000000000f01") && !expected.contains("TASK-000000000f03"),
        "the fixture must actually filter, or the comparison proves nothing:\n{expected}"
    );

    for argv in [
        vec!["find", "criterion", "-s", "open"],
        vec!["find", "criterion", "-s=open"],
    ] {
        let out = r.ank("claude-code@ank", &argv);
        assert_eq!(code(&out), 0, "{argv:?}: {}", stderr(&out));
        assert_eq!(stdout(&out), expected, "{argv:?} answered differently");
    }

    // And a global one, which is legal on every verb rather than declared per
    // verb -- the path through the parser is the same, the table is not.
    let out = r.ank("claude-code@ank", &["find", "criterion", "-j"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).trim_start().starts_with('{'),
        "-j is --json: {}",
        stdout(&out)
    );
}

/// Bundling is refused, and the refusal is the command to type instead.
#[test]
fn bundled_short_flags_are_refused_by_naming_the_flags_to_type_separately() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));

    let out = r.ank("claude-code@ank", &["find", "criterion", "-st", "task"]);
    let said = stderr(&out);
    assert_eq!(code(&out), 1, "{said}");
    assert!(said.contains("'-st' bundles"), "{said}");
    // The exact flags, each with its value placeholder: an error that only said
    // "no bundling" would leave the caller to work out what to write.
    assert!(
        said.contains("ank find -s <v> -t <v>"),
        "the hint is the command to run next: {said}"
    );

    // A letter that names nothing on this verb is reported as itself rather
    // than folded into the bundling message, which would advise a command that
    // would refuse too.
    let out = r.ank("claude-code@ank", &["find", "criterion", "-sz"]);
    let said = stderr(&out);
    assert_eq!(code(&out), 1, "{said}");
    assert!(said.contains("unknown flag '-z'"), "{said}");
}

/// A letter that is a real flag on another verb is a different mistake from a
/// letter that is nothing, and the error says which.
#[test]
fn a_short_flag_the_verb_does_not_take_names_the_flag_it_stands_for() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));

    let out = r.ank("claude-code@ank", &["claim", ID, "-s", "open"]);
    let said = stderr(&out);
    assert_eq!(code(&out), 1, "{said}");
    assert!(
        said.contains("'-s' is --status") && said.contains("'claim' does not take"),
        "{said}"
    );
    assert!(
        r.task_text(ID).contains("status: open"),
        "a refused parse claims nothing"
    );

    let out = r.ank("claude-code@ank", &["claim", ID, "-z"]);
    let said = stderr(&out);
    assert_eq!(code(&out), 1, "{said}");
    assert!(said.contains("unknown flag '-z'"), "{said}");
}

/// `ank help <verb>` shows both forms; `ank help` shows what it always did.
///
/// The second half is the one worth a test. The listing carries one line per
/// verb, and a second spelling of every flag is exactly the kind of addition
/// that arrives looking like an improvement. Grouping it (ADR-f61e2d2c75e8)
/// changed what stands between the lines and nothing on them.
#[test]
fn help_shows_both_forms_for_one_verb_and_leaves_the_listing_lines_alone() {
    let r = Repo::new();

    let one = stdout(&r.ank("claude-code@ank", &["help", "find"]));
    assert!(one.contains("-s, --status <v>"), "{one}");
    assert!(one.contains("-t, --type <v>"), "{one}");
    assert!(one.contains("-j, --json"), "the globals too: {one}");
    // `--scope` has no letter: `s` went to `--status`, and §4 says so rather
    // than leaving a reader to wonder whether it was forgotten.
    assert!(
        one.contains("--scope <v>") && !one.contains(", --scope"),
        "a flag with no short form shows none: {one}"
    );

    let all = stdout(&r.ank("claude-code@ank", &["help"]));
    assert!(
        all.contains("--status"),
        "the listing still names flags: {all}"
    );
    assert!(
        !all.contains("-s, ") && !all.contains("-j, "),
        "the listing's lines are unchanged: {all}"
    );

    // A script reads the mapping from --json, or it cannot use it at all.
    let j = stdout(&r.ank("claude-code@ank", &["help", "find", "--json"]));
    assert!(j.contains("\"name\":\"--status\",\"short\":\"-s\""), "{j}");
    assert!(j.contains("\"name\":\"--scope\",\"short\":null"), "{j}");
}

/// A single dash is a flag now, so the escape had to become reachable.
///
/// `ank log "-1 rebuilt"` used to be a message and is now an argument starting
/// with a dash. It is refused for what it is rather than reported as a bundle
/// of unknown letters, and `--` still carries it through.
#[test]
fn a_positional_that_starts_with_a_dash_is_refused_by_naming_the_escape() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);

    let message = "-1 rebuilt the index";
    let out = r.ank("claude-code@ank", &["log", message]);
    let said = stderr(&out);
    assert_eq!(code(&out), 1, "{said}");
    assert!(
        said.contains("is not a flag: it contains a space"),
        "{said}"
    );
    assert!(
        said.contains(&format!("ank log -- \"{message}\"")),
        "the hint is the exact command that works: {said}"
    );

    // And it does work. Invoked directly rather than through the harness,
    // because `--` terminates everything after it and the harness appends
    // `--repo` last -- which is the terminator behaving exactly as specified,
    // and worth seeing once from the caller's side.
    let out = ank_command()
        .args(["log", "--repo"])
        .arg(&r.0)
        .args(["--", message])
        .env("ANK_AGENT", "claude-code@ank")
        .current_dir(std::env::temp_dir())
        .output()
        .expect("the binary must have been built");
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(r.log_text(ID).contains(message), "{}", r.log_text(ID));

    // A value keeps taking its dash verbatim, whichever form the flag took:
    // only a positional ever needs the escape.
    let out = r.ank("claude-code@ank", &["find", "criterion", "-s=open"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
}

/// Verbs are never abbreviated, and the short forms did not open a door to it.
#[test]
fn a_verb_is_never_abbreviated() {
    let r = Repo::new();
    let out = r.ank("claude-code@ank", &["cl", ID]);
    let said = stderr(&out);
    assert_eq!(code(&out), 1, "{said}");
    assert!(said.contains("unknown command 'cl'"), "{said}");
}

/// A second claim under one identity is refused (TASK-a548c95261a5).
///
/// Observed while dogfooding: a task claimed in one terminal follows you into a
/// second one, because `ANK_AGENT` unset makes both `<user>@<hostname>`. The
/// identity is not bound to the session on purpose — a PID or a TTY in it would
/// break resuming a claim after a restart.
///
/// TASK-d79dc424c63d answered that with a warning naming the way out, and stays
/// `done` with its proof intact (§3). The warning was measured to be not
/// enough: it is printed once, at acquisition, and the claims do not
/// collide — they accumulate, until `log`, `release` and `done` pick the lowest
/// task id of the two in silence (TASK-97d8747416ea). §4 had said `claim`
/// *enforces* one live claim per agent for as long as this only warned.
///
/// Through the binary, because the environment variable under test is read by
/// the process and not by the function, and because a refusal that only exists
/// in a function is not a refusal.
#[test]
fn a_second_claim_under_one_identity_is_refused_and_names_both_ways_out() {
    let first = "TASK-000000000d01";
    let second = "TASK-000000000d02";
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task(first, Some("A criterion."));
    r.seed_task(second, Some("A criterion."));

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", first])), 0);

    let out = r.ank("claude-code@ank", &["claim", second]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(
        code(&out),
        7,
        "the prerequisite is missing, not the task: {said}"
    );
    assert!(
        said.contains(first),
        "the claim already held is named: {said}"
    );
    assert!(
        said.contains("ank release"),
        "the first way out is a command: {said}"
    );
    assert!(
        said.contains("ANK_AGENT"),
        "and the second is the one that fits a session that claimed nothing: {said}"
    );

    // Refused before anything was written. A refusal that leaves a ref behind
    // is a claim with a bad conscience.
    assert!(
        r.claim_ref(second).is_none(),
        "the refused task carries a claim ref"
    );
    assert!(
        r.task_text(second).contains("status: open"),
        "the refused task was moved anyway:\n{}",
        r.task_text(second)
    );

    // The other identity is the supported case and must pass: parallel agents,
    // one ref per task, is the design and not the anomaly.
    let third = "TASK-000000000d03";
    r.seed_task(third, Some("A criterion."));
    let out = r.ank("codex@ank", &["claim", third]);
    assert_eq!(
        code(&out),
        0,
        "a distinct identity holding its own task is not an anomaly: {}",
        stderr(&out)
    );

    // A lapsed claim is not a live one, so pickup after expiry (§3) passes
    // through untouched — the same task, refused a moment ago, now claimable.
    r.expire_claim(first);
    let out = r.ank("claude-code@ank", &["claim", second]);
    assert_eq!(
        code(&out),
        0,
        "an expired claim refuses the next one: {}",
        stderr(&out)
    );
}

/// The way out sends the reader to `getting-started`, so `getting-started` has
/// to be where the answer is (TASK-d79dc424c63d, TASK-a548c95261a5).
///
/// Driven by the binary rather than by a hand-copied string: the point is not
/// that the guide mentions a variable, it is that the exact line the binary
/// prints has somewhere to land. Naming a fix nobody wrote down is the defect
/// this task is about, one level up.
///
/// Read off the refusal now that the second claim is one, and off standard
/// error with it. What is under test is the sentence, not which stream carried
/// it — `way_out` is written once and both callers read it.
#[test]
fn the_guide_documents_the_identity_the_way_out_tells_you_to_set() {
    let first = "TASK-000000000d11";
    let second = "TASK-000000000d12";
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task(first, Some("A criterion."));
    r.seed_task(second, Some("A criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", first])), 0);

    let out = r.ank("claude-code@ank", &["claim", second]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    let warned: Vec<&str> = said.lines().filter(|l| l.contains("ANK_AGENT")).collect();
    assert!(!warned.is_empty(), "nothing to document: {said}");

    let guide = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/getting-started.md"),
    )
    .expect("the guide is in the repository the tests run from");

    // The variable the warning names, and the shape of an invocation that sets
    // it: naming it without showing how to use it is half an answer.
    // Trimmed of what punctuates the sentence rather than names the variable:
    // the way out reaches the reader inside a hint's parenthetical, and
    // `ANK_AGENT)` is the same variable as `ANK_AGENT`.
    let named: Vec<&str> = warned
        .iter()
        .flat_map(|l| l.split_whitespace())
        .filter(|w| w.contains("ANK_AGENT"))
        .map(|w| w.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '_'))
        .collect();
    assert!(!named.is_empty(), "the warning names no way out: {said}");
    for w in named {
        assert!(
            guide.contains(w),
            "the binary tells the reader about {w} and the guide never mentions it"
        );
    }
    assert!(
        guide.contains("ANK_AGENT=") && guide.contains("ank claim"),
        "the guide names the variable without showing an invocation that sets it"
    );
    assert!(
        guide.contains("concurrent session"),
        "the guide never says the fix is per session, which is the whole point"
    );
}

/// A blocker finished on another branch says which one (TASK-b5ad06f134f6).
///
/// Through the binary, because the defect is precisely a working tree being
/// asked a question only the refs can answer: `check_blockers` built its status
/// map from the files this branch carries, and on `main` the blocker's file
/// still reads `open`. Nothing below is observable from the function — it needs
/// a real repository holding a real completion ref for a commit `main` does not
/// have.
#[test]
fn a_blocker_finished_on_another_branch_is_named_with_its_branch_and_commit() {
    let blocker = "TASK-000000000c01";
    let dependent = "TASK-000000000c02";
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task_with(blocker, Some("A criterion."), &["ok"]);
    r.seed_task(dependent, Some("A criterion."));
    r.blocked(dependent, &[blocker]);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed tasks"]);

    // A commit of its own on the branch, so the commit the completion record
    // names is one `main` genuinely does not carry. Branching alone would leave
    // HEAD on a commit both branches share, and the assertion below would pass
    // on a repository where nothing was unmerged at all.
    r.git(&["checkout", "-q", "-b", "feature"]);
    std::fs::write(r.0.join("work.txt"), "y").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "work"]);
    let finished_at = r.head();

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", blocker])), 0);
    let out = r.ank("claude-code@ank", &["done"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "done"]);

    // Back where the work has not landed: the blocker's file says `open` again,
    // and the ref is the only thing that remembers otherwise.
    r.git(&["checkout", "-q", "main"]);
    assert!(
        r.task_text(blocker).contains("status: open"),
        "the fixture is wrong if main already carries the done"
    );

    let out = r.ank("claude-code@ank", &["claim", dependent]);
    let said = stderr(&out);
    assert_eq!(code(&out), 7, "the refusal itself does not move: {said}");
    assert!(
        said.contains(&format!("blocked by {blocker}")),
        "the blocker is still named: {said}"
    );
    assert!(
        said.contains("finished on feature"),
        "the branch is the half an agent cannot guess: {said}"
    );
    assert!(
        said.contains(&finished_at[..7]),
        "the commit is what makes the branch checkable: {said}"
    );
    assert!(
        said.contains("not merged here yet"),
        "and why it still blocks: {said}"
    );
    // The hint is never a command that refuses: the blocker carries the
    // completion ref, so claiming it is exactly what fails.
    assert!(
        !said.contains(&format!("ank claim {blocker}")),
        "offered a claim that would refuse on the spot: {said}"
    );
    assert!(
        r.task_text(dependent).contains("status: open"),
        "a refused claim moves nothing"
    );
}

#[test]
fn a_failing_done_through_the_binary_leaves_the_claim_intact() {
    let r = Repo::new().with_verifiers("verifiers:\n  nope:\n    run: exit 2\n");
    r.seed_task_with(ID, Some("A criterion."), &["nope"]);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);

    let out = r.ank("claude-code@ank", &["done"]);
    assert_eq!(
        code(&out),
        5,
        "a verifier that ran and refused: {}",
        stderr(&out)
    );

    let record = r.claim_ref(ID).expect("the ref is still there");
    assert!(record.contains("state: claim"), "still a claim: {record}");
    assert!(record.contains("claude-code@ank"), "{record}");
    assert!(
        r.task_text(ID).contains("status: in_progress"),
        "the task did not move"
    );
}

#[test]
fn done_refuses_proof_when_the_task_declares_verifiers() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task_with(ID, Some("A criterion."), &["ok"]);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);

    let out = r.ank(
        "claude-code@ank",
        &["done", "--proof", "assertion:trust me"],
    );
    assert_eq!(code(&out), 5, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("--proof is refused"),
        "{}",
        stderr(&out)
    );
    assert!(r.task_text(ID).contains("status: in_progress"));
}

/// The other branch of the same refusal: a task declaring no verifier demands a
/// `--proof`, and the hint is what the reader will type next.
///
/// It must name `commit:<sha>`, a proof the caller already holds. It named
/// `test:<ci-run-ref>` — push, wait for a pipeline, copy a run id back — which
/// is the workflow TASK-2dff950e5d51 replaced with a pipeline that attests the
/// run itself, and the rhythm an agent reconstructs from habit whenever the tool
/// suggests it. Through the binary, because the defect was never in the parser:
/// it was in the sentence a caller reads and obeys.
#[test]
fn done_with_no_verifier_asks_for_a_proof_the_caller_already_holds() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);

    let out = r.ank("claude-code@ank", &["done"]);
    let said = stderr(&out);
    assert_eq!(code(&out), 5, "{said}");
    assert!(
        said.contains("ank done --proof commit:<sha>"),
        "the hint must name a proof already in hand: {said}"
    );
    assert!(
        !said.contains("ci-run-ref"),
        "still sending the caller to wait for a run id: {said}"
    );
    assert!(r.task_text(ID).contains("status: in_progress"));
}

#[test]
fn the_whole_agent_loop_runs_through_the_binary() {
    // context -> new -> claim -> log -> release -> claim -> done, as an agent
    // actually types it. Each verb was tested in its module; what only exists
    // once the process does is that they compose.
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");

    let out = r.ank(
        "claude-code@ank",
        &["new", "task", "--title", "Rotate secrets"],
    );
    assert_eq!(code(&out), 7, "a scope is required: {}", stderr(&out));

    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--title",
            "Rotate secrets",
            "--scope",
            "src/**",
            "--criteria",
            "The rotation runs.",
        ],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let created = String::from_utf8_lossy(&out.stdout).to_string();
    let id = created
        .split_whitespace()
        .nth(1)
        .expect("created <id> <title>")
        .to_string();

    assert!(
        String::from_utf8_lossy(&r.ank("claude-code@ank", &["find", "rotate"]).stdout)
            .contains("Rotate secrets")
    );

    // log without a claim is refused, with it goes through and renews.
    assert_eq!(code(&r.ank("claude-code@ank", &["log", "early"])), 6);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", &id])), 0);
    assert_eq!(
        code(&r.ank("claude-code@ank", &["log", "made progress"])),
        0
    );

    // release refuses without a reason, and deletes the ref with one.
    let out = r.ank("claude-code@ank", &["release"]);
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(stderr(&out).contains("--reason"), "{}", stderr(&out));
    assert!(r.claim_ref(&id).is_some(), "a refusal deletes nothing");

    let out = r.ank(
        "claude-code@ank",
        &["release", "--reason", "needs staging access"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        r.claim_ref(&id).is_none(),
        "release deletes the ref, read back with git"
    );
    let file = r.task_text(&id);
    assert!(file.contains("status: open"), "{file}");
    assert!(
        r.log_text(&id).contains("needs staging access"),
        "the reason is in the log: {}",
        r.log_text(&id)
    );

    // And the next agent picks it up where the previous one stopped.
    assert_eq!(code(&r.ank("codex@host-9", &["claim", &id])), 0);
    let ctx = String::from_utf8_lossy(&r.ank("codex@host-9", &["context"]).stdout).to_string();
    assert!(ctx.contains("made progress"), "the log travels: {ctx}");
    assert!(ctx.contains("needs staging access"), "{ctx}");
}

/// `git log` reads, and now so does `ank log` given nothing but an id (§4).
///
/// Through the binary because the criterion says so, and because what has to be
/// true is a property of the process: a read that took the claim path would
/// still pass a module test run under the holder's identity, and fail for every
/// caller who is not it.
#[test]
fn log_with_an_id_and_no_message_reads_and_asks_for_no_claim() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    // An empty log is an answer, not a blank. Read here by an agent that holds
    // nothing, before any claim exists at all.
    let out = r.ank("marie@laptop", &["log", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("no log entry yet"),
        "{}",
        stdout(&out)
    );

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    assert_eq!(code(&r.ank("claude-code@ank", &["log", "first thing"])), 0);
    assert_eq!(code(&r.ank("claude-code@ank", &["log", "second thing"])), 0);

    // Still `marie@laptop`, who holds no claim on anything: the claim is what
    // writing needs, never reading.
    let before = r.task_text(ID);
    let out = r.ank("marie@laptop", &["log", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    let newest = text
        .find("second thing")
        .unwrap_or_else(|| panic!("the newer entry is missing:\n{text}"));
    let oldest = text
        .find("first thing")
        .unwrap_or_else(|| panic!("the older entry is missing:\n{text}"));
    assert!(newest < oldest, "newest first (§4):\n{text}");
    assert_eq!(
        r.task_text(ID),
        before,
        "a read that writes is not a read: {text}"
    );
    assert!(
        r.claim_ref(ID).is_some(),
        "reading neither takes nor drops the holder's claim"
    );

    // Scriptable like every other verb, and the same order.
    let out = r.ank("marie@laptop", &["log", ID, "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let j = stdout(&out);
    assert!(
        j.starts_with(&format!("{{\"task\":\"{ID}\",\"entries\":[")),
        "{j}"
    );
    assert!(
        j.find("second thing").unwrap() < j.find("first thing").unwrap(),
        "{j}"
    );

    // Only a task has a log, and the refusal names the verb that does answer.
    const ADR: &str = "ADR-0000000000ab";
    r.seed_adr(ADR, "Do not do X.", "src/**");
    let out = r.ank("marie@laptop", &["log", ADR]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(
        stderr(&out).contains(&format!("ank show {ADR}")),
        "{}",
        stderr(&out)
    );
}

/// The disambiguation of §4, exercised on all three of its branches. It is
/// stated rather than inferred precisely so that it can be asserted this way:
/// one question — does the argument resolve — and one answer.
#[test]
fn log_decides_between_reading_and_writing_by_what_resolves() {
    const OTHER: &str = "TASK-000000000002";
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.seed_task(OTHER, Some("Another verifiable criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);

    // Shaped like an id, resolving to nothing: a message.
    let out = r.ank("claude-code@ank", &["log", "TASK-000000000009"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        r.log_text(ID).contains("TASK-000000000009"),
        "{}",
        r.log_text(ID)
    );

    // An ambiguous prefix resolves to no single entity, so it is a message too.
    // "It resolved" is the whole test, and a second question — did it nearly
    // resolve — would be one an agent has to guess the answer to.
    let out = r.ank("claude-code@ank", &["log", "TASK-00000000000"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        r.log_text(ID)
            .lines()
            .any(|l| l.ends_with("TASK-00000000000")),
        "{}",
        r.log_text(ID)
    );

    // A message that also resolves: refused, naming both readings, writing
    // neither.
    let before = r.task_text(ID);
    let out = r.ank("claude-code@ank", &["log", ID, OTHER]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let e = stderr(&out);
    assert!(e.contains(OTHER), "the read it could have been: {e}");
    assert!(e.contains(ID), "the write it could have been: {e}");
    assert_eq!(r.task_text(ID), before, "a refusal writes nothing");

    // The redundant write form is untouched when the message is a message.
    let out = r.ank("claude-code@ank", &["log", ID, "a real message"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(r.log_text(ID).contains("a real message"));

    // And an id alone reads even for the agent holding it: what decides is the
    // argument, not who is asking.
    let out = r.ank("claude-code@ank", &["log", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("a real message"), "{}", stdout(&out));
    assert_eq!(
        stdout(&out).matches("a real message").count(),
        1,
        "reading appended an entry: {}",
        stdout(&out)
    );
}

#[test]
fn init_runs_where_there_is_no_ank_directory_yet() {
    // The one verb that precedes the foundation. Kept here because the reason
    // it bypasses `startup` is only observable from outside the process.
    let dir = std::env::temp_dir().join(format!("ank-cli-init-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    assert!(git_command(&dir)
        .args(["init", "-q"])
        .status()
        .unwrap()
        .success());

    let out = ank_command()
        .arg("init")
        .arg(&dir)
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(Path::new(&dir).join(".ank/config.yml").exists());
    let _ = std::fs::remove_dir_all(&dir);
}

/// `init` refuses `--repo` by name, and writes nowhere while doing it.
///
/// The shape of the fixture is the defect: the process runs **inside** one
/// repository with `--repo` naming another. Accepting the flag, `init`
/// initialised the repository it was standing in and left the named one empty —
/// the pointer paragraph appended to an `AGENTS.md` nobody was editing, and
/// `pointer added to AGENTS.md` printed as if it had worked
/// (TASK-b8a12d60686d). A test that ran from a neutral directory would pass
/// against that behaviour.
#[test]
fn init_refuses_repo_and_writes_into_neither_repository() {
    let inside = Repo::new();
    let named = std::env::temp_dir().join(format!("ank-cli-init-named-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&named);
    std::fs::create_dir_all(&named).unwrap();
    assert!(git_command(&named)
        .args(["init", "-q", "-b", "main"])
        .status()
        .unwrap()
        .success());

    // A file worth losing: `init` places its pointer in AGENTS.md, and the
    // silent write landed in exactly this file.
    std::fs::write(inside.0.join("AGENTS.md"), "Existing guidance.\n").unwrap();

    let out = ank_command()
        .arg("init")
        .arg("--repo")
        .arg(&named)
        .current_dir(&inside.0)
        .output()
        .expect("the binary must have been built");

    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(code(&out), 1, "{said}");
    assert!(said.contains("--repo"), "the flag is not named: {said}");
    assert!(
        said.contains("ank init") && said.contains(named.to_str().unwrap()),
        "the refusal must name the command to run next: {said}"
    );

    // Neither repository moved. The one it was standing in first, since that is
    // where the writes actually landed.
    assert_eq!(
        std::fs::read_to_string(inside.0.join("AGENTS.md")).unwrap(),
        "Existing guidance.\n",
        "init wrote into the repository it was merely standing in"
    );
    assert!(
        !inside.0.join(".gitattributes").exists() && !inside.0.join(".gitignore").exists(),
        "init wrote git files into the repository it was merely standing in"
    );
    let fetch = git_command(&inside.0)
        .args(["config", "--get-all", "remote.origin.fetch"])
        .output()
        .unwrap();
    assert!(
        !stdout(&fetch).contains("refs/ank/*"),
        "init added a refspec to the repository it was merely standing in"
    );

    // And the named one was not initialised behind the refusal either: a
    // refusal that half-acted would be worse than the acceptance it replaced.
    assert!(
        !named.join(".ank").exists(),
        "the refused init created the named repository anyway"
    );

    // The positional is the way, and it still works from inside another
    // repository -- the refusal is about the flag, not about the situation.
    let out = ank_command()
        .arg("init")
        .arg(&named)
        .current_dir(&inside.0)
        .output()
        .expect("the binary must have been built");
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(named.join(".ank/config.yml").exists());
    assert!(
        !inside.0.join(".gitattributes").exists(),
        "the positional form wrote into the wrong repository too"
    );

    let _ = std::fs::remove_dir_all(&named);
}

/// A second live claim on one identity stays visible after the moment it was
/// taken.
///
/// `claim` names it once, at acquisition, and never again. A convention that
/// announces itself only when it is taken fades exactly as a session lengthens,
/// and `status` is the verb a session runs when it has lost track — which is
/// how two sessions in one tree, both with `ANK_AGENT` unset, read as one agent
/// and produced a release taken on a misread (TASK-38b384543551).
///
/// Asserted on what reaches stdout, because a warning that never reaches stdout
/// is not a warning.
#[test]
fn status_names_every_live_claim_of_this_identity() {
    let r = Repo::new();
    const SECOND: &str = "TASK-000000000002";
    r.seed_task(ID, Some("A verifiable criterion."));
    r.seed_task(SECOND, Some("Another verifiable criterion."));

    // One identity, two live claims. `claim` refuses to produce that state now
    // (TASK-a548c95261a5), and the state still exists — ank is not a gatekeeper
    // (ADR-6b3fa9ba3a05), and a lapse revived is one of the ways in. Both
    // claims are taken by the binary; only the clock between them is forged.
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    r.expire_claim(ID);
    let out = r.ank("claude-code@ank", &["claim", SECOND]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    r.revive_claim(ID);

    // The moment passes; the state does not.
    let said = stdout(&r.ank("claude-code@ank", &["status"]));
    let other = said
        .lines()
        .filter(|l| l.contains(ID) || l.contains(SECOND))
        .count();
    assert!(
        other >= 2,
        "status names one claim and hides the other:\n{said}"
    );
    // The way out's own sentence and not the variable it names: `status` prints
    // the identity in effect on every run (TASK-6f4ff66894d2), so `ANK_AGENT`
    // alone stopped discriminating between an output that carries this advice
    // and one that does not.
    assert!(
        said.contains("sets its own ANK_AGENT"),
        "status must repeat the way out claim already names:\n{said}"
    );

    // The same information for a caller that scripts around it -- which is
    // exactly the caller running several sessions.
    let json = stdout(&r.ank("claude-code@ank", &["status", "--json"]));
    assert!(json.contains("\"also_held\""), "{json}");
    assert!(
        json.contains(SECOND) || json.contains(ID),
        "the second claim is absent from --json:\n{json}"
    );

    // And one identity holding one claim says none of it: the line exists to
    // report a state, not to decorate every status.
    let quiet = Repo::new();
    quiet.seed_task(ID, Some("A verifiable criterion."));
    assert_eq!(code(&quiet.ank("claude-code@ank", &["claim", ID])), 0);
    let said = stdout(&quiet.ank("claude-code@ank", &["status"]));
    assert!(!said.contains("sets its own ANK_AGENT"), "{said}");
    assert!(!said.contains("also"), "{said}");
}

/// `close` leaves nothing on the coordination plane where `done` leaves a
/// completion record, and that asymmetry is the decision (ADR-6d8736c04cfa).
///
/// The ADR it supersedes created the completion ref so that a task finished on
/// an unmerged branch would not look free everywhere else, and named this gap
/// without settling it -- on the ground that `close` is "a human act and a rare
/// one", which ADR-e17e1bbd93ff retired by dissolving the human side entirely.
/// That is history, and `.ank/` carries it (TASK-78326e2e3e89). The code has
/// behaved this way throughout; what it lacked was a decision saying so and a
/// test holding it.
///
/// Read **from a second checkout**, because that is where the question arises:
/// `refs/ank/` is shared by every worktree, and an agent on another branch is
/// exactly the reader the completion ref exists for. Both halves are asserted
/// in one test, since either alone would pass for the wrong reason -- an
/// absence proves nothing without the presence beside it.
#[test]
fn close_leaves_no_completion_ref_where_done_leaves_one() {
    const CLOSED: &str = "TASK-000000000b01";
    const FINISHED: &str = "TASK-000000000b02";
    const AGENT: &str = "claude-code@ank";

    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: git --version\n");
    r.seed_task_with(CLOSED, Some("A verifiable criterion."), &["ok"]);
    r.seed_task_with(FINISHED, Some("A verifiable criterion."), &["ok"]);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "the two tasks"]);

    // Not the default branch. The whole question is what another checkout can
    // see before the merge, so a test run on `main` would answer nothing.
    r.git(&["checkout", "-q", "-b", "feature"]);
    let other = r.0.with_extension("second-checkout");
    r.git(&[
        "worktree",
        "add",
        "--detach",
        other.to_str().unwrap(),
        "HEAD",
    ]);

    // close: the ref exists, and then it does not.
    assert_eq!(code(&r.ank(AGENT, &["claim", CLOSED])), 0);
    assert!(
        r.claim_ref_at(&other, CLOSED).is_some(),
        "the claim has to be visible from the other checkout to be worth losing"
    );
    let out = r.ank(
        AGENT,
        &["close", CLOSED, "--reason", "superseded by the pipeline"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        r.claim_ref_at(&other, CLOSED).is_none(),
        "close left a record on the plane:\n{:?}",
        r.claim_ref_at(&other, CLOSED)
    );

    // done, on the same branch and read from the same checkout: a completion
    // record, naming the commit and the branch it was finished on.
    assert_eq!(code(&r.ank(AGENT, &["claim", FINISHED])), 0);
    let out = r.ank(AGENT, &["done"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let record = r
        .claim_ref_at(&other, FINISHED)
        .expect("done leaves a completion record where close leaves nothing");
    assert!(
        record.contains("commit:") && record.contains("feature"),
        "the completion record names neither the commit nor the branch:\n{record}"
    );

    // And the price the decision accepts, stated rather than implied: the task
    // closed on this branch is claimable from a checkout the closure has not
    // reached. It is not a defect here, it is the thing being agreed to.
    let out = r.ank_at("codex@host-9", &["claim", CLOSED], &other);
    assert_eq!(
        code(&out),
        0,
        "a task closed on an unmerged branch is claimable elsewhere, which is \
         what the decision accepts:\n{}",
        stderr(&out)
    );

    r.git(&["worktree", "remove", "--force", other.to_str().unwrap()]);
}

/// Two clones of one repository arbitrate, which is the whole of level 1
/// (TASK-82c3341502c1).
///
/// Worktrees of one clone share `refs/ank/` and were always settled by the
/// local compare-and-swap. Clones do not share it: each held its own
/// `refs/ank/claims/<id>` and neither ever learned of the other, so both agents
/// claimed the same task, both succeeded, and nothing detected it — not then
/// and not later, since `check` prunes on the default branch, where two agents
/// having done the same work looks like two agents having done their work.
///
/// What settles it is the push, under `--force-with-lease` with the object the
/// caller read as the expectation: server-side, atomic, and on every host. The
/// test drives both clones through the binary because that is what the property
/// is about — a unit test over the push helper would prove the helper works and
/// leave two clones untested.
#[test]
fn two_clones_of_one_repository_arbitrate_over_a_claim() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: git --version\n");
    r.seed_task(ID, Some("A verifiable criterion."));
    let (_origin, other) = r.cloned();

    // The first clone takes it, and the claim reaches the remote.
    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        !format!("{}{}", stdout(&out), stderr(&out)).contains("not pushed"),
        "a reachable remote must not report an unsynchronised claim: {}",
        stderr(&out)
    );

    // The second clone, which has never seen that ref, is refused -- and told
    // by whom, which it can only know by having been given the winner's record.
    let out = r.ank_at("codex@host-9", &["claim", ID], &other);
    assert_eq!(
        code(&out),
        4,
        "the second clone took a task already held:\n{}{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("claude-code@ank"),
        "the refusal names no holder: {}",
        stderr(&out)
    );

    // And the loser leaves the durable state alone: a refused claim is not a
    // task moved to in_progress in a clone that does not hold it.
    let text =
        std::fs::read_to_string(other.join(".ank/entities").join(format!("{ID}.md"))).unwrap();
    assert!(
        text.contains("status: open"),
        "the refused clone moved the task anyway:\n{text}"
    );
}

/// A remote that exists and cannot be reached degrades, it does not fail
/// (TASK-82c3341502c1, §2).
///
/// The claim is taken locally and the risk is displayed rather than hidden --
/// what is at stake is that another clone can take the same task, so that is
/// what the warning says. Measured against a remote that is configured and
/// gone, which is the shape a laptop off the network actually has: a URL that
/// resolves to nothing.
#[test]
fn an_unreachable_remote_warns_and_the_claim_still_holds_locally() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.git(&[
        "remote",
        "add",
        "origin",
        // A path that will never exist: git fails to connect rather than
        // refusing a swap, which is exactly the distinction under test.
        &r.0.with_extension("gone.git").to_string_lossy(),
    ]);

    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(
        code(&out),
        0,
        "an unreachable remote must not fail the claim:\n{}",
        stderr(&out)
    );
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert!(
        said.contains("not pushed") && said.contains("another clone"),
        "the risk is displayed, not hidden: {said}"
    );
    assert!(
        r.claim_ref(ID).is_some(),
        "the claim did not hold locally, which is the half that must not degrade"
    );

    // And a repository with no remote at all says none of it: that is level 0,
    // the default mode, and a warning there would fire on every solo claim.
    let solo = Repo::new();
    solo.seed_task(ID, Some("A verifiable criterion."));
    let out = solo.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        !format!("{}{}", stdout(&out), stderr(&out)).contains("not pushed"),
        "level 0 is not a degradation and must stay silent"
    );
}

/// What a holder of a claim sees of the coordination plane (TASK-dacbcae6134c).
///
/// The agent best placed to notice a collision is the one currently working,
/// and it is the one `context` stops showing the plane to: execution mode drops
/// every other task, so `[claimed:holder]` has no listing left to sit on. §5
/// now says that is deliberate -- execution mode exists to remove choice, and a
/// list of what other agents hold is choice-shaped -- and that the information
/// is relocated rather than withheld, to `status`, which is off the loop and
/// costs nothing to skip.
///
/// The four assertions are the specification's four halves: orientation shows
/// it, execution does not, `status` does, and `status` says so even when there
/// is nothing to say -- silence and "this verb does not answer that" read
/// identically otherwise.
#[test]
fn a_holder_reads_the_plane_through_status_and_execution_context_stays_silent() {
    const MINE: &str = "TASK-000000000f01";
    const THEIRS: &str = "TASK-000000000f02";
    let r = Repo::new();
    r.seed_task(MINE, Some("A verifiable criterion."));
    r.seed_task(THEIRS, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("codex@host-9", &["claim", THEIRS])), 0);

    // Orientation, before claiming anything: the marker is here, and this is
    // the moment it is worth reading -- the agent is choosing.
    let said = stdout(&r.ank("claude-code@ank", &["context"]));
    assert!(
        said.contains("codex@host-9"),
        "orientation hides who holds what, which is when it matters:\n{said}"
    );

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", MINE])), 0);

    // Execution mode: nothing of the plane, deliberately.
    let said = stdout(&r.ank("claude-code@ank", &["context"]));
    assert!(
        said.contains(MINE),
        "execution mode lost its own task:\n{said}"
    );
    assert!(
        !said.contains(THEIRS) && !said.contains("codex@host-9"),
        "execution mode offers a task it exists to keep out of view:\n{said}"
    );

    // `status` is where the question was relocated to.
    let said = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(
        said.contains(THEIRS) && said.contains("codex@host-9"),
        "status names no claim but the caller's own:\n{said}"
    );

    let json = stdout(&r.ank("claude-code@ank", &["status", "--json"]));
    assert!(json.contains("\"elsewhere\""), "{json}");
    assert!(json.contains("codex@host-9"), "{json}");

    // And with nobody else on anything, it answers rather than going quiet.
    let solo = Repo::new();
    solo.seed_task(MINE, Some("A verifiable criterion."));
    assert_eq!(code(&solo.ank("claude-code@ank", &["claim", MINE])), 0);
    let said = stdout(&solo.ank("claude-code@ank", &["status"]));
    assert!(
        said.contains("no claim by another agent"),
        "an empty plane and an unanswered question read alike:\n{said}"
    );
}

/// An `elsewhere` line names the task, not only its id (TASK-028bcee93801).
///
/// The rows are already loaded where the line is built, so the join costs
/// nothing; without it a reader holds an id and has to run `show` once per claim
/// to learn what anybody is doing -- which is the question the section was
/// relocated here to answer.
///
/// Two tasks with two titles, because a fixture where every task is `Example
/// task` cannot tell a title from a coincidence.
#[test]
fn a_claim_held_elsewhere_is_named_with_the_title_of_its_task() {
    const MINE: &str = "TASK-000000000f21";
    const THEIRS: &str = "TASK-000000000f22";
    let r = Repo::new();
    r.seed_task_titled(MINE, "What this session is doing");
    r.seed_task_titled(THEIRS, "What the other session is doing");
    assert_eq!(code(&r.ank("codex@host-9", &["claim", THEIRS])), 0);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", MINE])), 0);

    let said = stdout(&r.ank("claude-code@ank", &["status"]));
    let line = said
        .lines()
        .find(|l| l.contains(THEIRS))
        .unwrap_or_else(|| panic!("no line for the claim held elsewhere:\n{said}"));
    assert!(
        line.contains("What the other session is doing"),
        "the line carries an id and nothing a reader can act on: {line}"
    );
    // After the id, which is what the criterion says and what a reader scans
    // by: the id is the handle, the title is what it means.
    assert!(
        line.find(THEIRS) < line.find("What the other session"),
        "the title runs ahead of the id it belongs to: {line}"
    );
    assert!(
        line.contains("codex@host-9"),
        "the holder was dropped for the title: {line}"
    );

    // Beside the three fields the entry already had, never instead of one: a
    // script that reads this surface is the caller most likely to be running
    // several agents.
    let json = stdout(&r.ank("claude-code@ank", &["status", "--json"]));
    assert!(
        json.contains(&format!(
            "{{\"id\":\"{THEIRS}\",\"title\":\"What the other session is doing\",\
             \"holder\":\"codex@host-9\","
        )),
        "{json}"
    );
    assert!(json.contains("\"expires\":\""), "{json}");
}

/// `status --remote` reads the claim refs origin holds, and says which of them
/// are only there (TASK-028bcee93801, ADR-47e2ac102f58).
///
/// Against a bare origin holding a claim this clone has never fetched, which is
/// the only shape that separates the two planes: a worktree shares `refs/ank/`
/// and a fetched clone has the record, so neither could tell a remote read from
/// a local one.
///
/// **The default is asserted as an absence.** Without the flag the claim the
/// other clone pushed is invisible here -- not because a call failed, but
/// because no call was made. A `status` that consulted origin unasked would find
/// that ref and print it, so the silence is the measurement rather than an
/// assumption about it.
#[test]
fn status_remote_names_the_claims_origin_holds_and_which_are_only_there() {
    const MINE: &str = "TASK-000000000f31";
    const THEIRS: &str = "TASK-000000000f32";
    let r = Repo::new();
    r.seed_task_titled(MINE, "Held in this clone");
    r.seed_task_titled(THEIRS, "Held in the other clone");
    let (_origin, other) = r.cloned();

    // Taken in the other clone, so the ref reaches origin and never this
    // checkout: `cloned` wires no `refs/ank/*` refspec, exactly as a clone made
    // by hand has none.
    let out = r.ank_at("codex@host-9", &["claim", THEIRS], &other);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));

    let said = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(
        said.contains("elsewhere no claim by another agent"),
        "status answered about a plane it was not asked to read:\n{said}"
    );
    assert!(
        !said.contains(THEIRS),
        "status paid for the network with no flag asking it to:\n{said}"
    );

    // With the flag, and marked: a claim seen only on origin and a claim seen
    // here are different facts, and a flag that made the output more confident
    // without making it more informative would be worse than none.
    let out = r.ank("claude-code@ank", &["status", "--remote"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let said = stdout(&out);
    let line = said
        .lines()
        .find(|l| l.contains(THEIRS))
        .unwrap_or_else(|| panic!("--remote read no claim off origin:\n{said}"));
    assert!(
        line.contains("Held in the other clone"),
        "a remote claim loses the title a local one carries: {line}"
    );
    assert!(
        line.contains("on origin only"),
        "nothing says the record is not in this clone: {line}"
    );
    assert!(
        said.contains("1 on origin only"),
        "the count says nothing about the plane it mixed in:\n{said}"
    );
    // The record is on origin and the objects are not here, so no holder can be
    // read without the fetch a reader must not perform. Saying one would be an
    // invention.
    assert!(
        !line.contains("codex@host-9"),
        "a holder was printed for a record this clone cannot read: {line}"
    );
    assert!(
        said.contains("git fetch origin"),
        "the way to read the record is not named:\n{said}"
    );

    // And a claim held here as well as there is not marked as either.
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", MINE])), 0);
    let said = stdout(&r.ank("someone-else@host-2", &["status", "--remote"]));
    let line = said
        .lines()
        .find(|l| l.contains(MINE))
        .unwrap_or_else(|| panic!("the local claim vanished under --remote:\n{said}"));
    assert!(
        line.contains("Held in this clone") && line.contains("claude-code@ank"),
        "a claim on both planes lost what the local one said: {line}"
    );
    assert!(
        !line.contains("on origin only"),
        "a claim this clone holds is reported as somebody else's: {line}"
    );

    let json = stdout(&r.ank("claude-code@ank", &["status", "--remote", "--json"]));
    assert!(
        json.contains("\"remote\":true"),
        "nothing says the remote plane was read, and --json has no other channel \
         for it: {json}"
    );
    assert!(
        json.contains(&format!(
            "{{\"id\":\"{THEIRS}\",\"title\":\"Held in the other clone\",\
             \"holder\":null,\"expires\":null,\"seen\":\"origin\"}}"
        )),
        "{json}"
    );
}

/// The flag degrades where there is no remote to read, and never fails
/// (TASK-028bcee93801, §2).
///
/// Both reasons, because the way out of each is a different command: a
/// repository with no `origin` is level 0 and nominal, and an `origin` that
/// cannot be reached is a laptop off the network. Both answer on the local plane
/// with code 0 -- `status` is what an agent runs when it does not know where it
/// is, which is exactly when a refusal is least useful.
#[test]
fn status_remote_warns_once_and_answers_locally_with_no_remote() {
    const THEIRS: &str = "TASK-000000000f41";
    let r = Repo::new();
    r.seed_task_titled(THEIRS, "Held by the other agent");
    assert_eq!(code(&r.ank("codex@host-9", &["claim", THEIRS])), 0);

    // No remote at all.
    let out = r.ank("claude-code@ank", &["status", "--remote"]);
    assert_eq!(
        code(&out),
        0,
        "a reader failed for want of a remote:\n{}{}",
        stdout(&out),
        stderr(&out)
    );
    let said = stdout(&out);
    assert_eq!(
        said.matches("warning: no remote named origin").count(),
        1,
        "the degradation is said once, or not at all:\n{said}"
    );
    assert!(
        said.contains("git remote add origin"),
        "a warning with no command to run next:\n{said}"
    );
    assert!(
        said.contains(THEIRS) && said.contains("Held by the other agent"),
        "the local plane was withheld because the remote one was missing:\n{said}"
    );
    assert!(
        stdout(&r.ank("claude-code@ank", &["status", "--remote", "--json"]))
            .contains("\"remote\":false"),
        "--json claimed a plane it never read"
    );

    // A remote that is configured and gone, which is the same shape the claim
    // path already degrades against.
    r.git(&[
        "remote",
        "add",
        "origin",
        &r.0.with_extension("gone.git").to_string_lossy(),
    ]);
    let out = r.ank("claude-code@ank", &["status", "--remote"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let said = stdout(&out);
    assert_eq!(
        said.matches("warning: origin could not be read").count(),
        1,
        "{said}"
    );
    assert!(said.contains(THEIRS), "{said}");

    // And with no flag, nothing of the remote is attempted: an unreachable
    // origin is exactly the instrument that would say so if it were.
    let out = r.ank("claude-code@ank", &["status"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let said = stdout(&out);
    assert!(
        !said.contains("origin could not be read"),
        "status reached for a remote nobody asked it to reach for:\n{said}"
    );
}

/// With two live claims under one identity, no verb acts on one of them without
/// the caller being able to tell which (TASK-97d8747416ea).
///
/// `on_task` returns the first live record and `ank_refs` goes through
/// `for-each-ref`, which sorts by refname, so HEAD is the lowest task id among
/// the agent's live claims -- chosen, and until now chosen in silence. `log`
/// wrote its entry there, `release` handed that one back, and `done` ran the
/// verifiers of that one and moved it to `done`, none of them saying which of
/// the two they had picked.
///
/// The state is built the way it stays reachable: `claim` refuses to create it
/// (TASK-a548c95261a5), and a lapse revived produces it. Both claims are taken
/// by the binary and only the clock between them is forged.
///
/// Through the binary because the assertion is on what reaches the caller, and
/// on a real verifier because the point about `done` is *when* it refuses: a
/// witness file the verifier writes is how "before a single verifier ran" is
/// asserted rather than assumed.
#[test]
fn with_two_live_claims_no_verb_picks_one_in_silence() {
    const FIRST: &str = "TASK-000000000e01";
    const SECOND: &str = "TASK-000000000e02";
    const THIRD: &str = "TASK-000000000e03";
    const AGENT: &str = "claude-code@ank";

    let r =
        Repo::new().with_verifiers("verifiers:\n  witness:\n    run: echo ran > verifier-ran\n");
    let witness = r.0.join("verifier-ran");
    for id in [FIRST, SECOND, THIRD] {
        r.seed_task_with(id, Some("A verifiable criterion."), &["witness"]);
    }

    // Two live claims, one identity. HEAD is FIRST, being the lower id.
    assert_eq!(code(&r.ank(AGENT, &["claim", FIRST])), 0);
    r.expire_claim(FIRST);
    assert_eq!(code(&r.ank(AGENT, &["claim", SECOND])), 0);
    r.revive_claim(FIRST);

    // `log` acts, and says so before it writes.
    let out = r.ank(AGENT, &["log", "one"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stderr(&out);
    assert!(said.contains(FIRST), "the task acted on is named: {said}");
    assert!(said.contains(SECOND), "and the other live claim: {said}");
    assert!(
        said.contains("ank log"),
        "and the command that names one explicitly: {said}"
    );
    assert!(said.contains("ANK_AGENT"), "and the way out: {said}");
    assert!(
        stdout(&out).contains(&format!("logged on {FIRST}")),
        "{}",
        stdout(&out)
    );
    assert!(r.log_text(FIRST).contains("one"), "the entry went to HEAD");

    // Naming a task is what picks the other one, and it costs the refusal
    // rather than a flag: the id used to have to equal HEAD.
    let out = r.ank(AGENT, &["log", SECOND, "two"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        r.log_text(SECOND).contains("two"),
        "the named task got the entry:\n{}",
        r.log_text(SECOND)
    );
    assert!(
        !stderr(&out).contains("live claims"),
        "a caller that named its task is told nothing back: {}",
        stderr(&out)
    );

    // The same sentences for a parser -- the caller that scripts around these
    // verbs is exactly the caller running several sessions.
    let json = stdout(&r.ank(AGENT, &["log", "--json", "three"]));
    assert!(json.contains("\"warnings\""), "{json}");
    assert!(json.contains(SECOND), "{json}");

    // `done` refuses instead, being the verb whose effect running it again
    // cannot undo -- and refuses before a verifier has run.
    let out = r.ank(AGENT, &["done"]);
    assert_eq!(code(&out), 6, "{}{}", stdout(&out), stderr(&out));
    let said = stderr(&out);
    assert!(said.contains(FIRST) && said.contains(SECOND), "{said}");
    assert!(
        said.contains(&format!("ank done {FIRST}")),
        "the refusal names a command to run: {said}"
    );
    assert!(
        !witness.exists(),
        "a verifier ran before the caller had answered which task"
    );
    assert!(
        r.task_text(FIRST).contains("status: in_progress")
            && r.task_text(SECOND).contains("status: in_progress"),
        "the refusal moved a task anyway"
    );

    // Named, it goes through, and it is the named one that moves.
    let out = r.ank(AGENT, &["done", SECOND]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(witness.exists(), "the verifier of the named task never ran");
    assert!(
        r.task_text(SECOND).contains("status: done"),
        "\n{}",
        r.task_text(SECOND)
    );
    assert!(
        r.task_text(FIRST).contains("status: in_progress"),
        "done moved a task nobody named:\n{}",
        r.task_text(FIRST)
    );

    // And `release`, which needs the state rebuilt: SECOND carries a completion
    // ref now, so the second live claim is a fresh one.
    r.expire_claim(FIRST);
    assert_eq!(code(&r.ank(AGENT, &["claim", THIRD])), 0);
    r.revive_claim(FIRST);

    let out = r.ank(AGENT, &["release", "--reason", "the criterion is wrong"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stderr(&out);
    assert!(said.contains(FIRST) && said.contains(THIRD), "{said}");
    assert!(said.contains("ank release"), "{said}");
    assert!(
        stdout(&out).contains(&format!("released {FIRST}")),
        "{}",
        stdout(&out)
    );
    assert!(
        r.task_text(THIRD).contains("status: in_progress"),
        "release handed back a task nobody named:\n{}",
        r.task_text(THIRD)
    );

    // One claim, which is the nominal case, says none of it.
    let out = r.ank(AGENT, &["log", "alone now"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        !stderr(&out).contains("live claims"),
        "the nominal case pays for the exceptional one: {}",
        stderr(&out)
    );
}

/// A holder returning to a lapsed claim carries on; one whose task was taken
/// over is told by whom.
///
/// Section 3 calls the first case normal and not a fault -- a build longer than
/// the lease expires the claim -- and both `log` and `done` answered
/// `no task in progress for this agent` instead, which is neither of the two
/// answers it specifies. Measured in the nominal flow: the claim lapsed during
/// a CI wait, and `done` was the next command (TASK-5bd23835d5a0).
#[test]
fn a_lapsed_claim_is_retaken_by_its_holder_and_lost_to_whoever_took_it() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: git --version\n");
    r.seed_task_with(ID, Some("A verifiable criterion."), &["ok"]);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    r.expire_claim(ID);

    // `status` says so in words before anything retakes it. It used to answer
    // `no claim`, which is false -- the task is still this agent's -- and an
    // expiry alone is a past timestamp a reader compares against nothing.
    let said = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(said.contains("lapsed"), "{said}");
    assert!(said.contains(ID), "the task is still the agent's: {said}");
    let json = stdout(&r.ank("claude-code@ank", &["status", "--json"]));
    assert!(json.contains("\"lapsed\":true"), "{json}");

    // `log` first: the renewal it performs is the re-acquisition, and an agent
    // that comes back to say what it learned must not be turned away.
    let out = r.ank("claude-code@ank", &["log", ID, "back from a long build"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let record = r.claim_ref(ID).expect("the ref survived");
    assert!(
        record.contains("holder: claude-code@ank"),
        "the ref no longer names the agent that retook it:\n{record}"
    );
    assert!(
        !record.contains("expires: 2020"),
        "the expiry was not moved:\n{record}"
    );

    // And `done` on a claim that lapsed again, which is the case that was
    // measured: the CI wait outlasts the lease, and `done` is what follows it.
    r.expire_claim(ID);
    let out = r.ank("claude-code@ank", &["done"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        r.task_text(ID).contains("status: done"),
        "{}",
        r.task_text(ID)
    );
}

/// The other half of section 3: taken over in the meantime is code 4, and it
/// names the new holder.
#[test]
fn a_lapsed_claim_taken_over_is_refused_with_the_new_holder_named() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: git --version\n");
    r.seed_task_with(ID, Some("A verifiable criterion."), &["ok"]);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    r.expire_claim(ID);

    // A lapsed claim is claimable, which is what makes the expiry useful.
    assert_eq!(code(&r.ank("codex@host-9", &["claim", ID])), 0);

    for verb in [vec!["done"], vec!["log", ID, "still here"]] {
        let out = r.ank("claude-code@ank", &verb);
        let said = format!("{}{}", stdout(&out), stderr(&out));
        assert_eq!(code(&out), 6, "{verb:?}: {said}");
        assert!(
            !r.task_text(ID).contains("status: done"),
            "{verb:?} finished a task it no longer holds"
        );
    }
    assert!(
        r.claim_ref(ID).unwrap().contains("holder: codex@host-9"),
        "the takeover did not survive"
    );
}

/// The over-constrained signal reports the threshold it actually applied.
///
/// The two numbers are parsed back out of the message and compared, which is
/// the only assertion that catches the defect: the signal used to report the
/// budget while testing half of it, so `check` said `5527 characters of
/// constraint against a budget of 8000` and every reader who checked the
/// arithmetic concluded the tool was miscounting (TASK-9ff86a0950bf). A test
/// asserting a fixed wording would have passed on that message.
///
/// The corpus is built to land in the gap the defect lived in: over half the
/// budget, under the budget. Anywhere else the two readings agree.
#[test]
fn the_over_constrained_signal_reports_the_limit_it_tested() {
    let r = Repo::new();
    r.set_config("schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\ncontext_budget: 400\n");
    r.seed_task(ID, Some("A verifiable criterion."));
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();

    // 300 characters: over the 200 that half of 400 allows, and well under 400.
    let rule = "x".repeat(300);
    r.seed_adr("ADR-0000000000ab", &rule, "src/**");
    let accepted = r
        .adr_text("ADR-0000000000ab")
        .replace("status: proposed", "status: accepted");
    std::fs::write(r.0.join(".ank/entities/ADR-0000000000ab.md"), accepted).unwrap();

    let out = r.ank("claude-code@ank", &["check"]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    let line = said
        .lines()
        .find(|l| l.contains("over-constrained"))
        .unwrap_or_else(|| panic!("the signal did not fire on a corpus built to trip it:\n{said}"));

    // From the message alone: the subject is an id, and an id is full of digits.
    let message = line
        .split_once("over-constrained scope:")
        .expect("the line was found by that needle")
        .1;
    let numbers: Vec<usize> = message
        .split(|c: char| !c.is_ascii_digit())
        .filter(|w| !w.is_empty())
        .map(|w| w.parse().unwrap())
        .collect();
    assert_eq!(
        numbers.len(),
        2,
        "the signal must name the quantity and the limit, and nothing else \
         numeric: {line}"
    );
    let (quantity, limit) = (numbers[0], numbers[1]);
    assert!(
        quantity > limit,
        "the signal fired while reporting a limit the quantity does not exceed, \
         which is arithmetic no reader can believe: {line}"
    );
    // The constraint as stored carries its trailing newline, so the count is
    // the 300 written plus it.
    assert_eq!(quantity, 301, "{line}");
    assert_eq!(limit, 200, "half of the configured budget: {line}");

    // And it stays silent just under the threshold, so the assertion above is
    // about the reported figures and not about a signal that always fires.
    let quiet = Repo::new();
    quiet.set_config("schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\ncontext_budget: 400\n");
    quiet.seed_task(ID, Some("A verifiable criterion."));
    std::fs::create_dir_all(quiet.0.join("src")).unwrap();
    std::fs::write(quiet.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    quiet.seed_adr("ADR-0000000000ab", &"x".repeat(150), "src/**");
    let accepted = quiet
        .adr_text("ADR-0000000000ab")
        .replace("status: proposed", "status: accepted");
    std::fs::write(quiet.0.join(".ank/entities/ADR-0000000000ab.md"), accepted).unwrap();
    let out = quiet.ank("claude-code@ank", &["check"]);
    assert!(
        !stdout(&out).contains("over-constrained"),
        "{}",
        stdout(&out)
    );
}

/// The over-constrained finding says which constraint is expensive, and what
/// the reader can do about it.
///
/// The total was the whole of the old message, and a total names a problem with
/// no path out of it: nothing in `14737 characters` says which of two dozen
/// constraints to stop matching, and a finding that names a number and no act
/// trains readers to skip it (TASK-19d82da76c78). Three assertions, and each is
/// a different half of that.
///
/// **Order, not merely presence.** The list is the treatment, so a breakdown in
/// id order would be a list the reader has to sort by hand — the charges are
/// parsed back out and checked descending, which a listing in the order
/// `applicable_constraints` happens to return would fail.
///
/// **Data, not prose, under `--json`.** A caller ranking constraints by cost
/// must not have to parse "charges 211 characters" back into an integer, so the
/// array is asserted whole and in order rather than by substring.
///
/// **An act, never a refusal.** Every constraint charged against a perimeter is
/// an accepted ADR, and `amend` exits 6 on one — so the finding says the
/// constraint is anchored and points the amend at the perimeter instead. The
/// test asserts the refusing command is absent, which is the assertion that
/// catches a future edit reaching for the obvious wording.
#[test]
fn the_over_constrained_signal_charges_each_constraint_and_names_an_act() {
    let r = Repo::new();
    r.set_config("schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\ncontext_budget: 400\n");
    r.seed_task(ID, Some("A verifiable criterion."));
    // Two globs, so the perimeter can lose one: the note branches on that, and
    // a one-glob fixture would exercise the other branch only.
    let text = r
        .task_text(ID)
        .replace("  - src/**", "  - docs/**\n  - src/**");
    std::fs::write(r.flat_task_path(ID), text).unwrap();
    for (dir, file) in [("src", "src/main.rs"), ("docs", "docs/guide.md")] {
        std::fs::create_dir_all(r.0.join(dir)).unwrap();
        std::fs::write(r.0.join(file), "content\n").unwrap();
    }

    // 211, 151 and 91 characters once the stored trailing newline is counted:
    // 453 against the 200 that half of 400 allows. Seeded in an order that is
    // neither the id order nor the charge order, so neither can pass by luck.
    let sizes = [
        ("ADR-0000000000ab", 150usize),
        ("ADR-0000000000cd", 90),
        ("ADR-0000000000ef", 210),
    ];
    for (id, size) in sizes {
        r.seed_adr(id, &"x".repeat(size), "src/**");
        let accepted = r
            .adr_text(id)
            .replace("status: proposed", "status: accepted");
        std::fs::write(r.0.join(".ank/entities").join(format!("{id}.md")), accepted).unwrap();
    }

    let out = r.ank("claude-code@ank", &["check"]);
    let said = stdout(&out);
    let charged: Vec<(String, usize)> = said
        .lines()
        .filter_map(|l| l.split_once(" charges "))
        .map(|(head, tail)| {
            let id = head.rsplit(' ').next().unwrap_or_default().to_string();
            let n = tail.trim_end_matches(" characters").parse().unwrap();
            (id, n)
        })
        .collect();
    assert_eq!(
        charged,
        vec![
            ("ADR-0000000000ef".to_string(), 211),
            ("ADR-0000000000ab".to_string(), 151),
            ("ADR-0000000000cd".to_string(), 91),
        ],
        "the breakdown must name every constraint and its cost, largest first:\n{said}"
    );

    // The act, on the entity that is still open to an edit.
    assert!(
        said.contains(&format!("ank amend {ID} --drop-scope \"<glob>\"")),
        "the finding names no act the reader can perform:\n{said}"
    );
    // And never on the one that is not. `amend` exits 6 on an accepted ADR, so
    // naming it would teach the reader the tool is wrong rather than that the
    // decision is settled.
    assert!(
        !said.contains("ank amend ADR-"),
        "the finding names a command that would refuse:\n{said}"
    );
    assert!(
        said.contains("ADR-0000000000ef costs the most, and is accepted"),
        "the heaviest constraint is not named as the one that cannot be amended:\n{said}"
    );

    // The same fact as data. Asserted as one substring so the order is part of
    // the assertion and the counts stay bare numbers.
    let out = r.ank("claude-code@ank", &["check", "--json"]);
    assert!(
        stdout(&out).contains(
            "\"charge\":[{\"id\":\"ADR-0000000000ef\",\"characters\":211},\
             {\"id\":\"ADR-0000000000ab\",\"characters\":151},\
             {\"id\":\"ADR-0000000000cd\",\"characters\":91}]"
        ),
        "the per-constraint charge is not structured data:\n{}",
        stdout(&out)
    );

    // A perimeter of one glob cannot be narrowed by dropping it — the entity
    // would attach to nothing and `amend` refuses — so the note offers the
    // replacement, and names the glob because there is only one it could be.
    let one = Repo::new();
    one.set_config("schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\ncontext_budget: 400\n");
    one.seed_task(ID, Some("A verifiable criterion."));
    std::fs::create_dir_all(one.0.join("src")).unwrap();
    std::fs::write(one.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    one.seed_adr("ADR-0000000000ab", &"x".repeat(300), "src/**");
    let accepted = one
        .adr_text("ADR-0000000000ab")
        .replace("status: proposed", "status: accepted");
    std::fs::write(one.0.join(".ank/entities/ADR-0000000000ab.md"), accepted).unwrap();
    let out = one.ank("claude-code@ank", &["check"]);
    assert!(
        stdout(&out).contains(&format!(
            "ank amend {ID} --scope \"<narrower>\" --drop-scope \"src/**\""
        )),
        "a one-glob perimeter was offered a drop that would refuse:\n{}",
        stdout(&out)
    );

    // Silent under the limit, breakdown included: a per-constraint listing on a
    // healthy perimeter is volume nobody asked for.
    let quiet = Repo::new();
    quiet.set_config("schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\ncontext_budget: 400\n");
    quiet.seed_task(ID, Some("A verifiable criterion."));
    std::fs::create_dir_all(quiet.0.join("src")).unwrap();
    std::fs::write(quiet.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    quiet.seed_adr("ADR-0000000000ab", &"x".repeat(150), "src/**");
    let accepted = quiet
        .adr_text("ADR-0000000000ab")
        .replace("status: proposed", "status: accepted");
    std::fs::write(quiet.0.join(".ank/entities/ADR-0000000000ab.md"), accepted).unwrap();
    let out = quiet.ank("claude-code@ank", &["check"]);
    assert!(
        !stdout(&out).contains(" charges "),
        "a perimeter under the limit reported a breakdown:\n{}",
        stdout(&out)
    );
    let out = quiet.ank("claude-code@ank", &["check", "--json"]);
    assert!(
        stdout(&out).contains("\"charge\":[]") || !stdout(&out).contains("over-constrained"),
        "{}",
        stdout(&out)
    );
}

/// A renewal reuses the lease the claim was granted, and re-caps it.
///
/// Through the binary and against the ref itself, because the lease is a fact
/// about `refs/ank/claims/<id>` and not about a struct: the record is what the
/// next process reads, and it is what an agent's survival across a silent
/// stretch actually depends on.
///
/// The defect this pins: renewal recomputed `DEFAULT_TTL.min(claim_ttl_max)`
/// and never read the granted lease, so an agent that asked for two hours held
/// them once and fell back to thirty minutes at its first `log` -- the command
/// the loop tells it to run often (TASK-1b45f41e7b99).
#[test]
fn a_renewal_keeps_the_lease_the_claim_was_granted() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    // Two hours is the default `claim_ttl_max` of the fixture, so this asks
    // for the most the cap allows and nothing more.
    let out = r.ank("claude-code@ank", &["claim", ID, "--ttl", "2h"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let granted = expiry_span(&r);
    assert!(
        (7000..=7300).contains(&granted),
        "the claim did not grant the two hours it was asked for: {granted}s"
    );

    let out = r.ank("claude-code@ank", &["log", ID, "still working"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let renewed = expiry_span(&r);
    assert!(
        (7000..=7300).contains(&renewed),
        "the renewal dropped the granted lease and fell back to the default: \
         {renewed}s from now, where the claim had {granted}s"
    );

    // The cap applies at renewal and not only at the claim, so lowering it
    // takes effect on the next log rather than waiting for the next claim.
    r.set_config("schema: 1\nclaim_ttl_max: 45m\ndefault_branch: main\n");
    let out = r.ank("claude-code@ank", &["log", ID, "still working"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let capped = expiry_span(&r);
    assert!(
        (2500..=2800).contains(&capped),
        "a lowered claim_ttl_max did not bind the renewal: {capped}s"
    );
}

/// A verb the holder runs against the task it holds renews that lease, and one
/// about anything else renews nothing (§3, ADR-0bb7ea8991bc).
///
/// Through the binary and against the ref, for the reason its neighbour above
/// is: what an agent's survival across a silent stretch depends on is the
/// record the next process reads.
///
/// **The expiry is forged far ahead and the renewal is what brings it back.**
/// A renewal that landed a second after the claim would be invisible against a
/// lease still running, and the honest wait is the lease itself; 2099 cannot be
/// confused with two hours from now, so nothing here has to be timed.
///
/// The defect this pins: the lease was renewed by `log` alone, and `log` is
/// reporting rather than working — so it lapsed precisely during the stretch
/// with nothing worth logging, which is also the stretch where the work is
/// least interruptible. Three parallel sessions hit it independently.
#[test]
fn a_verb_of_the_holder_on_the_task_it_holds_renews_the_lease() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    let out = r.ank("claude-code@ank", &["claim", ID, "--ttl", "2h"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // `show` on the held task: a read, and the holder's work on it.
    r.revive_claim(ID);
    let out = r.ank("claude-code@ank", &["show", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let renewed = expiry_span_of(&r, ID);
    assert!(
        (7000..=7300).contains(&renewed),
        "ank show on the held task did not renew the lease: {renewed}s from now"
    );

    // `context` with a claim in hand is about that task and nothing else, and
    // it names no id: the rule is not "a verb carrying the right id".
    r.revive_claim(ID);
    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let renewed = expiry_span_of(&r, ID);
    assert!(
        (7000..=7300).contains(&renewed),
        "ank context in execution mode did not renew the lease: {renewed}s"
    );

    // The cap binds this renewal as it binds `log`'s, so a lowered
    // `claim_ttl_max` takes effect on the next verb rather than the next claim.
    r.revive_claim(ID);
    r.set_config("schema: 1\nclaim_ttl_max: 45m\ndefault_branch: main\n");
    let out = r.ank("claude-code@ank", &["show", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let capped = expiry_span_of(&r, ID);
    assert!(
        (2500..=2800).contains(&capped),
        "a lowered claim_ttl_max did not bind the renewal a read performed: \
         {capped}s"
    );
}

/// The other half of the rule, and the half that keeps it a rule: a verb that
/// is not about the held task moves nothing (§3).
///
/// The record is compared byte for byte rather than by its expiry, because what
/// must be true is that the ref was not written at all — an expiry rewritten to
/// the same second would pass a comparison on seconds and would still be a
/// claim renewed by a verb that renews nothing.
#[test]
fn a_verb_about_another_task_or_the_repository_renews_nothing() {
    const OTHER: &str = "TASK-000000000002";
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.seed_task(OTHER, Some("Another verifiable criterion."));

    let out = r.ank("claude-code@ank", &["claim", ID, "--ttl", "2h"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    r.revive_claim(ID);
    let before = r.claim_ref(ID).expect("the claim ref must exist");

    for args in [
        // Named, and naming another one.
        vec!["show", OTHER],
        // Off the loop and about the repository, which is the case §3 spells
        // out: `status` does not renew.
        vec!["status"],
        vec!["find", "--status", "open"],
        vec!["check"],
    ] {
        let out = r.ank("claude-code@ank", &args);
        assert!(
            code(&out) == 0 || code(&out) == 8,
            "{args:?}: {}",
            stderr(&out)
        );
        assert_eq!(
            r.claim_ref(ID).as_deref(),
            Some(before.as_str()),
            "ank {args:?} renewed a lease it is not about"
        );
    }
}

/// `claim` with no `--ttl` grants what the repository states, capped (§3, §4).
///
/// Thirty minutes stays the shipped default. What changes is that a repository
/// can say its own number once, instead of every agent rediscovering `--ttl` by
/// losing a claim first.
#[test]
fn claim_without_a_ttl_takes_claim_ttl_default() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    // Unset, so the tool's value: the file the fixture carries names no
    // `claim_ttl_default` at all.
    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let granted = expiry_span_of(&r, ID);
    assert!(
        (1700..=1900).contains(&granted),
        "an unset claim_ttl_default did not resolve to the tool's thirty \
         minutes: {granted}s"
    );

    let out = r.ank("claude-code@ank", &["release", ID, "--reason", "again"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    r.set_config("schema: 1\nclaim_ttl_max: 2h\nclaim_ttl_default: 90m\ndefault_branch: main\n");
    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let stated = expiry_span_of(&r, ID);
    assert!(
        (5300..=5500).contains(&stated),
        "the repository's claim_ttl_default was not what the claim granted: \
         {stated}s"
    );

    let out = r.ank("claude-code@ank", &["release", ID, "--reason", "again"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // A default above the cap is not a configuration that fails to load: the
    // cap binds it exactly as it binds a value the caller typed.
    r.set_config("schema: 1\nclaim_ttl_max: 2h\nclaim_ttl_default: 4h\ndefault_branch: main\n");
    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let capped = expiry_span_of(&r, ID);
    assert!(
        (7000..=7300).contains(&capped),
        "claim_ttl_max did not cap a claim_ttl_default above it: {capped}s"
    );
}

/// Seconds from now to the expiry the claim ref carries, read with git.
fn expiry_span(r: &Repo) -> i64 {
    expiry_span_of(r, ID)
}

/// The same reading, of a named task's claim.
fn expiry_span_of(r: &Repo, id: &str) -> i64 {
    let record = r.claim_ref(id).expect("the claim ref must exist");
    let expires = record
        .lines()
        .find_map(|l| l.strip_prefix("expires: "))
        .unwrap_or_else(|| panic!("no expires in the record: {record}"))
        .trim()
        .to_string();

    // The record's own format, parsed here rather than through the crate: what
    // is under test is the bytes the next process reads.
    let stamp = |s: &str, at: usize, n: usize| -> i64 { s[at..at + n].parse().unwrap() };
    let (y, mo, d) = (
        stamp(&expires, 0, 4),
        stamp(&expires, 5, 2),
        stamp(&expires, 8, 2),
    );
    let (h, mi, s) = (
        stamp(&expires, 11, 2),
        stamp(&expires, 14, 2),
        stamp(&expires, 17, 2),
    );
    // Days since the epoch, by the civil-from-days algorithm the crate uses.
    let (y, mo) = if mo <= 2 { (y - 1, mo + 12) } else { (y, mo) };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * (mo - 3) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    let at = days * 86_400 + h * 3600 + mi * 60 + s;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    at - now
}

/// The listing describes every verb, and never as doing what it refuses.
///
/// Both surfaces through the binary, because the defect this guards against is
/// a disagreement *between* them: the flat listing is read fastest and checked
/// least, and `amend` advertised a criterion edit the binary refused always
/// (TASK-84cfad83c308). Two assertions, and the first is what makes the second
/// hold for good — the description in the listing is the same string the verb's
/// own page prints, so there is one text to keep true rather than two.
#[test]
fn the_listing_describes_each_verb_as_its_own_help_does() {
    let listing = stdout(&ank_command().arg("help").output().unwrap());

    // The listing's descriptions, folded lines rejoined. A line beginning with
    // `ank ` opens a verb; an indented one continues the description above it;
    // any other line at column 0 is a group heading and describes no verb.
    let mut described: Vec<(String, String)> = Vec::new();
    for line in listing_body(&listing) {
        match line.strip_prefix("ank ") {
            Some(rest) => {
                let verb: String = rest.chars().take_while(char::is_ascii_lowercase).collect();
                let desc = rest
                    .split_once("  ")
                    .map(|(_, d)| d.trim().to_string())
                    .unwrap_or_default();
                described.push((verb, desc));
            }
            None if !line.starts_with(' ') => continue,
            None => {
                let (_, desc) = described
                    .last_mut()
                    .expect("a continuation before any verb");
                desc.push(' ');
                desc.push_str(line.trim());
            }
        }
    }
    assert!(
        described.len() > 15,
        "the listing was not parsed: {listing}"
    );

    for (verb, desc) in described {
        assert!(
            !desc.is_empty(),
            "`ank {verb}` is listed with no description: the listing names every \
             verb and must say what each one does"
        );

        let page = stdout(&ank_command().args(["help", &verb]).output().unwrap());
        let mut lines = page.lines();
        lines.next();
        let summary = lines.next().unwrap_or("").trim();

        // One string, two surfaces. A listing that paraphrased the page would
        // be a second text, and the one that drifts is the one nobody rereads.
        assert_eq!(
            desc, summary,
            "the listing and `ank help {verb}` describe the verb differently"
        );

        // What the page offers, and what it says the verb refuses.
        let labelled = |label: &str| -> String {
            page.lines()
                .skip(2)
                .filter(|l| l.trim_start().starts_with(label))
                .collect::<Vec<_>>()
                .join(" ")
        };
        let offered = format!("{} {}", labelled("flags:"), labelled("global:"));

        // Every flag a description names is one the verb offers, or one it
        // names as refused. `refuses --repo` is honest; naming a flag the verb
        // rejects as though it were on offer is the defect.
        for (i, word) in desc.split_whitespace().enumerate() {
            let name = word.trim_end_matches(|c: char| !c.is_ascii_alphanumeric());
            if !name.starts_with("--") {
                continue;
            }
            let as_refusal = i > 0 && desc.split_whitespace().nth(i - 1) == Some("refuses");
            assert!(
                offered.contains(name) || as_refusal,
                "`ank {verb}` is described with {name}, which it does not offer:\n\
                 description: {desc}\n\
                 offered:     {offered}"
            );
        }
    }
}

/// A flag the verb rejects by design is in the refusals and not in the offer
/// (§9), and being global is not an exemption.
///
/// Through the binary, because the rule is about what a caller reads: `ank help
/// init` offering `--repo` is the offer that let the flag look supported while
/// the verb wrote somewhere else. Both renderings, since `--json` is how a
/// script reads the same surface.
#[test]
fn help_does_not_offer_init_the_global_it_refuses() {
    let text = stdout(&ank_command().args(["help", "init"]).output().unwrap());
    let line = |label: &str| -> String {
        text.lines()
            .find(|l| l.trim_start().starts_with(label))
            .unwrap_or_else(|| panic!("help init prints no {label} line:\n{text}"))
            .to_string()
    };
    // Absent from the offer, present in the refusals: the two halves of the
    // rule, and asserting only the first would pass on a page that says nothing
    // about the flag at all.
    let globals = line("global:");
    assert!(
        !globals.contains("--repo"),
        "help init offers a flag the verb refuses:\n{globals}"
    );
    assert!(
        globals.contains("--json") && globals.contains("--quiet"),
        "the other two globals are unaffected:\n{globals}"
    );
    assert!(
        line("refuses:").contains("--repo"),
        "the refusal is not stated where a caller looks for it:\n{text}"
    );

    let json = stdout(
        &ank_command()
            .args(["help", "init", "--json"])
            .output()
            .unwrap(),
    );
    assert!(
        !json.contains("\"--repo\""),
        "--json offers what the human page does not:\n{json}"
    );

    // Every other verb still carries all three: the exception is one verb wide.
    for verb in ["context", "claim", "done", "new", "find", "check", "config"] {
        let text = stdout(&ank_command().args(["help", verb]).output().unwrap());
        assert!(
            text.contains("--repo"),
            "help {verb} lost a global it accepts:\n{text}"
        );
    }

    // And the flat listing still states the three globals of §4, unqualified:
    // the exception belongs on the page of the verb that makes it.
    let flat = stdout(&ank_command().arg("help").output().unwrap());
    assert!(flat.contains("global: --json --quiet --repo"), "{flat}");
}

/// §6 calls the index derived, disposable and gitignored. The first two were
/// true of what `init` produced; the third was true of no repository it had
/// ever produced, because it wrote no ignore rule at all. This repository was
/// not the counterexample -- it carries the line by hand, written before
/// `init` existed to write it, which is exactly how the gap survived.
///
/// Through the binary end to end, and deliberately not by reading
/// `.gitignore`: what has to be true is that *git* treats the file as ignored,
/// and a line present in a file it does not consult would satisfy any
/// assertion about the file's content while proving nothing.
#[test]
fn an_initialised_repo_leaves_the_index_ignored_and_never_untracked() {
    let dir = std::env::temp_dir().join(format!("ank-cli-ignore-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let git = |args: &[&str]| -> Output {
        git_command(&dir)
            .args(args)
            .output()
            .expect("git must be installed: it is a hard dependency")
    };
    assert!(git(&["init", "-q", "-b", "main"]).status.success());

    // A `.gitignore` the user curated first: `init` has to append to it, not
    // replace it, and that is only observable when there is something to lose.
    std::fs::write(dir.join(".gitignore"), "/target\n").unwrap();

    let init = || -> Output {
        ank_command()
            .arg("init")
            .arg(&dir)
            .env("ANK_AGENT", "claude-code@ank")
            .current_dir(std::env::temp_dir())
            .output()
            .expect("the binary must have been built")
    };
    let out = init();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("wrote .gitignore"),
        "{}",
        stdout(&out)
    );

    // A command that builds the index, so the file under test actually exists.
    let out = ank_command()
        .args(["find", "--status", "open"])
        .arg("--repo")
        .arg(&dir)
        .env("ANK_AGENT", "claude-code@ank")
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        dir.join(".ank/index.db").exists(),
        "nothing built the index, so the assertion below would pass vacuously"
    );

    // Ignored, positively: `check-ignore` names a rule that matches.
    let ci = git(&["check-ignore", "-v", ".ank/index.db"]);
    assert!(
        ci.status.success(),
        "git does not consider the index ignored: {}{}",
        String::from_utf8_lossy(&ci.stdout),
        String::from_utf8_lossy(&ci.stderr)
    );

    // And untracked, negatively: `status` must not offer it. `-uall` because
    // the default collapses an untracked directory to its name, which would
    // hide the very path in question behind `.ank/`.
    let st = git(&["status", "--porcelain", "-uall"]);
    let st = String::from_utf8_lossy(&st.stdout).replace('\\', "/");
    assert!(
        !st.contains("index.db"),
        "the index is still offered for commit:\n{st}"
    );
    // The fixture is sound only if `status` sees the rest of `.ank/`.
    assert!(
        st.contains(".ank/config.yml"),
        "empty status proves nothing:\n{st}"
    );

    // The user's own rule survived, and a second init adds nothing.
    let gi = std::fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert!(gi.starts_with("/target\n"), "{gi:?}");

    let out = init();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        !stdout(&out).contains("wrote .gitignore"),
        "re-init rewrote it: {}",
        stdout(&out)
    );
    let again = std::fs::read_to_string(dir.join(".gitignore")).unwrap();
    assert_eq!(gi, again, "re-init changed .gitignore");

    let _ = std::fs::remove_dir_all(&dir);
}

/// The bug this exists for, end to end and through the binary: a corpus that a
/// Windows clone handed back in CRLF was entirely unreadable, and every entity
/// was reported as "missing frontmatter" -- a diagnostic that sends the reader
/// looking for a delimiter sitting right there on line one.
///
/// The criterion names `ank check` and an exit code, so it is the process that
/// answers here, not the function.
#[test]
fn a_crlf_corpus_is_read_signalled_and_exits_zero_through_the_binary() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    let p = r.0.join(".ank/entities").join(format!("{ID}.md"));
    let lf = std::fs::read_to_string(&p).unwrap();
    std::fs::write(&p, lf.replace('\n', "\r\n")).unwrap();

    let out = r.ank("claude-code@ank", &["check"]);
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));

    assert_eq!(code(&out), 0, "CRLF alone must not fail the build: {text}");
    assert!(text.contains("CRLF"), "{text}");
    assert!(text.contains("git config core.autocrlf input"), "{text}");
    assert!(text.contains("signal"), "{text}");
    assert!(
        !text.contains("missing frontmatter"),
        "the wrong diagnostic came back: {text}"
    );
    assert!(!text.contains("non-canonical"), "{text}");

    // And the entity was genuinely read, not merely tolerated: `show` prints
    // the task it could not previously parse at all.
    let out = r.ank("claude-code@ank", &["show", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(String::from_utf8_lossy(&out.stdout).contains("Example task"));
}

/// The one write §3 permits after `done`, performed by a command.
///
/// Through the binary and reading the file back, because the append is the
/// thing being asserted: what has to be true is what ends up on disk, not the
/// module's agreement with itself. Before this verb the same operation was done
/// by opening the file, where an append and a substitution are the same gesture.
#[test]
fn attest_appends_a_proof_to_a_finished_task_and_never_substitutes() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task_with(ID, Some("A verifiable criterion."), &["ok"]);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    assert_eq!(code(&r.ank("claude-code@ank", &["done"])), 0);

    let before = r.task_text(ID);
    assert!(before.contains("status: done"), "{before}");
    let original = before
        .lines()
        .find(|l| l.contains("ref: local/"))
        .expect("done wrote a proof")
        .to_string();

    let head = r.head();
    let out = r.ank(
        "claude-code@ank",
        &["attest", ID, "--proof", &format!("commit:{head}")],
    );
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
    assert_eq!(code(&out), 0, "{text}");

    let after = r.task_text(ID);
    // Appended, never substituted: the entry `done` wrote is still there, and
    // the new one sits beside it.
    assert!(
        after.contains(&original),
        "the original proof was rewritten:\n{after}"
    );
    assert!(after.contains("type: commit"), "{after}");
    assert!(after.contains(&head), "{after}");
    // Version increments and the log records what was added.
    assert!(!after.contains("version: 1"), "{after}");
    assert!(
        r.log_text(ID).contains("attested commit:"),
        "{}",
        r.log_text(ID)
    );

    // A commit that does not exist is refused, exactly as `done` refuses it.
    let out = r.ank(
        "claude-code@ank",
        &[
            "attest",
            ID,
            "--proof",
            "commit:0000000000000000000000000000000000000000",
        ],
    );
    assert_eq!(code(&out), 5, "{}", stderr(&out));
    assert_eq!(r.task_text(ID), after, "nothing was written");
}

#[test]
fn attest_refuses_a_task_that_is_not_finished_and_names_the_verb_that_applies() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    // A prerequisite rather than a transition: `attest` changes no status, it
    // adds to a record that only exists once the task is closed out.
    let out = r.ank("claude-code@ank", &["attest", ID, "--proof", "assertion:x"]);
    let err = stderr(&out);
    assert_eq!(code(&out), 7, "{err}");
    assert!(err.contains("open"), "the message names the state: {err}");
    assert!(
        err.contains(&format!("ank done {ID}")),
        "the refusal names the verb that applies: {err}"
    );
    assert!(
        !r.task_text(ID).contains("proof:"),
        "a refusal writes nothing"
    );
}

/// Writes a `done` task carrying exactly `proofs`, which the seeding helpers
/// cannot express: they build open tasks, and `done` is the one status whose
/// proof list is the subject here.
fn seed_done(r: &Repo, id: &str, proofs: &str) {
    std::fs::write(
        r.0.join(".ank/entities").join(format!("{id}.md")),
        format!(
            "---\nid: {id}\ntype: task\nslug: example\ntitle: Example task\n\
             created: 2026-07-28T00:00:00Z\nstatus: done\nscope:\n  - src/**\n\
             blocked_by: []\ndone_criteria: |\n  A verifiable criterion.\n\
             criteria_by: creator\nproof:\n{proofs}schema: 1\nversion: 1\n---\n\nFree body.\n"
        ),
    )
    .unwrap();
}

const ATTESTED: &str = "TASK-00000000a77e";
const UNATTESTED: &str = "TASK-00000000c0dd";

/// `done` records what ran on the machine that ran it; `attest` anchors the
/// same criterion to a run anybody can re-read. Nothing used to notice a task
/// that never got the second one: it reads `done`, its proof list is not empty,
/// and `commit` is not weak, so every existing finding stayed silent.
///
/// Through the binary, and the negative half carries the weight: a signal that
/// fired on the attested task too would be reporting every finished task in the
/// corpus, which is a line readers learn to skip rather than a finding.
#[test]
fn check_reports_a_done_task_that_was_never_attested_and_spares_one_that_was() {
    let r = Repo::new();
    seed_done(&r, UNATTESTED, "  - type: commit\n    ref: abc1234\n");
    seed_done(
        &r,
        ATTESTED,
        "  - type: commit\n    ref: abc1234\n  - type: test\n    ref: \"991\"\n",
    );
    // The seeded scope has to match something tracked, or a dead-scope fault
    // fires and the exit code under test stops being about attestation.
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/lib.rs"), "// x\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let said = format!("{}{}", stdout(&out), stderr(&out));

    // A signal and not a fault: the corpus is intact, the record is thin.
    // Exit 8 here would redden CI on the merge that introduced the task.
    assert_eq!(
        code(&out),
        0,
        "attestation is a signal, not a fault: {said}"
    );

    let reported: Vec<&str> = said
        .lines()
        .filter(|l| l.contains("no test proof"))
        .collect();
    assert_eq!(
        reported.len(),
        1,
        "exactly the unattested task should be named:\n{said}"
    );
    assert!(reported[0].contains(UNATTESTED), "{said}");
    assert!(
        reported[0].contains(&format!("ank attest {UNATTESTED}")),
        "a finding names the exact command that clears it: {said}"
    );
}

/// The gate, which is load-bearing and not decoration.
///
/// On a feature branch straight after `done`, attesting is impossible: no merge
/// run exists to cite yet. Reporting there would name work the reader cannot
/// do, and the completion ref already covers that window with a signal that
/// says the useful thing instead.
#[test]
fn an_unattested_task_is_not_reported_until_the_default_branch_carries_it() {
    let r = Repo::new();
    // A commit on `main` that does not carry the task, so the question asked of
    // the default branch is answerable and the answer is no. Without this, the
    // branch would carry no commit at all and the silence below would prove
    // only that git could not be asked.
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/lib.rs"), "// x\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    r.git(&["checkout", "-q", "-b", "feature"]);
    seed_done(&r, UNATTESTED, "  - type: commit\n    ref: abc1234\n");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "work"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(code(&out), 0, "{said}");
    assert!(
        !said.contains("no test proof"),
        "reported before the merge, when attesting is impossible:\n{said}"
    );

    // And it appears the moment the default branch catches up, so the silence
    // above is the gate and not the signal being broken outright.
    r.git(&["checkout", "-q", "main"]);
    r.git(&["merge", "-q", "--no-ff", "-m", "merge", "feature"]);
    let out = r.ank("claude-code@ank", &["check"]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert!(
        said.contains("no test proof") && said.contains(UNATTESTED),
        "the default branch caught up and nothing was reported:\n{said}"
    );
}

/// The body that is painful to type as a flag arrives on stdin instead.
///
/// The prose here is the whole point of the test, so it is the prose that hurts:
/// six lines, blank lines between them, and quotes of both kinds. Written as a
/// shell argument this is a fight with escaping — which is the friction
/// TASK-8e7c8e7724ee measured, not a hypothesis about one.
///
/// Asserted through `ank show` and not by reading the file, because `show` is
/// what an agent receives: what has to survive the pipe is the body a reader
/// gets back, byte for byte.
#[test]
fn a_body_piped_on_stdin_reaches_show_byte_for_byte() {
    let r = Repo::new();

    // A heredoc's trailing newline is absorbed by the canonical form, so the
    // text below is what `show` must return verbatim, and the pipe carries one
    // more newline than that.
    const BODY: &str = "Observed friction, not speculation: writing a body \
                        through a shell flag means \"fighting\" quoting.\n\
                        \n\
                        The '-' convention is the established Unix answer, and \
                        it costs nothing on the surface.\n\
                        \n\
                        - a bullet, indented\n\
                        - and a second one, with an apostrophe it doesn't need";

    let out = r.ank_stdin(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--title",
            "A piped task",
            "--scope",
            "src/**",
            "--criteria",
            "A verifiable criterion.",
            "--body",
            "-",
        ],
        &format!("{BODY}\n"),
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let id = stdout(&out)
        .split_whitespace()
        .nth(1)
        .expect("created <id> <title>")
        .to_string();

    let shown = stdout(&r.ank("claude-code@ank", &["show", &id]));
    let (_, after) = shown
        .split_once("\n---\n\n")
        .expect("show prints the frontmatter, then the body");
    let body = after
        .split("BLOCKED BY")
        .next()
        .expect("split yields at least one part");
    assert_eq!(
        body.trim_end_matches('\n'),
        BODY,
        "the piped body reached the entity changed:\n{shown}"
    );

    // And the two channels write the same file: a body that carried its origin
    // into the corpus would be a second spelling of the same field.
    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--title",
            "A flagged task",
            "--scope",
            "src/**",
            "--body",
            BODY,
        ],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let twin = stdout(&out).split_whitespace().nth(1).unwrap().to_string();
    assert_eq!(
        r.task_text(&id).split_once("---\n\n").unwrap().1,
        r.task_text(&twin).split_once("---\n\n").unwrap().1,
        "the pipe and the flag must produce the same body"
    );
}

/// `--body -` with nothing on stdin refuses, and names the flag it refuses on.
///
/// Accepting it would write the one entity `--body` exists to prevent — a task
/// with no reasoning, created silently by a pipeline that produced nothing. The
/// hint has to carry the flag, since the flag is what the caller has to fix.
#[test]
fn body_from_an_empty_stdin_is_refused_and_names_the_flag() {
    let r = Repo::new();

    let out = r.ank_stdin(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--title",
            "An empty task",
            "--scope",
            "src/**",
            "--body",
            "-",
        ],
        "",
    );
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(code(&out), 9, "{said}");
    assert!(said.contains("--body -"), "the flag is not named:\n{said}");
    assert!(
        said.contains("ank new task") && said.contains("| "),
        "the refusal must name the command to run next:\n{said}"
    );

    // Nothing was written: a refusal that left a half-made entity behind would
    // cost more than the one it refused.
    let out = r.ank("claude-code@ank", &["find", "--status", "open"]);
    assert!(
        !stdout(&out).contains("An empty task"),
        "the refused task was created anyway:\n{}",
        stdout(&out)
    );

    // Whitespace alone is the same emptiness, and the one a heredoc actually
    // produces when the variable it interpolates is unset.
    let out = r.ank_stdin(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--title",
            "A blank task",
            "--scope",
            "src/**",
            "--body",
            "-",
        ],
        "\n\n   \n",
    );
    assert_eq!(code(&out), 9, "{}{}", stdout(&out), stderr(&out));
}

/// A task created through the binary needs no hand finishing.
///
/// The assertion is about what lands on disk, so the file is read back rather
/// than the return value inspected: every task this repository created through
/// `ank new` had to be reopened afterwards to gain a `verify:` and a body, and
/// that reopening is the practice the task exists to end.
#[test]
fn a_task_created_by_new_declares_its_verifiers_and_carries_its_reasoning() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");

    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--title",
            "A created task",
            "--scope",
            "src/**",
            "--criteria",
            "A verifiable criterion.",
            "--verify",
            "ok",
            "--body",
            "Why this exists.\n\nAnd the trap worth naming.",
        ],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let id = String::from_utf8_lossy(&out.stdout)
        .split_whitespace()
        .nth(1)
        .expect("created <id> <title>")
        .to_string();

    let text = r.task_text(&id);
    assert!(text.contains("verify: [ok]"), "{text}");
    assert!(text.contains("Why this exists."), "{text}");
    assert!(text.contains("And the trap worth naming."), "{text}");
    // Canonical shape: a blank line after the frontmatter, and a final newline.
    // A creation that emitted anything else would be reformatted by the first
    // rewrite, and the round-trip is byte-identical on canonical form.
    assert!(text.contains("---\n\nWhy this exists."), "{text:?}");
    assert!(text.ends_with("worth naming.\n"), "{text:?}");

    // Complete means claimable and then finishable without touching the file:
    // `done` runs the declared verifier instead of demanding a proof the agent
    // supplies itself.
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", &id])), 0);
    let out = r.ank("claude-code@ank", &["done"]);
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
    assert_eq!(code(&out), 0, "{text}");
    assert!(text.contains("running: ok"), "{text}");
    assert!(
        r.task_text(&id).contains("verifier: ok@"),
        "{}",
        r.task_text(&id)
    );
}

/// `supersedes` existed in the model, `check` enforced the chain both ways and
/// `accept` completed it, while `commands.rs` wrote `supersedes: None`
/// unconditionally — everything built around a value nothing could write.
///
/// Through the binary and reading the file, because what is asserted is what
/// lands on disk: a resolution that never reached the serializer would pass a
/// unit test on the parsed entity and leave the field absent.
#[test]
fn new_adr_writes_the_succession_it_declares() {
    let r = Repo::new();
    let an_adr = |title: &str, extra: &[&str]| {
        let mut argv = vec![
            "new",
            "adr",
            "--title",
            title,
            "--scope",
            "src/**",
            "--constraint",
            "A binding rule.",
        ];
        argv.extend_from_slice(extra);
        let out = r.ank("claude-code@ank", &argv);
        (
            code(&out),
            String::from_utf8_lossy(&out.stdout).to_string(),
            stderr(&out),
        )
    };
    let id_of = |stdout: &str| {
        stdout
            .split_whitespace()
            .nth(1)
            .expect("created <id> <title>")
            .to_string()
    };

    let (c, stdout, err) = an_adr("The replaced", &[]);
    assert_eq!(c, 0, "{err}");
    let replaced = id_of(&stdout);

    // A short prefix resolves, exactly as `--blocked-by` does.
    let (c, stdout, err) = an_adr("The replacement", &["--supersedes", &replaced[..9]]);
    assert_eq!(c, 0, "{err}");
    let replacement = id_of(&stdout);

    let text = r.adr_text(&replacement);
    assert!(
        text.contains(&format!("supersedes: {replaced}")),
        "the prefix is resolved to the full id on disk:\n{text}"
    );
    // Proposed, and never born accepted: the succession happens at `accept`.
    assert!(text.contains("status: proposed"), "{text}");

    // A reference matching nothing is refused here rather than surfacing in
    // `check` as a corpus fault nobody can attribute to the act.
    let (c, _, err) = an_adr("Dangling", &["--supersedes", "ADR-ffffffffffff"]);
    assert_ne!(c, 0, "an unknown reference must not reach the file");
    assert!(err.contains("ffffffffffff"), "{err}");

    // And the flag is refused on a task, never dropped.
    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--title",
            "T",
            "--scope",
            "src/**",
            "--supersedes",
            &replaced,
        ],
    );
    assert_ne!(code(&out), 0, "a dropped flag teaches the caller it worked");
    assert!(stderr(&out).contains("--supersedes"), "{}", stderr(&out));
}

/// The two fields a plan actually changes on, and the two edits that were done
/// by hand this session because no command reached them.
///
/// Through the binary and reading the file, because what is asserted is that
/// the amendment lands on disk and that everything it did not name comes back
/// untouched — which is the whole difference between a verb and an editor.
#[test]
fn amend_adds_and_removes_without_disturbing_the_rest() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.seed_task("TASK-000000000002", Some("Another criterion."));
    r.seed_task("TASK-000000000003", Some("A third criterion."));
    let before = r.task_text(ID);

    // Nothing named is a refusal, not a silent no-op that bumps `version`.
    let out = r.ank("marie@laptop", &["amend", ID]);
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert_eq!(r.task_text(ID), before, "a refusal writes nothing");

    // A blocker added to a task that already exists, resolved by prefix.
    let out = r.ank(
        "marie@laptop",
        &[
            "amend",
            "0000000000 01",
            "--blocked-by",
            "TASK-000000000002",
            "--scope",
            "docs/**",
        ],
    );
    // The odd id above is deliberate: it must not resolve.
    assert_ne!(code(&out), 0, "a bad prefix is refused");

    let out = r.ank(
        "marie@laptop",
        &[
            "amend",
            ID,
            "--blocked-by",
            "TASK-000000000002",
            "--blocked-by",
            "TASK-000000000003",
            "--scope",
            "docs/**",
        ],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let text = r.task_text(ID);
    assert!(
        text.contains("blocked_by: [TASK-000000000002, TASK-000000000003]"),
        "{text}"
    );
    assert!(text.contains("  - src/**\n  - docs/**"), "{text}");
    assert!(text.contains("version: 2"), "the write increments: {text}");
    assert!(
        r.log_text(ID)
            .contains("amended: +blocked_by TASK-000000000002"),
        "the log says what changed: {}",
        r.log_text(ID)
    );
    assert!(
        r.log_text(ID).contains("+scope docs/**"),
        "{}",
        r.log_text(ID)
    );
    // Everything it did not name is still there, byte for byte.
    assert!(
        text.contains("done_criteria: |\n  A verifiable criterion."),
        "{text}"
    );

    // Removal is explicit, and removing what is not there is refused rather
    // than succeeding quietly.
    let out = r.ank(
        "marie@laptop",
        &["amend", ID, "--drop-blocked-by", "TASK-000000000003"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = r.task_text(ID);
    assert!(text.contains("blocked_by: [TASK-000000000002]"), "{text}");
    assert!(
        r.log_text(ID).contains("-blocked_by TASK-000000000003"),
        "{}",
        r.log_text(ID)
    );

    let out = r.ank("marie@laptop", &["amend", ID, "--drop-scope", "nowhere/**"]);
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("not in the scope"),
        "{}",
        stderr(&out)
    );

    // An entity attached to nothing is invisible, so the last glob cannot go.
    let out = r.ank(
        "marie@laptop",
        &[
            "amend",
            ID,
            "--drop-scope",
            "src/**",
            "--drop-scope",
            "docs/**",
        ],
    );
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(stderr(&out).contains("no scope"), "{}", stderr(&out));

    // Blanking the criterion is refused: a task with no done_criteria cannot be
    // claimed at all, so a flag that reads as an edit will not produce one.
    let out = r.ank("marie@laptop", &["amend", ID, "--criteria", "   "]);
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(
        r.task_text(ID).contains("A verifiable criterion."),
        "the criterion did not move"
    );
}

/// A criterion that turns out unmeasurable is corrected, by a caller holding no
/// claim, without the correction being recorded as the claimer's.
///
/// The case is real and is what TASK-7c2fa14284ff was filed for: a criterion
/// ending on a clause no repository of that shape could ever satisfy. The work
/// was right and the measurement was not, and the corrected criterion had no way
/// back in that did not lie about who wrote it.
#[test]
fn a_criterion_under_no_claim_is_amended_and_stays_the_creators() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion that cannot be measured."));

    let out = r.ank(
        "marie@laptop",
        &["amend", ID, "--criteria", "A criterion that can."],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let text = r.task_text(ID);
    assert!(
        text.contains("done_criteria: |\n  A criterion that can.\n"),
        "the criterion moved: {text}"
    );
    assert!(
        !text.contains("cannot be measured"),
        "the old criterion is gone: {text}"
    );
    // The heart of it. `criteria_by` answers whether the criterion was set at
    // claim time by the party the freeze constrains, and an amend is not a
    // claim — writing `claimer` here would launder a correction into the shape
    // the signal exists to expose.
    assert!(
        text.contains("criteria_by: creator"),
        "the correction was recorded as somebody else's: {text}"
    );
    assert!(
        r.log_text(ID).contains("amended: done_criteria"),
        "the log is what records the amend: {}",
        r.log_text(ID)
    );

    // And the corpus is clean afterwards: the point of the route is that it
    // leaves nothing for `check` to report.
    let out = r.ank("marie@laptop", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        !stdout(&out).contains("altered") && !stdout(&out).contains("claimer"),
        "{}",
        stdout(&out)
    );

    // Amending it to what it already says changes nothing, and says so rather
    // than writing a version nobody asked for.
    let out = r.ank(
        "marie@laptop",
        &["amend", ID, "--criteria", "A criterion that can."],
    );
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("already reads that way"),
        "{}",
        stderr(&out)
    );
}

/// The freeze still holds where it means something: under a live claim.
///
/// Through the binary and against a real claim ref, because "a live claim" is a
/// fact about `refs/ank/claims/`, not about a struct — and because the refusal
/// must not consult who is calling. The claimer itself is refused, which is the
/// whole point: it is the party the freeze constrains.
#[test]
fn a_criterion_under_a_live_claim_is_refused_to_everyone_alike() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task(ID, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);

    for who in ["marie@laptop", "claude-code@ank"] {
        let out = r.ank(who, &["amend", ID, "--criteria", "Something easier."]);
        let err = stderr(&out);
        assert_eq!(code(&out), 6, "{who}: {err}");
        assert!(err.contains("frozen"), "{who}: {err}");
        assert!(
            err.contains("claude-code@ank"),
            "the refusal names the holder: {who}: {err}"
        );
        assert!(
            err.contains("ank release"),
            "the command that applies: {who}: {err}"
        );
    }
    assert!(
        r.task_text(ID).contains("A verifiable criterion."),
        "the criterion did not move"
    );

    // Released, the same call goes through: the refusal was on the claim and
    // never on the caller.
    assert_eq!(
        code(&r.ank(
            "claude-code@ank",
            &["release", ID, "--reason", "unmeasurable"]
        )),
        0
    );
    let out = r.ank(
        "marie@laptop",
        &["amend", ID, "--criteria", "Something measurable."],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(r.task_text(ID).contains("Something measurable."));
}

/// `claim --criteria` sets a criterion, and never replaces one.
///
/// The door `amend` used to bolt shut was standing open here, and open to the
/// claimer -- the one party the freeze constrains. Closing it is what keeps
/// `criteria_by: claimer` meaning exactly one thing.
#[test]
fn claim_criteria_sets_an_absent_criterion_and_refuses_to_replace_one() {
    let r = Repo::new();
    r.seed_task(ID, Some("The criterion its creator wrote."));

    let out = r.ank(
        "claude-code@ank",
        &["claim", ID, "--criteria", "Something easier."],
    );
    let err = stderr(&out);
    assert_eq!(code(&out), 6, "{err}");
    assert!(err.contains("already carries a done_criteria"), "{err}");
    assert!(
        err.contains("ank amend") && err.contains("--criteria"),
        "the refusal names the route that applies: {err}"
    );
    let text = r.task_text(ID);
    assert!(text.contains("The criterion its creator wrote."), "{text}");
    assert!(text.contains("criteria_by: creator"), "{text}");
    // Refused before any ref was touched: the preconditions of §3 run before
    // the claim, so nothing is left half-taken.
    assert!(
        r.claim_ref(ID).is_none(),
        "a refused claim took the task anyway"
    );

    // A task carrying none is still claimed and set in one call, and that is
    // still recorded as the claimer's.
    const BARE: &str = "TASK-000000000009";
    r.seed_task(BARE, None);
    let out = r.ank(
        "claude-code@ank",
        &["claim", BARE, "--criteria", "A verifiable criterion."],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = r.task_text(BARE);
    assert!(text.contains("A verifiable criterion."), "{text}");
    assert!(text.contains("criteria_by: claimer"), "{text}");
}

#[test]
fn new_refuses_a_verifier_that_config_does_not_declare() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    let out = r.ank(
        "claude-code@ank",
        &[
            "new", "task", "--title", "T", "--scope", "src/**", "--verify", "nope",
        ],
    );
    let err = stderr(&out);
    // Resolved at creation for the same reason --blocked-by is: a name that
    // matches nothing would otherwise surface at `done`, far from its cause.
    assert_eq!(code(&out), 7, "{err}");
    assert!(err.contains("nope"), "{err}");
    assert!(err.contains("ok"), "the hint names what is declared: {err}");

    // And an ADR has no verify field, so the flag is refused, never dropped.
    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "adr",
            "--title",
            "T",
            "--scope",
            "src/**",
            "--constraint",
            "A binding rule.",
            "--verify",
            "ok",
        ],
    );
    assert_ne!(code(&out), 0, "a dropped flag teaches the caller it worked");
    assert!(stderr(&out).contains("--verify"), "{}", stderr(&out));
}

/// A claim that lapsed and left the file behind.
///
/// Through the binary because the thing being read is the ref namespace, and a
/// unit test would assert the module's agreement with itself. The state is built
/// the way it actually arises — a real claim, then the ref goes away — rather
/// than by writing `status: in_progress` into a file by hand, which would test a
/// corpus nobody produces.
///
/// TASK-daf25ab8a9b7 sat in exactly this state through fifteen signals on this
/// repository's own corpus, and not one of them was about it.
#[test]
fn a_task_in_progress_with_no_claim_ref_is_signalled_and_never_fails_ci() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    assert!(r.claim_ref(ID).is_some(), "the claim must have been taken");
    r.git(&["update-ref", "-d", &format!("refs/ank/claims/{ID}")]);
    assert!(r.claim_ref(ID).is_none(), "the ref must be gone");
    assert!(
        r.task_text(ID).contains("status: in_progress"),
        "the file must still claim to be held"
    );

    let out = r.ank("claude-code@ank", &["check"]);
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
    assert_eq!(
        code(&out),
        0,
        "a stale record is not a corrupt corpus and must never fail CI: {text}"
    );
    assert!(text.contains("signal:"), "{text}");
    assert!(text.contains(ID), "the finding must name the task: {text}");
    assert!(text.contains("no claim ref"), "{text}");
    assert!(
        text.contains(&format!("ank claim {ID}")),
        "the finding must carry the command that picks it up: {text}"
    );

    // And it stops the moment something holds the task again, so the signal
    // tracks the state rather than the history.
    assert_eq!(code(&r.ank("codex@host-9", &["claim", ID])), 0);
    let out = r.ank("claude-code@ank", &["check"]);
    let text = format!("{}{}", String::from_utf8_lossy(&out.stdout), stderr(&out));
    assert_eq!(code(&out), 0, "{text}");
    assert!(!text.contains("no claim ref"), "{text}");
}

// ---------------------------------------------------------------------------
// help (§9)
// ---------------------------------------------------------------------------

/// `help` outside any repository, which is the case that matters: the CLI it is
/// bundled with is installed as a skill, and SKILL.md sends the reader to
/// `ank help` before anything establishes where the reader is standing. A help
/// that demands a `.ank/` is a help withheld from the agent most likely to need
/// it.
///
/// `current_dir` is the system temp directory, which is not a git repository
/// and has no `.ank/`, and no `--repo` is passed. Anything that goes through
/// `startup` answers 9 or 1 from there.
fn help_from_nowhere(args: &[&str]) -> Output {
    ank_command()
        .args(args)
        .current_dir(std::env::temp_dir())
        .output()
        .expect("the binary must have been built")
}

/// The body of `ank help`: every non-empty line above the trailer, headings
/// included.
///
/// The trailer is the block opening with `global:` at column 0, and it is what
/// bounds the listing now that a blank line no longer does. The listing is
/// grouped (ADR-f61e2d2c75e8), so an empty line is a boundary *between* groups
/// -- a parser that stopped at the first one would read `run the loop`, find six
/// verbs, and call that the whole surface. A folded description is indented and
/// so can never be mistaken for the trailer.
fn listing_body(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .take_while(|l| !l.starts_with("global:"))
        .filter(|l| !l.trim().is_empty())
}

/// The verbs `ank help --json` carries: name, usage and group, in table order.
///
/// Scanned rather than parsed, since the suite carries no JSON dependency, and
/// read back from the binary rather than restated here for the reason
/// `tests/skill.rs` reads §4 rather than restating it — `ank-cli` has no library
/// target, so `COMMANDS` is unreachable from an integration test, and a second
/// hand-maintained copy of the surface is the drift being checked for.
///
/// Splitting on `{"name":"` bounds each object at the next one, so a flag —
/// which carries a `name` too — becomes a chunk of its own, and is told apart by
/// carrying no `usage`.
fn json_verbs(text: &str) -> Vec<(String, String, String)> {
    let mut verbs = Vec::new();
    for chunk in text.split("{\"name\":\"").skip(1) {
        let Some((name, tail)) = chunk.split_once('"') else {
            continue;
        };
        let quoted = |key: &str| -> Option<String> {
            let head = format!(",\"{key}\":\"");
            let at = tail.find(&head)? + head.len();
            tail[at..].find('"').map(|e| tail[at..at + e].to_string())
        };
        let (Some(usage), Some(group)) = (quoted("usage"), quoted("group")) else {
            continue;
        };
        verbs.push((name.to_string(), usage, group));
    }
    verbs
}

/// The listing parsed into its headings, each with the verbs printed under it.
fn listing_groups(listing: &str) -> Vec<(String, Vec<String>)> {
    let mut groups: Vec<(String, Vec<String>)> = Vec::new();
    for line in listing_body(listing) {
        match line.strip_prefix("ank ") {
            Some(rest) => {
                let verb: String = rest.chars().take_while(char::is_ascii_lowercase).collect();
                assert!(!verb.is_empty(), "unparseable listing line: {line}");
                groups
                    .last_mut()
                    .unwrap_or_else(|| panic!("a verb stands above every heading:\n{listing}"))
                    .1
                    .push(verb);
            }
            // A line at column 0 that opens no verb is a heading; an indented
            // one continues the description above it.
            None if !line.starts_with(' ') => groups.push((line.to_string(), Vec::new())),
            None => {}
        }
    }
    groups
}

#[test]
fn help_answers_outside_a_repository_and_lists_every_verb() {
    let out = help_from_nowhere(&["help"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(
        code(&out),
        0,
        "help must answer without a repository: {}",
        stderr(&out)
    );

    // Listed by hand here on purpose: cli.rs walks its own table, which would
    // agree with itself even if the table were wrong, and the criterion is
    // about what an agent reads.
    for verb in [
        "context", "claim", "log", "done", "release", "new", "find", "review", "accept", "close",
        "check", "show", "init", "help", "attest",
    ] {
        assert!(
            text.contains(&format!("ank {verb}")),
            "{verb} missing:\n{text}"
        );
    }

    // Grouped by the moment a verb is used, and never by who uses it
    // (ADR-f61e2d2c75e8). These four headings sorted callers -- agent loop,
    // agent off-loop, human -- which is the two-surface model speaking through
    // the output an agent reads, and a CLI that refuses on state and never on
    // identity has no such grouping to print. That refusal is untouched: what
    // came back is a map, not a gate.
    for heading in ["agent loop", "off-loop", "human", "setup"] {
        assert!(
            !text.contains(heading),
            "'{heading}' groups the listing by caller:\n{text}"
        );
    }

    // Grouped is not sorted: §4 puts the loop first, and its order survives
    // inside every group. Alphabetical would bury `context` between `close` and
    // `done`, so the order is asserted through the binary rather than trusted.
    let at = |needle: &str| {
        text.find(needle)
            .unwrap_or_else(|| panic!("{needle} missing:\n{text}"))
    };
    assert!(at("ank context") < at("ank done"), "{text}");
    assert!(at("ank done") < at("ank release"), "{text}");
    assert!(at("ank release") < at("ank review"), "{text}");
    assert!(at("ank review") < at("ank check"), "{text}");
    assert!(
        text.starts_with("run the loop\nank context"),
        "the listing opens on something other than the first moment and the \
         first verb of it:\n{text}"
    );
    // And a description beside each verb, which is what the listing says that
    // a bare flag name never did (§9, TASK-fe130d2b732c).
    for said in [
        "what binds this perimeter",
        "freezes its done_criteria by hash",
        "runs the verifiers the task's verify: list names",
    ] {
        assert!(text.contains(said), "{said:?} missing:\n{text}");
    }

    // The flags each verb takes are the part §9 keeps out of SKILL.md, and
    // they moved to the per-verb page when the description took their place.
    // Fetched from nowhere too: what is under test is that none of this needs
    // a repository.
    let page = |verb: &str| -> String {
        let out = help_from_nowhere(&["help", verb]);
        assert_eq!(code(&out), 0, "{}", stderr(&out));
        String::from_utf8_lossy(&out.stdout).to_string()
    };
    for (verb, flag) in [
        ("context", "--limit"),
        ("claim", "--ttl"),
        ("claim", "--criteria"),
        ("done", "--proof"),
        ("release", "--reason"),
        ("new", "--scope"),
    ] {
        let p = page(verb);
        assert!(
            p.contains(flag),
            "{flag} missing from ank help {verb}:\n{p}"
        );
    }
    assert!(text.contains("--json"), "{text}");
}

#[test]
fn help_for_one_verb_answers_and_an_unknown_one_is_a_two() {
    let out = help_from_nowhere(&["help", "claim"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(text.contains("ank claim <id>"), "{text}");
    assert!(text.contains("--ttl"), "{text}");
    assert!(text.contains("--criteria"), "{text}");
    assert!(
        !text.contains("audience"),
        "the audience line is what ADR-c656cbcc33a9 removes, and it is the \
         line an agent reads about itself:\n{text}"
    );
    assert!(
        !text.contains("ank accept"),
        "one verb means one verb:\n{text}"
    );

    // The exit code is the thing being asserted, and 2 is "entity not found"
    // in the table of §4 -- the verb looked for did not exist.
    let out = help_from_nowhere(&["help", "clam"]);
    let err = stderr(&out);
    assert_eq!(
        code(&out),
        2,
        "stdout={} stderr={err}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        err.contains("clam"),
        "the message must name what was looked for: {err}"
    );
    assert!(err.contains("ank help"), "{err}");
    assert!(
        out.stdout.is_empty(),
        "a silent fallback to the general listing: {}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// `ank help done` says where a verifier comes from.
///
/// It said the verb "runs the declared verifiers" and refuses when "no verifier
/// [is] declared to produce one", leaving out who declares. `config.yml` is
/// where verifiers are defined, so a reader filled the blank with it and
/// concluded `check-repo` and `cargo-test` would run; no task in this corpus
/// names them in a `verify:` list, so `done` always demands `--proof`. That
/// agent wrote the wrong reading into the project guide and found out by running
/// the command. Selection is the task's, and the page has to say so.
#[test]
fn help_done_says_a_verifier_comes_from_the_tasks_verify_list() {
    let page = String::from_utf8_lossy(&help_from_nowhere(&["help", "done"]).stdout).to_string();
    assert!(page.contains("verify:"), "{page}");
    assert!(
        page.contains("config.yml defines the verifiers"),
        "the page must name what config.yml does and does not decide:\n{page}"
    );
    assert!(
        !page.contains("the declared verifiers"),
        "'declared' with no declarer is the reading that misled:\n{page}"
    );
}

/// `ank help check` says the verb writes.
///
/// A verb called `check` reads as read-only, and this one prunes — it is the
/// only command that does (§7). An agent ran it freely in a loop on that
/// assumption and read `pruned refs/ank/claims/...` back mid-output. The
/// behaviour is correct and stays; the page is where a caller finds out before
/// scripting around it.
#[test]
fn help_check_says_the_verb_prunes_refs() {
    let page = String::from_utf8_lossy(&help_from_nowhere(&["help", "check"]).stdout).to_string();
    assert!(
        page.contains("prunes"),
        "a caller must not have to run it to learn it writes:\n{page}"
    );
    assert!(
        page.contains("refs/ank/claims"),
        "and which refs it prunes:\n{page}"
    );

    // The listing carries it too: an agent that scripts `check` reads the flat
    // page far more often than the per-verb one.
    let listing = String::from_utf8_lossy(&help_from_nowhere(&["help"]).stdout).to_string();
    assert!(listing.contains("prunes"), "{listing}");
}

#[test]
fn help_json_reaches_the_process_intact() {
    let out = help_from_nowhere(&["help", "--json"]);
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(text.starts_with("{\"verbs\":["), "{text}");
    assert!(text.trim_end().ends_with("]}"), "{text}");
    assert!(text.contains("\"name\":\"claim\""), "{text}");
    assert!(
        !text.contains("audience"),
        "a grouping by caller survived into the scripted output:\n{text}"
    );
}

/// Every verb of the table appears exactly once in the grouped listing, under a
/// heading, and in the table's order inside it (ADR-f61e2d2c75e8).
///
/// Through the binary, against `ank help --json`: the same table read out of the
/// same process, so the assertion is that the two renderings agree rather than
/// that either agrees with a list written here. A renderer that walks the groups
/// instead of the table is a renderer that can drop a verb — one whose group is
/// not a heading is printed by nothing, and `--json` goes on carrying it while
/// the surface an agent reads has quietly lost it. Exactly once, because the
/// other way for a group to be wrong is a verb filed under two moments.
#[test]
fn the_grouped_listing_prints_every_verb_exactly_once() {
    let listing = String::from_utf8_lossy(&help_from_nowhere(&["help"]).stdout).to_string();
    let json = String::from_utf8_lossy(&help_from_nowhere(&["help", "--json"]).stdout).to_string();

    let verbs = json_verbs(&json);
    assert!(verbs.len() > 15, "the table was not read back:\n{json}");

    let groups = listing_groups(&listing);
    assert!(!groups.is_empty(), "the listing has no heading:\n{listing}");
    let listed: Vec<&String> = groups.iter().flat_map(|(_, v)| v).collect();

    for (name, usage, _) in &verbs {
        assert_eq!(
            listed.iter().filter(|v| **v == name).count(),
            1,
            "`ank {name}` is not printed exactly once in the grouped listing:\n{listing}"
        );
        // The usage line and not only the name: a verb reduced to a heading
        // entry would satisfy the count and teach nothing.
        assert_eq!(
            listing.matches(usage.as_str()).count(),
            1,
            "`{usage}` is not printed exactly once:\n{listing}"
        );
    }
    assert_eq!(
        listed.len(),
        verbs.len(),
        "the listing prints something the table does not hold:\n{listing}"
    );

    // Inside a group the order is the table's: grouping is a second axis laid
    // over §4's order, never a re-sort, so a verb does not move relative to its
    // neighbours. Asserted per group, since the listing as a whole no longer
    // follows the table and a global check would have to be dropped rather than
    // weakened.
    for (heading, printed) in &groups {
        assert!(!printed.is_empty(), "'{heading}' has no verb under it");
        let expected: Vec<&String> = verbs
            .iter()
            .filter(|(_, _, g)| g == heading)
            .map(|(n, _, _)| n)
            .collect();
        let printed: Vec<&String> = printed.iter().collect();
        assert_eq!(
            printed, expected,
            "'{heading}' does not keep the order of the table:\n{listing}"
        );
    }

    // A blank line between groups, and a heading opening each one. The first
    // heading opens the listing: nothing is titled above the loop.
    assert!(
        listing.starts_with(&format!("{}\n", groups[0].0)),
        "something stands above the first heading:\n{listing}"
    );
    for (heading, _) in groups.iter().skip(1) {
        assert!(
            listing.contains(&format!("\n\n{heading}\n")),
            "'{heading}' does not open a group of its own:\n{listing}"
        );
    }
}

/// Every verb carries a non-empty group, and every group is a heading the
/// listing prints (ADR-f61e2d2c75e8).
///
/// This is what stops a twenty-second verb from being added with no home and
/// disappearing off the end unnoticed. The test above would pass on such a verb
/// only if the renderer happened to print it anyway; this one fails on the
/// table, which is where the omission is made, and it reads the table through
/// `--json` because that is where the field has to reach a caller.
#[test]
fn every_verb_carries_a_group_and_no_group_goes_unprinted() {
    let listing = String::from_utf8_lossy(&help_from_nowhere(&["help"]).stdout).to_string();
    let json = String::from_utf8_lossy(&help_from_nowhere(&["help", "--json"]).stdout).to_string();

    let verbs = json_verbs(&json);
    assert!(verbs.len() > 15, "the table was not read back:\n{json}");

    let headings: Vec<String> = listing_groups(&listing)
        .into_iter()
        .map(|(h, _)| h)
        .collect();
    assert!(
        !headings.is_empty(),
        "the listing has no heading:\n{listing}"
    );

    for (name, _, group) in &verbs {
        assert!(
            !group.trim().is_empty(),
            "`ank {name}` carries no group, so nothing prints it:\n{json}"
        );
        assert_eq!(
            *group,
            group.to_lowercase(),
            "'{group}' is titled rather than signposted: the headings are lowercase"
        );
        assert!(
            headings.contains(group),
            "`ank {name}` carries the group '{group}', which the listing gives no \
             heading:\n{listing}"
        );
        // And a group says when a verb is used, never who may use it: the wall
        // ADR-c656cbcc33a9 pulled down was built out of exactly these words.
        for who in ["agent", "human", "caller", "you"] {
            assert!(
                !group.split_whitespace().any(|w| w == who),
                "'{group}' sorts callers, which is the one thing a heading here \
                 must never do"
            );
        }
    }

    // The other half: a heading with nothing under it is a group that has lost
    // its verbs, which is the same defect seen from the listing's side.
    for heading in &headings {
        assert!(
            verbs.iter().any(|(_, _, g)| g == heading),
            "'{heading}' is printed with no verb of the table under it:\n{listing}"
        );
    }
}

// ---------------------------------------------------------------------------
// Dead constraints in the prose (ADR-c656cbcc33a9)
// ---------------------------------------------------------------------------

/// A comment citing a superseded ADR as the reason for a design is worse than
/// no comment: it hands the next reader a constraint that binds nobody, with
/// the authority of a decision record. Two of them went on asserting a frozen
/// agent surface -- at seven verbs in one, at eight in the other -- in module
/// headers and doc comments, long after the split they protected had been
/// dissolved and while the file around them had already stopped obeying it.
///
/// The needles are assembled from fragments so this file does not defeat its
/// own assertion by containing what it forbids. It reads as an affectation
/// until you picture the test failing on itself.
///
/// The third arrived the same way and is the reason the list is worth keeping:
/// thirteen comments, across six modules and this file, explained the
/// completion ref by the ADR that created it, and a human ratified its
/// successor on the default branch. The citations were right until that
/// signature and wrong the instant after, which is why re-pointing them is a
/// task of its own (TASK-d9d364bad929) and why the list holds the result rather
/// than a reader's memory.
///
/// This is not a general ban on naming a superseded ADR: history is worth
/// writing down, and `.ank/` is where it is written. It is a ban on these three,
/// which have no live claim left to make anywhere in this crate.
#[test]
fn no_superseded_adr_is_cited_in_the_crate() {
    const DEAD: [&str; 3] = [
        concat!("ADR-", "2f8a61c04b7d"),
        concat!("ADR-", "3859eb46bdc3"),
        concat!("ADR-", "bcf222a31525"),
    ];

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut walked = 0usize;
    for dir in ["src", "tests"] {
        for entry in std::fs::read_dir(root.join(dir)).expect("the crate has this directory") {
            let path = entry.expect("a readable entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a readable source file");
            for dead in DEAD {
                assert!(
                    !text.contains(dead),
                    "{} cites {dead}, which is superseded: name the ADR that \
                     binds today, or drop the citation",
                    path.display()
                );
            }
            walked += 1;
        }
    }
    // A walk that quietly found nothing would pass forever, which is the one
    // way a test like this fails at being a test.
    assert!(walked >= 8, "only {walked} source files walked");

    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("a readable manifest");
    for dead in DEAD {
        assert!(!manifest.contains(dead), "Cargo.toml cites {dead}");
    }
}

// ---------------------------------------------------------------------------
// edit
// ---------------------------------------------------------------------------
//
// Through the binary, and not only because the criterion says so: `edit` spends
// most of its life in a child process and in a file outside the corpus, and
// there is no in-process test that could reach either.

/// The seeded task, written back out of order and with two fields moved. What
/// comes out must be canonical form regardless of how the editor left it (§3).
const EDITED_TASK: &str = "---\ntype: task\nid: TASK-000000000001\nstatus: open\n\
     title: A better title\nslug: example\ncreated: 2026-07-28T00:00:00Z\n\
     scope:\n  - src/**\nblocked_by: []\ndone_criteria: |\n  A verifiable criterion.\n\
     criteria_by: creator\nschema: 1\nversion: 1\n---\n\nRewritten body.\n";

#[test]
fn edit_writes_back_what_the_editor_saved_in_canonical_form() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let editor = r.editor_saving(EDITED_TASK);

    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some(&editor));
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let said = stdout(&out);
    assert!(said.starts_with(&format!("edited {ID}")), "{said}");
    // Named field by field, so that a reader can tell a title fix from a scope
    // change without reaching for the diff.
    assert!(said.contains("title") && said.contains("body"), "{said}");
    assert!(said.contains("version 2"), "{said}");

    let on_disk = r.task_text(ID);
    // Canonical order, not the editor's: `id`, then `type`, then `slug`,
    // whatever the file that came back said.
    assert!(
        on_disk.starts_with(
            "---\nid: TASK-000000000001\ntype: task\nslug: example\ntitle: A better title\n"
        ),
        "{on_disk}"
    );
    assert!(on_disk.contains("\nversion: 2\n"), "{on_disk}");
    assert!(on_disk.ends_with("---\n\nRewritten body.\n"), "{on_disk}");
}

/// Code 9 and nothing guessed (§4). An editor chosen on the caller's behalf
/// would open something they never asked for, on a file they are about to
/// commit.
#[test]
fn edit_without_an_editor_is_an_environment_failure_and_touches_nothing() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let before = r.task_text(ID);

    let out = r.ank_edit("claude-code@ank", &["edit", ID], None);
    assert_eq!(code(&out), 9, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("EDITOR is not set"), "{err}");
    assert!(
        err.contains(&format!("EDITOR=vi ank edit {ID}")),
        "the exact command to run next, never generic help: {err}"
    );
    assert_eq!(r.task_text(ID), before, "the entity is untouched");
}

/// The clause the temporary copy exists for: nothing reaches `.ank/` until the
/// text has parsed, so a mistyped frontmatter costs a re-edit and never a
/// corrupt file. And the text survives the refusal, which is the difference
/// between a validation and a punishment.
#[test]
fn an_invalid_result_leaves_the_entity_untouched_and_says_why() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let before = r.task_text(ID);
    let broken = "---\nid: TASK-000000000001\ntype: task\ntitle: Half a file\n";
    let editor = r.editor_saving(broken);

    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some(&editor));
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("does not parse"), "{err}");
    assert!(err.contains("missing frontmatter"), "says why: {err}");
    assert_eq!(r.task_text(ID), before, "byte for byte");

    // The named file is real and holds what the editor saved. A message that
    // pointed at nothing would be worse than no message.
    let kept = err
        .split("kept at ")
        .nth(1)
        .and_then(|s| s.split(')').next())
        .expect("the refusal names where the text was kept");
    assert_eq!(
        std::fs::read_to_string(kept.trim()).expect("the kept file exists"),
        broken
    );
    let _ = std::fs::remove_file(kept.trim());
}

/// `done_criteria` is frozen at claim (§3), and the refusal names the command
/// that legally performs the change rather than the flag the caller reached for.
#[test]
fn a_claimed_criterion_is_refused_and_names_release() {
    let r = Repo::new().with_verifiers("");
    r.seed_task(ID, Some("A verifiable criterion."));
    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let before = r.task_text(ID);

    let weakened = EDITED_TASK.replace("A verifiable criterion.", "Something easier.");
    let editor = r.editor_saving(&weakened);
    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some(&editor));
    assert_eq!(code(&out), 6, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("frozen by the claim"), "{err}");
    assert!(err.contains("ank release --reason"), "{err}");
    assert_eq!(r.task_text(ID), before, "the entity is untouched");

    // The guard is on the field, not on the claim: the same task, the same
    // claim, an edit that leaves the criterion alone. Refusing this would make
    // `edit` useless exactly where the work happens.
    let editor = r.editor_saving(EDITED_TASK);
    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some(&editor));
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        r.task_text(ID).contains("A better title"),
        "{}",
        r.task_text(ID)
    );
}

/// `constraint` and `scope` are hashed into the signed ratification commit
/// (§8), so changing a ratified decision is a succession — and succession has
/// its own verb.
#[test]
fn a_ratified_constraint_is_refused_and_names_the_succession() {
    const ADR: &str = "ADR-0000000000ab";
    let r = Repo::new();
    r.enable_signing();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let before = r.adr_text(ADR);

    let altered = before.replace("Do not do X.", "Do not do Y.");
    let editor = r.editor_saving(&altered);
    let out = r.ank_edit("marie@laptop", &["edit", ADR], Some(&editor));
    assert_eq!(code(&out), 6, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("ratified"), "{err}");
    assert!(err.contains("ank new adr --supersedes"), "{err}");
    assert_eq!(r.adr_text(ADR), before, "the entity is untouched");

    // The body of an accepted ADR stays editable (§3): what is anchored is the
    // constraint and the scope, and the refusal must not reach past them.
    let reasoned = before.replace("Why.", "Why, at length.");
    let editor = r.editor_saving(&reasoned);
    let out = r.ank_edit("marie@laptop", &["edit", ADR], Some(&editor));
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("body"), "{}", stdout(&out));
}

/// An editor opened and closed without saving writes nothing at all — no
/// version bump, no rewrite. The alternative is a verb that dirties a file for
/// having been looked at.
#[test]
fn an_editor_that_saves_nothing_writes_nothing() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let before = r.task_text(ID);

    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some("true"));
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).starts_with(&format!("unchanged {ID}")),
        "{}",
        stdout(&out)
    );
    assert_eq!(r.task_text(ID), before);
}

/// An editor that fails is the environment failing, not the corpus refusing —
/// the same reading `verify` applies to a shell that cannot run what it is
/// given, and the same code.
#[test]
fn an_editor_that_exits_non_zero_leaves_the_entity_untouched() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let before = r.task_text(ID);

    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some("false"));
    assert_eq!(code(&out), 9, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("the editor exited 1"),
        "{}",
        stderr(&out)
    );
    assert_eq!(r.task_text(ID), before);
}

// ---------------------------------------------------------------------------
// new, interactive
// ---------------------------------------------------------------------------
//
// The `git commit` pattern: no flags, an editor (§4). The flag form is what
// SKILL.md teaches and what an agent uses, and the tests that matter most here
// are the ones asserting nothing about it moved.

/// The entities a repository holds, read off the disk rather than through the
/// tool: what has to be true is the state of the corpus, not `find`'s agreement
/// with itself.
/// The entity files of the corpus, of one kind. One directory holds every kind
/// since schema 3, so the kind is read off the file name rather than off the
/// path (§6).
fn entity_files(r: &Repo, kind: &str) -> Vec<String> {
    let prefix = match kind {
        "tasks" => "TASK-",
        "adr" => "ADR-",
        other => panic!("no such kind: {other}"),
    };
    let dir = r.0.join(".ank").join("entities");
    let mut v: Vec<String> = std::fs::read_dir(&dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .filter(|n| n.ends_with(".md") && n.starts_with(prefix))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

#[test]
fn new_task_without_flags_opens_a_template_and_writes_what_comes_back() {
    let r = Repo::new();
    let editor = editor_filling("A task typed in an editor", "crates/**", "The reasoning.");

    let out = r.ank_edit("claude-code@ank", &["new", "task"], Some(&editor));
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("created TASK-")
            && stdout(&out).contains("A task typed in an editor"),
        "{}",
        stdout(&out)
    );

    let files = entity_files(&r, "tasks");
    assert_eq!(files.len(), 1, "exactly one task: {files:?}");
    let text = std::fs::read_to_string(r.0.join(".ank/entities").join(&files[0])).unwrap();

    // The guidance rides in YAML comments and the parser drops them. A template
    // whose help text reached the corpus would put it in every `show` forever.
    assert!(!text.contains('#'), "the comments are stripped: {text}");
    assert!(text.contains("title: A task typed in an editor"), "{text}");
    assert!(
        text.contains("  - crates/**"),
        "canonical block scope: {text}"
    );
    // Derived, not typed: the template never asks for a slug.
    assert!(text.contains("slug: a-task-typed-in-an-editor"), "{text}");
    assert!(
        text.ends_with("The reasoning.\n"),
        "the body survives: {text}"
    );
    assert!(
        text.contains("status: open") && text.contains("version: 1"),
        "{text}"
    );
}

#[test]
fn new_adr_without_flags_opens_a_template_and_writes_what_comes_back() {
    let r = Repo::new();
    // The constraint is the ADR's mandatory field, and the template leaves it an
    // empty block for the caller to fill.
    let editor = editor_filling_and(
        "A decision typed in an editor",
        "docs/**",
        r"s|^  $|  Do not do X.|",
    );

    let out = r.ank_edit("marie@laptop", &["new", "adr"], Some(&editor));
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let files = entity_files(&r, "adr");
    assert_eq!(files.len(), 1, "exactly one adr: {files:?}");
    let text = std::fs::read_to_string(r.0.join(".ank/entities").join(&files[0])).unwrap();
    assert!(text.contains("Do not do X."), "{text}");
    // Never born accepted and never born anchored: ratification is a signed
    // commit produced by `accept`, and an ADR that arrived ratified would bind
    // before anyone agreed to it.
    assert!(text.contains("status: proposed"), "{text}");
    assert!(!text.contains("ratified:"), "{text}");
}

/// The clause that keeps the interactive form from being the hole in the wall:
/// it refuses what the flag form refuses, in the flag form's words and with the
/// flag form's code.
#[test]
fn a_template_saved_untouched_is_refused_and_creates_nothing() {
    let r = Repo::new();

    let out = r.ank_edit("claude-code@ank", &["new", "task"], Some("true"));
    assert_eq!(
        code(&out),
        7,
        "the same code the flag form gives an empty scope: {}",
        stderr(&out)
    );
    let err = stderr(&out);
    assert!(err.contains("empty scope"), "says why: {err}");
    assert!(
        err.contains("ank new task --title"),
        "names the flag form: {err}"
    );
    assert!(
        entity_files(&r, "tasks").is_empty(),
        "nothing is created: {:?}",
        entity_files(&r, "tasks")
    );
}

/// §4 requires this one by name: `$EDITOR` unset is an environment failure, and
/// what it names as the way through is the flag form.
#[test]
fn new_without_an_editor_names_the_flag_form() {
    let r = Repo::new();

    let out = r.ank_edit("claude-code@ank", &["new", "task"], None);
    assert_eq!(code(&out), 9, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("EDITOR is not set"), "{err}");
    assert_eq!(
        err.lines().nth(1).map(str::trim),
        Some(r#"-> ank new task --title "<t>" --scope "<glob>""#),
        "{err}"
    );

    let out = r.ank_edit("marie@laptop", &["new", "adr"], None);
    assert_eq!(code(&out), 9, "{}", stderr(&out));
    assert!(stderr(&out).contains("--constraint"), "{}", stderr(&out));
    assert!(entity_files(&r, "tasks").is_empty());
}

/// The property the whole trigger rule was chosen for. A script that forgot
/// `--scope` has to stop, not sit in an editor waiting for somebody who is not
/// there — so a partial invocation stays the flag form and fails as it always
/// did, with an editor set and willing.
#[test]
fn the_flag_form_is_untouched_by_the_interactive_one() {
    let r = Repo::new();
    // An editor that would happily create a valid entity if it were ever run.
    // Reaching it is the failure this test exists to catch.
    let editor = editor_filling("Should never be created", "src/**", "x");

    let out = r.ank_edit(
        "claude-code@ank",
        &["new", "task", "--title", "Half an invocation"],
        Some(&editor),
    );
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("a scope is required"),
        "{}",
        stderr(&out)
    );
    assert!(
        entity_files(&r, "tasks").is_empty(),
        "the editor was reached: {:?}",
        entity_files(&r, "tasks")
    );

    // And the complete flag form still writes exactly what it always wrote.
    let out = r.ank_edit(
        "claude-code@ank",
        &["new", "task", "--title", "Scripted", "--scope", "src/**"],
        Some(&editor),
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let files = entity_files(&r, "tasks");
    assert_eq!(files.len(), 1, "{files:?}");
    let text = std::fs::read_to_string(r.0.join(".ank/entities").join(&files[0])).unwrap();
    assert!(text.contains("title: Scripted"), "{text}");
    assert!(text.contains("  - src/**"), "{text}");
}

/// The id hashes the act of creation (§3) and the file name carries it. Letting
/// the caller choose one would let them collide with an entity that exists, or
/// mint a reference nothing can resolve.
#[test]
fn a_template_that_comes_back_with_another_id_is_refused() {
    let r = Repo::new();
    let editor = editor_filling_and(
        "Renamed",
        "src/**",
        r"s|^id: TASK-.*|id: TASK-000000000009|",
    );

    let out = r.ank_edit("claude-code@ank", &["new", "task"], Some(&editor));
    assert_eq!(code(&out), 6, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("cannot be chosen"),
        "{}",
        stderr(&out)
    );
    assert!(entity_files(&r, "tasks").is_empty());
}

/// Reaching the editor does not mean nothing was said. A flag that is not one of
/// the mandatory ones carries into the template rather than being dropped.
#[test]
fn the_flags_that_were_given_are_carried_into_the_template() {
    let r = Repo::new().with_verifiers("verifiers:\n  unit:\n    run: 'true'\n    timeout: 1m\n");
    let editor = editor_filling("Pre-filled", "src/**", "x");

    let out = r.ank_edit(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--criteria",
            "The binary answers.",
            "--verify",
            "unit",
        ],
        Some(&editor),
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let files = entity_files(&r, "tasks");
    assert_eq!(files.len(), 1, "{files:?}");
    let text = std::fs::read_to_string(r.0.join(".ank/entities").join(&files[0])).unwrap();
    assert!(text.contains("The binary answers."), "{text}");
    assert!(text.contains("verify: [unit]"), "{text}");
    // Set by the creator, because that is who typed it — the same thing the flag
    // form records.
    assert!(text.contains("criteria_by: creator"), "{text}");
}

/// The references are resolved at the point of creation, exactly as the flag
/// form resolves `--blocked-by`: an unknown one would otherwise surface in
/// `check`, long afterwards, as a corpus error nobody can attribute to the act.
#[test]
fn a_blocker_that_does_not_exist_is_refused_at_creation() {
    let r = Repo::new();
    let editor = editor_filling_and(
        "Blocked on nothing",
        "src/**",
        r"s|^blocked_by: .*|blocked_by: [TASK-000000000404]|",
    );

    let out = r.ank_edit("claude-code@ank", &["new", "task"], Some(&editor));
    assert_eq!(code(&out), 7, "{}", stderr(&out));
    let err = stderr(&out);
    assert!(err.contains("TASK-000000000404"), "{err}");
    assert!(
        err.contains("ank find"),
        "the exact command to run next: {err}"
    );
    assert!(entity_files(&r, "tasks").is_empty());
}

// ---------------------------------------------------------------------------
// Color: the guarantee is negative (§4, ADR-962c25797569)
// ---------------------------------------------------------------------------

/// Every verb worth styling, exercised against a corpus that actually has
/// something to style: a claimed task, an ADR, a blocker, a finding.
///
/// Kept as one fixture and one list because the property under test is
/// universal — "no verb" is the claim, so a list that quietly omits one is the
/// hole. `--version` and `help` are in it for the same reason they answer
/// before the foundation: they are reachable when nothing else is.
fn styled_surface() -> Vec<Vec<&'static str>> {
    vec![
        vec!["context"],
        vec!["status"],
        vec!["find", "Example"],
        vec!["find", "nothing-matches-this"],
        vec!["show", ID],
        // The ADR too: its `constraint` is a block scalar, which is the shape
        // the entity painter has to walk past without touching, and a task's
        // `show` alone never reaches it.
        vec!["show", "ADR-0000000000ab"],
        vec!["log", ID],
        vec!["graph"],
        vec!["scope", "src"],
        vec!["check"],
        vec!["review"],
        vec!["help"],
        vec!["--version"],
    ]
}

fn color_fixture() -> Repo {
    let r = Repo::new();
    r.seed_task(ID, Some("a criterion to freeze"));
    r.seed_task("TASK-000000000002", Some("another criterion"));
    r.seed_adr("ADR-0000000000ab", "a rule that binds", "src/**");
    r.blocked("TASK-000000000002", &[ID]);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    // A live claim: markers, warnings and the execution mode of `context` are
    // all unreachable without one, and they are exactly what carries colour.
    r.ank("claude-code@ank", &["claim", ID]);
    r
}

/// The one assertion the whole feature exists to satisfy.
///
/// A spawned process writes to a pipe, so this suite *is* the piped case — the
/// same shape an agent shelling out to `ank` sees. An escape byte anywhere in
/// either stream is the defect ADR-962c25797569 forbids, and `--json` is
/// checked beside the plain form because it is the one surface that must stay
/// clean even at a terminal.
#[test]
fn no_verb_writes_an_escape_sequence_to_a_pipe() {
    let r = color_fixture();

    for base in styled_surface() {
        for json in [false, true] {
            let mut args = base.clone();
            if json {
                args.push("--json");
            }
            let out = r.ank("claude-code@ank", &args);
            assert!(
                !out.stdout.contains(&0x1b),
                "{args:?} put an escape sequence on stdout: {:?}",
                stdout(&out)
            );
            assert!(
                !out.stderr.contains(&0x1b),
                "{args:?} put an escape sequence on stderr: {:?}",
                stderr(&out)
            );
        }
    }
}

/// Structure ships in the bytes, and `--json` carries none of it.
///
/// The other half of ADR-0c8ab846d262, and the half a test can only make in a
/// pipe: this suite spawns the binary, so its stdout *is* the pipe an agent
/// reads. Colour must be absent from it and the connectors must be present —
/// the two assertions point in opposite directions on purpose, because a
/// gate that confused the two layers would satisfy either one alone.
#[test]
fn a_pipe_receives_the_drawing_and_never_an_escape_sequence() {
    let r = color_fixture();

    let drawn = r.ank("claude-code@ank", &["graph"]);
    let said = stdout(&drawn);
    assert!(
        !drawn.stdout.contains(&0x1b),
        "graph coloured a pipe: {said:?}"
    );
    assert!(
        said.contains("└── ") || said.contains("├── "),
        "the connectors are text, so a pipe gets them: {said}"
    );

    // `show` draws the same relation from one task's point of view.
    let shown = stdout(&r.ank("claude-code@ank", &["show", ID]));
    assert!(
        shown.contains("UNBLOCKS (1)") && shown.contains("└── "),
        "show draws its edges: {shown}"
    );

    // The machine surface carries neither layer. Asserted over the whole
    // alphabet rather than over one connector: a new glyph reaching `--json`
    // through some other verb is exactly what this is here to catch.
    for verb in [
        vec!["graph", "--json"],
        vec!["show", ID, "--json"],
        vec!["find", "Example", "--json"],
        vec!["context", "--json"],
    ] {
        let out = r.ank("claude-code@ank", &verb);
        let j = stdout(&out);
        assert!(!out.stdout.contains(&0x1b), "{verb:?} coloured json: {j:?}");
        for glyph in ['│', '├', '└', '─'] {
            assert!(!j.contains(glyph), "{verb:?} drew {glyph:?} into json: {j}");
        }
    }
}

/// The held row is marked, and the margin it is drawn in was already there.
#[test]
fn a_listing_marks_the_row_the_caller_holds() {
    let r = color_fixture();

    // `color_fixture` leaves the claim with claude-code@ank.
    let mine = stdout(&r.ank("claude-code@ank", &["find", "Example"]));
    let theirs = stdout(&r.ank("someone-else@ank", &["find", "Example"]));

    assert!(
        mine.lines().any(|l| l.starts_with("* TASK-")),
        "the holder sees their own row marked: {mine}"
    );
    assert!(
        theirs.lines().all(|l| !l.starts_with("* ")),
        "and nobody else does: {theirs}"
    );
    // Same width for both readers: the marker is spent out of the margin, not
    // added to it, so a listing does not reflow depending on who is asking.
    for (a, b) in mine.lines().zip(theirs.lines()) {
        assert_eq!(a.len(), b.len(), "{a:?} and {b:?} are not the same width");
    }
}

/// A value that is valid for the flag, so that what is measured is the flag
/// and not the parse of its argument.
///
/// Measured the hard way first: a generic `x` makes `--limit` a code 1 and
/// `--type` a code 1, neither of which is a refusal of the flag -- it is the
/// verb correctly rejecting a value that is not a number and not a kind. A test
/// that read those as refusals would have failed on four verbs that are right.
fn valid_value(flag: &str) -> &'static str {
    match flag {
        "--limit" => "5",
        "--type" => "task",
        "--status" => "open",
        "--ttl" => "1h",
        "--proof" => "test:1",
        "--scope" | "--drop-scope" => "src/**",
        "--blocked-by" | "--drop-blocked-by" => "TASK-000000000001",
        "--supersedes" => "ADR-000000000001",
        "--verify" => "cargo-test",
        "--criteria" => "A measurable thing.",
        "--reason" => "a reason",
        "--title" => "A title",
        "--constraint" => "A rule.",
        "--body" => "Some prose.",
        other => panic!("no valid value declared for {other}: add one rather than guess"),
    }
}

/// The flags a verb's own help offers, read off the human output because that
/// is the surface the claim is about, with whether each takes a value.
///
/// The pair matters and is not decoration: help renders a value-taking flag as
/// `--name <v>` and a switch as `--name` alone, so handing a switch an argument
/// would push a stray positional at the verb and measure the positional's
/// refusal instead of the flag's. Read off the same line rather than from the
/// specs, because the claim under test is about what help offers.
fn listed_flags(r: &Repo, verb: &str) -> Vec<(String, bool)> {
    let out = stdout(&r.ank("claude-code@ank", &["help", verb]));
    let Some(line) = out
        .lines()
        .find(|l| l.trim_start().starts_with("flags:"))
        .map(str::to_string)
    else {
        return Vec::new();
    };
    let tokens: Vec<&str> = line.split_whitespace().collect();
    tokens
        .iter()
        .enumerate()
        .filter(|(_, t)| t.starts_with("--"))
        .map(|(i, t)| {
            let takes_value = tokens.get(i + 1).is_some_and(|n| n.starts_with('<'));
            (t.to_string(), takes_value)
        })
        .collect()
}

/// **A flag `ank help` offers is a flag the verb can actually be given** (§9).
///
/// The defect this exists for: `amend` listed `--criteria` among its flags and
/// refused it unconditionally, by name and by design. Help did not merely say
/// too little there, it made an offer the verb rejects -- and the refusal's own
/// hint pointed at `ank release`, which a task that was never claimed cannot
/// run. It was found by reading `human.rs`, which is the recovery route that
/// does not exist in a repository where ank arrives as a binary and a SKILL.md.
///
/// The shape is the one `msrv-tight` uses: a negative test that attributes the
/// drift rather than leaving it for a human to notice two revisions later.
///
/// **The invariant.** Against a fixed repository state, adding one flag the
/// help lists must not change the verb's exit code. A flag that is refused by
/// name fires before the verb looks at the repository, so it replaces the
/// ordinary answer -- `amend 9999` is a 2, and `amend 9999 --criteria ...` was a
/// 6, which is exactly the signature.
///
/// **One exception, and it is the flag doing its job.** When the baseline is a
/// 7 -- a missing prerequisite -- a flag is allowed to change the code, because
/// supplying the prerequisite is what `close --reason` and `release --reason`
/// are for. Measured rather than assumed: those two are the only verbs where it
/// happens.
#[test]
fn every_flag_the_help_offers_can_be_given_to_the_verb() {
    let r = color_fixture();
    // `sh -c true` on all three platforms: git supplies the shell on Windows.
    // Without it `new` opens an editor and the run has no deterministic end.
    let env = [("EDITOR", Some("true"))];

    // Every verb §4 lists, with a positional that resolves to nothing so the
    // baseline is the verb's ordinary "not found" rather than real work.
    let verbs: [(&str, &[&str]); 20] = [
        ("context", &[]),
        ("claim", &["TASK-999999999999"]),
        ("show", &["TASK-999999999999"]),
        ("log", &["TASK-999999999999"]),
        ("done", &["TASK-999999999999"]),
        ("release", &["TASK-999999999999"]),
        ("new", &["task"]),
        ("find", &["nothing-matches-this"]),
        ("status", &[]),
        ("review", &[]),
        ("accept", &["ADR-999999999999"]),
        ("close", &["TASK-999999999999"]),
        ("amend", &["TASK-999999999999"]),
        ("attest", &["TASK-999999999999"]),
        ("edit", &["TASK-999999999999"]),
        ("graph", &[]),
        ("scope", &["src"]),
        ("check", &[]),
        ("init", &[]),
        ("help", &["find"]),
    ];

    let mut walked = 0;
    for (verb, positionals) in verbs {
        walked += 1;
        let flags = listed_flags(&r, verb);
        if flags.is_empty() {
            continue;
        }
        let mut base_args = vec![verb];
        base_args.extend_from_slice(positionals);
        let baseline = code(&r.ank_env("claude-code@ank", &base_args, &env));

        for (flag, takes_value) in &flags {
            let mut args = base_args.clone();
            args.push(flag);
            if *takes_value {
                args.push(valid_value(flag));
            }
            let got = code(&r.ank_env("claude-code@ank", &args, &env));
            assert!(
                got == baseline || baseline == 7,
                "`ank {verb}` offers {flag} and then answers {got} where it \
                 answers {baseline} without it: either the verb refuses a flag \
                 its help advertises, or the help advertises a flag the verb \
                 refuses"
            );
        }
    }
    assert_eq!(
        walked, 20,
        "a verb was added and this walk did not learn of it"
    );
}

/// One fact, one string, whichever verb prints it.
///
/// `context` reads the claim refs and every other listing read the index, so a
/// claimed task said `[claimed:who]` under one verb and `[in_progress]` under
/// four others — one state wearing two words, chosen by whichever verb the
/// reader happened to type. Asserted through the binary because the divergence
/// was between whole commands and not inside one function: each of these opens
/// the corpus its own way, and only running them proves they agree.
#[test]
fn every_verb_says_the_same_thing_about_a_claimed_task() {
    let r = color_fixture();

    // `color_fixture` leaves ID claimed by claude-code@ank, and the marker
    // names the holder whoever is asking — so a stranger reads the same words.
    let held = format!("[claimed:{}]", "claude-code@ank");

    for args in [
        vec!["context"],
        vec!["find", "Example"],
        vec!["scope", "src"],
        vec!["graph"],
        // The reverse edge of the task ID blocks: `show` reaches ID through
        // another task's `BLOCKED BY`, which is the fourth listing that used to
        // print the index's word instead.
        vec!["show", "TASK-000000000002"],
    ] {
        let said = stdout(&r.ank("someone-else@ank", &args));
        assert!(
            said.contains(&held),
            "`ank {}` does not say {held}: {said}",
            args.join(" ")
        );
        assert!(
            !said.contains("[in_progress]"),
            "`ank {}` still prints the index's word: {said}",
            args.join(" ")
        );
    }
}

/// The transition grammar of §4, walked through the binary.
///
/// Every one of these lines was bare before TASK-4601ed18d84e, and none of them
/// can live in `styled_surface`: that list is replayed against one fixture, and
/// each line here is produced by a verb that mutates. So they are walked once,
/// in the loop's own order — a task created, taken, logged, released, retaken,
/// finished and attested; a second one amended and closed.
///
/// Two assertions per line, and the second is the one that would survive a
/// wrong colour. Absence of an escape byte proves the pipe is clean; the shape
/// proves the grammar is the one the specification declares, because a call
/// site that painted the wrong token would still be escape-free here.
#[test]
fn every_transition_line_reads_one_grammar_and_stays_plain_in_a_pipe() {
    let r = Repo::new();
    r.seed_task(ID, Some("a criterion to freeze"));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let run = |args: &[&str]| -> String {
        let out = r.ank("claude-code@ank", args);
        assert_eq!(code(&out), 0, "{args:?}: {}", stderr(&out));
        assert!(
            !out.stdout.contains(&0x1b),
            "{args:?} coloured a pipe: {:?}",
            stdout(&out)
        );
        assert!(
            !out.stderr.contains(&0x1b),
            "{args:?} coloured stderr: {:?}",
            stderr(&out)
        );
        stdout(&out)
    };

    let created = run(&[
        "new",
        "task",
        "--title",
        "A second task",
        "--scope",
        "src/**",
        "--criteria",
        "The binary answers.",
    ]);
    assert!(
        created.starts_with("created TASK-"),
        "created names what it made: {created:?}"
    );
    let second = created
        .split_whitespace()
        .nth(1)
        .expect("created <id>")
        .to_string();

    let claimed = run(&["claim", &second]);
    assert!(claimed.starts_with("claimed TASK-"), "{claimed:?}");

    let logged = run(&["log", &second, "something learned"]);
    assert!(logged.starts_with("logged on TASK-"), "{logged:?}");

    let released = run(&["release", &second, "--reason", "the criterion is wrong"]);
    assert!(
        released.starts_with("released TASK-") && released.trim_end().ends_with("-> open"),
        "a release names the state it lands on: {released:?}"
    );

    run(&["claim", &second]);
    let finished = run(&["done", &second, "--proof", "test:ci-run-1"]);
    assert!(finished.trim_end().ends_with("-> done"), "{finished:?}");

    let attested = run(&["attest", &second, "--proof", "test:ci-run-2"]);
    assert!(attested.starts_with("attested TASK-"), "{attested:?}");

    let amended = run(&["amend", ID, "--scope", "docs/**"]);
    assert!(amended.starts_with("amended TASK-"), "{amended:?}");

    let closed = run(&["close", ID, "--reason", "superseded by the second"]);
    assert!(
        closed.starts_with("closed TASK-") && closed.trim_end().ends_with("-> closed"),
        "{closed:?}"
    );
}

/// The error envelope travels the same rule, and it is the one line that does
/// not go through the writer every verb shares.
#[test]
fn a_refusal_is_plain_in_a_pipe_too() {
    let r = color_fixture();

    // One refusal per stream-producing shape: an unknown entity, a held claim,
    // and a flag error caught before any verb runs.
    for args in [
        vec!["claim", "TASK-00000000dead"],
        vec!["claim", "TASK-000000000002"],
        vec!["find", "bug", "-st", "task"],
        vec!["context", "--limit", "not-a-number"],
    ] {
        let out = r.ank("someone-else@ank", &args);
        assert_ne!(code(&out), 0, "{args:?} was supposed to refuse");
        assert!(
            !out.stderr.contains(&0x1b),
            "{args:?} coloured a refusal into a pipe: {:?}",
            stderr(&out)
        );
        assert!(
            stderr(&out).starts_with("error["),
            "{args:?} lost the envelope: {:?}",
            stderr(&out)
        );
    }
}

/// The detection rule read from the other side: nothing in the environment can
/// turn colour *on*, because the terminal test has already failed.
///
/// This is what catches a wiring inversion. `NO_COLOR` implemented backwards,
/// or a Windows allowlist consulted instead of the terminal check, would leave
/// every assertion above green and change the bytes here.
#[test]
fn no_environment_makes_a_pipe_colored() {
    let r = color_fixture();

    let environments: [&[(&str, Option<&str>)]; 6] = [
        &[("NO_COLOR", None), ("TERM", None), ("WT_SESSION", None)],
        &[("NO_COLOR", Some("1"))],
        &[("NO_COLOR", Some(""))],
        &[("TERM", Some("dumb"))],
        &[("TERM", Some("xterm-256color"))],
        // The Windows allowlist, set on every platform: it is only ever an
        // additional condition, never a substitute for the terminal itself.
        &[("WT_SESSION", Some("1")), ("ANSICON", Some("1"))],
    ];

    for base in styled_surface() {
        let reference = r.ank_env(
            "claude-code@ank",
            &base,
            &[("NO_COLOR", None), ("TERM", None), ("WT_SESSION", None)],
        );
        for env in environments {
            let out = r.ank_env("claude-code@ank", &base, env);
            assert!(
                !out.stdout.contains(&0x1b),
                "{base:?} under {env:?} coloured a pipe"
            );
            assert_eq!(
                out.stdout, reference.stdout,
                "{base:?} answered differently under {env:?}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// `ank config` (§4, ADR-e64dfaafd578)
// ---------------------------------------------------------------------------
//
// Through the binary, and not only through the module, because every clause of
// the criterion is phrased about `ank config <key>` -- and because the sharpest
// of them is a claim about dispatch: the verb answers on a `config.yml` that
// fails `startup`, which no unit test on the writer could reach.

/// Trimmed stdout of one invocation, which is what a caller reads.
fn said(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).trim().to_string()
}

fn erred(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).to_string()
}

/// A `config.yml` carrying every awkward form at once: comments on their own
/// line and after a value, blank lines, verifiers out of alphabetical order, a
/// quoted `run`, and a verifier with no `timeout`.
const AWKWARD: &str = "\
# Reviewed like code, and it stays reviewable.
schema: 1

claim_ttl_max: 2h   # renewed by ank log
default_branch: main

verifiers:
  # Out of alphabetical order on purpose.
  fmt-check:
    run: \"cargo fmt --check\"
  cargo-test:
    run: cargo test --workspace -q
    timeout: 30m
";

#[test]
fn config_reads_the_value_in_effect_and_marks_a_resolved_default() {
    let r = Repo::new();
    r.set_config(AWKWARD);
    let say = |args: &[&str]| said(&r.ank("claude-code@ank", args));

    assert_eq!(say(&["config", "claim_ttl_max"]), "2h");
    assert_eq!(say(&["config", "default_branch"]), "main");
    // A quoted value reads as its value, not as its spelling.
    assert_eq!(
        say(&["config", "verifiers.fmt-check.run"]),
        "cargo fmt --check"
    );
    assert_eq!(say(&["config", "verifiers.cargo-test.timeout"]), "30m");

    // Absent from the file and resolved by the tool, said to be so: the
    // difference between "the tool's value" and "this repository's value" is
    // the whole question a reader is asking, and a release that moves a default
    // moves the first and not the second.
    assert_eq!(say(&["config", "context_budget"]), "8000 (default)");
    assert_eq!(
        say(&["config", "verifiers.fmt-check.timeout"]),
        "10m (default)"
    );
    // The key a repository states its own rhythm through, and the fixture
    // states nothing: the thirty minutes it reads as are the tool's
    // (ADR-0bb7ea8991bc).
    assert_eq!(say(&["config", "claim_ttl_default"]), "30m (default)");

    // Absent with nothing to resolve.
    r.set_config("schema: 1\n");
    assert_eq!(say(&["config", "default_branch"]), "(unset)");

    // The marker is on the human surface alone: --json splits the two, which
    // is what a script reads.
    r.set_config(AWKWARD);
    let json = say(&["config", "context_budget", "--json"]);
    assert!(json.contains("\"value\":\"8000\""), "{json}");
    assert!(json.contains("\"source\":\"default\""), "{json}");
    let json = say(&["config", "claim_ttl_max", "--json"]);
    assert!(json.contains("\"source\":\"file\""), "{json}");
}

#[test]
fn config_writes_the_key_and_no_byte_beside_it() {
    let r = Repo::new();
    r.set_config(AWKWARD);

    let out = r.ank("claude-code@ank", &["config", "claim_ttl_max", "4h"]);
    assert!(out.status.success(), "{}", erred(&out));
    assert_eq!(said(&out), "claim_ttl_max 2h -> 4h");

    // Every line but the one named is byte-identical, comment column included.
    let after = r.config_text();
    let moved: Vec<(&str, &str)> = AWKWARD
        .lines()
        .zip(after.lines())
        .filter(|(a, b)| a != b)
        .collect();
    assert_eq!(
        moved,
        vec![(
            "claim_ttl_max: 2h   # renewed by ank log",
            "claim_ttl_max: 4h   # renewed by ank log"
        )],
        "more than the named key moved"
    );
    assert_eq!(AWKWARD.lines().count(), after.lines().count());

    // A key the file does not carry is written where the surgery appends, and
    // reads back as this repository's value rather than the tool's.
    let out = r.ank("claude-code@ank", &["config", "claim_ttl_default", "90m"]);
    assert!(out.status.success(), "{}", erred(&out));
    assert_eq!(said(&out), "claim_ttl_default 30m (default) -> 90m");
    assert!(r.config_text().contains("claim_ttl_default: 90m"));
    assert_eq!(
        said(&r.ank("claude-code@ank", &["config", "claim_ttl_default"])),
        "90m"
    );
    let out = r.ank(
        "claude-code@ank",
        &["config", "--unset", "claim_ttl_default"],
    );
    assert!(out.status.success(), "{}", erred(&out));
    assert!(!r.config_text().contains("claim_ttl_default"));

    // No default was materialised on the way: context_budget was absent and
    // stays absent, and fmt-check still declares no timeout.
    let after = r.config_text();
    assert!(!after.contains("context_budget"), "{after}");
    let fmt = after.split("fmt-check:").nth(1).unwrap();
    assert!(!fmt.split("cargo-test:").next().unwrap().contains("timeout"));

    // Nested values are addressed by dotted path, and a new verifier is
    // declared by writing its run -- which is what the six hints now name.
    let out = r.ank(
        "claude-code@ank",
        &["config", "verifiers.audit.run", "cargo audit"],
    );
    assert!(out.status.success(), "{}", erred(&out));
    assert_eq!(said(&out), "verifiers.audit.run (unset) -> cargo audit");
    assert!(r.config_text().contains("  audit:\n    run: cargo audit\n"));

    // Writing what the file already says touches nothing at all.
    let before = r.config_text();
    let out = r.ank(
        "claude-code@ank",
        &["config", "verifiers.audit.run", "cargo audit"],
    );
    assert_eq!(said(&out), "verifiers.audit.run cargo audit (unchanged)");
    assert_eq!(r.config_text(), before);
}

#[test]
fn config_unset_removes_the_key_and_gives_the_file_back() {
    let r = Repo::new();
    r.set_config(AWKWARD);

    let out = r.ank("claude-code@ank", &["config", "--unset", "default_branch"]);
    assert!(out.status.success(), "{}", erred(&out));
    assert_eq!(said(&out), "default_branch main -> (unset)");
    assert!(!r.config_text().contains("default_branch"));

    // Set then unset is the identity, byte for byte -- and the short form of
    // §4's table does the same thing as the long one.
    r.ank("claude-code@ank", &["config", "default_branch", "main"]);
    r.ank("claude-code@ank", &["config", "-u", "default_branch"]);
    let stripped = AWKWARD.replace("default_branch: main\n", "");
    assert_eq!(r.config_text(), stripped);

    // A whole verifier is what --unset addresses, which is what makes
    // declaring one reversible.
    r.set_config(AWKWARD);
    r.ank(
        "claude-code@ank",
        &["config", "verifiers.audit.run", "cargo audit"],
    );
    let out = r.ank("claude-code@ank", &["config", "--unset", "verifiers.audit"]);
    assert!(out.status.success(), "{}", erred(&out));
    assert_eq!(r.config_text(), AWKWARD);

    // And `run` alone is refused, naming the command that does the job.
    let out = r.ank(
        "claude-code@ank",
        &["config", "--unset", "verifiers.cargo-test.run"],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(
        erred(&out).contains("ank config --unset verifiers.cargo-test"),
        "{}",
        erred(&out)
    );
    assert_eq!(r.config_text(), AWKWARD, "a refusal wrote to the file");
}

#[test]
fn config_refuses_an_unknown_key_by_name_and_writes_nothing() {
    let r = Repo::new();
    r.set_config(AWKWARD);

    let out = r.ank("claude-code@ank", &["config", "budget_context", "10"]);
    assert_eq!(out.status.code(), Some(1));
    let err = erred(&out);
    assert!(err.contains("budget_context"), "{err}");
    for key in [
        "schema",
        "context_budget",
        "claim_ttl_max",
        "claim_ttl_default",
        "default_branch",
        "verifiers.<name>.run",
        "verifiers.<name>.timeout",
    ] {
        assert!(err.contains(key), "{key} missing from the refusal: {err}");
    }
    assert_eq!(r.config_text(), AWKWARD);

    // Known to the parser, structured, and refused by name rather than guessed
    // at -- a different message from "no such key".
    for key in ["roles", "identities", "roles.agent.can"] {
        let out = r.ank("claude-code@ank", &["config", key, "x"]);
        assert_eq!(out.status.code(), Some(1), "{key}");
        assert!(erred(&out).contains("structured"), "{key}: {}", erred(&out));
    }

    // A timeout cannot declare a verifier, and the refusal names what can.
    let out = r.ank(
        "claude-code@ank",
        &["config", "verifiers.nope.timeout", "5m"],
    );
    assert_eq!(out.status.code(), Some(7));
    assert!(
        erred(&out).contains("ank config verifiers.nope.run"),
        "{}",
        erred(&out)
    );
    assert_eq!(r.config_text(), AWKWARD);
}

#[test]
fn config_refuses_a_write_that_would_leave_the_file_unreadable() {
    let r = Repo::new();
    r.set_config(AWKWARD);

    // Each of these parses as YAML and fails the configuration's own reading:
    // a duration in no unit it knows, and a schema it does not support.
    for (key, value) in [("claim_ttl_max", "30w"), ("schema", "2")] {
        let out = r.ank("claude-code@ank", &["config", key, value]);
        assert_eq!(out.status.code(), Some(1), "{key} was accepted");
        assert!(erred(&out).contains("unreadable"), "{key}: {}", erred(&out));
        assert_eq!(r.config_text(), AWKWARD, "{key} left the file changed");
    }

    // A block scalar is refused before any of that: flattening it would move
    // the definition hash that anchors historical proofs.
    r.set_config("schema: 1\nverifiers:\n  ci:\n    run: |\n      cargo test\n");
    let before = r.config_text();
    let out = r.ank(
        "claude-code@ank",
        &["config", "verifiers.ci.run", "cargo test -q"],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(erred(&out).contains("verifiers.ci.run"), "{}", erred(&out));
    assert_eq!(r.config_text(), before);
}

#[test]
fn config_is_the_one_verb_a_config_that_does_not_parse_does_not_stop() {
    let r = Repo::new();
    let broken = "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\nbudget_context: 10\n";
    r.set_config(broken);

    // `startup` loads the configuration for every other verb, so the file
    // fails all of them -- `check`, the one an agent would reach for, included.
    for args in [
        &["check"][..],
        &["context"][..],
        &["status"][..],
        &["find", "anything"][..],
    ] {
        let out = r.ank("claude-code@ank", args);
        assert_eq!(
            out.status.code(),
            Some(1),
            "{args:?} ran on a config.yml that does not parse"
        );
    }

    // The verb that exists to repair the file is the one the broken file does
    // not stop. It reads,
    let out = r.ank("claude-code@ank", &["config", "claim_ttl_max"]);
    assert!(out.status.success(), "{}", erred(&out));
    assert_eq!(said(&out), "2h");

    // and it writes, saying that the file is still unreadable for a reason it
    // was not asked about. Refusing here instead would be refusing exactly
    // where the verb is needed.
    let out = r.ank("claude-code@ank", &["config", "claim_ttl_max", "4h"]);
    assert!(out.status.success(), "{}", erred(&out));
    assert!(r.config_text().contains("claim_ttl_max: 4h"));
    assert!(said(&out).contains("warning:"), "{}", said(&out));
    assert!(said(&out).contains("budget_context"), "{}", said(&out));
}

#[test]
fn the_errors_that_named_the_file_now_name_the_command() {
    // ADR-01b6dd05f0db closed `.ank/` to agents and stopped at the
    // configuration, which left these telling their caller to open a file the
    // same tool forbids them to open (ADR-e64dfaafd578).
    let r = Repo::new();

    // `new`, on a --verify that matches nothing. The criterion calls the two
    // verify sites "new's and amend's"; they are in fact both `new`'s -- the
    // flag form here, and the `verify:` filled into the $EDITOR template,
    // which `commands::check_verifiers` guards and a unit test covers. `amend`
    // takes no --verify at all.
    let out = r.ank(
        "claude-code@ank",
        &[
            "new", "task", "--title", "T", "--scope", "src/**", "--verify", "nope",
        ],
    );
    assert_eq!(out.status.code(), Some(7), "{}", erred(&out));
    let err = erred(&out);
    assert!(err.contains("ank config verifiers.nope.run"), "{err}");
    assert!(
        !err.contains("under verifiers: in .ank/config.yml"),
        "it still tells the caller to open the file: {err}"
    );

    // `done`, on a task declaring a verifier the configuration does not.
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: git --version\n");
    r.seed_task_with(
        "TASK-000000000c01",
        Some("A verifiable criterion."),
        &["ok"],
    );
    r.ank("claude-code@ank", &["claim", "TASK-000000000c01"]);
    r.set_config("schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n");
    let out = r.ank("claude-code@ank", &["done", "TASK-000000000c01"]);
    assert_eq!(out.status.code(), Some(9), "{}", erred(&out));
    assert!(
        erred(&out).contains("ank config verifiers.ok.run"),
        "{}",
        erred(&out)
    );

    // The four sites that ask for a default branch. There is no
    // `default_branch` and no `refs/remotes/origin/HEAD` to fall back on, so
    // each of them has to name the command that sets one.
    let r = Repo::new();
    r.set_config("schema: 1\nclaim_ttl_max: 2h\n");
    std::fs::write(r.0.join("seed.txt"), "x").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    r.seed_adr("ADR-000000000c02", "A binding rule.", "src/**");
    for args in [
        &["status"][..],
        &["check"][..],
        &["context"][..],
        &["accept", "ADR-000000000c02"][..],
    ] {
        let out = r.ank("claude-code@ank", args);
        let text = format!("{}{}", said(&out), erred(&out));
        assert!(
            text.contains("ank config default_branch"),
            "{args:?} does not name the command: {text}"
        );
        assert!(
            !text.contains("\"default_branch: <name>\""),
            "{args:?} still quotes the line to add by hand: {text}"
        );
    }
}

// ---------------------------------------------------------------------------
// Path and glob normalisation, over the whole surface (TASK-8dd89053fa33)
// ---------------------------------------------------------------------------
//
// TASK-df4c39031583 measured this defect, fixed it, and wrote a criterion
// naming four verbs where the property was general. The fix satisfied that text
// exactly; the flag values were never in it, and the freeze made the
// enumeration authoritative. So the tests here are built the other way round:
// the surface is read back from the binary, every argument on it has to be
// classified, and an argument nobody classified fails the suite.

/// The verbs `ank help` prints, each with its usage line and its listed flags.
///
/// Read from the binary rather than restated here, for the reason
/// `tests/skill.rs` reads §4 rather than restating it: a second hand-maintained
/// copy of the surface is the drift being checked for.
fn surface() -> Vec<(String, String, Vec<String>)> {
    let out = ank_command()
        .arg("help")
        .output()
        .expect("the binary must have been built");
    assert!(out.status.success(), "ank help must succeed");
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    let mut rows = Vec::new();
    for line in listing_body(&text) {
        // The listing is `<usage>` padded, then the description. Its folded
        // continuation lines are indented and open no verb (TASK-fe130d2b732c),
        // and a group heading opens none either (ADR-f61e2d2c75e8).
        let Some(rest) = line.strip_prefix("ank ") else {
            continue;
        };
        let verb: String = rest.chars().take_while(char::is_ascii_lowercase).collect();
        assert!(!verb.is_empty(), "unparseable listing line: {line}");
        let usage = match rest.find("  ") {
            Some(i) => format!("ank {}", &rest[..i]),
            None => format!("ank {rest}"),
        };

        // The flags come from the verb's own page, which is where they live
        // now that the description has their place in the listing. The `flags:`
        // line only: the globals have a line of their own, as they did here.
        let page = stdout(&ank_command().args(["help", &verb]).output().unwrap());
        let flags: Vec<String> = page
            .lines()
            .find(|l| l.trim_start().starts_with("flags:"))
            .unwrap_or("")
            .split_whitespace()
            .filter(|w| w.starts_with("--"))
            .map(|w| {
                w.trim_end_matches(|c: char| !c.is_ascii_alphanumeric())
                    .to_string()
            })
            .collect();
        rows.push((verb, usage.trim().to_string(), flags));
    }
    assert!(!rows.is_empty(), "ank help printed no verb");
    rows
}

/// Positionals naming a perimeter inside the corpus, which are normalised.
const PATH_POSITIONALS: [&str; 5] = ["context", "review", "graph", "scope", "check"];

/// Positionals naming a place on the machine rather than a perimeter in the
/// corpus. `init` creates `.ank/` somewhere, and that somewhere is legitimately
/// absolute — normalising it would refuse the ordinary call.
const MACHINE_POSITIONALS: [&str; 1] = ["init"];

/// Flag values matched against scopes, or stored as one.
const PATH_FLAGS: [(&str, &str); 1] = [("find", "--scope")];
const GLOB_FLAGS: [(&str, &str); 3] = [
    ("new", "--scope"),
    ("amend", "--scope"),
    ("amend", "--drop-scope"),
];

/// Every other flag on the surface, declared as carrying no path.
///
/// Enumerated rather than inferred from the name. A heuristic — "it carries a
/// path if it is called `--scope`" — is exactly what would let the next
/// `--under <glob>` through in silence, which is the failure this whole task is
/// a correction of.
const NOT_A_PATH: [&str; 20] = [
    "--limit",
    "--criteria",
    "--ttl",
    "--proof",
    // Carries no value either: the perimeters it compares are the scopes the
    // corpus already holds, and nothing here comes off the command line
    // (ADR-052accd6e3b2).
    "--free",
    // Carries no value at all, let alone a path: it names where the proof is
    // written, and that address is a ref (ADR-493471d64ba0).
    "--detached",
    // Carries no value either: the remote it reads is `origin` by name, the
    // refs it asks for are the claims namespace, and neither comes off the
    // command line (ADR-47e2ac102f58).
    "--remote",
    "--reason",
    "--title",
    "--blocked-by",
    "--constraint",
    "--supersedes",
    "--verify",
    "--body",
    "--type",
    "--status",
    "--drop-blocked-by",
    "--unset",
    "--json",
    "--quiet",
];

/// **The guard.** Every argument the binary offers is classified, and one that
/// is not fails here.
///
/// This is what the criterion asks for in as many words: a call site added
/// later cannot skip the normalisation, because adding it to the surface makes
/// this test red until somebody says which kind of argument it is.
///
/// Measured rather than assumed: dropping `--limit` from `NOT_A_PATH` turns
/// this red with the message naming `ank context --limit`, which is the whole
/// point of writing it this way.
#[test]
fn every_argument_on_the_surface_is_classified_as_carrying_a_path_or_not() {
    for (verb, usage, flags) in surface() {
        let takes_positional_path = usage.contains("<path>");
        let classified = PATH_POSITIONALS.contains(&verb.as_str())
            || MACHINE_POSITIONALS.contains(&verb.as_str());
        assert_eq!(
            takes_positional_path, classified,
            "`{usage}` takes a positional path and is not classified (or is \
             classified and does not take one): add `{verb}` to \
             PATH_POSITIONALS or MACHINE_POSITIONALS, and route it through \
             context::normalised if it names a perimeter"
        );

        for flag in flags {
            let known = PATH_FLAGS.contains(&(verb.as_str(), flag.as_str()))
                || GLOB_FLAGS.contains(&(verb.as_str(), flag.as_str()))
                || NOT_A_PATH.contains(&flag.as_str());
            assert!(
                known,
                "`ank {verb} {flag}` is on the surface and nobody has said \
                 whether it carries a path: add it to PATH_FLAGS, GLOB_FLAGS \
                 or NOT_A_PATH. A flag that carries one must go through \
                 context::normalised before matching or storage."
            );
        }
    }
    // `--repo` is the third global and is deliberately in none of the lists: it
    // names the repository to resolve, on the machine, and is legitimately
    // absolute. Asserting it is absent keeps that a decision rather than an
    // omission.
    assert!(
        !NOT_A_PATH.contains(&"--repo"),
        "--repo is excluded on purpose, not by classification"
    );
}

/// The spellings a shell actually produces for one directory.
const SPELLINGS: [&str; 5] = ["docs", "docs/", "docs\\", "./docs", ".\\docs\\"];

/// The globs an entity stores, read out of `ank show`.
///
/// The `scope:` block alone, and not the whole entity: `amend` writes a log
/// entry naming the glob it added or dropped, so a substring search over the
/// output reports a scope that is only mentioned in the history as one that is
/// still bound. That mistake cost this test one red run.
fn stored_scope(shown: &str) -> Vec<String> {
    shown
        .lines()
        .skip_while(|l| l.trim() != "scope:")
        .skip(1)
        .take_while(|l| l.starts_with("  - "))
        .map(|l| l.trim_start_matches("  - ").to_string())
        .collect()
}

/// A repository with `docs/` bound by an entity, so a perimeter question has a
/// non-empty answer to give and a partial one to get wrong.
fn scoped_repo() -> Repo {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("docs")).unwrap();
    std::fs::write(r.0.join("docs/guide.md"), "x").unwrap();
    r.seed_task_scoped("TASK-000000000d01", "docs/**");
    r.seed_adr("ADR-000000000d02", "Docs are prose.", "docs/**");
    r
}

#[test]
fn every_positional_path_answers_the_same_however_the_directory_is_typed() {
    let r = scoped_repo();
    for verb in PATH_POSITIONALS {
        let reference = r.ank("claude-code@ank", &[verb, "docs"]);
        assert!(
            reference.status.success() || reference.status.code() == Some(8),
            "{verb} docs: {}",
            erred(&reference)
        );
        for spelling in SPELLINGS {
            let out = r.ank("claude-code@ank", &[verb, spelling]);
            assert_eq!(
                out.stdout, reference.stdout,
                "`ank {verb} {spelling}` answered differently from `ank {verb} docs`"
            );
        }
        // Never a silently partial set: a form naming nothing in the repository
        // is refused with the command to run next.
        let out = r.ank("claude-code@ank", &[verb, "../elsewhere"]);
        assert_eq!(
            out.status.code(),
            Some(1),
            "{verb} answered about ../elsewhere"
        );
        assert!(
            erred(&out).contains("does not name a path"),
            "{}",
            erred(&out)
        );
    }
}

#[test]
fn find_scope_answers_the_same_however_the_directory_is_typed() {
    let r = scoped_repo();
    let reference = r.ank("claude-code@ank", &["find", "--scope", "docs"]);
    assert!(reference.status.success(), "{}", erred(&reference));
    assert!(
        !said(&reference).contains("no match"),
        "the fixture must have something to find, or this proves nothing"
    );
    for spelling in SPELLINGS {
        let out = r.ank("claude-code@ank", &["find", "--scope", spelling]);
        assert_eq!(
            out.stdout, reference.stdout,
            "`ank find --scope {spelling}` answered differently from `docs`"
        );
    }
    let out = r.ank("claude-code@ank", &["find", "--scope", "/etc"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(erred(&out).contains("ank find --scope"), "{}", erred(&out));
}

/// `find --free` is the overlap `claim` names, read from the other side, and it
/// says how many candidates it withheld (ADR-052accd6e3b2).
///
/// The count is not decoration. One session read seven task files by hand to
/// discover that a single held task, scoped `crates/ank-cli/tests/**`, made
/// five of the seven unworkable — and a filter that silently returns two
/// candidates out of seven reads as a corpus with two tasks left in it. It
/// would be trusted, for the wrong reason.
#[test]
fn find_free_lists_what_no_live_claim_covers_and_says_how_many_it_hid() {
    let r = Repo::new();
    let held = "TASK-a00000000002";
    let collides = "TASK-b00000000002";
    let elsewhere = "TASK-c00000000002";
    r.seed_task_scoped(held, "crates/ank-cli/tests/**");
    r.seed_task_scoped(collides, "crates/ank-cli/**");
    r.seed_task_scoped(elsewhere, "docs/**");

    // Without the flag `find` is unchanged, and this is the baseline the
    // filtered listing is read against.
    let all = stdout(&r.ank("claude-code@ank", &["find", "--status", "open"]));
    for id in [held, collides, elsewhere] {
        assert!(
            all.contains(&id[..9]),
            "{id} is missing from the listing: {all}"
        );
    }
    assert!(
        !all.contains("hidden"),
        "an unfiltered find hides nothing: {all}"
    );

    assert_eq!(code(&r.ank("mia@laptop", &["claim", held])), 0);

    let out = r.ank("claude-code@ank", &["find", "--free"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let free = stdout(&out);
    assert!(
        free.contains(&elsewhere[..9]),
        "a task no live claim covers is listed: {free}"
    );
    assert!(
        !free.contains(&collides[..9]),
        "a task whose scope meets a live claim is not: {free}"
    );
    assert!(
        !free.contains(&held[..9]),
        "and neither is the claimed task itself, which is no longer open: {free}"
    );
    assert!(
        free.contains("1 hidden"),
        "the filter says what it withheld: {free}"
    );

    // The machine surface carries the same number, so a caller scripting the
    // choice reads it rather than parsing the sentence.
    let j = stdout(&r.ank("claude-code@ank", &["find", "--free", "--json"]));
    assert!(j.contains("\"hidden\":1"), "{j}");

    // A lapsed claim covers nothing: the withheld task comes back, and with it
    // the claimed one, whose file says open again is not true -- it says
    // in_progress, so only the collision returns.
    r.expire_claim(held);
    let free = stdout(&r.ank("claude-code@ank", &["find", "--free"]));
    assert!(
        free.contains(&collides[..9]),
        "a lapsed claim is not a live one: {free}"
    );
    assert!(
        !free.contains("hidden"),
        "and nothing is hidden by it: {free}"
    );
}

/// A task finished on another branch is not free, whatever its file says here
/// (ADR-6d8736c04cfa).
///
/// Measured on the real corpus while building `--free`, which is why it is
/// written down: the listing offered a task carrying a completion ref, because
/// the file this branch carries still reads `open`. Following the offer is a
/// code 4 — an exact command that refuses the moment it is run, which is the
/// generic help by another route, and which `claim`'s own "another ready task"
/// hint had already learned to skip.
///
/// It is not counted as hidden either: the count answers what the scope filter
/// cost, and this task was never a candidate.
#[test]
fn find_free_does_not_offer_a_task_finished_on_another_branch() {
    let finished = "TASK-a00000000003";
    let candidate = "TASK-b00000000003";
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task_with(finished, Some("A criterion."), &["ok"]);
    r.seed_task_scoped(candidate, "docs/**");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed tasks"]);

    // A commit of its own on the branch, so the completion record names one
    // `main` genuinely does not carry.
    r.git(&["checkout", "-q", "-b", "feature"]);
    std::fs::write(r.0.join("work.txt"), "y").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "work"]);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", finished])), 0);
    let out = r.ank("claude-code@ank", &["done"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "done"]);

    r.git(&["checkout", "-q", "main"]);
    assert!(
        r.task_text(finished).contains("status: open"),
        "the fixture is wrong if main already carries the done"
    );

    let listed = stdout(&r.ank("someone@ank", &["find", "--free"]));
    assert!(
        listed.contains(&candidate[..9]),
        "the ordinary candidate is offered: {listed}"
    );
    assert!(
        !listed.contains(&finished[..9]),
        "a task the refs say is finished is not claimable, so it is not free: \
         {listed}"
    );
    assert!(
        !listed.contains("hidden"),
        "and it was never a candidate, so the scope filter is not charged for \
         it: {listed}"
    );
}

#[test]
fn new_stores_the_normalised_glob_and_never_the_string_as_typed() {
    let r = scoped_repo();

    // The form Windows tab-completion produces, which used to be stored
    // verbatim and then match nothing for the life of the corpus.
    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--title",
            "Completed by the shell",
            "--scope",
            ".\\docs\\**",
            "--criteria",
            "It matches.",
        ],
    );
    assert!(out.status.success(), "{}", erred(&out));
    let id = said(&out).split_whitespace().nth(1).unwrap().to_string();
    let shown = said(&r.ank("claude-code@ank", &["show", &id]));
    assert_eq!(
        stored_scope(&shown),
        ["docs/**"],
        "the scope was stored as typed: {shown}"
    );

    // And it is reachable, which is the point of storing the normal form.
    let found = said(&r.ank("claude-code@ank", &["find", "--scope", "docs"]));
    assert!(found.contains(&id[..9]), "{found}");

    // An ADR takes the same route.
    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "adr",
            "--title",
            "Also completed",
            "--scope",
            "./docs/**",
            "--constraint",
            "A rule.",
        ],
    );
    assert!(out.status.success(), "{}", erred(&out));
    let id = said(&out).split_whitespace().nth(1).unwrap().to_string();
    assert_eq!(
        stored_scope(&said(&r.ank("claude-code@ank", &["show", &id]))),
        ["docs/**"]
    );

    // A glob naming nothing in the repository is refused, and nothing is
    // written -- a scope pointing outside the tree is not a scope.
    let before = std::fs::read_dir(r.0.join(".ank/entities"))
        .unwrap()
        .count();
    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "task",
            "--title",
            "Outside",
            "--scope",
            "../elsewhere/**",
        ],
    );
    assert_eq!(out.status.code(), Some(1), "{}", said(&out));
    assert_eq!(
        std::fs::read_dir(r.0.join(".ank/entities"))
            .unwrap()
            .count(),
        before,
        "a refused scope still created the task"
    );

    // The repository root is a perimeter, not a pattern, and the refusal says
    // which pattern was meant.
    let out = r.ank(
        "claude-code@ank",
        &["new", "task", "--title", "Root", "--scope", "."],
    );
    assert_eq!(out.status.code(), Some(7));
    assert!(erred(&out).contains("\"**\""), "{}", erred(&out));
}

#[test]
fn amend_normalises_both_the_scope_it_adds_and_the_one_it_drops() {
    let r = scoped_repo();
    let id = "TASK-000000000d01";

    // Added: stored in normal form beside the one already there.
    let out = r.ank(
        "claude-code@ank",
        &["amend", id, "--scope", ".\\src\\auth\\**"],
    );
    assert!(out.status.success(), "{}", erred(&out));
    let scope = stored_scope(&said(&r.ank("claude-code@ank", &["show", id])));
    assert_eq!(scope, ["docs/**", "src/auth/**"], "stored as typed");

    // Dropped: the stored form is normal, so a raw argument would match no
    // glob and be refused as absent from a scope that carries it.
    let out = r.ank(
        "claude-code@ank",
        &["amend", id, "--drop-scope", "./src/auth/**"],
    );
    assert!(out.status.success(), "{}", erred(&out));
    let scope = stored_scope(&said(&r.ank("claude-code@ank", &["show", id])));
    assert_eq!(
        scope,
        ["docs/**"],
        "the drop did not take, or took too much"
    );
}

// ---------------------------------------------------------------------------
// The git boundary (TASK-2f01baf94632)
// ---------------------------------------------------------------------------
//
// Through the binary and **without `--repo`**, which is the only way to reach
// this at all: the finding is about the walk `discover` performs from the
// working directory, and every other test in this file short-circuits that walk
// by naming the repository.

/// Runs `ank` from `dir` with no `--repo`, so resolution is the walk.
fn ank_walking(dir: &Path, args: &[&str]) -> Output {
    ank_command()
        .args(args)
        .env("ANK_AGENT", "claude-code@ank")
        .current_dir(dir)
        .output()
        .expect("the binary must have been built")
}

/// A git repository of its own, with no `.ank/`, nested inside `outer`.
///
/// The layout nobody sets up on purpose — a checkout cloned inside another
/// checkout — and the one where the walk succeeds at the wrong place.
fn nest_a_repository_in(outer: &Repo) -> PathBuf {
    let inner = outer.0.join("inner");
    std::fs::create_dir_all(inner.join("src")).unwrap();
    for args in [
        &["init", "-q", "-b", "main"][..],
        &["config", "user.email", "t@ank.local"][..],
        &["config", "user.name", "T"][..],
    ] {
        let out = git_command(&inner)
            .args(args)
            .output()
            .expect("git is a hard dependency");
        assert!(out.status.success(), "git {args:?}");
    }
    inner
}

#[test]
fn a_verb_resolving_a_corpus_across_a_git_boundary_names_the_root() {
    let outer = Repo::new();
    let inner = nest_a_repository_in(&outer);
    let root = outer.0.file_name().unwrap().to_string_lossy().to_string();

    let out = ank_walking(&inner.join("src"), &["status"]);
    // Degrade, do not fail (§2): the walk succeeded, and the caller may well
    // have meant it.
    assert!(out.status.success(), "{}", erred(&out));

    let err = erred(&out);
    assert!(err.contains("warning:"), "nothing was said at all: {err}");
    assert!(
        err.contains(&root),
        "the resolved root is not named, which is the whole criterion: {err}"
    );
    // Claims are git refs, and that is why this is not merely reading the wrong
    // files: the refs land in the outer repository while the code being changed
    // is versioned by the inner one.
    assert!(err.contains("claims"), "{err}");
    // Both ways out, named.
    assert!(err.contains("ank init") && err.contains("--repo"), "{err}");
}

#[test]
fn the_boundary_warning_never_reaches_standard_output() {
    // §4 requires --json to stay byte-for-byte what a caller's parser already
    // reads. A line on stdout would break every one of them to say something no
    // parser asked for -- so the warning is on stderr, and this is what holds it
    // there.
    let outer = Repo::new();
    let inner = nest_a_repository_in(&outer);

    for args in [
        &["status", "--json"][..],
        &["find", "--status", "open", "--json"][..],
    ] {
        let out = ank_walking(&inner.join("src"), args);
        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
        assert!(
            !stdout.contains("warning"),
            "{args:?} put the warning on stdout: {stdout}"
        );
        assert!(
            stdout.trim().is_empty() || stdout.trim_start().starts_with('{'),
            "{args:?} left stdout unparseable: {stdout}"
        );
        // It was still said, on the stream that carries it.
        assert!(erred(&out).contains("warning:"), "{args:?} said nothing");
    }
}

#[test]
fn naming_the_repository_is_what_silences_the_boundary_warning() {
    let outer = Repo::new();
    let inner = nest_a_repository_in(&outer);

    // `--repo` is the caller saying which corpus they mean. Warning about an
    // answer that was asked for by name would fire forever in the one layout
    // this behaviour makes usable: a single `.ank/` above several checkouts.
    let out = ank_walking(
        &inner.join("src"),
        &["status", "--repo", outer.0.to_str().unwrap()],
    );
    assert!(
        !erred(&out).contains("warning:"),
        "an explicit --repo was still warned about: {}",
        erred(&out)
    );

    // And --quiet means no chatter, here as everywhere.
    let out = ank_walking(&inner.join("src"), &["status", "--quiet"]);
    assert!(!erred(&out).contains("warning:"), "{}", erred(&out));
}

#[test]
fn an_ordinary_subdirectory_of_the_same_repository_is_not_warned_about() {
    // The guard against a warning that fires when nothing is wrong. Walking up
    // from a subdirectory is the nominal case -- §6 specifies it -- and a
    // warning there would be noise nobody reads the day something is actually
    // wrong.
    let outer = Repo::new();
    let deep = outer.0.join("crates").join("ank-cli").join("src");
    std::fs::create_dir_all(&deep).unwrap();

    let out = ank_walking(&deep, &["status"]);
    assert!(out.status.success(), "{}", erred(&out));
    assert!(
        !erred(&out).contains("warning:"),
        "the nominal walk was warned about: {}",
        erred(&out)
    );
}

// ---------------------------------------------------------------------------
// A corpus written by a newer binary (TASK-ca7b61b00896)
// ---------------------------------------------------------------------------

// The second thing `startup` says before a verb runs, and it is asserted here
// for the reason the boundary warning above is: it exists to reach somebody
// running an ordinary verb, and no unit test can establish that a verb reaches
// it. The failure it names is silent by construction -- a listing that came up
// short says nothing about what it left out -- so the assertions below check
// both halves at once: that the warning is there, and that the entity really
// is missing from the answer it accompanies.

/// A corpus one schema past this build, plus one entity this build reads.
///
/// Both, deliberately: a corpus made only of unreadable entities would be
/// indistinguishable from an empty one, and the case that costs a reader time
/// is precisely the one where the listing answers and looks complete.
fn a_corpus_one_schema_ahead() -> Repo {
    let r = Repo::new();
    r.seed_task_at_schema("TASK-aaaaaaaaaaaa", ank_core::SCHEMA_VERSION + 1);
    r.seed_task("TASK-bbbbbbbbbbbb", Some("A verifiable criterion."));
    r
}

#[test]
fn a_corpus_one_schema_ahead_is_named_by_every_verb_that_reads_it() {
    let r = a_corpus_one_schema_ahead();
    let ahead = (ank_core::SCHEMA_VERSION + 1).to_string();
    let reads = ank_core::SCHEMA_VERSION.to_string();

    // Verbs that list, and verbs that show -- the two shapes the criterion
    // names, because they fail differently. A listing drops the entity and
    // still answers; `show` on the entity itself is the one place the parse
    // failure was already visible.
    for args in [
        &["context"][..],
        &["find"][..],
        &["status"][..],
        &["graph"][..],
        &["show", "TASK-bbbbbbbbbbbb"][..],
        &["show", "TASK-aaaaaaaaaaaa"][..],
    ] {
        let err = erred(&r.ank("claude-code@ank", args));
        assert!(err.contains("warning:"), "{args:?} said nothing: {err}");
        // The schema found and the newest supported, both named: one number
        // alone tells a reader nothing about which side is behind.
        assert!(
            err.contains(&format!("schema {ahead}")) && err.contains(&format!("reads {reads}")),
            "{args:?} named neither schema: {err}"
        );
        // And what to do, which is not a migration: the corpus is fine and the
        // binary is old.
        assert!(
            err.contains("ank --version") && err.contains("npm install -g @haksolot/ank"),
            "{args:?} left the reader with no next step: {err}"
        );
    }

    // The half that makes the warning necessary. `find` answers, exits zero,
    // and its answer is short by exactly the entity it cannot read -- which is
    // what "answering as if the file were understood" looks like from the
    // outside.
    let out = r.ank("claude-code@ank", &["find"]);
    assert!(out.status.success(), "{}", erred(&out));
    let listed = said(&out);
    assert!(listed.contains("TASK-bbbb"), "{listed}");
    assert!(
        !listed.contains("TASK-aaaa"),
        "the unreadable entity was listed after all, so the premise is wrong: {listed}"
    );
}

#[test]
fn a_corpus_this_binary_reads_is_not_warned_about() {
    // The guard against a warning that fires when nothing is wrong. Every
    // other fixture in this file seeds schema 1, which this build reads, so a
    // false positive here would be a warning printed by the whole suite.
    let r = Repo::new();
    r.seed_task("TASK-cccccccccccc", Some("A verifiable criterion."));
    let err = erred(&r.ank("claude-code@ank", &["context"]));
    assert!(!err.contains("warning:"), "{err}");
}

#[test]
fn the_schema_warning_survives_repo_and_yields_to_quiet() {
    let r = a_corpus_one_schema_ahead();

    // Unlike the boundary warning, `--repo` does not silence this one. Naming
    // the corpus is the caller saying they meant this one; it says nothing
    // about whether the binary can read it. `Repo::ank` passes `--repo`
    // already, so every assertion above rode that path -- this states it.
    let err = erred(&r.ank("claude-code@ank", &["status"]));
    assert!(err.contains("warning:"), "{err}");

    // And --quiet means no chatter, here as everywhere.
    let err = erred(&r.ank("claude-code@ank", &["status", "--quiet"]));
    assert!(!err.contains("warning:"), "{err}");
}

#[test]
fn the_schema_warning_never_reaches_standard_output() {
    // §4 requires --json to stay byte-for-byte what a caller's parser already
    // reads, so the warning goes on stderr and this is what holds it there.
    let r = a_corpus_one_schema_ahead();

    for args in [&["status", "--json"][..], &["find", "--json"][..]] {
        let out = r.ank("claude-code@ank", args);
        let stdout = said(&out);
        assert!(
            !stdout.contains("warning"),
            "{args:?} put the warning on stdout: {stdout}"
        );
        assert!(
            stdout.trim_start().starts_with('{'),
            "{args:?} left stdout unparseable: {stdout}"
        );
        assert!(erred(&out).contains("warning:"), "{args:?} said nothing");
    }
}

/// The guide carries the sentence the binary prints, and not a paraphrase of it
/// (TASK-ca7b61b00896).
///
/// Same shape as `the_guide_documents_the_identity_the_way_out_tells_you_to_set`
/// and for the same reason: the warning tells a reader their binary is behind,
/// and `getting-started.md` is where they will look for what that means. A
/// guide that describes a different message is a guide that has drifted.
///
/// Compared with the digits removed from both sides. The numbers belong to one
/// fixture's corpus, and pinning them would turn this red the day
/// `SCHEMA_VERSION` moves — which is the one day the sentence itself is still
/// exactly right.
#[test]
fn the_guide_carries_the_warning_the_binary_prints_about_a_newer_corpus() {
    fn skeleton(s: &str) -> String {
        s.chars().filter(|c| !c.is_ascii_digit()).collect()
    }

    let r = a_corpus_one_schema_ahead();
    let err = erred(&r.ank("claude-code@ank", &["find"]));
    let warned: Vec<&str> = err
        .lines()
        .filter(|l| l.contains("warning:") || l.trim_start().starts_with("-> the binary"))
        .collect();
    assert_eq!(warned.len(), 2, "the warning is two lines: {err}");

    let guide = skeleton(
        &std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/getting-started.md"),
        )
        .expect("the guide is in the repository the tests run from"),
    );
    for line in warned {
        assert!(
            guide.contains(skeleton(line.trim()).trim()),
            "the binary prints {line:?} and the guide never says it"
        );
    }

    // And the half no corpus can announce, which is why it is documented at
    // all: an old binary reading an old corpus looks exactly like a current
    // one, so `--version` is the only answer there.
    assert!(
        guide.contains("tracks the published release, not the tree"),
        "the guide never says where the binary a contributor runs comes from"
    );
}

#[test]
fn help_answers_about_config_the_way_it_answers_about_every_verb() {
    let r = Repo::new();
    let text = said(&r.ank("claude-code@ank", &["help", "config"]));
    assert!(text.contains("ank config <key> [<value>]"), "{text}");
    // The short form of §4's table, on the surface that teaches one verb.
    assert!(text.contains("-u, --unset"), "{text}");
    // What it refuses, with the code -- the question is asked before the call.
    assert!(text.contains("(1)") && text.contains("(7)"), "{text}");
    // The key set, which is the one thing a caller cannot guess.
    assert!(text.contains("verifiers.<name>.run"), "{text}");

    // And the listing keeps one line for the verb: the heading names the moment
    // a group of verbs belongs to, never a verb of its own.
    let listing = said(&r.ank("claude-code@ank", &["help"]));
    assert!(listing.contains("ank config <key> [<value>]"), "{listing}");
    assert!(listing.contains("--unset"), "{listing}");
}

// ---------------------------------------------------------------------------
// Constraint drift at done (TASK-bfa325e55424)
// ---------------------------------------------------------------------------
//
// Through the binary, and the task said why in advance: asserting that a hash
// comparison returns false proves nothing about the path `done` actually takes,
// and that is precisely how two earlier defects in this repo passed green unit
// tests. So the fixture accepts a real constraint, with a real signature, over
// a scope a real claim already froze.

/// A repository where a task is claimed and one constraint can be landed over
/// its scope afterwards.
fn drift_fixture(task: &str) -> Repo {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: git --version\n");
    r.enable_signing();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_task_with(task, Some("A verifiable criterion."), &["ok"]);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    r
}

#[test]
fn done_warns_when_a_constraint_landed_over_the_scope_while_the_claim_was_held() {
    const TASK: &str = "TASK-0000000000dd";
    const ADR: &str = "ADR-0000000000ce";
    let r = drift_fixture(TASK);

    // The claim freezes the constraints applicable to `src/**` -- none yet.
    let out = r.ank("claude-code@ank", &["claim", TASK]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // A rule lands over those files while the work is in progress. Accepted for
    // real: `bearing_on` counts accepted ADRs and nothing else, so a proposed
    // one would leave the hash where it was and this test would pass on a
    // binary that never looked.
    r.seed_adr(ADR, "Every session goes through the store.", "src/**");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "adr"]);
    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let out = r.ank("claude-code@ank", &["done", TASK]);

    // It warns, and it does not block: a rule that landed after the work
    // started does not necessarily concern work already finished, and refusing
    // would punish exactly the case §7 singles out.
    assert_eq!(code(&out), 0, "done was blocked: {}", stderr(&out));
    assert!(
        stdout(&out).contains("-> done"),
        "the transition did not happen: {}",
        stdout(&out)
    );

    let err = stderr(&out);
    assert!(
        err.contains("constraints over this scope changed"),
        "done completed in silence, which is the whole defect: {err}"
    );
    assert!(
        err.contains("ank context"),
        "the warning names no next command: {err}"
    );

    // On stderr and not on stdout: the `running:` lines already make
    // `done --json` unparseable, and a second line there would deepen a defect
    // rather than avoid it.
    assert!(
        !stdout(&out).contains("constraints over this scope"),
        "the warning reached stdout: {}",
        stdout(&out)
    );
}

#[test]
fn done_says_nothing_when_the_constraints_did_not_move() {
    // The control, and it is what makes the test above mean anything: a warning
    // that fires on every `done` would be a warning nobody reads on the one
    // that matters.
    const TASK: &str = "TASK-0000000000de";
    const ADR: &str = "ADR-0000000000cf";
    let r = drift_fixture(TASK);

    // Accepted *before* the claim, so it is inside the frozen hash.
    r.seed_adr(ADR, "Every session goes through the store.", "src/**");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "adr"]);
    assert_eq!(code(&r.ank("marie@laptop", &["accept", ADR])), 0);

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", TASK])), 0);
    let out = r.ank("claude-code@ank", &["done", TASK]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        !stderr(&out).contains("constraints over this scope changed"),
        "warned with nothing to warn about: {}",
        stderr(&out)
    );
}

// ---------------------------------------------------------------------------
// --json is data, on every verb (TASK-2eefcdd80124)
// ---------------------------------------------------------------------------
//
// §4: `--json` "is data, and it stays byte-for-byte what a caller's parser
// already reads". Three verbs contradicted it -- `done`'s progress lines, and
// the takeover warnings of `log` and `amend` -- and `cli.rs` named them in a
// comment that treated the situation as a given.
//
// The tests come in two halves, and the criterion asks for both. The sweep
// walks every verb the binary offers, so a line added later to a fourth is
// caught without anybody remembering to name it. The three cases below it reach
// the states the sweep cannot set up, and assert the information was moved
// rather than dropped.

/// Standard output holds one JSON document and nothing else, or nothing at all.
///
/// Ank emits its JSON on a single line, so this is exact rather than an
/// approximation of a parser: any second line, and any line not opening a
/// document, is what a caller would choke on.
fn assert_json_only(out: &Output, what: &str) {
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    if text.trim().is_empty() {
        return;
    }
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(
        lines.len(),
        1,
        "{what} put {} lines on stdout under --json; a parser reads the first \
         one:\n{text}",
        lines.len()
    );
    let line = lines[0].trim();
    assert!(
        line.starts_with('{') && line.ends_with('}'),
        "{what} put something other than a JSON document on stdout: {line}"
    );
    assert!(!line.contains('\u{1b}'), "{what} coloured its JSON: {line}");
}

/// Every verb, invoked under `--json` in a repository where it has work to do.
///
/// The arguments are the smallest that reach past parsing; a verb that refuses
/// on state still proves the point, because a refusal writes to stderr and must
/// leave stdout empty rather than half a document.
///
/// `$EDITOR` is removed for the sweep. `edit` and the interactive form of `new`
/// otherwise spawn whatever the machine running the suite has exported and wait
/// for it — the sweep hung on exactly that. Removed, they refuse with code 9,
/// which is a state worth sweeping too: a refusal must leave stdout empty
/// rather than half a document.
///
/// **What it catches, measured rather than claimed.** A line printed
/// unconditionally by a verb is caught: adding one to `find` turns this red
/// with `` `ank find criterion --json` put 2 lines on stdout ``. A line printed
/// only on a branch these arguments do not take is not — the same line placed
/// inside `find`'s `no match` arm passed, because `--json` never reaches it.
/// That is the honest boundary of a sweep, and it is where the three cases
/// below take over: they reach the states this one cannot set up. All three
/// offenders §4 was contradicted by were unconditional, which is what makes the
/// sweep the right first net.
#[test]
fn no_verb_puts_anything_but_json_on_stdout_under_json() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: git --version\n");
    r.seed_task(ID, Some("A verifiable criterion."));
    r.seed_adr("ADR-0000000000ef", "A binding rule.", "src/**");

    // One invocation per verb the listing offers, so a verb added later is
    // swept without anybody adding it here. The map supplies only what parsing
    // demands.
    for (verb, _usage, _flags) in surface() {
        let extra: &[&str] = match verb.as_str() {
            "claim" | "show" | "accept" | "close" | "amend" | "attest" | "edit" => &[ID],
            "log" => &[ID],
            "find" => &["criterion"],
            "scope" => &["src"],
            "config" => &["claim_ttl_max"],
            "new" => &["task", "--title", "T", "--scope", "src/**"],
            "help" => &[],
            _ => &[],
        };
        let mut args: Vec<&str> = vec![verb.as_str()];
        args.extend_from_slice(extra);
        args.push("--json");
        let out = r.ank_edit("claude-code@ank", &args, None);
        assert_json_only(&out, &format!("`ank {}`", args.join(" ")));
    }
}

#[test]
fn done_reports_its_progress_on_standard_error_and_still_reports_it() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: git --version\n");
    r.seed_task_with(
        "TASK-0000000000f1",
        Some("A verifiable criterion."),
        &["ok"],
    );
    assert_eq!(
        code(&r.ank("claude-code@ank", &["claim", "TASK-0000000000f1"])),
        0
    );

    let out = r.ank("claude-code@ank", &["done", "TASK-0000000000f1", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // The document a caller parses, alone.
    assert_json_only(&out, "ank done --json");
    assert!(
        stdout(&out).contains("\"status\":\"done\""),
        "{}",
        stdout(&out)
    );

    // Moved, not dropped: progress on a long verifier run is exactly what a
    // human wants, and it is still there.
    assert!(
        stderr(&out).contains("running: ok ... ok"),
        "the progress line was dropped rather than moved: {}",
        stderr(&out)
    );
}

#[test]
fn the_takeover_warnings_of_log_and_amend_are_on_standard_error() {
    // Both fire only while another agent holds the claim, which is a state the
    // sweep above cannot set up -- so they are reached deliberately here.
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("codex@host-9", &["claim", ID])), 0);

    // `amend`, whose scope change moves what the live claim anchors.
    let out = r.ank("marie@laptop", &["amend", ID, "--scope", "docs/**"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("codex@host-9"),
        "silence would be worse, and it must name the holder: {}",
        stderr(&out)
    );
    assert!(
        !stdout(&out).contains("warning"),
        "the warning reached stdout: {}",
        stdout(&out)
    );

    // And under --json the document stands alone.
    let out = r.ank(
        "marie@laptop",
        &["amend", ID, "--scope", "src/extra/**", "--json"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_json_only(&out, "ank amend --json");
    assert!(stderr(&out).contains("codex@host-9"), "{}", stderr(&out));
}

// ---------------------------------------------------------------------------
// A dead scope, and where git says the file went (ADR-97beaf55e73a)
// ---------------------------------------------------------------------------

const DEAD_ADR: &str = "ADR-00000000aaaa";

/// A body long enough for git's rename detection to fire on it.
///
/// `-M` is a similarity heuristic: a one-line file renamed and a one-line file
/// deleted beside a new one-line file are the same event to it. A fixture
/// sitting under the threshold would exercise the fallback while claiming to
/// exercise the detection, and would pass for the wrong reason.
const SIMILAR: &str = "fn main() {\n    let a = 1;\n    let b = 2;\n    println!(\"{a}{b}\");\n}\n";

/// A repository whose history holds one commit that moved `from` — or deleted
/// it, when `to` is `None` — with `from` named literally in a proposed ADR.
fn moved_fixture(from: &str, to: Option<&str>) -> Repo {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join(from), SIMILAR).unwrap();
    r.seed_adr(DEAD_ADR, "Do not do X.", from);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    match to {
        Some(to) => std::fs::rename(r.0.join(from), r.0.join(to)).unwrap(),
        None => std::fs::remove_file(r.0.join(from)).unwrap(),
    }
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "move it"]);
    r
}

/// The note lines of the one finding that has any, connector stripped.
///
/// Reads the drawn output rather than `--json`, because what these tests are
/// about is what a reader is shown, and the structure layer is part of that.
/// §4's alphabet is closed, so the two leads below are the two that exist and a
/// third would fail here rather than pass unnoticed.
fn proposal(text: &str) -> Option<Vec<String>> {
    let mut lines: Vec<String> = Vec::new();
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("└── ") {
            assert!(lines.is_empty(), "two notes in one output: {text}");
            lines.push(rest.to_string());
        } else if !lines.is_empty() {
            match line.strip_prefix("    ") {
                Some(rest) => lines.push(rest.to_string()),
                None => break,
            }
        }
    }
    (!lines.is_empty()).then_some(lines)
}

/// The finding keeps its wording, and the note under it answers the question
/// the reader has at that moment.
///
/// Through the binary because the walk is two `git` processes and a parser
/// agreeing with each other, and neither half proves the answer reached
/// standard output carrying the other.
///
/// **The proposed command is run, not merely matched.** ADR-97beaf55e73a
/// requires a command that will not refuse on the spot, and the only way to
/// assert that is to spend it: the text is split back into arguments and handed
/// to the binary, which must exit 0 and leave the scope alive.
#[test]
fn a_renamed_file_names_where_it_went_and_the_command_that_repairs_it() {
    let r = moved_fixture("src/old.rs", Some("src/new.rs"));
    let head = r.git(&["rev-parse", "HEAD"]);

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(
        code(&out),
        0,
        "a dead scope git can explain is a signal (TASK-27cf26cbc414): {}",
        stderr(&out)
    );
    let text = stdout(&out);

    // The wording is untouched, and only the severity moved. TASK-1e79ff3738df
    // asked that neither change; TASK-27cf26cbc414 changed exactly one of them,
    // and this assertion is what keeps the other where it was.
    assert!(
        text.contains("dead scope 'src/old.rs': no file matches it"),
        "{text}"
    );
    assert!(
        text.contains("signal: ") && !text.contains("error: "),
        "the finding is a signal and nothing here is a fault: {text}"
    );

    let note = proposal(&text).unwrap_or_else(|| panic!("the rename is named: {text}"));
    let named = note[0].rsplit(' ').next().unwrap();
    assert!(
        note[0].contains("src/new.rs") && head.starts_with(named),
        "the note must name the new path and the commit that moved it, and \
         the commit is {head}: {note:?}"
    );

    let command = note[1].clone();
    assert!(
        command.starts_with("ank amend ") && command.contains(DEAD_ADR),
        "a proposed ADR is repaired by amend: {command}"
    );

    // Spent rather than read. A command that refuses here is the exact defect
    // ADR-97beaf55e73a names, and no amount of matching on the string sees it.
    let args: Vec<String> = command
        .split_whitespace()
        .skip(1)
        .map(|a| a.trim_matches('"').to_string())
        .collect();
    let argv: Vec<&str> = args.iter().map(String::as_str).collect();
    let repaired = r.ank("claude-code@ank", &argv);
    assert_eq!(
        code(&repaired),
        0,
        "the proposal must not refuse on the spot: `{command}` exited {} — {}",
        code(&repaired),
        stderr(&repaired)
    );

    // And it repaired what it was proposed for.
    let after = stdout(&r.ank("claude-code@ank", &["check"]));
    assert!(
        !after.contains("dead scope"),
        "the proposed command must leave the scope alive: {after}"
    );
}

/// The other half of the criterion, and the one that must add nothing.
///
/// A deletion, a move under the similarity threshold and a scope that never
/// named a real file are one silence to git. The reader is left exactly where
/// they stand today — no proposal, and above all no sentence asserting the file
/// was deleted, because this code cannot know that.
#[test]
fn a_deleted_file_leaves_the_finding_exactly_as_it_was() {
    let deleted = moved_fixture("src/old.rs", None);
    let out = deleted.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 8, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.contains("dead scope 'src/old.rs': no file matches it"),
        "{text}"
    );
    assert_eq!(
        proposal(&text),
        None,
        "git cannot explain a deletion, and nothing may be proposed: {text}"
    );
    for word in ["renamed", "delet", "removed"] {
        assert!(
            !text.contains(word),
            "'{word}' claims to know what became of the file: {text}"
        );
    }

    // What makes the silence mean something: the same corpus and the same
    // finding, differing by the rename and by nothing else.
    let renamed = moved_fixture("src/old.rs", Some("src/new.rs"));
    assert!(
        proposal(&stdout(&renamed.ank("claude-code@ank", &["check"]))).is_some(),
        "the renamed fixture must be explained, or the deleted one proves \
         nothing about the walk"
    );
}

/// `amend` refuses the scope of an accepted ADR with code 6, so proposing it
/// there would name a command that fails on the spot. The proposal is a
/// supersession, and the refusal it avoids is asserted rather than assumed.
#[test]
fn an_accepted_adr_is_offered_a_supersession_and_never_an_amend() {
    let r = Repo::new();
    r.enable_signing();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/old.rs"), SIMILAR).unwrap();
    r.seed_adr(DEAD_ADR, "Do not do X.", "src/old.rs");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    let out = r.ank("marie@laptop", &["accept", DEAD_ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    std::fs::rename(r.0.join("src/old.rs"), r.0.join("src/new.rs")).unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "move it"]);

    let text = stdout(&r.ank("claude-code@ank", &["check"]));
    let note = proposal(&text).unwrap_or_else(|| panic!("the rename is named: {text}"));
    assert!(
        note[1].starts_with("ank new adr --supersedes ") && note[1].contains("src/new.rs"),
        "an accepted ADR is changed by a succession: {note:?}"
    );
    assert!(
        !note[1].contains("ank amend"),
        "amend refuses this with code 6, and naming it would be the defect: {note:?}"
    );

    let refused = r.ank(
        "claude-code@ank",
        &[
            "amend",
            DEAD_ADR,
            "--drop-scope",
            "src/old.rs",
            "--scope",
            "src/new.rs",
        ],
    );
    assert_eq!(
        code(&refused),
        6,
        "the branch exists because amend refuses here: {}",
        stderr(&refused)
    );
}

/// The two task states, which `amend` treats as opposites.
///
/// An open task is amended, and the rename is what tells a typo from a file
/// that moved under the work. A finished one is not: §3 allows a single write
/// to it and that write is a proof, so `amend` exits 7 — and a proposal naming
/// it would be the refusal this feature exists to avoid. The rename is named
/// either way, because where the file went is the answer in both states.
#[test]
fn a_finished_task_is_told_where_the_file_went_and_offered_no_command() {
    for (status, expected) in [("open", Some("ank amend ")), ("done", None)] {
        let r = Repo::new();
        std::fs::create_dir_all(r.0.join("src")).unwrap();
        std::fs::write(r.0.join("src/old.rs"), SIMILAR).unwrap();
        r.seed_task_scoped(ID, "src/old.rs");
        let seeded = r
            .task_text(ID)
            .replace("status: open", &format!("status: {status}"));
        std::fs::write(r.0.join(".ank/entities").join(format!("{ID}.md")), seeded).unwrap();
        r.git(&["add", "-A"]);
        r.git(&["commit", "-qm", "seed"]);
        std::fs::rename(r.0.join("src/old.rs"), r.0.join("src/new.rs")).unwrap();
        r.git(&["add", "-A"]);
        r.git(&["commit", "-qm", "move it"]);

        let text = stdout(&r.ank("claude-code@ank", &["check"]));
        let note = proposal(&text).unwrap_or_else(|| panic!("{status}: no note in {text}"));
        assert!(
            note[0].contains("src/new.rs"),
            "{status}: the rename is named whatever the state: {note:?}"
        );
        match expected {
            Some(verb) => assert!(
                note.get(1).is_some_and(|c| c.starts_with(verb)),
                "{status}: {note:?}"
            ),
            None => assert_eq!(
                note.len(),
                1,
                "{status}: amend refuses a settled plan, so nothing is proposed: {note:?}"
            ),
        }
    }
}

/// Seeds a `done` task scoped to `glob`, which is the state the severity rule
/// of TASK-27cf26cbc414 decides something about: an open task is a signal
/// either way, so a fixture built on one would pass whatever the rule does.
/// The proof is not decoration: a `done` task carrying none is a fault of its
/// own, and these fixtures assert an exit code. Without it the corpus would exit
/// 8 for a reason that has nothing to do with the scope, and the renamed half
/// would fail while the rule it tests worked.
fn finished_task_scoped(r: &Repo, glob: &str) {
    r.seed_task_scoped(ID, glob);
    let seeded = r
        .task_text(ID)
        .replace("status: open", "status: done")
        .replace(
            "schema:",
            "proof:\n  - type: assertion\n    ref: seeded\nschema:",
        );
    std::fs::write(r.0.join(".ank/entities").join(format!("{ID}.md")), seeded).unwrap();
}

/// The severity rule, on the state where it decides something.
///
/// Two fixtures differing by one act, because a single one proves the exit code
/// and not the reason for it. **Renamed:** git names the commit, so the corpus
/// is outdated in a way the reader can follow rather than broken — a signal.
/// **Deleted:** git explains nothing, the reader has nothing, and the fault is
/// what says so.
///
/// Through the binary because the exit code is the whole claim, and no unit test
/// of the severity reaches it.
#[test]
fn a_finished_tasks_dead_scope_faults_only_when_git_cannot_explain_it() {
    for (to, expected) in [(Some("src/new.rs"), 0), (None, 8)] {
        let r = Repo::new();
        std::fs::create_dir_all(r.0.join("src")).unwrap();
        std::fs::write(r.0.join("src/old.rs"), SIMILAR).unwrap();
        finished_task_scoped(&r, "src/old.rs");
        r.git(&["add", "-A"]);
        r.git(&["commit", "-qm", "seed"]);
        match to {
            Some(to) => std::fs::rename(r.0.join("src/old.rs"), r.0.join(to)).unwrap(),
            None => std::fs::remove_file(r.0.join("src/old.rs")).unwrap(),
        }
        r.git(&["add", "-A"]);
        r.git(&["commit", "-qm", "move it"]);

        let out = r.ank("claude-code@ank", &["check"]);
        let text = stdout(&out);
        assert_eq!(code(&out), expected, "to={to:?}: {text}{}", stderr(&out));
        assert!(
            text.contains("dead scope 'src/old.rs': no file matches it"),
            "the wording does not move with the severity: {text}"
        );
        assert_eq!(
            proposal(&text).is_some(),
            to.is_some(),
            "to={to:?}: the explanation and the severity are the same fact: {text}"
        );
    }
}

/// A glob is answered through its literal prefix, and only when one directory
/// answers.
///
/// **This is the half that makes the rule real.** `scope_moved` returned nothing
/// for any glob before this, so the severity change alone would have left four
/// of the six dead scopes of the flat-layout move — all `.ank/adr/**` — faulting
/// while the rule looked implemented.
///
/// **Scattered is the negative control.** Two destinations is not a directory
/// that moved, and "the prefix moved mostly there" is not a sentence this is
/// allowed to print. The files carry distinct bodies so that git pairs each
/// rename with its own source rather than by coincidence of identical content.
#[test]
fn a_glob_is_explained_only_when_its_prefix_moved_to_one_place() {
    for (scattered, expected) in [(false, 0), (true, 8)] {
        let r = Repo::new();
        std::fs::create_dir_all(r.0.join("old")).unwrap();
        for n in ["a.rs", "b.rs"] {
            std::fs::write(r.0.join("old").join(n), format!("{SIMILAR}// {n}\n")).unwrap();
        }
        finished_task_scoped(&r, "old/**");
        r.git(&["add", "-A"]);
        r.git(&["commit", "-qm", "seed"]);

        let second = if scattered { "elsewhere" } else { "new" };
        for dir in ["new", second] {
            std::fs::create_dir_all(r.0.join(dir)).unwrap();
        }
        std::fs::rename(r.0.join("old/a.rs"), r.0.join("new/a.rs")).unwrap();
        std::fs::rename(r.0.join("old/b.rs"), r.0.join(second).join("b.rs")).unwrap();
        r.git(&["add", "-A"]);
        r.git(&["commit", "-qm", "move it"]);

        let out = r.ank("claude-code@ank", &["check"]);
        let text = stdout(&out);
        assert_eq!(code(&out), expected, "scattered={scattered}: {text}");
        assert!(
            text.contains("dead scope 'old/**': no file matches it"),
            "scattered={scattered}: {text}"
        );
        match scattered {
            false => {
                let note = proposal(&text).unwrap_or_else(|| panic!("the prefix is named: {text}"));
                assert!(
                    note[0].starts_with("git records old renamed to new in"),
                    "the note names the directory git recorded, not the glob: {note:?}"
                );
            }
            true => {
                assert_eq!(
                    proposal(&text),
                    None,
                    "two destinations is not a directory that moved: {text}"
                );
                for word in ["renamed", "moved", "elsewhere"] {
                    assert!(
                        !text.contains(word),
                        "'{word}' claims more than git recorded: {text}"
                    );
                }
            }
        }
    }
}

/// A `file://` URL for a local path, and the one form of it this suite uses.
///
/// The URL is the path with `file://` in front and nothing else done to it. On
/// Unix the path opens with a slash and the result is the ordinary three-slash
/// form; on Windows it opens with a drive letter and the result is
/// `file://C:/...`, two slashes. The three-slash form is what a reader expects
/// there and it is the one that fails: git-for-Windows rewrites it, unless
/// `MSYS_NO_PATHCONV` or `MSYS2_ARG_CONV_EXCL` is set -- either alone is enough
/// -- and then git reads the literal `/C:/...` as a path and refuses. Measured
/// both ways on git 2.54.0.windows.1: the two-slash form clones, and stays
/// shallow, in all four combinations of those two variables. So the form below
/// is not a Windows workaround but the one that does not depend on the
/// environment an agent's shell happens to export, which is how two sessions
/// reported this test red while a third watched it pass (TASK-143a310de8b6).
///
/// **One function and not one expression per caller**, because that measurement
/// is the whole of what makes it right: a second site deriving the URL again is
/// a second chance to write the three-slash form back in.
fn file_url(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy().replace('\\', "/"))
}

/// A clone of `r`, truncated to `depth` when one is given.
///
/// Through a `file://` URL and not a path, because git ignores `--depth` on a
/// local path clone: without the URL the shallow fixture would quietly be a
/// whole one, and the test would pass while testing nothing.
fn clone_of(r: &Repo, depth: Option<u32>) -> PathBuf {
    let name = r.0.file_name().unwrap().to_string_lossy().to_string();
    let dest = r.0.with_file_name(match depth {
        Some(d) => format!("{name}-clone{d}"),
        None => format!("{name}-clone"),
    });
    let url = file_url(&r.0);
    let dest_s = dest.to_string_lossy().to_string();
    let d = depth.map(|d| d.to_string());
    let mut args = vec!["clone", "-q"];
    if let Some(d) = d.as_deref() {
        args.extend_from_slice(&["--depth", d]);
    }
    args.extend_from_slice(&[url.as_str(), dest_s.as_str()]);
    r.git(&args);
    // The truncation is the fixture, so it is asserted and not assumed: a clone
    // that succeeded whole would leave every assertion below still passing for
    // the wrong reason, which is the failure the URL exists to prevent.
    assert_eq!(
        dest.join(".git/shallow").exists(),
        depth.is_some(),
        "clone of {url} at depth {depth:?}: .git/shallow is what makes it the third state"
    );
    dest
}

/// The third state (§4): a history that cannot answer.
///
/// A shallow clone holds no commit that could record the rename, so "git has the
/// history and recorded none" and "there is no history to ask" are different
/// answers, and only the first is evidence. Faulting on the second makes the
/// health of a corpus depend on how it was cloned -- measured on this project's
/// own pipeline, where a depth-1 checkout turned six signals into six faults with
/// no note at all (TASK-2ce5554d6ed0).
///
/// Three clones of two histories, because the claim is a *difference* between
/// clones and no single one can show it.
#[test]
fn a_shallow_clone_cannot_explain_a_dead_scope_and_says_so_instead_of_faulting() {
    let r = moved_fixture("src/old.rs", Some("src/new.rs"));

    // Whole: git has the history, and the answer is the one TASK-27cf26cbc414
    // established.
    let whole = clone_of(&r, None);
    let out = r.ank_at("claude-code@ank", &["check"], &whole);
    let text = stdout(&out);
    assert_eq!(code(&out), 0, "{text}{}", stderr(&out));
    assert!(
        text.contains("src/new.rs"),
        "a whole clone names where it went: {text}"
    );

    // The same history, truncated. The corpus is byte for byte the one above.
    let shallow = clone_of(&r, Some(1));
    let out = r.ank_at("claude-code@ank", &["check"], &shallow);
    let text = stdout(&out);
    assert_eq!(
        code(&out),
        0,
        "the shape of a clone is not a defect in the corpus: {text}"
    );
    let note = proposal(&text).unwrap_or_else(|| panic!("the third state is named: {text}"));
    assert!(
        note[0].contains("shallow") && note[0].contains("git fetch --unshallow"),
        "it says the history cannot answer, and what to run: {note:?}"
    );
    for word in ["src/new.rs", "renamed", "delet"] {
        assert!(
            !text.contains(word),
            "'{word}': a clone that cannot see the rename must claim nothing about it: {text}"
        );
    }

    // And the fault survives where git does have the history and records
    // nothing, or this would have bought the green by giving up the check.
    let deleted = moved_fixture("src/old.rs", None);
    let whole = clone_of(&deleted, None);
    let out = deleted.ank_at("claude-code@ank", &["check"], &whole);
    assert_eq!(
        code(&out),
        8,
        "a deletion git can see is still a fault: {}",
        stdout(&out)
    );
}

/// No repository, no walk, and no line saying so.
///
/// The cost clause of ADR-97beaf55e73a has a silence clause beside it, and the
/// silence is the part a test has to hold: `check` already says once that the
/// coordination half was skipped, and a second sentence about a question that
/// could not be asked is noise in the one output an agent reads whole.
#[test]
fn outside_a_repository_the_rename_walk_is_skipped_without_a_word() {
    let b = Bare::new();
    std::fs::write(
        b.0.join(".ank/entities").join(format!("{DEAD_ADR}.md")),
        format!(
            "---\nid: {DEAD_ADR}\ntype: adr\nslug: example\ntitle: A decision\n\
             created: 2026-07-20T00:00:00Z\nstatus: proposed\nscope:\n  - src/old.rs\n\
             constraint: |\n  Do not do X.\nschema: 1\nversion: 1\n---\n\nWhy.\n"
        ),
    )
    .unwrap();

    let out = b.ank(&["check"]);
    assert_eq!(
        code(&out),
        8,
        "the dead scope is still a fault: {}",
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(
        text.contains("dead scope 'src/old.rs': no file matches it"),
        "{text}"
    );
    assert_eq!(proposal(&text), None, "{text}");
    for word in ["rename", "git records"] {
        assert!(
            !text.contains(word),
            "'{word}' is a word about a walk that never ran: {text}"
        );
    }
}

/// `--json` carries the note as data, and no structure character with it.
#[test]
fn the_note_reaches_json_as_a_list_and_not_as_drawn_text() {
    let r = moved_fixture("src/old.rs", Some("src/new.rs"));
    let out = r.ank("claude-code@ank", &["check", "--json"]);
    assert_json_only(&out, "ank check --json");
    let text = stdout(&out);
    assert!(
        text.contains("\"note\":[\"git records src/old.rs renamed to src/new.rs in "),
        "{text}"
    );
    assert!(
        text.contains("ank amend "),
        "the command a caller would run is data too: {text}"
    );
    for glyph in ["└", "├", "│"] {
        assert!(
            !text.contains(glyph),
            "--json carries no structure layer (ADR-0c8ab846d262): {text}"
        );
    }
    // A finding with nothing to add carries the key and an empty list, so a
    // parser reads one shape rather than two.
    assert!(text.contains("\"note\":[]"), "{text}");
}

// ---------------------------------------------------------------------------
// The previous layout, read for one window (§6, ADR-c9f9d0d6f05d)
// ---------------------------------------------------------------------------
//
// Three fixtures, and the third is the one that bites: a corpus in the previous
// layout, one in the flat layout, and one holding both at once. Through the
// binary, because what has to be true is what a reader of a real corpus gets,
// and every one of these paths is reached by dispatch and not by a function.

const LEGACY_TASK: &str = "TASK-00000000fa01";
const LEGACY_ADR: &str = "ADR-00000000fa02";

/// A reader accepts the previous layout. Every verb answers, and `check` says
/// so **once**, as a signal, naming the command that moves it.
#[test]
fn a_corpus_in_the_previous_layout_is_read_and_reported_once() {
    let r = Repo::new();
    r.seed_task_legacy(LEGACY_TASK, "A task written before the move");
    r.seed_adr_legacy(LEGACY_ADR, "Do not do the thing.");
    // A scope that matches something: a dead scope is a fault of its own and
    // would drown the one thing this fixture is about.
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/lib.rs"), "fn main() {}\n").unwrap();

    let out = r.ank("claude-code@ank", &["show", LEGACY_ADR]);
    assert_eq!(code(&out), 0, "both kinds are read: {}", stderr(&out));
    assert!(
        stdout(&out).contains("Do not do the thing."),
        "{}",
        stdout(&out)
    );

    let out = r.ank("claude-code@ank", &["show", LEGACY_TASK]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("A task written before the move"),
        "{}",
        stdout(&out)
    );

    let out = r.ank("claude-code@ank", &["find", "--status", "open"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains(&LEGACY_TASK[..9]), "{}", stdout(&out));

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(
        code(&out),
        0,
        "a corpus that still reads is not broken: {}",
        stderr(&out)
    );
    let text = stdout(&out);
    let mentions: Vec<&str> = text
        .lines()
        .filter(|l| l.contains("previous layout"))
        .collect();
    assert_eq!(
        mentions.len(),
        1,
        "once for the corpus, never per file: {text}"
    );
    let line = mentions[0];
    assert!(
        line.starts_with("signal:"),
        "a signal and not a fault: {line}"
    );
    assert!(line.contains('2'), "it counts what is left: {line}");
    assert!(
        line.contains("git mv") && line.contains(".ank/entities/"),
        "it names the command that moves it: {line}"
    );
}

/// The flat layout is what a writer produces, and it earns no finding at all.
#[test]
fn a_corpus_in_the_flat_layout_carries_no_leftover_finding() {
    let r = Repo::new();
    r.seed_task(LEGACY_TASK, Some("A verifiable criterion."));

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        !stdout(&out).contains("previous layout"),
        "{}",
        stdout(&out)
    );

    // And a new entity lands there rather than in a directory named after its
    // kind, whichever kind it is.
    let out = r.ank(
        "claude-code@ank",
        &["new", "task", "--title", "Fresh", "--scope", "src/**"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(!r.0.join(".ank/tasks").exists(), "no writer produces it");
    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "adr",
            "--title",
            "Fresh",
            "--scope",
            "src/**",
            "--constraint",
            "Do not.",
        ],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(!r.0.join(".ank/adr").exists(), "nor for the other kind");
}

/// Both at once: **one corpus, no entity counted twice, and the flat copy
/// wins.** An id resolving in two directories that produced two entities, or
/// silently preferred whichever the filesystem enumerated first, is how a corpus
/// grows two versions of a task that disagree.
#[test]
fn a_corpus_holding_both_layouts_is_one_corpus_and_the_flat_copy_wins() {
    let r = Repo::new();
    r.seed_task_legacy(LEGACY_TASK, "The copy left behind");
    r.seed_task(LEGACY_TASK, Some("A verifiable criterion."));
    assert!(r.legacy_task_path(LEGACY_TASK).exists());
    assert!(r.flat_task_path(LEGACY_TASK).exists());

    // Listed once.
    let out = r.ank("claude-code@ank", &["find", "--status", "open"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let hits = stdout(&out)
        .lines()
        .filter(|l| l.contains(&LEGACY_TASK[..9]))
        .count();
    assert_eq!(hits, 1, "one entity, one line: {}", stdout(&out));

    // And read from the copy that counts.
    let out = r.ank("claude-code@ank", &["show", LEGACY_TASK]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("Example task") && !stdout(&out).contains("The copy left behind"),
        "the flat copy is the newer one by construction: {}",
        stdout(&out)
    );

    // check counts the entity once and still reports the leftover.
    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(text.contains("1 tasks"), "one entity, not two: {text}");
    assert_eq!(
        text.lines()
            .filter(|l| l.contains("previous layout"))
            .count(),
        1,
        "{text}"
    );
}

/// A write lands in `entities/` **and leaves nothing behind**, which is what
/// keeps the both-at-once state from being something a normal loop produces.
#[test]
fn a_write_moves_an_entity_out_of_the_previous_layout() {
    let r = Repo::new().with_verifiers("");
    r.seed_task_legacy(LEGACY_TASK, "A task written before the move");
    assert!(r.legacy_task_path(LEGACY_TASK).exists());

    let out = r.ank("claude-code@ank", &["claim", LEGACY_TASK]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    assert!(
        r.flat_task_path(LEGACY_TASK).exists(),
        "every write lands in .ank/entities/"
    );
    assert!(
        !r.legacy_task_path(LEGACY_TASK).exists(),
        "and does not leave the old copy to disagree with it"
    );
    let text = std::fs::read_to_string(r.flat_task_path(LEGACY_TASK)).unwrap();
    assert!(text.contains("status: in_progress"), "{text}");

    // The corpus is whole again, so the finding goes with it.
    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        !stdout(&out).contains("previous layout"),
        "{}",
        stdout(&out)
    );
}

/// `index.db` is derived: it rebuilds from either layout, and deleting it stays
/// safe. Asserted by deleting it between two reads that must agree.
#[test]
fn the_index_rebuilds_from_either_layout_and_deleting_it_stays_safe() {
    for legacy in [true, false] {
        let r = Repo::new();
        if legacy {
            r.seed_task_legacy(LEGACY_TASK, "A task written before the move");
        } else {
            r.seed_task(LEGACY_TASK, Some("A verifiable criterion."));
        }

        let first = r.ank("claude-code@ank", &["find", "--status", "open"]);
        assert_eq!(code(&first), 0, "{}", stderr(&first));

        let db = r.0.join(".ank/index.db");
        assert!(db.exists(), "the read builds one");
        std::fs::remove_file(&db).unwrap();

        let second = r.ank("claude-code@ank", &["find", "--status", "open"]);
        assert_eq!(code(&second), 0, "{}", stderr(&second));
        assert_eq!(
            stdout(&first),
            stdout(&second),
            "the index is a cache and never the source of truth (legacy: {legacy})"
        );
    }
}

// ---------------------------------------------------------------------------
// review, and the ratification queue it exists for (TASK-e3d00a6e62bb)
// ---------------------------------------------------------------------------

/// `review` is described by `ank help`, by `SKILL.md` and by §4 as the
/// ratification queue, and it printed no queue at all.
///
/// The consequence was not cosmetic. `accept` is the one human authority act in
/// the system, `review` is the only surface meant to say what is waiting for
/// it, and a maintainer running `review` before ratifying was told there was
/// nothing to ratify. The failure was silent in the direction that matters: an
/// empty queue and an unprinted queue were the same bytes.
///
/// Both halves of the criterion are here in one fixture, because the second is
/// reached by ratifying the first — which is also the transition a reader would
/// perform, and the one that must move an entry from the queue into the
/// constraints rather than duplicating it into both.
#[test]
fn review_prints_the_ratification_queue_it_is_described_by() {
    const BOUND: &str = "ADR-00000000b0b0";
    const WAITING: &str = "ADR-00000000c0c0";
    const ELSEWHERE: &str = "ADR-00000000d0d0";

    let r = Repo::new();
    r.enable_signing();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    std::fs::create_dir_all(r.0.join("docs")).unwrap();
    std::fs::write(r.0.join("docs/guide.md"), "# guide\n").unwrap();
    r.seed_adr(BOUND, "Do not do X.", "src/**");
    r.seed_adr(WAITING, "Do not do Y.", "src/**");
    r.seed_adr(ELSEWHERE, "Do not do Z.", "docs/**");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    let out = r.ank("marie@laptop", &["accept", BOUND]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let text = stdout(&r.ank("claude-code@ank", &["review"]));
    assert!(text.contains("PROPOSED (2)"), "{text}");
    assert!(text.contains(WAITING) && text.contains(ELSEWHERE), "{text}");
    assert!(
        text.find("PROPOSED (2)") < text.find("LIVE CONSTRAINTS"),
        "the queue is what a maintainer opens this for, and §4 opens the \
         description with it: {text}"
    );
    assert!(
        text.contains("LIVE CONSTRAINTS (1)"),
        "a proposal binds nobody and must not be counted as a constraint: {text}"
    );

    // The two surfaces answer one corpus. `status` counted the queue correctly
    // on the same tree while `review` printed none, and that disagreement is
    // exactly the shape of the defect.
    let status = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(
        status.contains("queue 2 proposal(s)"),
        "status and review must agree on the queue: {status}"
    );

    // The perimeter binds the queue as it binds the constraints, or `review`
    // would be deciding for itself what a path contains — the disagreement
    // TASK-df4c39031583 removed from three other places.
    let narrowed = stdout(&r.ank("claude-code@ank", &["review", "src"]));
    assert!(narrowed.contains("PROPOSED (1)"), "{narrowed}");
    assert!(narrowed.contains(WAITING), "{narrowed}");
    assert!(!narrowed.contains(ELSEWHERE), "{narrowed}");

    let out = r.ank("claude-code@ank", &["review", "--json"]);
    assert_json_only(&out, "ank review --json");
    let json = stdout(&out);
    assert!(
        json.contains(&format!("\"proposed\":[{{\"id\":\"{WAITING}\"")),
        "a caller parsing review gets the queue as data: {json}"
    );
    assert!(
        json.contains("\"live\":[") && json.contains("\"dead\":"),
        "alongside the two keys that were already there: {json}"
    );

    // The other half of the criterion: a corpus holding no proposal at all.
    for id in [WAITING, ELSEWHERE] {
        let out = r.ank("marie@laptop", &["accept", id]);
        assert_eq!(code(&out), 0, "{}", stderr(&out));
    }

    let text = stdout(&r.ank("claude-code@ank", &["review"]));
    assert!(
        text.contains("nothing proposed for ratification"),
        "an empty queue is said in one line rather than vanishing, on the \
         reasoning status already applies to `elsewhere`: {text}"
    );
    assert!(
        !text.contains("PROPOSED"),
        "a header over nothing is not the one line asked for: {text}"
    );
    assert!(
        text.contains("LIVE CONSTRAINTS (3)"),
        "ratifying moves an entry from the queue into the constraints: {text}"
    );

    let json = stdout(&r.ank("claude-code@ank", &["review", "--json"]));
    assert!(
        json.contains("\"proposed\":[]"),
        "the key stays, so a parser reads one shape rather than two: {json}"
    );
}

// ---------------------------------------------------------------------------
// A proof that lives in a ref (ADR-493471d64ba0)
// ---------------------------------------------------------------------------

/// A finished task in a clone, and a second clone that can attest to it.
///
/// Three properties the fixture has to have, each of which cost a red run to
/// discover. `done` happens before `cloned`, so both clones carry the task as
/// `done` — `attest` applies to a finished task, and a clone reading it as open
/// would test the refusal instead of the feature. `src/` exists, or the task's
/// own scope is dead and `check` exits 8 before anything here is reached. And
/// the completion carries a **commit** proof rather than a verifier's: `done`
/// with a declared verifier records a `test` proof itself, which would leave
/// the signal this feature silences unable to fire in the first place.
fn attestable() -> (Repo, PathBuf) {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    let sha = r.head();

    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let out = r.ank(
        "claude-code@ank",
        &["done", ID, "--proof", &format!("commit:{sha}")],
    );
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));

    let (_origin, other) = r.cloned();
    // What `ank init` installs, and the reason ADR-493471d64ba0 says an
    // existing repository needs no configuration change: one refspec already
    // carries every namespace under refs/ank/, this one included. Both sides
    // get it, because `cloned` wires the remote on the original too and the
    // reading clone is the one that needs the fetch to bring anything.
    for at in [&r.0, &other] {
        let out = git_command(at)
            .args(["config", "--add", "remote.origin.fetch", init_refspec()])
            .output()
            .unwrap();
        assert!(out.status.success(), "{}", stderr(&out));
    }
    (r, other)
}

/// Read from the binary's own constant through its help of `init`? No — it is
/// not on any surface. Written here and asserted against `init` by
/// `init_writes_the_same_refspec_this_suite_assumes`.
fn init_refspec() -> &'static str {
    "+refs/ank/*:refs/ank/*"
}

/// The refspec above is the one `ank init` actually installs.
///
/// A constant restated in a test is a constant free to drift; this is the one
/// assertion that makes the restatement safe, and it fails the day `init`
/// changes its mind.
#[test]
fn init_writes_the_same_refspec_this_suite_assumes() {
    let dir = std::env::temp_dir().join(format!("ank-cli-refspec-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let out = git_command(&dir)
        .args(["init", "-q", "-b", "main"])
        .output();
    assert!(out.unwrap().status.success());
    // A positional and not `--repo`, which `init` refuses by name: it is the
    // verb that makes a repository, so naming an existing one is a
    // contradiction (TASK-b8a1a3d0d47c).
    let out = ank_command()
        .arg("init")
        .arg(&dir)
        .env("ANK_AGENT", "claude-code@ank")
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let config = std::fs::read_to_string(dir.join(".git/config")).unwrap();
    assert!(
        config.contains(init_refspec()),
        "init no longer writes {}: {config}",
        init_refspec()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The criterion, end to end: clone B attests, clone A reads it, and neither
/// repository grows a commit (ADR-493471d64ba0).
///
/// **What is being tested is an absence.** A green pipeline is a statement
/// about a tree made by an environment with no branch, and §12 forbids ank from
/// committing — so the value of this feature is entirely in what does not
/// happen. Hence the byte comparison of the task file and the empty
/// `git status`: an implementation that wrote the file and happened to leave it
/// looking similar would pass a looser assertion.
#[test]
fn a_detached_proof_crosses_two_clones_and_neither_grows_a_commit() {
    let (r, other) = attestable();
    let file = other.join(".ank/entities").join(format!("{ID}.md"));
    let before = std::fs::read(&file).unwrap();
    let head_a = r.head();
    let head_b = git_command(&other)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    let head_b = String::from_utf8_lossy(&head_b.stdout).trim().to_string();

    let out = ank_command()
        .args(["attest", ID, "--proof", "test:ci-run-4242", "--detached"])
        .arg("--repo")
        .arg(&other)
        .env("ANK_AGENT", "process:github-actions")
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        !stderr(&out).contains("not pushed"),
        "a reachable remote must take the proof: {}",
        stderr(&out)
    );

    // Byte for byte, and no commit: the two halves of "writes no file".
    assert_eq!(
        std::fs::read(&file).unwrap(),
        before,
        "the task file moved, and a detached proof writes no file"
    );
    let status = git_command(&other)
        .args(["status", "--porcelain"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&status.stdout).trim(),
        "",
        "the working tree is not clean after a detached attestation"
    );

    // Clone A has never seen the ref. One ordinary fetch is what brings it,
    // through the refspec and not through anything ank had to be told.
    let out = git_command(&r.0)
        .args(["fetch", "--quiet", "origin"])
        .output();
    assert!(out.unwrap().status.success());

    let shown = stdout(&r.ank("claude-code@ank", &["show", ID]));
    assert!(shown.contains("test:ci-run-4242"), "{shown}");
    assert!(
        shown.contains("detached") && shown.contains("process:github-actions"),
        "the display must say which source a proof came from, and who stood \
         behind it: {shown}"
    );
    // And the file's own proof is listed beside it, because the union prefers
    // neither source.
    assert!(shown.contains("PROOFS (2)"), "{shown}");

    let json = stdout(&r.ank("claude-code@ank", &["show", ID, "--json"]));
    assert!(
        json.contains("\"detached_proofs\":[{\"type\":\"test\",\"ref\":\"ci-run-4242\""),
        "{json}"
    );

    // Neither repository grew a commit, which is the whole point.
    assert_eq!(r.head(), head_a, "clone A committed something");
    let after_b = git_command(&other)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&after_b.stdout).trim(),
        head_b,
        "clone B committed something"
    );
}

/// A signal that counts proofs counts both, or it fires on work that is
/// anchored (ADR-493471d64ba0).
///
/// The window this covers is most of a branch's life: between a `done` landing
/// and its merge, the ref is the only place a CI reference exists. A reader
/// preferring the file would tell somebody to attest work already attested.
#[test]
fn a_detached_test_proof_silences_the_signal_that_counts_proofs() {
    let (r, other) = attestable();
    // The task is `done` on the default branch here, which is the gate the
    // signal is behind — so it fires before the attestation and must stop
    // after it. Without that, this test would pass on a signal that never ran.
    let before = stdout(&r.ank("claude-code@ank", &["check"]));
    assert!(
        before.contains("done with no test proof"),
        "the signal must fire first, or its silence proves nothing: {before}"
    );

    let out = ank_command()
        .args(["attest", ID, "--proof", "test:ci-run-4242", "--detached"])
        .arg("--repo")
        .arg(&other)
        .env("ANK_AGENT", "process:github-actions")
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let out = git_command(&r.0)
        .args(["fetch", "--quiet", "origin"])
        .output();
    assert!(out.unwrap().status.success());

    let after = stdout(&r.ank("claude-code@ank", &["check"]));
    assert!(
        !after.contains("done with no test proof"),
        "the task is anchored by a ref and the signal still fires: {after}"
    );
}

/// A test reference a caller typed is not the proof a pipeline attested
/// (ADR-b6b69053a47b), and the binary is where that has to be true.
///
/// **The two halves are one test because the route is the only difference
/// between them.** The same verb, the same proof type, the same task: what
/// changes is `--detached`, and therefore where the entry goes and what it
/// records about how it got there. A test asserting only the silence would
/// pass on a rule that had stopped firing altogether, and a test asserting
/// only the firing would pass on one that never stops.
///
/// The wording is asserted too, and not as decoration. A task carrying
/// `test:<something>` that is told it has no test proof is told something
/// false, and the hint has to name a command that actually clears the finding
/// — a plain `ank attest` records `via: submitted` and would leave the reader
/// running it twice.
#[test]
fn a_submitted_test_reference_leaves_the_signal_firing_and_an_attested_one_clears_it() {
    let (r, other) = attestable();
    let before = stdout(&r.ank("claude-code@ank", &["check"]));
    assert!(
        before.contains("done with no test proof"),
        "the signal must fire first, or the rest proves nothing: {before}"
    );

    // A caller types a run reference into the file. Accepted, recorded, and
    // still not an anchor: this changes what is reported, never what is
    // refused.
    let out = r.ank(
        "claude-code@ank",
        &["attest", ID, "--proof", "test:31666088871"],
    );
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "a typed reference"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(code(&out), 0, "still a signal, never a fault: {said}");
    let reported: Vec<&str> = said
        .lines()
        .filter(|l| l.contains("test proof") && l.contains(ID))
        .collect();
    assert_eq!(
        reported.len(),
        1,
        "a typed reference silenced the signal it exists to leave standing:\n{said}"
    );
    assert!(
        reported[0].contains("no attested test proof") && reported[0].contains("test:31666088871"),
        "the finding must say what it declined to count, and why: {}",
        reported[0]
    );
    assert!(
        reported[0].contains(&format!("ank attest {ID} --proof test:<run-id> --detached")),
        "the hint must name the command that clears it: {}",
        reported[0]
    );

    // And the file says so, which is where the distinction lives: it is a
    // field and not a guess made at read time from a ref that gets pruned.
    let shown = stdout(&r.ank("claude-code@ank", &["show", ID]));
    assert!(shown.contains("via: submitted"), "{shown}");

    // The same reference, the same verb, arriving by the other route.
    let out = ank_command()
        .args(["attest", ID, "--proof", "test:31666088871", "--detached"])
        .arg("--repo")
        .arg(&other)
        .env("ANK_AGENT", "process:github-actions")
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let out = git_command(&r.0)
        .args(["fetch", "--quiet", "origin"])
        .output();
    assert!(out.unwrap().status.success());

    let after = stdout(&r.ank("claude-code@ank", &["check"]));
    assert!(
        !after.contains("test proof"),
        "a pipeline attested it and the signal still fires: {after}"
    );

    // Twice, because `check` is the command that prunes and the answer must
    // not depend on how often it has run. A ref retired in favour of a file
    // entry that anchors nothing would bring the finding back on a task
    // nobody touched.
    let again = stdout(&r.ank("claude-code@ank", &["check"]));
    assert!(
        !again.contains("test proof"),
        "the second run of check disagreed with the first: {again}"
    );
}

/// The attestation copied out of the ref into the file keeps its route, and
/// that is what makes the ref safe to retire.
///
/// A proof ref lives exactly as long as what it carries is not yet where
/// everyone reads it, and the file catching up is the one thing that retires
/// it. Ank can see that the entry being appended is the one the ref already
/// holds — the same check it already performs on a `commit:` against git — so
/// it records the route it verified rather than the route it was handed.
/// Without that, the prune would delete the only place the attestation was
/// written down and the signal would come back on a finished task.
#[test]
fn copying_a_detached_attestation_into_the_file_keeps_it_an_attestation() {
    let (r, other) = attestable();
    let out = ank_command()
        .args(["attest", ID, "--proof", "test:ci-run-4242", "--detached"])
        .arg("--repo")
        .arg(&other)
        .env("ANK_AGENT", "process:github-actions")
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let out = git_command(&r.0)
        .args(["fetch", "--quiet", "origin"])
        .output();
    assert!(out.unwrap().status.success());

    // The same reference, now written into the file by a caller who did not
    // run it. Ank checks the ref before believing the flag.
    let out = r.ank(
        "claude-code@ank",
        &["attest", ID, "--proof", "test:ci-run-4242"],
    );
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let shown = stdout(&r.ank("claude-code@ank", &["show", ID]));
    assert!(
        shown.contains("via: attested"),
        "the ref carries this exact entry and the file called it submitted: {shown}"
    );

    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "the proof lands in the file"]);
    // The ref retires here, and the finding must not come back with it.
    assert_eq!(code(&r.ank("claude-code@ank", &["check"])), 0);
    let after = stdout(&r.ank("claude-code@ank", &["check"]));
    assert!(
        !after.contains("test proof"),
        "the prune took the attestation with it: {after}"
    );
}

/// A corpus written before the distinction existed is left exactly as it was.
///
/// **This is the failure mode the task was written about.** This repository's
/// own corpus carries twenty-one completions anchored by `test:` references,
/// ten typed by hand and eleven written by the pipeline, and nothing in those
/// files tells the two apart. A rule reading the absent field as `submitted`
/// would redden every one of them on the day it landed, which is a rule
/// everybody turns off. So absent is not `submitted`, and it counts as it
/// always counted.
///
/// Asserted on both halves of "unchanged": the signal that does not fire, and
/// the bytes that do not move. A reader that filled the absence in with a
/// default would be migrating a corpus by reading it.
#[test]
fn a_corpus_written_before_the_route_existed_is_neither_reinterpreted_nor_rewritten() {
    let r = Repo::new();
    // Typed by hand at a keyboard, before `via` existed. Under the new rule it
    // would not anchor; under the reading its own schema earns, it does.
    seed_done(&r, ATTESTED, "  - type: test\n    ref: \"991\"\n");
    seed_done(&r, UNATTESTED, "  - type: commit\n    ref: abc1234\n");
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/lib.rs"), "// x\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let older = r.0.join(".ank/entities").join(format!("{ATTESTED}.md"));
    let bytes = std::fs::read(&older).unwrap();

    let out = r.ank("claude-code@ank", &["check"]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(code(&out), 0, "{said}");

    let reported: Vec<&str> = said.lines().filter(|l| l.contains("test proof")).collect();
    assert_eq!(
        reported.len(),
        1,
        "the signal started firing on a corpus that did nothing:\n{said}"
    );
    assert!(
        reported[0].contains(UNATTESTED),
        "the task named is the one with no test proof at all: {}",
        reported[0]
    );
    assert!(
        !said.contains(ATTESTED),
        "an entry predating the field was reinterpreted:\n{said}"
    );

    // Read whole, and byte for byte where it was. The field is optional and
    // its absence means the entry predates it — never a default written in on
    // the first rewrite.
    let shown = stdout(&r.ank("claude-code@ank", &["show", ATTESTED]));
    assert!(shown.contains("type: test"), "{shown}");
    assert!(!shown.contains("via:"), "a route was invented: {shown}");
    assert_eq!(
        std::fs::read(&older).unwrap(),
        bytes,
        "reading the corpus rewrote it"
    );
}

/// Pruned when the file catches up, and at no other time.
///
/// **No TTL, and the negative half is the one worth testing.** A proof ref
/// pruned on time would delete the record precisely during the window it exists
/// to cover, so `check` is run twice against a corpus the default branch has
/// not caught up with, and the ref has to survive both.
#[test]
fn a_detached_proof_outlives_check_until_the_file_carries_it() {
    let (r, other) = attestable();
    let out = ank_command()
        .args(["attest", ID, "--proof", "test:ci-run-4242", "--detached"])
        .arg("--repo")
        .arg(&other)
        .env("ANK_AGENT", "process:github-actions")
        .current_dir(std::env::temp_dir())
        .output()
        .unwrap();
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    let out = git_command(&r.0)
        .args(["fetch", "--quiet", "origin"])
        .output();
    assert!(out.unwrap().status.success());

    let refname = format!("refs/ank/proof/{ID}");
    let present = |where_: &Path| {
        git_command(where_)
            .args(["rev-parse", "--verify", "--quiet", &refname])
            .output()
            .unwrap()
            .status
            .success()
    };
    assert!(present(&r.0), "the fetch brought nothing");

    for _ in 0..2 {
        assert_eq!(code(&r.ank("claude-code@ank", &["check"])), 0);
        assert!(
            present(&r.0),
            "check pruned a proof the default branch does not carry"
        );
    }

    // Now the file catches up on the default branch, which is the one condition
    // that retires the ref.
    let out = r.ank(
        "claude-code@ank",
        &["attest", ID, "--proof", "test:ci-run-4242"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "the proof lands in the file"]);

    assert_eq!(code(&r.ank("claude-code@ank", &["check"])), 0);
    assert!(
        !present(&r.0),
        "the file on the default branch carries the same proof, and the ref \
         is still there"
    );
}

/// A detached proof that never reached the remote **fails**, and a claim
/// against the same remote does not (ADR-af533e7a3e03).
///
/// **The two halves are one test because the decision is the difference between
/// them.** A change that failed both would satisfy the first assertion alone
/// and would be exactly the generalisation the ADR refuses: `claim` leaves a
/// record that still governs this clone, so it degrades and displays the risk,
/// while `--detached` produces a ref and nothing else, and a ref no other clone
/// can read is not an attestation.
///
/// Against a `file://` remote that is configured and gone, which is the shape a
/// pipeline off the network actually has: a URL that resolves to nothing, so
/// git fails to connect rather than refusing a swap.
#[test]
fn a_detached_proof_that_missed_the_remote_fails_where_a_claim_degrades() {
    const FINISHED: &str = "TASK-000000000e01";
    const FREE: &str = "TASK-000000000e02";
    let refname = format!("refs/ank/proof/{FINISHED}");

    let r = Repo::new();
    r.seed_task(FINISHED, Some("A verifiable criterion."));
    r.seed_task(FREE, Some("A verifiable criterion."));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    let sha = r.head();
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", FINISHED])), 0);
    let out = r.ank(
        "claude-code@ank",
        &["done", FINISHED, "--proof", &format!("commit:{sha}")],
    );
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));

    // The remote is added after `done`, so nothing before this point had one to
    // reach and the failure below is about the attestation alone.
    r.git(&[
        "remote",
        "add",
        "origin",
        &file_url(&r.0.with_extension("gone.git")),
    ]);

    let out = r.ank(
        "process:github-actions",
        &[
            "attest",
            FINISHED,
            "--proof",
            "test:ci-run-4242",
            "--detached",
        ],
    );
    assert_eq!(
        code(&out),
        9,
        "a proof no other clone can read reported success:\n{}{}",
        stdout(&out),
        stderr(&out)
    );
    let said = stderr(&out);
    assert!(
        said.contains("proof not pushed") && said.contains("no other clone can read it"),
        "the failure dropped the sentence naming what went wrong: {said}"
    );
    // The hint, and it is the exact command rather than advice: the record is
    // in this clone already, so what is missing is one push and not a re-run.
    assert!(
        said.contains(&format!("git push origin {refname}")),
        "the failure names no command to run next: {said}"
    );

    // And that hint is only right if the record really is local. It is: the
    // local swap is what the push failed to carry, not what it undid.
    assert!(
        git_command(&r.0)
            .args(["rev-parse", "--verify", "--quiet", &refname])
            .output()
            .unwrap()
            .status
            .success(),
        "the local ref went missing, and the hint would send the caller to \
         push nothing"
    );

    // `--json` says the same thing the exit code does. An integration reading
    // the flag and one reading the code must not be able to disagree.
    let out = r.ank(
        "process:github-actions",
        &[
            "attest",
            FINISHED,
            "--proof",
            "test:ci-run-4242",
            "--detached",
            "--json",
        ],
    );
    assert_eq!(code(&out), 9, "{}{}", stdout(&out), stderr(&out));
    assert!(
        stdout(&out).contains("\"pushed\":false"),
        "the flag and the exit code disagree: {}",
        stdout(&out)
    );

    // The other half, and the one that keeps the change from spreading: the
    // same unreachable remote, a verb whose write also landed on disk.
    let out = r.ank("claude-code@ank", &["claim", FREE]);
    assert_eq!(
        code(&out),
        0,
        "an unreachable remote must not fail a claim:\n{}{}",
        stdout(&out),
        stderr(&out)
    );
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert!(
        said.contains("claim not pushed") && said.contains("another clone"),
        "the claim degraded in silence: {said}"
    );
    assert!(
        r.claim_ref(FREE).is_some(),
        "the claim did not hold locally, which is the half that must not degrade"
    );
}

/// The help says which side of that rule the verb is on, so a caller never has
/// to infer it from what the verb happens to touch (ADR-af533e7a3e03).
#[test]
fn the_help_of_attest_says_a_detached_proof_fails_on_an_unreachable_remote() {
    let r = Repo::new();
    let page = stdout(&r.ank("claude-code@ank", &["help", "attest"]));
    assert!(
        page.contains("--detached") && page.contains("unreachable") && page.contains("(9)"),
        "the page does not say the verb fails on an unreachable remote: {page}"
    );
}

// ---------------------------------------------------------------------------
// The log is a file, and an append is not a transition (§3, ADR-ff294eff4d1a)
// ---------------------------------------------------------------------------

const LOGGED: &str = "TASK-00000000c10a";

/// The property the move was made for: **`ank log` does not touch the entity**.
///
/// Not "writes little" -- writes nothing. The entity file carries the frozen
/// criterion, and it was the file that churned most because the loop tells an
/// agent to log whenever it learns something. Byte-for-byte equality is the
/// only assertion that says so; a version check alone would pass on a rewrite
/// that happened to land the same number.
#[test]
fn an_append_leaves_the_entity_file_byte_for_byte() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", LOGGED])), 0);

    let before = r.task_text(LOGGED);
    let out = r.ank("claude-code@ank", &["log", "learned something"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    assert_eq!(
        r.task_text(LOGGED),
        before,
        "no frontmatter, no version bump, nothing touched that carries a freeze"
    );
    let log = r.log_text(LOGGED);
    assert!(log.contains("learned something"), "{log}");
    assert!(
        log.lines().count() == 1 && log.starts_with("- "),
        "one entry, one line, and the file is nothing but entries: {log:?}"
    );

    // A second append is a one-line diff over the first.
    assert_eq!(code(&r.ank("claude-code@ank", &["log", "and again"])), 0);
    let after = r.log_text(LOGGED);
    assert!(
        after.starts_with(&log),
        "an append rewrites nothing: {after}"
    );
    assert_eq!(after.lines().count(), 2, "{after}");
}

/// Every verb that logs writes to the same place, and none of them puts a log
/// section back into a body that has none.
#[test]
fn every_verb_that_logs_writes_to_the_log_file() {
    let r = Repo::new().with_verifiers("");
    r.seed_task(LOGGED, Some("A verifiable criterion."));

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", LOGGED])), 0);
    assert_eq!(
        code(&r.ank(
            "claude-code@ank",
            &["release", LOGGED, "--reason", "needs staging access"]
        )),
        0
    );
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", LOGGED])), 0);
    assert_eq!(
        code(&r.ank(
            "claude-code@ank",
            &["done", "--proof", "assertion:it works"]
        )),
        0
    );
    assert_eq!(
        code(&r.ank(
            "claude-code@ank",
            &["attest", LOGGED, "--proof", "test:ci-run-9"]
        )),
        0
    );

    let log = r.log_text(LOGGED);
    for expected in [
        "released: needs staging access",
        "done, proof",
        "attested test:",
    ] {
        assert!(log.contains(expected), "{expected} missing from:\n{log}");
    }
    assert!(
        !r.task_text(LOGGED).contains("## Log"),
        "no verb puts a log section back into a body: {}",
        r.task_text(LOGGED)
    );
}

/// Read from wherever it is, by every surface that shows it.
#[test]
fn show_log_and_context_read_the_log_from_the_file() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", LOGGED])), 0);
    assert_eq!(
        code(&r.ank("claude-code@ank", &["log", "learned something"])),
        0
    );

    // `show`: under the entity, which stays byte for byte the file above it.
    let out = r.ank("claude-code@ank", &["show", LOGGED]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.starts_with(&r.task_text(LOGGED)),
        "the entity is still verbatim: {text}"
    );
    assert!(text.contains("LOG (1)"), "{text}");
    assert!(text.contains("learned something"), "{text}");

    // `ank log <id>` with no message.
    let out = r.ank("claude-code@ank", &["log", LOGGED]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("learned something"),
        "{}",
        stdout(&out)
    );

    // `context` in execution mode.
    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("learned something"),
        "{}",
        stdout(&out)
    );

    // And as data.
    let out = r.ank("claude-code@ank", &["show", LOGGED, "--json"]);
    assert_json_only(&out, "ank show --json");
    assert!(
        stdout(&out).contains("\"log\":[{\"timestamp\":"),
        "{}",
        stdout(&out)
    );
}

/// A missing log file is an empty log and never an error -- the state every
/// entity is in until somebody logs against it.
#[test]
fn an_entity_with_no_log_file_reads_as_an_empty_log() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    assert!(r.log_text(LOGGED).is_empty(), "the fixture has no log file");

    let out = r.ank("claude-code@ank", &["log", LOGGED]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("no log entry yet"),
        "{}",
        stdout(&out)
    );

    let out = r.ank("claude-code@ank", &["show", LOGGED]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        !stdout(&out).contains("LOG ("),
        "no empty heading: {}",
        stdout(&out)
    );
}

/// **One history never splits.** An entity whose body still carries a `## Log`
/// section keeps it: the entry lands there, no file appears beside it, and the
/// entries already written stay reachable.
///
/// Writing the entry into a new file instead would leave the older half
/// unreachable, since reading prefers the file -- which is the exact failure
/// the schema bump exists to prevent, arriving through the tool rather than
/// through an old reader.
#[test]
fn an_entity_whose_log_is_in_its_body_keeps_it_there() {
    let r = Repo::new();
    r.seed_task_with_body_log(LOGGED, "an entry written before the move");
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", LOGGED])), 0);
    assert_eq!(
        code(&r.ank("claude-code@ank", &["log", "learned something"])),
        0
    );

    assert!(
        r.log_text(LOGGED).is_empty(),
        "no file appears beside a body that already holds the log: {}",
        r.log_text(LOGGED)
    );
    let text = r.task_text(LOGGED);
    assert!(
        text.contains("an entry written before the move") && text.contains("learned something"),
        "both halves of one history, in one place: {text}"
    );

    // And every reader still sees both, exactly once.
    let out = r.ank("claude-code@ank", &["log", LOGGED]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let listed = stdout(&out);
    assert_eq!(listed.matches("learned something").count(), 1, "{listed}");
    assert_eq!(
        listed.matches("an entry written before the move").count(),
        1,
        "{listed}"
    );

    // `show` prints the body's own section and adds no second copy under it.
    let out = stdout(&r.ank("claude-code@ank", &["show", LOGGED]));
    assert_eq!(out.matches("learned something").count(), 1, "{out}");
    assert!(!out.contains("LOG ("), "the body already carries it: {out}");
}

// ---------------------------------------------------------------------------
// The orientation budget (§5, TASK-1ead0e19fb73)
// ---------------------------------------------------------------------------

/// The default of §5, restated here because the fixture writes no
/// `context_budget` and the assertions are arithmetic on it.
const BUDGET: usize = 8000;

/// A corpus whose constraints alone would swallow the page.
///
/// Forty accepted ADRs with multi-sentence rules and forty open tasks. Rendered
/// the old way — every rule in full, tasks cut first — this is far past 8000
/// characters, which is what makes the allocation observable rather than
/// hypothetical.
fn crowded() -> Repo {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    for i in 0..40 {
        let id = format!("ADR-0000000{i:05}");
        let rule = format!(
            "Rule number {i} is stated at length, because a real constraint is \
             a paragraph and not a slogan. It goes on for a second sentence so \
             that rendering it in full costs what a real one costs."
        );
        std::fs::write(
            r.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: adr\nslug: example\n\
                 title: Decision number {i}, phrased as a title of ordinary length\n\
                 created: 2026-07-20T00:00:00Z\nstatus: accepted\nscope:\n  - src/**\n\
                 constraint: |\n  {rule}\nschema: 1\nversion: 1\n---\n\nWhy.\n"
            ),
        )
        .unwrap();

        let tid = format!("TASK-0000000{i:05}");
        std::fs::write(
            r.0.join(".ank/entities").join(format!("{tid}.md")),
            format!(
                "---\nid: {tid}\ntype: task\nslug: example\n\
                 title: Task number {i}, with a title of the length titles really have\n\
                 created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
                 blocked_by: []\ndone_criteria: |\n  A verifiable criterion.\n\
                 criteria_by: creator\nschema: 1\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
    }
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "a crowded corpus"]);
    r
}

/// The characters a named section spends, counted as the renderer counts them.
fn section_chars(text: &str, header: &str) -> usize {
    text.lines()
        .skip_while(|l| !l.starts_with(header))
        .take_while(|l| {
            l.starts_with(header)
                || !(l.starts_with("CONSTRAINTS")
                    || l.starts_with("PROPOSED")
                    || l.starts_with("TASKS"))
        })
        .map(|l| l.chars().count() + 1)
        .sum()
}

/// Orientation allocates the budget the way §5 now promises, through the
/// binary and on a corpus large enough to exceed it.
///
/// **The measurement this replaces is in the task's log.** On this repository,
/// at the same 8000 characters, orientation spent 7357 on seven constraints
/// rendered in full and 157 on tasks — one task line printed and eleven cut,
/// with the closing suggestion naming the only candidate it had room for. The
/// mode whose purpose is choosing offered one option out of twelve.
///
/// Asserted through the binary rather than on `render`, because the allocation
/// is only worth anything if it survives the whole path an agent actually
/// takes: config, perimeter, index, and the writer that prints it.
#[test]
fn orientation_spends_at_most_a_third_on_constraints_and_the_rest_on_tasks() {
    let r = crowded();
    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);

    // The control, and it is the assertion that catches a missing ceiling.
    // Forty constraints named on one line each are past a third of 8000, so
    // some of them must have been counted away; if they all fit, the rule was
    // never under load and every assertion below would pass on a page that
    // never had to choose anything.
    assert!(
        text.contains("broad constraints, ank find --type adr"),
        "the constraints all fit, so the third was never enforced: {text}"
    );

    let constraints = section_chars(&text, "CONSTRAINTS");
    let tasks = section_chars(&text, "TASKS");
    assert!(
        constraints <= BUDGET / 3,
        "constraints took {constraints} characters of {BUDGET}, over the \
         third §5 allows them"
    );
    assert!(
        tasks > constraints,
        "the tasks got {tasks} characters against {constraints} for the \
         constraints, and orientation is for choosing"
    );

    // Named, never quoted: the rule's text is one `ank show` away.
    assert!(
        text.contains("Decision number 0, phrased as a title"),
        "a constraint must be named: {text}"
    );
    assert!(
        !text.contains("because a real constraint is"),
        "orientation quoted a rule instead of naming it: {text}"
    );

    // And the page offers a choice rather than a single candidate.
    let listed = text
        .lines()
        .filter(|l| l.trim_start().starts_with("TASK-"))
        .count();
    assert!(
        listed > 10,
        "orientation offered {listed} candidates out of 40: {text}"
    );
    assert!(
        text.chars().count() <= BUDGET,
        "the page is {} characters, over budget",
        text.chars().count()
    );
}

/// Execution mode keeps the opposite rule, on the same corpus.
///
/// The two halves of §5 are one decision, and a change to the orientation half
/// that quietly truncated a binding constraint would be the failure the whole
/// design exists to prevent. Same fixture, one claim, opposite guarantee.
#[test]
fn execution_still_renders_a_binding_constraint_in_full() {
    let r = crowded();
    let out = r.ank("claude-code@ank", &["claim", "TASK-000000000000"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let text = stdout(&r.ank("claude-code@ank", &["context"]));
    assert!(
        text.contains("because a real constraint is a paragraph and not a slogan"),
        "a binding constraint was truncated in execution mode: {text}"
    );
    assert!(
        !text.contains("broad constraints, ank find"),
        "execution mode counted constraints away instead of printing them: {text}"
    );
}

// ---------------------------------------------------------------------------
// Corpus drift from the default branch (ADR-47e2ac102f58)
// ---------------------------------------------------------------------------

/// A repository whose corpus is committed on `main`, with `feature` branched off
/// the same commit: the two carry the same corpus until a test moves one of
/// them.
///
/// Committed on purpose. The question is what this checkout carries against what
/// the default branch carries, and a fixture that never committed would compare
/// a corpus against a branch that has none — which is the third state, not the
/// one under test.
fn drifting_repo() -> Repo {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_task("TASK-aaaaaaaaaaaa", Some("A verifiable criterion."));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "corpus"]);
    r.git(&["branch", "feature"]);
    r
}

/// The finding that caused a failure rather than friction: a checkout whose
/// corpus is not the one the default branch carries, and nothing saying so.
///
/// Two ways of differing in one corpus, because they are one question: an entity
/// the default branch has and this checkout does not, and an entity both carry
/// with different content. One line either way, and the count is what makes it
/// actionable.
#[test]
fn check_names_a_corpus_behind_the_default_branch_and_status_says_how_far() {
    let r = drifting_repo();
    r.seed_task("TASK-bbbbbbbbbbbb", Some("A criterion only main carries."));
    r.seed_task_titled("TASK-aaaaaaaaaaaa", "Retitled on the default branch");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "main moves"]);
    r.git(&["checkout", "-q", "feature"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let text = stdout(&out);
    assert_eq!(code(&out), 0, "drift is a signal, never a fault: {text}");
    assert!(
        text.contains("2 entity file(s) differ from main"),
        "check said nothing about a corpus two entities behind main: {text}"
    );
    assert!(
        text.contains("git merge main"),
        "the signal must name the command that closes the gap: {text}"
    );
    // Once for the corpus. Two entities differ, and two lines saying one thing
    // is the volume that teaches a reader to stop reading `check`.
    assert_eq!(
        text.lines()
            .filter(|l| l.contains("differ from main"))
            .count(),
        1,
        "the drift is reported once for the corpus, never per entity: {text}"
    );

    let text = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(
        text.contains("2 entity file(s) differ from main"),
        "status carries the same fact on one line: {text}"
    );
}

/// Ahead is drift too, and level is silence in `check` and a line in `status`.
///
/// The corpus is a comparison and not an ordering: a checkout carrying an entity
/// the default branch has never seen is as far from it as one missing an entity,
/// and a reader told about only one of the two learns to trust neither.
#[test]
fn a_corpus_ahead_of_the_default_branch_is_named_and_a_level_one_says_so() {
    let r = drifting_repo();
    r.git(&["checkout", "-q", "feature"]);
    r.seed_task(
        "TASK-cccccccccccc",
        Some("A criterion only this branch has."),
    );
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "feature moves"]);

    let text = stdout(&r.ank("claude-code@ank", &["check"]));
    assert!(
        text.contains("1 entity file(s) differ from main"),
        "a corpus ahead of the default branch is drift as well: {text}"
    );

    // Level, on the default branch itself and with nothing uncommitted.
    r.git(&["checkout", "-q", "main"]);
    let out = r.ank("claude-code@ank", &["check"]);
    let text = stdout(&out);
    assert_eq!(code(&out), 0, "{text}");
    assert!(
        !text.contains("differ from main"),
        "a level corpus must produce no drift signal at all: {text}"
    );

    // `status` says it either way. An absent line is what "not asked" looks
    // like, so a level corpus cannot be reported by silence.
    let text = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(
        text.contains("level with main"),
        "status must say the corpus is level rather than say nothing: {text}"
    );
}

/// No default branch to compare against: the question is skipped in silence, on
/// the rule the rename walk already follows.
///
/// Silence and not a warning. A corpus with no resolvable default branch already
/// gets one line saying what that costs the coordination plane; a second about
/// the corpus would report a consequence as though it were a separate finding.
#[test]
fn corpus_drift_is_skipped_in_silence_with_no_resolvable_default_branch() {
    let r = drifting_repo();
    r.set_config("schema: 1\nclaim_ttl_max: 2h\n");
    r.git(&["checkout", "-q", "feature"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let text = stdout(&out);
    assert_eq!(code(&out), 0, "{text}");
    assert!(
        !text.contains("entity file(s) differ"),
        "nothing can be compared without a default branch: {text}"
    );
    let text = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(
        !text.contains("entity file(s) differ") && !text.contains("level with"),
        "status has no drift to report either: {text}"
    );
}

/// A `default_branch` that names no commit in this clone is said out loud, and
/// never rendered as a corpus that has not moved.
///
/// That distinction is the whole reason the comparison goes through `file_at`,
/// which tells an absent path from an unresolvable revision. A mistyped branch
/// answering "level" is a reader told the corpus agrees with something that was
/// never read.
#[test]
fn a_default_branch_naming_no_commit_is_named_rather_than_read_as_level() {
    let r = drifting_repo();
    r.set_config("schema: 1\nclaim_ttl_max: 2h\ndefault_branch: mian\n");

    let out = r.ank("claude-code@ank", &["check"]);
    let text = stdout(&out);
    assert_eq!(code(&out), 0, "{text}");
    assert!(
        text.contains("mian") && text.contains("not compared"),
        "a branch naming no commit must be reported, not passed over: {text}"
    );
    assert!(
        !text.contains("level with"),
        "unable to compare is never 'nothing has moved': {text}"
    );
}

/// Neither verb fetches to answer, and the assertion is on the absence of the
/// network call rather than on the output.
///
/// A verb that fetched would rewrite the coordination plane underneath every
/// other agent in the clone, which is the argument that made `status --remote`
/// read with `ls-remote`. Origin is moved ahead first, so a fetch introduced
/// here would visibly succeed: it would write `FETCH_HEAD` and advance
/// `refs/remotes/origin/main`, and this test fails on either.
#[test]
fn neither_check_nor_status_fetches_to_compare_the_corpus() {
    // `cloned` is what commits and pushes this corpus, so the fixture seeds and
    // leaves the committing to it.
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_task("TASK-aaaaaaaaaaaa", Some("A verifiable criterion."));
    let (_origin, other) = r.cloned();

    // Origin moves, and the clone is told nothing about it.
    r.seed_task(
        "TASK-dddddddddddd",
        Some("A criterion pushed after the clone."),
    );
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "origin moves"]);
    r.git(&["push", "-q", "origin", "main"]);

    let tracking = |at: &Path| -> String {
        let out = git_command(at)
            .args(["rev-parse", "refs/remotes/origin/main"])
            .output()
            .unwrap();
        assert!(out.status.success(), "rev-parse: {}", stderr(&out));
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    let before = tracking(&other);
    assert!(
        !other.join(".git/FETCH_HEAD").exists(),
        "the fixture must start with no FETCH_HEAD, or the assertion is empty"
    );

    for args in [["check"], ["status"]] {
        let out = r.ank_at("claude-code@ank", &args, &other);
        assert_eq!(code(&out), 0, "{args:?}: {}", stderr(&out));
        assert!(
            !other.join(".git/FETCH_HEAD").exists(),
            "{args:?} fetched: FETCH_HEAD appeared"
        );
        assert_eq!(
            tracking(&other),
            before,
            "{args:?} fetched: refs/remotes/origin/main advanced"
        );
    }
}
