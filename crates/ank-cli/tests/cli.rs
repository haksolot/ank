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
        // What `ank init` writes, and the fixture was unrepresentative without
        // it: the index is derived, disposable and **gitignored** (§6). Any verb
        // that opens one leaves `.ank/index.db` behind, and a fixture that
        // tracks it turns an ordinary `git merge` into a refusal about an
        // untracked file the tool owns. `init` has its own tests for the
        // appending behaviour; this is only the line those tests are about.
        std::fs::write(r.0.join(".gitignore"), ".ank/index.db\n").unwrap();
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

    /// The entries about an entity, rendered as the lines a reader sees, oldest
    /// first — empty when there are none.
    ///
    /// Since ADR-25f977377fa0 an entry is an entity of its own, so this walks
    /// the corpus for the entries naming this id rather than opening one file.
    /// It renders rather than returning the entities, because the assertions
    /// below are about **what somebody wrote**, and the line is where that has
    /// always been legible. The message is whole here: what a lister elides for
    /// width, an assertion must still see.
    ///
    /// The previous log directory answers for an entity that has no entries,
    /// which is the same rule the CLI applies (§3) and what keeps the fixtures
    /// seeding one meaningful.
    fn log_text(&self, id: &str) -> String {
        let mut rows: Vec<((String, u64, String), String)> = Vec::new();
        for entry in std::fs::read_dir(self.0.join(".ank/entities"))
            .into_iter()
            .flatten()
            .flatten()
        {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("md") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(ank_core::Entity::Log(l)) = ank_core::parse_entity(&text) else {
                continue;
            };
            if l.about.to_string() != id {
                continue;
            }
            rows.push((
                // The order of §3, and the same key the tool uses: `created`,
                // then `seq`, then the identifier. **A helper that sorted on
                // the timestamp alone reproduced, inside the thing doing the
                // catching, the very defect these tests exist to catch** — two
                // entries of one second came out in identifier order, and it
                // failed 1 run in 10 until this line named `seq`.
                (l.created.clone(), l.seq, l.id.to_string()),
                format!(
                    "- {} {} \u{2014} {}\n",
                    l.created,
                    l.author.clone().unwrap_or_default(),
                    l.message()
                ),
            ));
        }
        if rows.is_empty() {
            return std::fs::read_to_string(self.0.join(".ank/log").join(format!("{id}.md")))
                .unwrap_or_default();
        }
        rows.sort();
        rows.into_iter().map(|(_, line)| line).collect()
    }

    /// The entries about an entity, as entities: the identifiers of the files
    /// the corpus actually holds. What `log_text` renders, this counts.
    fn entry_ids(&self, id: &str) -> Vec<String> {
        let mut ids: Vec<String> = Vec::new();
        for entry in std::fs::read_dir(self.0.join(".ank/entities"))
            .into_iter()
            .flatten()
            .flatten()
        {
            let path = entry.path();
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Ok(ank_core::Entity::Log(l)) = ank_core::parse_entity(&text) {
                if l.about.to_string() == id {
                    ids.push(l.id.to_string());
                }
            }
        }
        ids.sort();
        ids
    }

    /// The machinery entries about an entity, oldest first, as the messages
    /// they carry (ADR-16813b3bcf37).
    ///
    /// Read off the files rather than through the binary, on the same reasoning
    /// `log_text` is: what has to be true is the state of the corpus, and a
    /// helper that asked the tool would be asking the writer whether it wrote.
    fn machinery_of(&self, id: &str) -> Vec<String> {
        let mut rows: Vec<(u64, String)> = Vec::new();
        for entry in std::fs::read_dir(self.0.join(".ank/entities"))
            .into_iter()
            .flatten()
            .flatten()
        {
            let Ok(text) = std::fs::read_to_string(entry.path()) else {
                continue;
            };
            if let Ok(ank_core::Entity::Log(l)) = ank_core::parse_entity(&text) {
                if l.about.to_string() == id && l.records.is_some() {
                    rows.push((l.seq, l.message()));
                }
            }
        }
        rows.sort();
        rows.into_iter().map(|(_, m)| m).collect()
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

    /// A spec, in canonical form, with the two fields a citation test varies.
    ///
    /// Written by hand rather than through `ank new spec`, and necessarily so:
    /// the states under test are a reference to an entity this corpus does not
    /// hold and a citation left behind by a supersession, and `new` resolves
    /// every reference it is given — so no writer will produce either of them.
    /// That is the division of labour the field rests on: the write refuses what
    /// it can attribute, and `check` reports the corpus moving underneath a
    /// citation that was good when it was written.
    fn seed_spec(&self, id: &str, status: &str, references: &[&str], supersedes: Option<&str>) {
        let references = match references {
            [] => String::new(),
            r => format!("references: [{}]\n", r.join(", ")),
        };
        let supersedes = supersedes
            .map(|s| format!("supersedes: {s}\n"))
            .unwrap_or_default();
        std::fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: spec\nslug: a-document\ntitle: A document\n\
                 created: 2026-08-01T00:00:00Z\nauthor: human:marie\nstatus: {status}\n\
                 scope:\n  - docs/**\n{references}{supersedes}schema: 3\nversion: 1\n---\n\
                 \nThe document itself.\n"
            ),
        )
        .unwrap();
    }

    /// The golden corpus's own seeders (TASK-e89613d66284).
    ///
    /// Schema 3 and an author on every one, where the shared helpers above write
    /// schema 1 and no author: this corpus is a published example, and a fixture
    /// demonstrating a layout the store no longer writes teaches the wrong thing
    /// to whoever reads it.
    fn seed_golden_adr(&self, id: &str) {
        std::fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: adr\nslug: example\ntitle: A decision\n\
                 created: 2026-07-20T00:00:00Z\nauthor: human:marie\nstatus: proposed\n\
                 scope:\n  - src/**\nconstraint: |\n\
                 {constraint}\nschema: 3\nversion: 1\n---\n\nWhy.\n",
                constraint = GOLDEN_CONSTRAINT
            ),
        )
        .unwrap();
    }

    /// Scoped `src/**` and not `docs/**` as [`Repo::seed_spec`] is: `scope.specs`
    /// and `context.specs` are what a perimeter rests on, and both fixtures ask
    /// about the source tree.
    fn seed_golden_spec(&self, id: &str) {
        std::fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: spec\nslug: a-document\ntitle: A document\n\
                 created: 2026-08-01T00:00:00Z\nauthor: human:marie\nstatus: proposed\n\
                 scope:\n  - src/**\nschema: 3\nversion: 1\n---\n\
                 \nThe document itself.\n"
            ),
        )
        .unwrap();
    }

    fn seed_golden_task(&self, id: &str, title: &str, blocked_by: &[&str]) {
        let blocked_by = format!("blocked_by: [{}]\n", blocked_by.join(", "));
        std::fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\nslug: example\ntitle: {title}\n\
                 created: 2026-07-28T00:00:00Z\nauthor: human:marie\nstatus: open\n\
                 scope:\n  - src/**\n{blocked_by}\
                 done_criteria: |\n  A verifiable criterion.\ncriteria_by: creator\n\
                 schema: 3\nversion: 1\n---\n\nFree body.\n"
            ),
        )
        .unwrap();
    }

    /// A second live claim under one identity, forged.
    ///
    /// `claim` refuses one (TASK-a548c95261a5), and rightly: `status.also_held`
    /// exists for two machines pushing under one identity, which is a state the
    /// refs can hold and this binary will not produce. So the record is written
    /// the way that state arrives, by another clone updating the ref.
    fn forge_claim(&self, id: &str, from: &str) {
        let record = self
            .claim_ref(from)
            .expect("the claim being copied has to exist")
            .replace(&format!("task: {from}"), &format!("task: {id}"));
        self.write_ref(&format!("refs/ank/claims/{id}"), &record);
    }

    /// A detached proof, forged for the same reason.
    ///
    /// `attest --detached` refuses when the remote is unreachable, the ref being
    /// the whole product, and this corpus has no remote: a fixture that grew one
    /// would be pinning a clone rather than a document. The record is the one
    /// `serialize_record` writes.
    fn forge_detached_proof(&self, id: &str) {
        let record = format!(
            "state: proof{n}task: {id}{n}proofs:{n}- identity: process:github-actions{n}  \
             attested: '2026-07-30T00:00:00Z'{n}  proof:{n}    type: test{n}    \
             ref: ci-run-4242{n}",
            n = "\n"
        );
        self.write_ref(&format!("refs/ank/proof/{id}"), &record);
    }

    /// Writes `content` as a blob and points `name` at it, which is how a record
    /// arrives on a ref from anywhere but this process.
    fn write_ref(&self, name: &str, content: &str) {
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
            .write_all(content.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success(), "hash-object: {}", stderr(&out));
        let blob = String::from_utf8_lossy(&out.stdout).trim().to_string();
        self.git(&["update-ref", name, &blob]);
    }

    /// A log entity, for `show.log` and `log-read.entries`.
    ///
    /// Written as a file rather than made with `ank log`, which would need a
    /// live claim: a claim in this corpus is a decision of its own, and the
    /// coordination fixtures make it deliberately further down.
    fn seed_golden_log(&self, id: &str, about: &str) {
        std::fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: log\ntitle: what the last holder learned\n\
                 created: 2026-07-29T00:00:00Z\nauthor: human:marie\n\
                 scope:\n  - src/**\nabout: {about}\n\
                 seq: 0\nschema: 3\nversion: 1\n---\n\nThe entry itself.\n"
            ),
        )
        .unwrap();
    }

    /// An entry that records machinery rather than work, which is what
    /// `show.machinery` and `log-read.machinery` are (ADR-16813b3bcf37).
    ///
    /// Written by hand and at schema 4, because no verb writes one yet: the
    /// verbs that will are TASK-3c12e0ced2c0, and a declaration no fixture
    /// reaches is a declaration nothing verifies, which the conformance test at
    /// the end of this file refuses by name.
    fn seed_golden_machinery(&self, id: &str, about: &str) {
        std::fs::write(
            self.0.join(".ank/entities").join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: log\n\
                 title: \"constraint, body: 1 -> 2, was 6f1d9c04a7b2\"\n\
                 created: 2026-07-29T00:00:01Z\nauthor: human:marie\n\
                 scope:\n  - src/**\nabout: {about}\n\
                 seq: 1\nrecords: edit\nschema: 4\nversion: 1\n---\n\nThe entry itself.\n"
            ),
        )
        .unwrap();
    }

    /// A file under `docs/`, so that a seeded spec's scope names something and
    /// the dead-scope machinery stays out of the fixture under test.
    fn seed_docs(&self) {
        std::fs::create_dir_all(self.0.join("docs")).unwrap();
        std::fs::write(self.0.join("docs/doc.md"), "The document.\n").unwrap();
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

/// An ADR written through the binary, so it carries the `author` the signals
/// under test read.
///
/// `seed_adr` writes a schema 1 file with no author at all, which is the corpus
/// that predates the field — and every actor signal skips it by design (§3). A
/// fixture for *who ratified what* cannot be built out of entities that say
/// nobody wrote them, so this one goes through `new` and lets the verb resolve
/// `$ANK_AGENT` the way a real write does.
fn new_adr(r: &Repo, author: &str, constraint: &str) -> String {
    let out = r.ank(
        author,
        &[
            "new",
            "adr",
            "--title",
            "A decision",
            "--scope",
            "src/**",
            "--constraint",
            constraint,
        ],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    stdout(&out)
        .split_whitespace()
        .nth(1)
        .expect("created <id> <slug>")
        .to_string()
}

/// A repository whose `src/**` exists and whose signing key is declared, which
/// is the ground every ratification fixture below needs: a scope matching
/// nothing is a fault of its own, and an undeclared key puts §8 in advisory
/// mode where no signature is judged at all.
fn ready_to_ratify() -> Repo {
    let r = Repo::new();
    r.enable_signing();
    declare_signing_key(&r);
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r
}

/// The hole TASK-5d38636bb4e5 names, closed: a ratification records the actor
/// that ran it, and the record is in the corpus rather than only in the commit.
///
/// Three ratifications in this project's own corpus were typed by an agent
/// under a passphrase the maintainer's gpg-agent had cached, at the
/// maintainer's instruction. Every mechanism in §8 reported exactly what it was
/// built to report, and none of them could say that: the signature says *this
/// key authorised it*, and a cached passphrase makes that true of an agent's
/// keystroke as well as of a human's.
///
/// Through the binary because that is where the identity comes from. The record
/// is whatever `$ANK_AGENT` resolved to in the process that ran `accept`, and
/// no unit test over `promote` reaches an environment variable.
#[test]
fn a_ratification_by_a_human_is_recorded_as_a_human_reading() {
    let r = ready_to_ratify();
    let id = new_adr(&r, "claude-code/opus-5", "Do not do X.");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let out = r.ank("human:marie", &["accept", &id]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let text = r.adr_text(&id);
    assert!(
        text.contains("verified:\n  - by: human:marie\n    at: "),
        "the actor that ran accept is on the entity, typed: {text}"
    );

    // The record survives a read through the binary, which is the half that
    // makes it a record rather than a byte in a file nobody is allowed to open.
    let out = r.ank("claude-code/opus-5", &["show", &id]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).contains("by: human:marie"), "{}", stdout(&out));

    // And it is not the self-ratification case: an agent wrote the decision, a
    // human ratified it, and that is the shape §8 exists to produce.
    let out = r.ank("claude-code/opus-5", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        !stdout(&out).contains("ratified by its own author"),
        "{}",
        stdout(&out)
    );

    // The reading a human left is also what silences the signal that says an
    // agent wrote this and nobody read it — because now somebody has.
    assert!(
        !stdout(&out).contains("read by no human"),
        "a human ratification is a human reading: {}",
        stdout(&out)
    );
}

/// The other half of the same distinction: an agent ratifying leaves a record
/// that says so, and it does not pass for a human one.
///
/// This is the case that was invisible. The commit is signed, `check` verifies
/// it against `.ank/allowed_signers`, and the corpus used to read back as a
/// decision a human stood behind — because the only thing recorded was the
/// signature, and the key is not the hand that typed.
///
/// **The record is not a defence and this test does not pretend otherwise.**
/// `$ANK_AGENT` is declared and never proved, so the agent below could have
/// written `human:` in front of its own name, exactly as ADR-6b3f19e08a24
/// already concedes for every freeze in the system. What is asserted is that an
/// honest ratification leaves a trace a reader can tell apart.
#[test]
fn a_ratification_by_an_agent_is_recorded_as_an_agent_reading() {
    let r = ready_to_ratify();
    let id = new_adr(&r, "human:marie", "Do not do X.");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let out = r.ank("claude-code/opus-5", &["accept", &id]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let text = r.adr_text(&id);
    assert!(
        text.contains("verified:\n  - by: claude-code/opus-5\n    at: "),
        "the agent that ran accept is named as one: {text}"
    );
    assert!(
        !text.contains("by: human:"),
        "nothing turns an agent's keystroke into a human act: {text}"
    );

    // The signature is real and declared, so §8 is satisfied and says nothing.
    // That is the point: the ratification is valid, and the record is what
    // distinguishes it from the one above rather than what refuses it.
    let out = r.ank("human:marie", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(!stdout(&out).contains("not signed"), "{}", stdout(&out));
}

/// Self-ratification: the entity's author is the actor that ratified it, and
/// `check` says so as a signal.
///
/// **A signal and never a fault**, and the reason is in the corpus rather than
/// in a preference. A solo maintainer writes the decision and ratifies it,
/// legitimately and every time; a rule that reddened over it would redden this
/// project's own corpus wholesale and be silenced within a week. What is worth
/// reporting is that the one act meant to come from outside the entity came
/// from inside it, and the reader is who decides what that is worth.
#[test]
fn ratifying_your_own_decision_is_a_signal_and_never_a_fault() {
    let r = ready_to_ratify();
    let id = new_adr(&r, "human:marie", "Do not do X.");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let out = r.ank("human:marie", &["accept", &id]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let out = r.ank("human:marie", &["check"]);
    assert_eq!(
        code(&out),
        0,
        "a signal is not a finding that reddens: {}{}",
        stdout(&out),
        stderr(&out)
    );
    let said = stdout(&out);
    assert!(
        said.contains(&format!(
            "signal: {id}: ratified by its own author (human:marie)"
        )),
        "the signal names the entity and the actor: {said}"
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

/// A ratification git refused to commit leaves nothing behind
/// (TASK-1dbb6e7843f1).
///
/// Measured on this corpus on 2026-08-18, ratifying ADR-24e306277bd4 on a
/// machine whose ratification key was protected by a passphrase nothing
/// supplied. `git commit` failed, `accept` exited 9 carrying git's message, and
/// the ADR came out `accepted` carrying the anchor of a commit that does not
/// exist. `check` calls that a signal, which is right for the bootstrap case
/// the clause was written for and generous here; `accept` will not repair it,
/// an existing anchor being the one thing it refuses to overwrite; so the only
/// route out was `ank edit`, which no message names.
///
/// The lever is the `gpg.format` git rejects, the same one the test above uses
/// and surgical for the same reason: every other question git is asked here
/// still answers, and only the signing fails.
///
/// Through the binary, because what is under test is what the process leaves on
/// disk. A unit test would be asserting about a call `accept` made rather than
/// about a file, which is the shape CLAUDE.md warns about and the shape that let
/// this defect through in the first place.
#[test]
fn a_ratification_that_could_not_be_committed_leaves_the_entity_untouched() {
    const ADR: &str = "ADR-00000000dbb1";
    let r = Repo::new();
    r.enable_signing();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    declare_signing_key(&r);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let path = r.0.join(".ank/entities").join(format!("{ADR}.md"));
    // Read as text, and compared as text: the bytes are the assertion, and a
    // failure printing two byte arrays says nothing a reader can act on.
    // `read_to_string` translates no newline on any platform, so it is the same
    // comparison spelled legibly.
    let before = std::fs::read_to_string(&path).unwrap();
    let head = r.git(&["rev-parse", "HEAD"]);

    r.git(&["config", "gpg.format", "bogus"]);
    let out = r.ank("marie@laptop", &["accept", ADR]);
    let said = stderr(&out);
    assert_ne!(code(&out), 0, "a ratification that did not happen: {said}");
    assert!(
        said.contains("gpg.format"),
        "and the message names what failed, in git's own words: {said}"
    );

    // The criterion, in as many words: byte for byte, `version` included.
    let after = std::fs::read_to_string(&path).unwrap();
    assert_eq!(
        after, before,
        "the entity is what it was before the accept that failed"
    );
    assert!(after.contains("status: proposed"), "{after}");
    assert!(!after.contains("ratified:"), "{after}");
    assert!(!after.contains("verified:"), "{after}");
    assert_eq!(
        head,
        r.git(&["rev-parse", "HEAD"]),
        "and no commit was made"
    );

    // Nothing is left staged either: `git add` ran before `git commit` refused,
    // and an index holding the write would commit it under the next commit
    // anybody makes in this repository, which is the same corpus by another
    // route.
    assert_eq!(
        r.git(&["diff", "--cached", "--name-only"]),
        "",
        "the write git refused to commit is not left in the index"
    );

    // And the failure cost the caller nothing but the failure: with signing
    // working, the same command still ratifies. This is what the byte-for-byte
    // restore buys — a `version` bumped by the failed attempt would have left
    // the compare-and-swap of the next one refusing.
    r.git(&["config", "gpg.format", "ssh"]);
    let out = r.ank("marie@laptop", &["accept", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains("status: accepted"), "{text}");
    assert!(text.contains("ratified:"), "{text}");
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

// ---------------------------------------------------------------------------
// References between documents (TASK-50dd8f9b565c, ADR-c88f99e1c16e)
// ---------------------------------------------------------------------------

const CITING: &str = "SPEC-00000000a001";
const GONE: &str = "SPEC-00000000ffff";
const DRAFT: &str = "SPEC-00000000b002";
const REPLACED: &str = "SPEC-00000000c003";
const SUCCESSOR: &str = "SPEC-00000000d004";
const FOLLOWER: &str = "SPEC-00000000e005";

/// A corpus holding all three states a reference can be in, and one document
/// that has already followed its chain.
fn cited_fixture() -> Repo {
    let r = Repo::new();
    r.seed_docs();
    r.seed_spec(CITING, "proposed", &[GONE, DRAFT, REPLACED], None);
    r.seed_spec(DRAFT, "proposed", &[], None);
    r.seed_spec(REPLACED, "superseded", &[], None);
    r.seed_spec(SUCCESSOR, "accepted", &[], Some(REPLACED));
    r.seed_spec(FOLLOWER, "proposed", &[REPLACED, SUCCESSOR], None);
    r
}

/// Every finding `check` prints about one entity.
fn findings_about(said: &str, id: &str) -> Vec<String> {
    said.lines()
        .filter(|l| l.contains(id))
        .map(str::to_string)
        .collect()
}

/// **`check` resolves what a specification declares it rests on**
/// (ADR-c88f99e1c16e, TASK-50dd8f9b565c).
///
/// This is the mechanism that decision rests on. It argues that cutting a
/// specification into documents is safe because the drift it risks is
/// *detected*, and until this runs through the binary that is a promise with
/// nothing behind it.
///
/// Through the binary, because the claim is about what `check` reports and what
/// its exit code then means to a pipeline: a fault has to reach exit 8 and a
/// signal has to leave it at 0, and no unit test on the report can say whether
/// the process agreed.
#[test]
fn check_resolves_the_references_a_spec_declares_to_another() {
    let r = cited_fixture();
    let out = r.ank("claude-code@ank", &["check"]);
    let said = stdout(&out);

    // Absent: a fault, the same condition `blocked_by` naming nothing is.
    let about = findings_about(&said, GONE);
    assert_eq!(about.len(), 1, "one line for the missing target: {said}");
    assert!(about[0].starts_with("error:"), "{said}");
    assert!(about[0].contains("does not exist"), "{said}");
    assert!(
        about[0].contains(&format!("ank amend {CITING} --drop-reference {GONE}")),
        "a finding names the command that repairs it: {said}"
    );

    // Unaccepted: a signal. Two specifications are legitimately written at
    // once, and refusing that would make it impossible to write the second.
    let about = findings_about(&said, DRAFT);
    assert_eq!(about.len(), 1, "{said}");
    assert!(about[0].starts_with("signal:"), "{said}");
    assert!(about[0].contains("not accepted"), "{said}");
    assert!(about[0].contains(&format!("ank accept {DRAFT}")), "{said}");

    // Superseded: nothing at all, because the reference resolves
    // (ADR-c88f99e1c16e, TASK-e2da6b0cc817). A reference names a document and
    // not a revision of it, so the chain is followed by the reader and the
    // citation is left exactly as its author wrote it.
    assert!(
        findings_about(&said, REPLACED)
            .iter()
            .all(|l| !l.contains(CITING)),
        "a reference that resolves through its chain was reported: {said}"
    );

    // And the document that had already spelled the resolution out by hand is
    // silent for the same reason rather than for a special one: the branch that
    // let it off has become the general rule.
    assert!(findings_about(&said, FOLLOWER).is_empty(), "{said}");

    // The severities reach the process: one fault, so exit 8.
    assert_eq!(code(&out), 8, "{said}");
}

/// The commands those findings name are commands the verb accepts.
///
/// A finding naming a repair that refuses on the spot is worse than a finding
/// with no repair at all, and two of the three here land on documents `amend`
/// used to turn down outright: a citation is not covered by the ratification
/// anchor, so following a chain on an accepted document is an amend and never a
/// supersession.
#[test]
fn the_repair_a_reference_finding_names_is_one_amend_accepts() {
    let r = cited_fixture();

    // The absent target, dropped. It cannot be resolved — that is what the
    // fault says — so the flag matches what the entity stores.
    let out = r.ank(
        "claude-code@ank",
        &["amend", CITING, "--drop-reference", GONE],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // The chain, followed in one call.
    let out = r.ank(
        "claude-code@ank",
        &[
            "amend",
            CITING,
            "--reference",
            SUCCESSOR,
            "--drop-reference",
            REPLACED,
        ],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // And an accepted document's citations are reachable, where its scope is
    // not: the anchor covers the body and the scope, and a reference is
    // neither.
    let out = r.ank(
        "claude-code@ank",
        &["amend", SUCCESSOR, "--reference", DRAFT],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let out = r.ank(
        "claude-code@ank",
        &["amend", SUCCESSOR, "--scope", "src/**"],
    );
    assert_eq!(code(&out), 6, "{}", stdout(&out));

    // What is left is the draft, which was a signal before and still is.
    let said = stdout(&r.ank("claude-code@ank", &["check"]));
    assert!(findings_about(&said, GONE).is_empty(), "{said}");
    assert!(
        findings_about(&said, REPLACED)
            .iter()
            .all(|l| !l.contains(CITING)),
        "{said}"
    );
    assert_eq!(
        code(&r.ank("claude-code@ank", &["check"])),
        0,
        "the faults are gone, so the exit code is: {said}"
    );
}

// ---------------------------------------------------------------------------
// A reference resolves through its succession (ADR-c88f99e1c16e,
// TASK-e2da6b0cc817)
// ---------------------------------------------------------------------------

const FIRST: &str = "SPEC-00000000c101";
const SECOND: &str = "SPEC-00000000c102";
const THIRD: &str = "SPEC-00000000c103";
const PENDING: &str = "SPEC-00000000c104";
const RETIRED_END: &str = "SPEC-00000000c105";
const BEHIND: &str = "SPEC-00000000c106";
const BEHIND_TWO: &str = "SPEC-00000000c107";
const BEHIND_DEAD: &str = "SPEC-00000000c108";

/// **A citation two hops behind resolves, and nothing is written to make it.**
///
/// The length is the point. One hop was already let off by the branch this
/// replaces, on condition that the citing document also stored the end of the
/// chain — a reader following a succession, spelled by hand and stored twice.
/// Two hops is what that branch could never do without the corpus re-pointing
/// every citation after every revision, which is the churn ADR-c88f99e1c16e
/// measured: four citations after superseding two documents, nine hours later.
#[test]
fn a_citation_two_hops_behind_resolves_and_the_file_is_not_touched() {
    let r = Repo::new();
    r.seed_docs();
    r.seed_spec(FIRST, "superseded", &[], None);
    r.seed_spec(SECOND, "superseded", &[], Some(FIRST));
    r.seed_spec(THIRD, "accepted", &[], Some(SECOND));
    r.seed_spec(BEHIND, "accepted", &[FIRST], None);

    let before = std::fs::read(r.0.join(".ank/entities").join(format!("{BEHIND}.md"))).unwrap();

    let out = r.ank("claude-code/1.0", &["check"]);
    let said = both_streams(&out);
    assert!(
        findings_about(&said, BEHIND)
            .iter()
            .all(|l| !l.contains("references")),
        "the chain ends on an accepted document, whatever its length: {said}"
    );
    assert_eq!(code(&out), 0, "and nothing else is wrong with it: {said}");

    // **Nothing is written to make a reference resolve.** The whole argument
    // against repairing citations in place was that one `accept` would write to
    // nine entities and, under ADR-16813b3bcf37, leave nine machinery entries
    // behind. A read that writes is the same defect one verb further along.
    let after = std::fs::read(r.0.join(".ank/entities").join(format!("{BEHIND}.md"))).unwrap();
    assert_eq!(before, after, "check wrote to the citing document");
    let text = String::from_utf8(after).unwrap();
    assert!(
        text.contains("version: 1"),
        "the version did not move: {text}"
    );
    assert!(
        text.contains(&format!("references: [{FIRST}]")),
        "the file keeps the identifier its author wrote: {text}"
    );
}

/// The chain is followed, and then the ordinary rule applies to where it ends.
///
/// A succession ending on a document nobody has ratified is the `not accepted`
/// signal one link further along, and it names the same command: two
/// specifications are legitimately drafted at once, and this is that case seen
/// through a citation written before either.
#[test]
fn a_chain_ending_on_a_draft_is_the_unaccepted_signal_one_link_along() {
    let r = Repo::new();
    r.seed_docs();
    r.seed_spec(SECOND, "superseded", &[], None);
    r.seed_spec(PENDING, "proposed", &[], Some(SECOND));
    r.seed_spec(BEHIND_TWO, "accepted", &[SECOND], None);

    let out = r.ank("claude-code/1.0", &["check"]);
    let said = both_streams(&out);
    let about: Vec<String> = findings_about(&said, BEHIND_TWO)
        .into_iter()
        .filter(|l| l.contains("references"))
        .collect();
    assert_eq!(about.len(), 1, "{said}");
    assert!(about[0].starts_with("signal:"), "{said}");
    assert!(about[0].contains("not accepted"), "{said}");
    assert!(
        about[0].contains(&format!("ank accept {PENDING}")),
        "the command names where the chain ends, which is what a reader can act \
         on: {said}"
    );
}

/// A chain leading nowhere keeps the signal it has today, in the same words.
///
/// The entity at the end says it was replaced and nothing replaced it, which is
/// already a fault against that entity. For whoever cites it, the statement is
/// that the citation has nowhere to follow to — and it is a signal, because the
/// corpus defect is the other entity's and is reported there.
#[test]
fn a_chain_ending_on_a_superseded_entity_that_nothing_replaces_keeps_its_signal() {
    let r = Repo::new();
    r.seed_docs();
    r.seed_spec(RETIRED_END, "superseded", &[], None);
    r.seed_spec(BEHIND_DEAD, "accepted", &[RETIRED_END], None);

    let out = r.ank("claude-code/1.0", &["check"]);
    let said = both_streams(&out);
    let about: Vec<String> = findings_about(&said, BEHIND_DEAD)
        .into_iter()
        .filter(|l| l.contains("references"))
        .collect();
    assert_eq!(about.len(), 1, "{said}");
    assert!(about[0].starts_with("signal:"), "{said}");
    assert!(
        about[0].contains(&format!(
            "references {RETIRED_END}, which is superseded and names no successor \
             (ank show {RETIRED_END})"
        )),
        "the words are the ones it had: {said}"
    );
}

/// The three findings that remain, each in its own case and at its own
/// severity.
///
/// Removing a finding is easy to do too widely, and these three sit beside the
/// one that went. A change that quietly took a fault down to silence would be
/// worse than the churn it was meant to end.
#[test]
fn the_three_findings_beside_it_keep_their_severity() {
    let r = cited_fixture();
    // The task exists, so that the refusal below is about the kind and not
    // about an id nothing resolves.
    r.seed_task(ID, Some("A verifiable criterion."));
    let out = r.ank("claude-code/1.0", &["check"]);
    let said = both_streams(&out);

    // Absent: a fault, and the repair deletes.
    let about = findings_about(&said, GONE);
    assert_eq!(about.len(), 1, "{said}");
    assert!(about[0].starts_with("error:"), "{said}");
    assert!(about[0].contains("does not exist"), "{said}");

    // Not yet accepted: a signal, naming `ank accept`.
    let about = findings_about(&said, DRAFT);
    assert_eq!(about.len(), 1, "{said}");
    assert!(about[0].starts_with("signal:"), "{said}");
    assert!(about[0].contains(&format!("ank accept {DRAFT}")), "{said}");

    // A kind a specification may not cite: a fault, refused at every door that
    // writes one and reported here for a file that arrived some other way.
    let out = r.ank("claude-code/1.0", &["amend", CITING, "--reference", ID]);
    assert_eq!(code(&out), 1, "{}", both_streams(&out));

    // The fault reaches the process: the severities are what a pipeline reads.
    assert_eq!(code(&r.ank("claude-code/1.0", &["check"])), 8, "{said}");
}

const RETIRED: &str = "SPEC-00000000f006";
const HEIR: &str = "SPEC-00000000a007";

/// The same three states, cited twice: once by a live document, once by one
/// that has itself been replaced.
fn retired_citer_fixture() -> Repo {
    let r = Repo::new();
    r.seed_docs();
    r.seed_spec(CITING, "proposed", &[GONE, DRAFT, REPLACED], None);
    r.seed_spec(DRAFT, "proposed", &[], None);
    r.seed_spec(REPLACED, "superseded", &[], None);
    r.seed_spec(SUCCESSOR, "accepted", &[], Some(REPLACED));
    r.seed_spec(RETIRED, "superseded", &[GONE, DRAFT, REPLACED], None);
    r.seed_spec(HEIR, "accepted", &[SUCCESSOR], Some(RETIRED));
    r
}

/// **A citer that is itself superseded is not asked to follow anything**
/// (TASK-a6c643216f51).
///
/// Measured the first time a spec was replaced: three live documents followed
/// the chain, which is ADR-c88f99e1c16e working as designed, and the fourth had
/// been retired in the same operation. What the finding asked of it was that a
/// document nobody reads any more be edited to cite one written after it was
/// retired — and the repair it named would have worked, bumping the `version` of
/// a document that is supposed to be settled.
///
/// A superseded entity is history: it records what was decided and what it
/// rested on at the time. So the whole reference half is skipped for it — the
/// absent target, the unaccepted one and the superseded one alike — and what
/// must not move is the live case, which is the entire reason the signal exists.
///
/// Through the binary, because the claim is about what `check` prints and about
/// the exit code that print then carries to a pipeline.
#[test]
fn a_superseded_document_is_not_asked_to_follow_a_chain() {
    let r = retired_citer_fixture();
    let out = r.ank("claude-code@ank", &["check"]);
    let said = stdout(&out);

    // Nothing whatsoever against the retired citer, and nothing naming it.
    assert!(
        findings_about(&said, RETIRED).is_empty(),
        "a superseded document was asked to repair its citations: {said}"
    );

    // The three states it cites are reported once each, and the once is the
    // live document: a citation is counted for the reader who can still act on
    // it, and never twice.
    for target in [GONE, DRAFT] {
        let about = findings_about(&said, target);
        assert_eq!(
            about.len(),
            1,
            "only the live citer owes a repair for {target}: {said}"
        );
        assert!(about[0].contains(CITING), "{said}");
    }
    // The superseded target is reported by nobody now, live citer included: it
    // resolves through its chain to an accepted document (TASK-e2da6b0cc817).
    assert!(
        findings_about(&said, REPLACED)
            .iter()
            .all(|l| !l.contains("references")),
        "{said}"
    );

    // The live fault reaches the process. Skipping the retired citer silences a
    // finding, never a corpus.
    assert_eq!(code(&out), 8, "{said}");
}

/// A specification cites a spec or an adr, and nothing that is meant to be
/// retired. The rule is one function, read by the two writers and by `check`.
#[test]
fn a_reference_names_a_document_or_a_decision_and_no_other_kind() {
    let r = cited_fixture();
    r.seed_task(ID, Some("A verifiable criterion."));

    // An ADR is citable, and it is the case the rule exists to allow: a
    // document resting on a binding decision.
    r.seed_adr("ADR-00000000a0a0", "A rule.", "docs/**");
    let out = r.ank(
        "claude-code@ank",
        &["amend", CITING, "--reference", "ADR-00000000a0a0"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // A task is not. It is work that finishes, so a document citing one would
    // cite something the corpus is designed to retire.
    let out = r.ank("claude-code@ank", &["amend", CITING, "--reference", ID]);
    assert_eq!(code(&out), 1, "{}", stdout(&out));
    assert!(stderr(&out).contains("spec or an adr"), "{}", stderr(&out));

    // The same refusal at creation, where the citation is first written.
    let out = r.ank(
        "claude-code@ank",
        &[
            "new",
            "spec",
            "--title",
            "A document",
            "--scope",
            "docs/**",
            "--reference",
            ID,
        ],
    );
    assert_eq!(code(&out), 1, "{}", stdout(&out));
    assert!(stderr(&out).contains("spec or an adr"), "{}", stderr(&out));

    // And the field belongs to the one kind that carries it.
    let out = r.ank(
        "claude-code@ank",
        &["amend", ID, "--reference", "ADR-00000000a0a0"],
    );
    assert_eq!(code(&out), 1, "{}", stdout(&out));
    assert!(
        stderr(&out).contains("references applies to a spec"),
        "{}",
        stderr(&out)
    );
}

// ---------------------------------------------------------------------------
// A description that enumerates output (TASK-213979c9df67)
// ---------------------------------------------------------------------------

/// The word a description must use to name a group `scope` prints.
///
/// A mapping and not a substring test, because the two surfaces speak
/// deliberately different languages: the listing groups by kind and prints
/// `ADR`, while a description addresses the reader in the vocabulary of §3 and
/// says *constraints*. Asserting the heading itself appeared would demand that
/// `help` say `ADR (20)`, which is the machine's word for it.
///
/// A group with no word declared here fails loudly rather than passing, the way
/// `valid_value` does above: a fourth heading added to `scope` should make
/// somebody say what a description has to call it, not slip through a test that
/// only knew three.
fn word_for_group(heading: &str) -> &'static str {
    match heading {
        "ADR" => "constraint",
        "SPECIFICATIONS" => "specification",
        "TASKS" => "task",
        other => panic!(
            "`ank scope` prints a group `{other}` and nobody has said what a \
             description must call it: add it to word_for_group rather than \
             guess"
        ),
    }
}

/// **A description that enumerates the verb's output is checked against the
/// output** (§9, TASK-213979c9df67).
///
/// §9 already rules that a description is a fourth surface able to misinform,
/// and the test that enforces it walks *flags*: it fails when a description
/// advertises a flag the verb does not offer. Nothing compared a description
/// that lists what a verb *prints* against what it prints, and one had gone
/// stale exactly there — `scope` announced "the constraints that bind it and
/// the tasks that touch it" while printing three groups, the third being the
/// specifications. A reader of `help` therefore could not learn that `scope` is
/// the verb that says which document governs a file, which is one of the two
/// things it exists for.
///
/// Both surfaces are read from the binary, because the claim is about what the
/// process prints and not about the table it is derived from.
#[test]
fn the_description_of_scope_names_every_group_it_prints() {
    let r = Repo::new();
    r.seed_docs();
    // One entity of each kind that `scope` groups, all on one perimeter: the
    // comparison is only worth making where every group is printed.
    r.seed_task_scoped(ID, "docs/**");
    r.seed_adr("ADR-00000000a0a0", "A rule.", "docs/**");
    r.seed_spec("SPEC-00000000d0c1", "accepted", &[], None);

    let printed = stdout(&r.ank("claude-code@ank", &["scope", "docs/doc.md"]));
    let groups: Vec<String> = printed
        .lines()
        .filter_map(|l| l.split_once(" ("))
        .filter(|(head, _)| !head.is_empty() && !head.starts_with(' '))
        .map(|(head, _)| head.to_string())
        .collect();
    assert!(
        groups.len() >= 3,
        "the fixture must make `scope` print every group for the comparison to \
         mean anything: {printed}"
    );

    let described = stdout(&r.ank("claude-code@ank", &["help", "scope"])).to_lowercase();
    for heading in &groups {
        let word = word_for_group(heading);
        assert!(
            described.contains(word),
            "`ank scope` prints a `{heading}` group and `ank help scope` never \
             says {word:?}: a description that enumerates what a verb emits and \
             leaves one out is the fourth surface §9 refuses.\n\nprinted:\n{printed}\ndescribed:\n{described}"
        );
    }
}

// ---------------------------------------------------------------------------
// Concurrent readers of one corpus (TASK-e9dfaf187a1b)
// ---------------------------------------------------------------------------

/// **Two agents reading one corpus at the same time both get an answer.**
///
/// The nominal execution model is one working tree per agent (§7), and
/// worktrees of a repository share its `.ank/` — so two agents running
/// `ank context` inside the same second is the ordinary shape of a parallel
/// session, not an exotic one. It did not work: measured in CI on all three
/// platforms at once, two of three concurrent readers came back
/// `error[1]: index: attempt to write a readonly database` and
/// `error[1]: index: disk I/O error`.
///
/// The symptom names the cause. `attempt to write a readonly database` is what
/// SQLite reports when the file underneath an open connection has been unlinked,
/// and `open_raw` deletes the index whenever opening it fails — so one reader
/// losing a lock race deleted the database the others were using, and the
/// disposability rule of §6 turned a moment of contention into an error for
/// everybody else.
///
/// Through the binary and with real processes, because that is the only place
/// the defect exists: every in-process test opened one connection at a time and
/// passed throughout.
#[test]
fn concurrent_readers_of_one_corpus_all_answer() {
    let r = Repo::new();
    for i in 0..12 {
        r.seed_task_titled(&format!("TASK-00000000{i:04x}"), &format!("Task {i}"));
    }

    // Every verb here opens the index, and each of them refreshes it while
    // reading (§6), so all three are writers whatever their names suggest.
    let verbs: [&[&str]; 3] = [&["find", "Task"], &["context"], &["scope", "src"]];
    let mut running = Vec::new();
    for i in 0..12 {
        running.push(
            ank_command()
                .args(verbs[i % verbs.len()])
                .arg("--repo")
                .arg(&r.0)
                .env("ANK_AGENT", format!("agent-{i}@host"))
                .current_dir(std::env::temp_dir())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("the binary must have been built"),
        );
    }

    let mut refused = Vec::new();
    for (i, child) in running.into_iter().enumerate() {
        let out = child.wait_with_output().expect("a spawned ank must finish");
        if !out.status.success() {
            refused.push(format!(
                "#{i} exited {:?}: {}",
                code(&out),
                stderr(&out).trim()
            ));
        }
    }
    assert!(
        refused.is_empty(),
        "readers of one corpus refused each other. The index is derived, \
         disposable and rebuildable (§6), so contention on it is never a reason \
         to fail a reader:\n{}",
        refused.join("\n")
    );
}

/// The same twelve readers, with **no wait allowed at all**
/// (TASK-4111dfae8a87).
///
/// The test above proves the invariant on whatever machine happens to run it,
/// and that is its weakness: it passes when the hardware was fast enough for the
/// readers never to have met. Measured on CI, run 32061737531 on
/// `windows-latest`, it failed with twelve refusals out of twelve on a commit
/// that changed no code -- so the invariant was resting on a five-second wall
/// being wider than the queue, which is a margin and not a guarantee.
///
/// Setting the wall to zero is what an arbitrarily loaded runner is, and it is
/// deterministic. With it, any contention at all refuses immediately, so this
/// passes only if the readers take no write lock -- which is the property the
/// fix installed: a healthy schema is confirmed by a read, and a refresh with
/// nothing to write opens no transaction. Measured on this tree: four of sixty
/// refused before, none of a hundred and twenty after.
///
/// The index is warmed first, on purpose. A cold corpus has real work to
/// serialise and one process must wait for it; what must never contend is the
/// steady state, which is the state a board polling every thirty seconds and a
/// dozen agents running `find` are in.
#[test]
fn readers_of_a_warm_corpus_take_no_write_lock() {
    let r = Repo::new();
    for i in 0..12 {
        r.seed_task_titled(&format!("TASK-00000000{i:04x}"), &format!("Task {i}"));
    }
    // One reader with a normal wall, to build the index and leave it current.
    let warm = r.ank("warm@host", &["find", "Task"]);
    assert_eq!(code(&warm), 0, "warming: {}", stderr(&warm));

    let verbs: [&[&str]; 3] = [&["find", "Task"], &["context"], &["scope", "src"]];
    let mut running = Vec::new();
    for i in 0..12 {
        running.push(
            ank_command()
                .args(verbs[i % verbs.len()])
                .arg("--repo")
                .arg(&r.0)
                .env("ANK_AGENT", format!("agent-{i}@host"))
                .env("ANK_INDEX_BUSY_MS", "0")
                .current_dir(std::env::temp_dir())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("the binary must have been built"),
        );
    }

    let mut refused = Vec::new();
    for (i, child) in running.into_iter().enumerate() {
        let out = child.wait_with_output().expect("a spawned ank must finish");
        if !out.status.success() {
            refused.push(format!(
                "#{i} exited {:?}: {}",
                code(&out),
                stderr(&out).trim()
            ));
        }
    }
    assert!(
        refused.is_empty(),
        "a reader of a warm corpus asked for the write lock. Nothing diverged and          the schema was already installed, so there was nothing to write and no          lock to take; whatever asked for one is what makes this invariant rest          on a deadline again:
{}",
        refused.join("
")
    );
}

/// A corpus is keyed on its root commit, so a path cannot change what it is
/// (ADR-621a7fd96ce1).
///
/// Driven through the binary because the claim is about what a reader gets back,
/// and the reader in question is a board keying its rows on the answer. Four
/// halves, and the two middle ones are the whole point: the same repository
/// reached by a second path is the same corpus, and a second repository inside
/// the same working tree is a different one. A path answers both of those wrong.
#[test]
fn a_corpus_is_keyed_on_its_root_commit_and_not_on_its_path() {
    let corpus_of = |out: &std::process::Output| -> serde_yaml::Value {
        let doc: serde_yaml::Value =
            serde_yaml::from_str(&stdout(out)).expect("status --json must be readable");
        doc["corpus"].clone()
    };

    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    // A tree with no commits has no root commit, so it has no identity and says
    // so. Falling back to the path here would reintroduce, for the one case that
    // cannot be answered, exactly the defect this field removes.
    let out = r.ank(AGENT, &["status", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        corpus_of(&out).is_null(),
        "a tree with no history invented a value: {}",
        stdout(&out)
    );

    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "the root commit"]);
    let out = r.ank(AGENT, &["status", "--json"]);
    let here = corpus_of(&out);
    let sha = here
        .as_str()
        .expect("an identity, once there is history")
        .to_string();
    assert_eq!(sha.len(), 40, "the root commit, whole: {sha}");
    assert_eq!(
        sha,
        r.git(&["rev-list", "--max-parents=0", "HEAD"]).trim(),
        "the identity is the root commit and not a hash of something else"
    );

    // The same repository, a second path. A worktree is the case the ADR names
    // first, and it is the one a path gets wrong every time: two directories,
    // one corpus.
    let second = std::env::temp_dir().join(format!("ank-wt-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&second);
    r.git(&["worktree", "add", "-q", second.to_str().unwrap(), "HEAD"]);
    let out = ank_command()
        .args(["status", "--json", "--repo"])
        .arg(&second)
        .env("ANK_AGENT", AGENT)
        .current_dir(std::env::temp_dir())
        .output()
        .expect("the binary must have been built");
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(
        corpus_of(&out).as_str(),
        Some(sha.as_str()),
        "one repository reached by two paths answered two corpora"
    );

    // A second corpus inside the same working tree. Its own git repository, so
    // its own root commit, so its own identity -- and a reader keying on the
    // enclosing path would have merged the two.
    let nested = r.0.join("vendor");
    std::fs::create_dir_all(nested.join(".ank/entities")).unwrap();
    std::fs::write(
        nested.join(".ank/config.yml"),
        "schema: 1
claim_ttl_max: 2h
default_branch: main
",
    )
    .unwrap();
    for args in [
        vec!["init", "-q", "-b", "main"],
        vec!["config", "user.email", "test@ank.local"],
        vec!["config", "user.name", "Test"],
        vec!["config", "commit.gpgsign", "false"],
        vec!["add", "-A"],
        vec!["commit", "-qm", "another root"],
    ] {
        let out = git_command(&nested)
            .args(&args)
            .output()
            .expect("git must be installed");
        assert!(out.status.success(), "{args:?}: {}", stderr(&out));
    }
    let out = ank_command()
        .args(["status", "--json", "--repo"])
        .arg(&nested)
        .env("ANK_AGENT", AGENT)
        .current_dir(std::env::temp_dir())
        .output()
        .expect("the binary must have been built");
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let inner = corpus_of(&out);
    assert!(
        inner.as_str().is_some() && inner.as_str() != Some(sha.as_str()),
        "two corpora in one tree answered one identity: {inner:?} against {sha}"
    );

    let _ = std::fs::remove_dir_all(&second);
}

/// A closure with a perimeter that names nothing is a record, not a defect
/// (TASK-4c031f7b44ed).
///
/// Driven through the binary because the thing being judged is `check`'s exit
/// code, which is what CI routes on. Both terminal states in one test, because
/// the point is that they are *not* the same fact: a `done` task claimed to touch
/// files, a `closed` one claimed nothing.
///
/// The perimeter names a directory no commit ever carried, so neither
/// ADR-3094538d831e's rename walk nor its deletion clause has anything to lower
/// a severity with. That is the case with no way out, and it is
/// the one that used to redden a corpus for good: `amend` refuses a finished task,
/// so the fault could never be cleared.
#[test]
fn a_closed_task_whose_scope_names_nothing_leaves_check_green() {
    const GONE: &str = "TASK-00000000c105";
    let r = Repo::new();
    r.seed_task_scoped(GONE, "nowhere/**");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "a task whose perimeter never existed"]);

    // Open, it is work not started, and green. This is the state the corpus was
    // stuck in: honest, and permanent, because closing it was punished.
    let out = r.ank(AGENT, &["check"]);
    assert_eq!(code(&out), 0, "open: {}{}", stdout(&out), stderr(&out));
    assert!(
        stdout(&out).contains("work not started"),
        "open: {}",
        stdout(&out)
    );

    let out = r.ank(AGENT, &["close", GONE, "--reason", "it ships elsewhere"]);
    assert_eq!(code(&out), 0, "close: {}", stderr(&out));

    let out = r.ank(AGENT, &["check"]);
    let said = stdout(&out);
    assert_eq!(
        code(&out),
        0,
        "a closure with a dead perimeter must not redden a corpus: {said}{}",
        stderr(&out)
    );
    assert!(
        said.contains("the task is closed: nothing is owed"),
        "the closure needs its own sentence, not the open task's: {said}"
    );
    assert!(
        !said.contains("work not started"),
        "a closed task did not fail to start: {said}"
    );

    // `done` is the other terminal state and keeps the fault, because it *did*
    // claim to touch those files. Same corpus, same missing directory, so the
    // only variable is the status.
    let text = r.task_text(GONE).replace("status: closed", "status: done");
    std::fs::write(r.0.join(".ank/entities").join(format!("{GONE}.md")), text).unwrap();
    let out = r.ank(AGENT, &["check"]);
    assert_eq!(
        code(&out),
        8,
        "a done task claimed to touch files that are not there: {}{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(
        stdout(&out).contains("dead scope"),
        "done: {}",
        stdout(&out)
    );
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
// Short forms (TASK-f3e92656b5df, ADR-0c8ab846d262)
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
/// that arrives looking like an improvement. Grouping it (ADR-91b77f036884)
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
    // A second apart, because `created` is what orders the two and it has
    // one-second resolution (§3). Two entries inside one second are a real
    // case and have a test of their own below; here the question is the
    // direction, which needs two instants to have one.
    std::thread::sleep(std::time::Duration::from_millis(1100));
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
        j.starts_with(&format!(
            "{{\"contract\":1,\"about\":\"{ID}\",\"total\":2,\"shown\":2,\"entries\":["
        )),
        "{j}"
    );
    assert!(
        j.find("second thing").unwrap() < j.find("first thing").unwrap(),
        "{j}"
    );

    // **Any kind carries entries, an ADR included** (ADR-25f977377fa0). The
    // refusal that named a task by name went with the per-entity file it
    // guarded: `about` names an entity, so there is no kind that cannot be one.
    // Writing one asks for no claim either -- an ADR has none to hold, and
    // refusing there would be refusing on the absence of a state.
    const ADR: &str = "ADR-0000000000ab";
    r.seed_adr(ADR, "Do not do X.", "src/**");
    let out = r.ank("marie@laptop", &["log", ADR]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("no log entry yet"),
        "{}",
        stdout(&out)
    );

    let out = r.ank("marie@laptop", &["log", ADR, "the constraint bit here"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).starts_with("logged LOG-") && stdout(&out).contains(&format!("on {ADR}")),
        "{}",
        stdout(&out)
    );
    let out = r.ank("marie@laptop", &["show", ADR]);
    assert!(
        stdout(&out).contains("the constraint bit here"),
        "an ADR shows its entries like anything else: {}",
        stdout(&out)
    );
}

/// A control character a shell put in a message never reaches the corpus
/// (TASK-f3910718320a).
///
/// Through the binary because that is where it was measured, and because no
/// module test could have caught it: the message was correct when it was typed
/// and wrong when it arrived. PowerShell reads a backtick as its escape
/// character, so `` `rev-list `` inside a double-quoted argument became a
/// carriage return followed by `ev-list` before `ank` saw an argument at all.
/// `ank log` wrote it, `ank check` called the corpus healthy, and the line
/// endings guard of the pipeline refused the branch on all three runners.
///
/// Refused rather than stripped: ank records what a caller states and does not
/// silently delete bytes from a message somebody wrote, so the refusal names
/// the character by its escape and gives the command to run again — the
/// empty-message refusal above is the precedent and the wording follows it.
#[test]
fn log_refuses_a_message_carrying_a_control_character() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);

    let entries_before = r.entry_ids(ID);
    let record_before = r.claim_ref(ID).expect("the claim is live");

    // The message as it actually arrived, one line with a lone carriage return
    // in the middle of it. Built rather than written literally, so that what is
    // under test is legible in the source instead of hiding in an escape.
    let mangled = format!("walked the corpus with git {}ev-list", '\r');
    let out = r.ank("claude-code@ank", &["log", &mangled]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let said = stderr(&out);
    assert!(
        said.contains("\\r"),
        "the character is named by its escape: {said}"
    );
    assert!(
        said.contains("ank log \""),
        "the command to run again, never generic help: {said}"
    );
    assert_eq!(r.entry_ids(ID), entries_before, "nothing was written");
    assert_eq!(
        r.claim_ref(ID).as_deref(),
        Some(record_before.as_str()),
        "a refused write renews no claim"
    );

    // The same message without it logs, so what is refused is the character and
    // not the sentence.
    let clean = "walked the corpus with git rev-list";
    let out = r.ank("claude-code@ank", &["log", clean]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(r.log_text(ID).contains(clean), "{}", r.log_text(ID));

    // And the file that landed holds no byte outside the grammar of §3: an
    // entity is UTF-8, its lines end in a line feed, and nothing else below
    // U+0020 belongs in one.
    let written = r
        .entry_ids(ID)
        .into_iter()
        .find(|id| !entries_before.contains(id))
        .expect("the clean message produced an entry");
    let bytes = std::fs::read(r.0.join(".ank/entities").join(format!("{written}.md"))).unwrap();
    let stray: Vec<String> = bytes
        .iter()
        .filter(|b| **b < 0x20 && **b != b'\n')
        .map(|b| format!("{b:#04x}"))
        .collect();
    assert!(stray.is_empty(), "control bytes in {written}.md: {stray:?}");

    // The other door a caller's own text goes through, and the reason it is a
    // second door rather than the same one: `release` writes the transition
    // first and records the reason after it, so a refusal that came only at the
    // entry would hand the task back with nothing in the corpus saying why —
    // the gap `--reason` exists to close.
    let out = r.ank("claude-code@ank", &["release", "--reason", &mangled]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    let said = stderr(&out);
    assert!(
        said.contains("\\r"),
        "the same wording at both doors: {said}"
    );
    assert!(
        said.contains("ank release --reason \""),
        "and this verb's own command to run again: {said}"
    );
    assert!(
        r.task_text(ID).contains("status: in_progress"),
        "the transition did not happen: {}",
        r.task_text(ID)
    );
    assert!(
        r.claim_ref(ID).is_some(),
        "and the task was not handed back"
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
        !inside.0.join(".gitattributes").exists(),
        "init wrote git files into the repository it was merely standing in"
    );
    // The fixture writes this one, so the question is whether `init` touched
    // it, not whether it is there: `init` appends its own lines to a
    // `.gitignore` it finds, and this one is exactly as the fixture left it.
    assert_eq!(
        std::fs::read_to_string(inside.0.join(".gitignore")).unwrap(),
        ".ank/index.db\n",
        "init appended to the .gitignore of the repository it was merely standing in"
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
/// one", which the skill decision retired by dissolving the human side entirely
/// (ADR-91b77f036884).
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
        stdout(&out).contains(&format!("on {FIRST}")),
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

/// The task offered as another thing to take is one whose ref is free, and a
/// live claim someone else holds is not free (§7).
///
/// Found by running the tool. In a sandbox of two clones and two worktrees,
/// `claim` refused and pointed at a task the same listing had rendered
/// `[claimed:agent-a2/1]` one command earlier: the hint printed an exact
/// command that refuses with code 4 the moment it is run, which is the generic
/// help by another route — the very thing the completion-ref skip next to it
/// was written to prevent (TASK-f601ba59229e).
///
/// The fixture builds the only state in which the defect is visible: the claim
/// is taken on a branch, so the ref is live for every checkout of this
/// repository while the file `main` carries still reads `open`. Claiming on
/// `main` alone would move the file to `in_progress` and the status filter
/// would hide the candidate before the ref was ever consulted, which is a
/// fixture testing nothing.
///
/// A lapsed claim is the other half, and it is not a variation: pickup after
/// expiry is a legal transition, `claim` takes it, and a fix that skipped a
/// lapsed candidate would trade one wrong answer for another.
#[test]
fn the_refusal_does_not_offer_a_task_a_live_claim_already_holds() {
    let wanted = "TASK-a00000000005";
    let held = "TASK-b00000000005";
    let r = Repo::new();
    r.seed_task(wanted, Some("A verifiable criterion."));
    r.seed_task(held, Some("A verifiable criterion."));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed tasks"]);

    r.git(&["checkout", "-q", "-b", "feature"]);
    assert_eq!(code(&r.ank("agent-a2@ank", &["claim", held])), 0);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "claim on a branch"]);
    r.git(&["checkout", "-q", "main"]);
    assert!(
        r.task_text(held).contains("status: open"),
        "the fixture is wrong if main already carries the in_progress"
    );
    assert!(
        r.claim_ref(held).unwrap().contains("holder: agent-a2@ank"),
        "the ref is shared by every checkout, so the claim is live here"
    );

    // The task under refusal is held too, and by a third identity, so that the
    // only thing the hint could name is the one held on the branch.
    assert_eq!(code(&r.ank("agent-a1@ank", &["claim", wanted])), 0);

    let out = r.ank("agent-a3@ank", &["claim", wanted]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(code(&out), 4, "{said}");
    assert!(
        !said.contains(&held[..9]),
        "a task a live claim holds is not another ready task: {said}"
    );
    assert!(
        said.contains("ank context"),
        "with nothing free to offer, the hint is the listing: {said}"
    );

    // Lapsed, the same ref stops covering anything: the candidate comes back,
    // and following the hint is a claim that succeeds.
    r.expire_claim(held);
    let out = r.ank("agent-a3@ank", &["claim", wanted]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(code(&out), 4, "{said}");
    assert!(
        said.contains(held),
        "a lapsed claim covers nothing, and pickup after expiry is legal: {said}"
    );
    assert!(
        said.contains("another ready task"),
        "and it is offered as one: {said}"
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
/// a disagreement *between* them: the listing is read fastest and checked
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

    // And the listing still states the three globals of §4, unqualified: the
    // exception belongs on the page of the verb that makes it.
    let listing = stdout(&ank_command().arg("help").output().unwrap());
    assert!(
        listing.contains("global: --json --quiet --repo"),
        "{listing}"
    );
}

/// Every refusal the table declares is on the page of the verb that declares
/// it, with its code, and a verb declaring none says nothing about refusals.
///
/// §9 gives `ank help <verb>` the job of carrying "the state conditions on which
/// it refuses, each with its exit code", and until TASK-106dccc7f71c seven verbs
/// declared none, so nothing connected the field to the rendering for most of
/// the table. This walks the whole table rather than a sample: a row added to
/// any verb is on its page or this is red, which is what makes the declaration
/// reach a caller instead of only the JSON.
///
/// Through the binary, because the claim is about what a caller reads. A unit
/// test on the renderer would pass over a page the process never printed.
#[test]
fn every_declared_refusal_is_printed_on_the_page_that_declares_it() {
    for spec in ank_contract::COMMANDS {
        let page = stdout(&ank_command().args(["help", spec.name]).output().unwrap());
        // `refuses:` is the last block of the page, so everything from it to the
        // end is the block. Taken as a whole because a row wraps onto a
        // continuation line, and a per-line search would miss a long one.
        let block: String = page
            .lines()
            .skip_while(|l| !l.trim_start().starts_with("refuses:"))
            .collect::<Vec<_>>()
            .join(" ");

        if spec.refuses.is_empty() {
            assert!(
                block.is_empty(),
                "`ank help {}` prints a refusals block for a verb that declares none:
{page}",
                spec.name
            );
            continue;
        }

        for row in spec.refuses {
            assert!(
                block.contains(row.when),
                "`ank help {}` does not state a refusal the table declares:
                 declared: {}
page:
{page}",
                spec.name,
                row.when
            );
            assert!(
                block.contains(&format!("({})", row.code.code())),
                "`ank help {}` states {:?} without the code a caller reacts to:
{page}",
                spec.name,
                row.when
            );
        }
    }
}

/// The seven readers exit with the codes their pages now declare, and `status`
/// answers where the other six refuse.
///
/// This is the measurement TASK-106dccc7f71c was opened for, kept as a test so
/// it stays true. Six of the seven refused on states they declared nowhere; the
/// declaration is now in the table, and what binds the two is asserting the
/// observed code **against the row**, never against a number written here. A
/// row whose code is corrected and whose behaviour is not turns this red.
///
/// Through the binary, because an exit code exists only once there is a process
/// (§4).
#[test]
fn the_readers_exit_with_the_codes_their_pages_declare() {
    let r = Repo::new();

    // The invocation, and the row it is the measurement of. `../outside` climbs
    // above the root on every platform: `normalize_path` pops nothing and
    // answers `None`, where an absolute path would have to be spelled twice.
    let cases: &[(&str, &[&str], &str)] = &[
        (
            "context",
            &["context", "../outside"],
            "the path names nothing inside this repository",
        ),
        (
            "context",
            &["context", "--limit", "soon"],
            "--limit is not a number",
        ),
        (
            "find",
            &["find", "--type", "nope"],
            "--type names a kind the registry does not declare",
        ),
        (
            "find",
            &["find", "--scope", "../outside"],
            "--scope names nothing inside this repository",
        ),
        (
            "review",
            &["review", "../outside"],
            "the path names nothing inside this repository",
        ),
        (
            "graph",
            &["graph", "../outside"],
            "the path names nothing inside this repository",
        ),
        (
            "scope",
            &["scope"],
            "no path given, and this verb answers about one",
        ),
        (
            "scope",
            &["scope", "../outside"],
            "the path names nothing inside this repository",
        ),
        (
            "check",
            &["check", "../outside"],
            "the path names nothing inside this repository",
        ),
    ];

    for (verb, args, when) in cases {
        let declared = ank_contract::spec_of(verb)
            .unwrap_or_else(|| panic!("no verb named {verb}"))
            .refuses
            .iter()
            .find(|row| row.when == *when)
            .unwrap_or_else(|| panic!("`{verb}` declares no refusal reading {when:?}"))
            .code
            .code();
        let out = r.ank(AGENT, args);
        assert_eq!(
            code(&out),
            declared,
            "{args:?} does not exit with the code `ank help {verb}` promises for              {when:?}: {}",
            stderr(&out)
        );
    }

    // The other half of the comparison, and the reason `status` keeps an empty
    // array: it refuses on nothing, and a `--status` value naming no status is a
    // listing of nothing rather than a refusal.
    for args in [&["status"][..], &["find", "--status", "nope"][..]] {
        let out = r.ank(AGENT, args);
        assert_eq!(
            code(&out),
            0,
            "{args:?} refuses where the table declares no refusal: {}",
            stderr(&out)
        );
    }
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

const REMOVED_SCOPE: &str = "TASK-00000000de1e";
const REMOVED_TREE: &str = "TASK-00000000d132";
const NEVER_SCOPE: &str = "TASK-00000000f00d";

/// The same `done` seed with a scope of its own, which is the only field a
/// dead-scope fixture varies.
fn seed_done_scoped(r: &Repo, id: &str, scope: &str) {
    seed_done(r, id, "  - type: commit\n    ref: abc1234\n");
    let text = r
        .task_text(id)
        .replace("  - src/**", &format!("  - {scope}"));
    std::fs::write(r.0.join(".ank/entities").join(format!("{id}.md")), text).unwrap();
}

/// A dead scope whose path git records as deleted is a signal naming the commit
/// that removed it; a death git cannot name at all stays a fault
/// (TASK-ec579d3a566e).
///
/// Both halves, because only the pair says anything. A check that stopped
/// checking would pass the first assertion and fail the second, and the corpus
/// this exists for is one where the second case is the whole point: a path git
/// never knew leaves the reader with nothing, which is what the fault is for.
///
/// Through the binary, because the criterion is about what the process prints
/// and what it exits with, and because the exit code is the thing a pipeline
/// reads.
#[test]
fn a_scope_deleted_by_a_commit_is_a_signal_and_a_scope_git_never_knew_is_a_fault() {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/lib.rs"), "// x\n").unwrap();
    std::fs::write(r.0.join("src/gone.rs"), "pub fn gone() {}\n").unwrap();
    seed_done_scoped(&r, REMOVED_SCOPE, "src/gone.rs");
    // And the glob, which is the shape the real instance has: a task that put a
    // directory there, and a later commit that removed the directory. git has no
    // answer for "where did `tools/**` go", so this is asked about the literal
    // prefix and is a different path through the code, not a second spelling of
    // the same one.
    std::fs::create_dir_all(r.0.join("tools")).unwrap();
    std::fs::write(r.0.join("tools/hook.sh"), "#!/bin/sh\n").unwrap();
    seed_done_scoped(&r, REMOVED_TREE, "tools/**");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    // The commit that kills both scopes, and it deletes and nothing else: an
    // addition alongside it could be paired by rename detection, and the finding
    // under test would be the rename one.
    std::fs::remove_file(r.0.join("src/gone.rs")).unwrap();
    std::fs::remove_dir_all(r.0.join("tools")).unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "remove the file"]);
    let sha: String = r.head().chars().take(12).collect();

    let out = r.ank("claude-code@ank", &["check"]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(
        code(&out),
        0,
        "a death git can name is a signal, and exit 8 here is a fault no verb \
         clears: {said}"
    );
    let reported: Vec<&str> = said
        .lines()
        .filter(|l| l.contains("dead scope 'src/gone.rs'"))
        .collect();
    assert_eq!(reported.len(), 1, "exactly one finding names it:\n{said}");
    assert!(
        reported[0].starts_with(&format!("signal: {REMOVED_SCOPE}:")),
        "a deletion git records lowers the severity: {said}"
    );
    assert!(
        said.contains(&format!("git records src/gone.rs deleted in {sha}")),
        "the note names the commit that removed it, or the reader still has \
         nothing: {said}"
    );
    let tree: Vec<&str> = said
        .lines()
        .filter(|l| l.contains("dead scope 'tools/**'"))
        .collect();
    assert_eq!(tree.len(), 1, "exactly one finding names it:\n{said}");
    assert!(
        tree[0].starts_with(&format!("signal: {REMOVED_TREE}:")),
        "a glob whose files git records as deleted is lowered too: {said}"
    );
    assert!(
        said.contains(&format!(
            "git records the files tools/** matched deleted in {sha}"
        )),
        "and its note names the same commit: {said}"
    );

    // The half that must not move. A path git has nothing to say about is where
    // the reader is left with nothing, and lowering that too would be a check
    // that stopped checking.
    seed_done_scoped(&r, NEVER_SCOPE, "src/never_written.rs");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "a scope naming a path that never existed"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(
        code(&out),
        8,
        "a death git cannot name is still a fault: {said}"
    );
    assert!(
        said.contains(&format!(
            "error: {NEVER_SCOPE}: dead scope 'src/never_written.rs'"
        )),
        "the unexplained death is the fault: {said}"
    );
    assert!(
        said.contains(&format!(
            "signal: {REMOVED_SCOPE}: dead scope 'src/gone.rs'"
        )),
        "and the explained one is still a signal: {said}"
    );
}

const DETACHED: &str = "TASK-00000000d1ed";
const LIVE_ANCHOR: &str = "TASK-00000000a11e";

/// A `commit:` proof is validated once, when `done` writes it, and the routine
/// this project prescribes detaches it in silence (§4).
///
/// The fixture is that routine run for real: work on a branch, the default
/// branch moves, rebase. The sha `done` checked against git is replaced, the
/// recorded reference names a commit no ref reaches any more, and nothing in
/// the corpus says so.
///
/// The negative half carries the weight. A signal that fired on the live
/// anchor too would be reporting every finished task in the corpus, which is a
/// line readers learn to skip rather than a finding.
#[test]
fn check_reports_a_commit_proof_a_rebase_detached_and_spares_a_live_one() {
    let r = Repo::new();
    // The seeded scope has to match something tracked, or a dead-scope fault
    // fires and the exit code under test stops being about the proof.
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/lib.rs"), "// x\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    r.git(&["checkout", "-q", "-b", "feature"]);
    std::fs::write(r.0.join("src/work.rs"), "// work\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "work"]);
    let anchored_on = r.head();
    r.git(&["checkout", "-q", "main"]);
    std::fs::write(r.0.join("src/other.rs"), "// other\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "other"]);
    let live = r.head();
    r.git(&["checkout", "-q", "feature"]);
    r.git(&["rebase", "-q", "main"]);
    // The rebase is the fixture, so it is asserted and not assumed: a rebase
    // that replayed nothing would leave every assertion below passing for the
    // wrong reason.
    assert_ne!(
        r.head(),
        anchored_on,
        "the rebase must have replaced the commit the proof names"
    );

    seed_done(
        &r,
        DETACHED,
        &format!("  - type: commit\n    ref: {anchored_on}\n"),
    );
    seed_done(
        &r,
        LIVE_ANCHOR,
        &format!("  - type: commit\n    ref: {live}\n"),
    );

    let out = r.ank("claude-code@ank", &["check"]);
    let said = format!("{}{}", stdout(&out), stderr(&out));

    // A signal and never a fault: an unreachable commit here is a shallow
    // clone, a branch never fetched or a rebase on somebody else's machine.
    assert_eq!(
        code(&out),
        0,
        "the shape of a clone is not a defect in the corpus: {said}"
    );

    let reported: Vec<&str> = said
        .lines()
        .filter(|l| l.contains("no commit reachable"))
        .collect();
    assert_eq!(
        reported.len(),
        1,
        "exactly the detached proof should be named:\n{said}"
    );
    assert!(reported[0].contains(DETACHED), "it names the task: {said}");
    assert!(
        reported[0].contains(&anchored_on),
        "it names the reference: {said}"
    );
    assert!(
        reported[0].contains(&format!("ank attest {DETACHED}")),
        "a finding names the exact command that clears it: {said}"
    );
    assert!(
        !said.contains(&live),
        "a proof git still reaches is anchored, and nothing is owed about it: {said}"
    );
    // Reported and never repaired: which commit carries the work now is a
    // judgement, and appending a proof is the only legal post-`done` write.
    assert!(
        r.task_text(DETACHED).contains(&anchored_on),
        "the dead entry stays: a proof list is append-only"
    );
}

/// A clone that cannot see is not a corpus that is wrong (§4).
///
/// A depth-1 clone reaches almost no history, so a check that asked the
/// question there would report every commit proof in the corpus at once — the
/// volume failure §4 keeps legislating against, and the reading this project
/// has already settled twice for a dead scope a truncated history cannot
/// explain (TASK-03eaa26bddd1, TASK-2ce5554d6ed0).
///
/// Two clones of one history, because the claim is a *difference* between
/// clones and no single one can show it.
#[test]
fn a_shallow_clone_reports_no_detached_commit_proof() {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/lib.rs"), "// x\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    let early = r.head();

    // The corpus anchors on that first commit and then main moves past it, so
    // a truncated clone carries the proof and not the commit it names.
    seed_done(
        &r,
        DETACHED,
        &format!("  - type: commit\n    ref: {early}\n"),
    );
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "corpus"]);
    std::fs::write(r.0.join("src/other.rs"), "// other\n").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "other"]);

    // Whole: git has the commit, the proof is anchored, and there is nothing
    // to say. Without this half, the silence below would prove only that the
    // signal never fires.
    let whole = clone_of(&r, None);
    let out = r.ank_at("claude-code@ank", &["check"], &whole);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(code(&out), 0, "{said}");
    assert!(
        !said.contains("no commit reachable"),
        "a whole clone reaches the commit the proof names:\n{said}"
    );

    let shallow = clone_of(&r, Some(1));
    // The truncation is only a fixture if the clone genuinely cannot see it:
    // the object is absent there, so a check that asked would have this proof
    // — and every other commit proof in the corpus — to report at once.
    let seen = git_command(&shallow)
        .args(["cat-file", "-e", &format!("{early}^{{commit}}")])
        .output()
        .unwrap();
    assert!(
        !seen.status.success(),
        "a depth-1 clone must not carry {early}, or this fixture asks nothing"
    );

    let out = r.ank_at("claude-code@ank", &["check"], &shallow);
    let said = format!("{}{}", stdout(&out), stderr(&out));
    assert_eq!(
        code(&out),
        0,
        "the shape of a clone is not a defect in the corpus: {said}"
    );
    assert!(
        !said.contains("no commit reachable"),
        "a clone that cannot see must accuse nothing:\n{said}"
    );
    assert!(
        !said.contains(&early),
        "a proof it cannot see is not a proof it may name:\n{said}"
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
    // The record of an amend is machinery since TASK-3c12e0ced2c0, so it names
    // the fields alone and the `amended:` opening is gone. What it gained is
    // the version transition and the hash of the state replaced, which is what
    // makes the entry account for the write (ADR-16813b3bcf37).
    assert!(
        r.log_text(ID).contains("+blocked_by TASK-000000000002"),
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
        r.log_text(ID)
            .contains("done_criteria (version 1 to 2, replaced "),
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
/// grouped (ADR-91b77f036884), so an empty line is a boundary *between* groups
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
    // (ADR-91b77f036884). These four headings sorted callers -- agent loop,
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
        "the audience line is what ADR-91b77f036884 removes, and it is the \
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
    // The envelope leads, and the verbs follow it (ADR-6fd69efb629c).
    assert!(text.starts_with("{\"contract\":1,\"verbs\":["), "{text}");
    assert!(text.trim_end().ends_with("]}"), "{text}");
    assert!(text.contains("\"name\":\"claim\""), "{text}");
    assert!(
        !text.contains("audience"),
        "a grouping by caller survived into the scripted output:\n{text}"
    );
}

/// Every verb of the table appears exactly once in the grouped listing, under a
/// heading, and in the table's order inside it (ADR-91b77f036884).
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
/// listing prints (ADR-91b77f036884).
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
        // ADR-91b77f036884 pulled down was built out of exactly these words.
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
// Dead constraints in the prose (ADR-91b77f036884)
// ---------------------------------------------------------------------------

/// The workspace root: two levels above this crate's manifest.
///
/// Derived rather than configured, so a crate added to the workspace is walked
/// by the guard below without anybody wiring it in — which is the same property
/// the dead set gains from being asked of the corpus.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/<name> sits two levels under the workspace root")
        .to_path_buf()
}

/// Every source file and manifest of every crate in the workspace.
fn workspace_sources(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.join("crates")];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Build output is not source, and it is large.
                if path.file_name().and_then(|n| n.to_str()) == Some("target") {
                    continue;
                }
                stack.push(path);
                continue;
            }
            if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("rs") | Some("toml")
            ) {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// The number a `--json` document carries under `name`.
fn json_number(text: &str, name: &str) -> u64 {
    let at = text
        .find(name)
        .unwrap_or_else(|| panic!("{name} is in the document: {text}"));
    text[at + name.len()..]
        .chars()
        .take_while(char::is_ascii_digit)
        .collect::<String>()
        .parse()
        .expect("a number")
}

/// The identifiers of every superseded document in a corpus, asked of the
/// binary (ADR-01b6dd05f0db: `.ank/` is reached only through the CLI).
///
/// **Derived and never transcribed.** A list in this file protects the cases
/// somebody remembered and no fourth: superseding an entity does not add to it,
/// so the guard goes quiet on exactly the citation that has just gone stale.
/// The corpus already knows what is superseded, and a test that asks it needs
/// nobody to remember.
fn superseded_ids(repo: &Path) -> Vec<String> {
    let out = ank_command()
        .args(["find", "--status", "superseded", "--json"])
        .arg("--repo")
        .arg(repo)
        .current_dir(std::env::temp_dir())
        .output()
        .expect("the binary must have been built");
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    let said = stdout(&out);
    // A listing the budget cut would be read as a shorter corpus, and the guard
    // would go quiet on whatever it dropped. The document carries both numbers
    // for exactly this.
    assert_eq!(
        json_number(&said, "\"shown\":"),
        json_number(&said, "\"total\":"),
        "the listing was cut, so the set is not the whole one: {said}"
    );
    let mut ids = Vec::new();
    let mut rest = said.as_str();
    while let Some(at) = rest.find("\"id\":\"") {
        rest = &rest[at + "\"id\":\"".len()..];
        let end = rest.find('"').expect("a closed string");
        ids.push(rest[..end].to_string());
        rest = &rest[end..];
    }
    ids
}

/// Every citation of one of `dead` in `files`, as the line a reader would fix.
fn stale_citations(files: &[PathBuf], dead: &[String]) -> Vec<String> {
    let mut found = Vec::new();
    for path in files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        for (n, line) in text.lines().enumerate() {
            for id in dead {
                if line.contains(id.as_str()) {
                    found.push(format!("{}:{}: {id}", path.display(), n + 1));
                }
            }
        }
    }
    found
}

/// A comment citing a superseded document as the reason for a design is worse
/// than no comment: it hands the next reader a constraint that binds nobody,
/// with the authority of a decision record. Two of them went on asserting a
/// frozen agent surface -- at seven verbs in one, at eight in the other -- in
/// module headers and doc comments, long after the split they protected had been
/// dissolved and while the file around them had already stopped obeying it.
///
/// **This is not a ban on writing history down.** It is a ban on writing it in
/// the source, because `.ank/` is where the corpus keeps it and `ank show
/// <successor>` carries `supersedes:`. A comment says what binds today; the
/// chain behind it is one command away and does not have to be pasted into a
/// module header where nothing will ever notice it going stale.
///
/// **The set is asked of the corpus and the walk covers the workspace**
/// (TASK-7a1c92961465). Both were holes and each on its own was enough. The
/// list used to be three identifiers transcribed by whoever last remembered, so
/// superseding an entity protected nothing new; and the walk used to be this
/// crate's `src` and `tests`, so `ank-contract`, `ank-core` and `ank-mcp` were
/// never read. The citation that prompted this was in one of the three, which is
/// not a coincidence: the guard had never looked there, so nothing there had
/// ever been kept honest. Re-pointing what the wider walk then found came to a
/// hundred and six citations across eighteen files.
#[test]
fn no_superseded_document_is_cited_in_the_workspace() {
    let root = workspace_root();
    let dead = superseded_ids(&root);
    // A corpus that answered with nothing would make this pass forever, which is
    // the one way a test like this fails at being a test.
    assert!(dead.len() >= 30, "only {} superseded documents", dead.len());

    let files = workspace_sources(&root);
    // The same guard for the walk, and the number rose with it: this crate's
    // `src` and `tests` alone were eight.
    assert!(
        files.len() >= 40,
        "only {} source files walked",
        files.len()
    );
    assert!(
        files
            .iter()
            .any(|p| p.components().any(|c| c.as_os_str() == "ank-core")),
        "the walk never left ank-cli: {files:?}"
    );

    let stale = stale_citations(&files, &dead);
    assert!(
        stale.is_empty(),
        "these cite a superseded document: name the one that binds today, or \
         drop the citation and leave the history to `ank show`:\n{}",
        stale.join("\n")
    );
}

/// The falsification, and it is the hole the task was filed for: a citation
/// planted in a crate that is not this one is found.
///
/// The identifier comes from the corpus rather than from a literal, because a
/// literal written here would be a citation in this very file and the guard
/// above would fail on it. That is the same reason the old list was assembled
/// out of `concat!` fragments, arrived at from the other end.
#[test]
fn the_walk_reaches_a_crate_that_is_not_this_one() {
    let root = workspace_root();
    let dead = superseded_ids(&root);
    let planted = dead.first().expect("the corpus has superseded documents");

    let tree = std::env::temp_dir().join(format!("ank-guard-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tree);
    for crate_name in ["ank-cli", "ank-other"] {
        std::fs::create_dir_all(tree.join("crates").join(crate_name).join("src")).unwrap();
    }
    std::fs::write(
        tree.join("crates/ank-cli/src/lib.rs"),
        "// nothing stale here\n",
    )
    .unwrap();
    std::fs::write(
        tree.join("crates/ank-other/src/lib.rs"),
        format!("//! A design justified by {planted}.\n"),
    )
    .unwrap();

    let files = workspace_sources(&tree);
    assert_eq!(files.len(), 2, "{files:?}");
    let stale = stale_citations(&files, &dead);
    assert_eq!(stale.len(), 1, "{stale:?}");
    assert!(
        stale[0].contains("ank-other") && stale[0].contains(planted),
        "the finding names the file and the identifier: {}",
        stale[0]
    );

    let _ = std::fs::remove_dir_all(&tree);
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
        "spec" => "SPEC-",
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

/// The document a spec fixture carries, written so that every line of it is a
/// string no other output could produce. The negative assertion below is only
/// worth as much as that: a body made of ordinary words would let a page pass
/// this test by coincidence.
const SPEC_BODY_LINES: [&str; 3] = [
    "Quoting-this-line-would-mean-the-budget-is-gone.",
    "Section-two-of-a-document-nobody-should-see-in-context.",
    "And-a-third-line-for-good-measure.",
];

/// The whole criterion of TASK-3e68786fa443, driven through the built binary:
/// `new spec` creates, `show` reads whole, `find --type spec` lists, `context`
/// names — and **no line of the document reaches `context` in either mode**.
///
/// The negative half is the point, and it is not the same assertion as the
/// positive one read backwards. `context` names a spec exactly as it names an
/// ADR, one line of id and title, and unlike an ADR there is no execution mode
/// that serves the body either: there is no `constraint` field to serve, and the
/// body is the document (§3, §5). The specification this repository stores is
/// over two hundred thousand bytes against a budget of eight thousand
/// characters, so a single line getting through is the budget gone and the mode
/// broken — which a test asserting only that the title appears would never
/// notice.
#[test]
fn a_spec_is_created_read_listed_and_named_but_never_quoted() {
    let r = Repo::new();
    let body = SPEC_BODY_LINES.join("\n");

    // Created through the surface, with the body on stdin: a document is what
    // `--body -` exists for, and a spec is the kind that has nothing else.
    let out = r.ank_stdin(
        "marie@laptop",
        &[
            "new",
            "spec",
            "--title",
            "The session protocol",
            // What the document governs, never where it lives (§3).
            "--scope",
            "src/auth/**",
            "--body",
            "-",
        ],
        &body,
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let id = stdout(&out)
        .split_whitespace()
        .nth(1)
        .expect("created <id> <title>")
        .to_string();
    assert!(id.starts_with("SPEC-"), "the id carries the kind: {id}");

    let files = entity_files(&r, "spec");
    assert_eq!(files.len(), 1, "exactly one spec: {files:?}");
    let text = std::fs::read_to_string(r.0.join(".ank/entities").join(&files[0])).unwrap();
    assert!(text.contains("type: spec"), "{text}");
    // The absence that justifies the kind: a spec describes, an ADR binds.
    assert!(!text.contains("constraint:"), "{text}");
    // Never born accepted and never born anchored, exactly as an ADR is not:
    // ratification is a signed commit produced by `accept`.
    assert!(text.contains("status: proposed"), "{text}");
    assert!(!text.contains("ratified:"), "{text}");
    assert!(text.contains("  - src/auth/**"), "{text}");

    // `show` is the reader, byte for byte. Not "contains the body": the whole
    // file, exactly, which is what the split with `context` rests on.
    let out = r.ank("marie@laptop", &["show", &id]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert_eq!(stdout(&out), text, "show prints the entity whole");

    // Listed by its own kind, and not by another's.
    let out = r.ank("marie@laptop", &["find", "--type", "spec"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("The session protocol") && stdout(&out).contains("SPEC-"),
        "{}",
        stdout(&out)
    );
    let out = r.ank("marie@laptop", &["find", "--type", "task"]);
    assert!(
        !stdout(&out).contains("The session protocol"),
        "a spec is not a task: {}",
        stdout(&out)
    );
    // And the refusal names the kind rather than the two it used to know.
    let out = r.ank("marie@laptop", &["find", "--type", "epic"]);
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(stderr(&out).contains("spec"), "{}", stderr(&out));

    // Orientation: named beside the constraints, and not one line of the
    // document with it.
    let out = r.ank("marie@laptop", &["context", "src/auth/"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let oriented = stdout(&out);
    assert!(
        oriented.contains("SPECIFICATIONS") && oriented.contains("The session protocol"),
        "named: {oriented}"
    );
    assert_no_spec_body(&oriented, "orientation");

    // Execution: HEAD is set, the perimeter is the task's, and the spec is
    // named there too — still without a line of its body, because there is no
    // mode that serves one.
    r.seed_task_scoped("TASK-000000000001", "src/auth/**");
    let out = r.ank("marie@laptop", &["claim", "TASK-000000000001"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let out = r.ank("marie@laptop", &["context"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let executing = stdout(&out);
    assert!(
        executing.contains("SPECIFICATIONS") && executing.contains("The session protocol"),
        "named in execution too: {executing}"
    );
    assert_no_spec_body(&executing, "execution");

    // The machine surface answers to the same rule: a `--json` caller is
    // exactly the one that would pipe a document into an agent's context
    // without noticing.
    let out = r.ank("marie@laptop", &["context", "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let json = stdout(&out);
    assert!(json.contains("\"specs\":[{"), "{json}");
    assert!(json.contains("The session protocol"), "{json}");
    assert_no_spec_body(&json, "execution --json");

    // And `help` lists the kind wherever it lists task and adr.
    let out = r.ank("marie@laptop", &["help"]);
    assert!(
        stdout(&out).contains("ank new <task|adr|spec>"),
        "{}",
        stdout(&out)
    );
}

/// No line of the document, in `where`. Every line, not the first: a renderer
/// that cut the body after one line would still be quoting it.
fn assert_no_spec_body(page: &str, mode: &str) {
    for line in SPEC_BODY_LINES {
        assert!(
            !page.contains(line),
            "{mode} quotes a line of the spec body: {line}\n{page}"
        );
    }
}

/// The file of an entity of any kind, read off the disk rather than through the
/// tool: what has to be true is the state of the corpus.
fn entity_text(r: &Repo, id: &str) -> String {
    std::fs::read_to_string(r.0.join(".ank/entities").join(format!("{id}.md"))).unwrap()
}

/// The other half of the lifecycle, the one that costs a signature, driven
/// through the built binary: `accept`, `amend` and `check` on a spec.
///
/// **The anchor is the one place a spec differs from an ADR** (§3). `ratified`
/// holds the hash of the body and the scope, because no narrower field carries
/// the authority — so the commit key says `body+scope` rather than naming a
/// `constraint` the kind does not declare, and a body that moves afterwards is
/// altered on the same terms and by the same walk.
///
/// **And the consequence of that is deliberate, not an exception.** An accepted
/// spec's scope is refused by `amend` exactly as an accepted ADR's is, with the
/// same code 6 and a succession in its own kind's words: revising an accepted
/// specification is a supersession, and a working draft stays `proposed` while
/// it is one.
///
/// The negative at the end is the half that is not the positive read backwards.
/// An altered ADR stops being injected, because injecting a rewritten rule would
/// let whoever edits the file rewrite what every agent works under. A spec binds
/// nobody, so **there is no constraint to suspend**: `context` names it after
/// the alteration exactly as it named it before, and quotes no more of it.
#[test]
fn a_spec_is_ratified_amended_and_checked_like_an_adr() {
    let r = Repo::new();
    r.enable_signing();
    declare_signing_key(&r);
    // A scope matching no file is a fault of its own, and it would redden
    // `check` before anything under test ran.
    for (dir, file) in [
        ("src/auth", "src/auth/session.rs"),
        ("src/session", "src/session/store.rs"),
    ] {
        std::fs::create_dir_all(r.0.join(dir)).unwrap();
        std::fs::write(r.0.join(file), "fn main() {}\n").unwrap();
    }

    let body = SPEC_BODY_LINES.join("\n");
    let out = r.ank_stdin(
        "marie@laptop",
        &[
            "new",
            "spec",
            "--title",
            "The session protocol",
            "--scope",
            "src/auth/**",
            "--body",
            "-",
        ],
        &body,
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let id = stdout(&out)
        .split_whitespace()
        .nth(1)
        .expect("created <id> <title>")
        .to_string();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    // `proposed`: nothing anchors the scope, so `amend` reaches it — the same
    // rule an ADR's scope is under, and the reason the verb refused a spec by
    // name for exactly as long as the kind could not be ratified.
    let out = r.ank("marie@laptop", &["amend", &id, "--scope", "src/session/**"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        entity_text(&r, &id).contains("  - src/session/**"),
        "{}",
        entity_text(&r, &id)
    );

    // Silent before, or the fault at the end would prove nothing.
    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));

    let out = r.ank("marie@laptop", &["accept", &id]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("accepted") && stdout(&out).contains(&id),
        "{}",
        stdout(&out)
    );
    let text = entity_text(&r, &id);
    assert!(text.contains("status: accepted"), "{text}");
    let anchor = text
        .lines()
        .find_map(|l| l.strip_prefix("ratified: "))
        .expect("accept writes the anchor into the file")
        .trim()
        .to_string();

    // The commit is the anchor that counts, and its key names what was hashed.
    // `constraint+scope` over a kind that declares no constraint would name a
    // field the file does not have.
    let message = r.git(&["log", "-1", "--format=%B"]);
    assert!(message.starts_with(&format!("ratify {id}")), "{message}");
    assert!(
        message.contains(&format!("body+scope: {anchor}")),
        "{message}"
    );
    assert!(!message.contains("constraint+scope"), "{message}");

    // Ratified and untouched. Exit 0 here is also what says the signature was
    // judged and trusted: `.ank/allowed_signers` declares the key, so an
    // unsigned or undeclared ratification would be a fault rather than silence.
    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));

    // And the scope stops being amendable, in the accepted ADR's words and with
    // its code.
    let out = r.ank("marie@laptop", &["amend", &id, "--scope", "docs/**"]);
    assert_eq!(code(&out), 6, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("anchored in the ratification commit"),
        "{}",
        stderr(&out)
    );
    assert!(
        stderr(&out).contains("ank new spec --supersedes"),
        "the succession is named in the kind's own words: {}",
        stderr(&out)
    );

    // `edit` is the paved road for everything else, and §4 says a change to a
    // frozen field is refused there by naming the command that legally performs
    // it. What the anchor covers is where the two kinds part: the body of an
    // accepted ADR stays editable, and a spec's is the thing that was ratified.
    let before = entity_text(&r, &id);
    let editor = r.editor_saving(&before.replace(SPEC_BODY_LINES[0], "A-rewritten-first-line."));
    let out = r.ank_edit("marie@laptop", &["edit", &id], Some(&editor));
    assert_eq!(code(&out), 6, "{}", stderr(&out));
    assert!(stderr(&out).contains("ratified"), "{}", stderr(&out));
    assert_eq!(entity_text(&r, &id), before, "the entity is untouched");

    // And the refusal reaches no further than the anchor: a title is neither
    // the body nor the scope.
    let editor =
        r.editor_saving(&before.replace("The session protocol", "The session protocol v2"));
    let out = r.ank_edit("marie@laptop", &["edit", &id], Some(&editor));
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    // The document moves in the file, and no second signature follows it.
    let moved = entity_text(&r, &id).replace(SPEC_BODY_LINES[1], "A-line-nobody-ever-ratified.");
    std::fs::write(r.0.join(".ank/entities").join(format!("{id}.md")), moved).unwrap();

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(
        code(&out),
        8,
        "a divergence is a fault: {}{}",
        stdout(&out),
        stderr(&out)
    );
    assert!(
        stdout(&out).contains("altered since ratification"),
        "{}",
        stdout(&out)
    );
    // The suspension an ADR gets exists to stop an edited rule from binding, and
    // a spec binds nobody.
    assert!(
        !stdout(&out).contains("no longer injected"),
        "a spec has no constraint to suspend: {}",
        stdout(&out)
    );
    let out = r.ank("claude-code@ank", &["context", "src/auth/"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("SPECIFICATIONS") && stdout(&out).contains("The session protocol"),
        "an altered spec is still named: {}",
        stdout(&out)
    );
    assert_no_spec_body(&stdout(&out), "altered");
}

/// The two findings `check` owes a spec beyond the freeze itself.
///
/// **Accepted with no anchor is a signal**, the bootstrap exception §3 states
/// for an ADR and states for the same reason here: a document promoted by
/// editing the file, or one predating the verb, must not condemn the corpus and
/// block every `done` behind it.
///
/// **A dead scope names the repair**, and which repair depends on the status —
/// which is the whole point of `repair` branching rather than printing one
/// command. The draft is amendable and the command says `amend`; the accepted
/// one is not, and naming a command that exits 6 would be worse than naming
/// none, so it names the supersession instead.
#[test]
fn a_spec_carries_the_bootstrap_signal_and_names_the_repair_for_a_dead_scope() {
    const BOOTSTRAP: &str = "SPEC-0000000000ab";
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src/auth")).unwrap();
    std::fs::write(r.0.join("src/auth/session.rs"), "fn main() {}\n").unwrap();

    let out = r.ank_stdin(
        "marie@laptop",
        &[
            "new",
            "spec",
            "--title",
            "A draft",
            "--scope",
            "src/auth/session.rs",
            "--body",
            "-",
        ],
        "The draft.\n",
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let draft = stdout(&out)
        .split_whitespace()
        .nth(1)
        .expect("created <id> <title>")
        .to_string();

    // Accepted and never anchored: the state no verb produces and every
    // bootstrap corpus is in. Written by hand for that reason.
    std::fs::write(
        r.0.join(".ank/entities").join(format!("{BOOTSTRAP}.md")),
        format!(
            "---\nid: {BOOTSTRAP}\ntype: spec\nslug: bootstrap\n\
             title: A specification accepted by hand\ncreated: 2026-07-20T00:00:00Z\n\
             status: accepted\nscope:\n  - src/auth/session.rs\nschema: 3\nversion: 1\n\
             ---\n\nThe document.\n"
        ),
    )
    .unwrap();

    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    std::fs::create_dir_all(r.0.join("src/session")).unwrap();
    r.git(&["mv", "src/auth/session.rs", "src/session/store.rs"]);
    r.git(&["commit", "-qm", "the file moves"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let said = stdout(&out);
    // Both are signals: a dead scope git can explain is not a broken corpus,
    // and a bootstrap anchor is not a violation.
    assert_eq!(code(&out), 0, "{said}{}", stderr(&out));
    assert!(
        said.contains("accepted with no ratification commit"),
        "{said}"
    );
    assert!(
        said.contains("git records src/auth/session.rs renamed to src/session/store.rs"),
        "{said}"
    );
    assert!(
        said.contains(&format!(
            "ank amend {draft} --drop-scope \"src/auth/session.rs\" \
             --scope \"src/session/store.rs\""
        )),
        "the draft is amendable, and the repair says so:\n{said}"
    );
    assert!(
        said.contains(&format!(
            "ank new spec --supersedes {BOOTSTRAP} --title \"<t>\" \
             --scope \"src/session/store.rs\""
        )),
        "the accepted one is a supersession, and naming an amend would refuse:\n{said}"
    );
    assert!(
        !said.contains(&format!("ank amend {BOOTSTRAP}")),
        "the finding names a command that would refuse:\n{said}"
    );
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
// Color: the guarantee is negative (§4, ADR-0c8ab846d262)
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
/// either stream is the defect ADR-0c8ab846d262 forbids, and `--json` is
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
        // A whole id and not a prefix: `--drop-reference` matches what the
        // entity stores rather than resolving against the corpus, which is what
        // makes dropping a citation to a deleted document possible at all.
        "--reference" | "--drop-reference" => "SPEC-000000000001",
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
    assert!(
        logged.starts_with("logged LOG-") && logged.contains(" on TASK-"),
        "an entry is an entity, and the line names the one it wrote: {logged:?}"
    );

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
        "peers.<name>",
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

// ---------------------------------------------------------------------------
// Corpora federate by reading (§7, ADR-a1de673043b4, TASK-13e802e46050)
// ---------------------------------------------------------------------------

/// Every file under `dir`, by path relative to it, with its bytes.
///
/// The whole subtree and not just `.ank/`: the assertion this exists for is that
/// reading a peer writes **nothing** there, and an index quietly created beside
/// the entities, a config rewritten, or a ref moved would each be a different
/// way of failing the same promise. `.git/` is included for exactly that reason
/// — claims do not cross, and the cheapest way to say so is to notice if one
/// ever did.
fn snapshot(dir: &Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    fn walk(
        base: &Path,
        dir: &Path,
        out: &mut std::collections::BTreeMap<String, Vec<u8>>,
    ) -> std::io::Result<()> {
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                walk(base, &path, out)?;
            } else {
                let key = path
                    .strip_prefix(base)
                    .expect("walked from base")
                    .to_string_lossy()
                    .replace('\\', "/");
                out.insert(key, std::fs::read(&path)?);
            }
        }
        Ok(())
    }
    let mut out = std::collections::BTreeMap::new();
    walk(dir, dir, &mut out).expect("the fixture is readable");
    out
}

/// The path `from` uses to name `to`, relative and with forward slashes.
///
/// Relative because that is the reviewable form: an absolute path is a fact
/// about one machine, which is the failure a declared peer exists to avoid. The
/// fixtures are siblings in the temp directory, so one `..` is the whole of it.
fn sibling_path(to: &Path) -> String {
    format!(
        "../{}",
        to.file_name()
            .expect("a fixture has a name")
            .to_string_lossy()
    )
}

/// The read half of federation, both halves of it asserted (TASK-13e802e46050).
///
/// Two corpora, and the direction is the part worth reading slowly. `reader`
/// declares `home` as a peer, which is what lets it read that corpus at all.
/// The ADR lives in `home` and nowhere else, and it declares that it binds a
/// peer through its own scope — `app:src/**`, where `app` is a name `home`
/// declares and resolves to `reader`. So the constraint has exactly one home,
/// nothing is copied, and it is served where it binds.
///
/// The second assertion is the one that makes the task's title true and the one
/// that will still be right in a year: after `reader` has answered, every byte
/// under `home` is the byte that was there before.
#[test]
fn a_declared_peer_is_read_and_never_written() {
    let home = Repo::new();
    let reader = Repo::new();

    // The peer declares the corpus its decision binds. Reviewed here, where the
    // scope that uses the name is written.
    home.set_config(&format!(
        "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\npeers:\n  app: {}\n",
        sibling_path(&reader.0)
    ));
    // Accepted, and written by hand: `accept` signs and commits, which is a
    // different verb's test. What matters here is a binding rule whose scope
    // reaches across.
    std::fs::write(
        home.0.join(".ank/entities/ADR-aaaaaaaaaaaa.md"),
        "---\nid: ADR-aaaaaaaaaaaa\ntype: adr\nslug: one-home\n\
         title: The session cookie is opaque on both sides\n\
         created: 2026-08-01T00:00:00Z\nstatus: accepted\nscope:\n  - app:src/**\n\
         constraint: |\n  A session identifier is opaque and carries no claims.\n\
         schema: 1\nversion: 1\n---\n\nWhy.\n",
    )
    .unwrap();

    // The reader declares the corpus it reads. Reviewed here, in the repository
    // that wants to read.
    reader.set_config(&format!(
        "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\npeers:\n  core: {}\n",
        sibling_path(&home.0)
    ));
    reader.seed_task_titled(ID, "Local work");

    let before = snapshot(&home.0);
    let out = reader.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", erred(&out));

    let text = stdout(&out);
    // Served, and named as what it is: the full id, because a short prefix is
    // computed per corpus and means nothing here, and the peer it came from.
    assert!(
        text.contains("ADR-aaaaaaaaaaaa@core"),
        "the peer's constraint was not served: {text}"
    );
    assert!(
        text.contains("The session cookie is opaque on both sides"),
        "{text}"
    );
    // The local corpus still answers about itself.
    assert!(text.contains("Local work"), "{text}");

    // Exactly one home. The reading never produced a copy.
    assert!(
        !reader.0.join(".ank/entities/ADR-aaaaaaaaaaaa.md").exists(),
        "the ADR was copied into the reader's corpus"
    );

    // The assertion the title rests on.
    let after = snapshot(&home.0);
    let changed: Vec<&String> = after
        .keys()
        .chain(before.keys())
        .filter(|k| before.get(*k) != after.get(*k))
        .collect();
    assert!(
        changed.is_empty(),
        "reading the peer modified {changed:?} under {}",
        home.0.display()
    );

    // And `--json` carries the same fact where a suffix has nowhere to go.
    let out = reader.ank("claude-code@ank", &["context", "--json"]);
    assert_eq!(code(&out), 0, "{}", erred(&out));
    assert!(
        stdout(&out).contains("\"home\":\"core\""),
        "{}",
        stdout(&out)
    );
}

/// A scope entry naming a peer the corpus that wrote it does not declare means
/// nothing at all (§7), and the same entry read from a third repository binds
/// nothing there.
#[test]
fn a_peer_name_the_writing_corpus_does_not_declare_binds_nobody() {
    let home = Repo::new();
    let reader = Repo::new();

    // No `peers` at all in the corpus that wrote the scope, so `app` resolves to
    // nothing and the entry is a name with no referent.
    std::fs::write(
        home.0.join(".ank/entities/ADR-bbbbbbbbbbbb.md"),
        "---\nid: ADR-bbbbbbbbbbbb\ntype: adr\nslug: dangling\n\
         title: A rule naming a peer nobody declared\n\
         created: 2026-08-01T00:00:00Z\nstatus: accepted\nscope:\n  - app:src/**\n\
         constraint: |\n  This must not bind anybody.\nschema: 1\nversion: 1\n---\n\nWhy.\n",
    )
    .unwrap();
    reader.set_config(&format!(
        "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\npeers:\n  core: {}\n",
        sibling_path(&home.0)
    ));

    let out = reader.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", erred(&out));
    assert!(
        !stdout(&out).contains("ADR-bbbbbbbbbbbb"),
        "an unresolved peer name bound the reader: {}",
        stdout(&out)
    );
}

/// **Degrade, never fail** (§2): a peer that is not there costs one line and the
/// local answer, the way `status --remote` answers an unreachable remote.
#[test]
fn a_peer_that_cannot_be_read_warns_once_and_answers_locally() {
    let reader = Repo::new();
    reader.set_config(
        "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\npeers:\n  gone: ../not-a-corpus\n",
    );
    reader.seed_task_titled(ID, "Local work");

    let out = reader.ank("claude-code@ank", &["context"]);
    assert_eq!(
        code(&out),
        0,
        "a missing peer must not fail: {}",
        erred(&out)
    );

    let text = stdout(&out);
    assert_eq!(
        text.matches("peer 'gone'").count(),
        1,
        "warned more than once, or not at all: {text}"
    );
    assert!(text.contains("ank config --unset peers.gone"), "{text}");
    // Answered locally, which is the whole of "degrade".
    assert!(text.contains("Local work"), "{text}");
}

/// The key set stays closed: a peer is a key the parser knows, and everything
/// else is still refused by name.
#[test]
fn config_declares_a_peer_and_the_key_set_stays_closed() {
    let r = Repo::new();
    r.set_config(AWKWARD);

    // Declared through the verb, into a file that never carried the key.
    let out = r.ank("claude-code@ank", &["config", "peers.core", "../core"]);
    assert_eq!(code(&out), 0, "{}", erred(&out));
    assert!(stdout(&out).contains("../core"), "{}", stdout(&out));
    assert!(
        r.config_text().contains("core: ../core"),
        "{}",
        r.config_text()
    );
    // Every byte the caller did not name is still there.
    assert!(r.config_text().starts_with(AWKWARD), "{}", r.config_text());

    let out = r.ank("claude-code@ank", &["config", "peers.core"]);
    assert_eq!(stdout(&out).trim(), "../core");

    // A second peer joins the block rather than starting another one.
    let out = r.ank("claude-code@ank", &["config", "peers.web", "../web"]);
    assert_eq!(code(&out), 0, "{}", erred(&out));
    assert_eq!(r.config_text().matches("peers:").count(), 1);

    // And an unknown key is still refused, with nothing written.
    let before = r.config_text();
    let out = r.ank("claude-code@ank", &["config", "peer.core", "../core"]);
    assert_eq!(code(&out), 1);
    assert!(
        erred(&out).contains("unknown key 'peer.core'"),
        "{}",
        erred(&out)
    );
    assert_eq!(r.config_text(), before);

    // A file carrying an unknown key still fails every other verb, which is the
    // strictness this feature added a key to rather than removed.
    r.set_config("schema: 1\npeerz:\n  core: ../core\n");
    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 1);
    assert!(erred(&out).contains("peerz"), "{}", erred(&out));

    // Removing the last peer leaves a mapping the parser reads, not a key with
    // no children.
    r.set_config("schema: 1\nclaim_ttl_max: 2h\npeers:\n  core: ../core\n");
    let out = r.ank("claude-code@ank", &["config", "--unset", "peers.core"]);
    assert_eq!(code(&out), 0, "{}", erred(&out));
    assert!(r.config_text().contains("peers: {}"), "{}", r.config_text());
    let out = r.ank("claude-code@ank", &["context"]);
    assert_eq!(code(&out), 0, "{}", erred(&out));
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
        // and a group heading opens none either (ADR-91b77f036884).
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
const NOT_A_PATH: [&str; 22] = [
    // Entity ids, both of them: what a document rests on is another entity of
    // this corpus, never a file (ADR-c88f99e1c16e). The scope is what names
    // paths on a spec, and it is in GLOB_FLAGS.
    "--reference",
    "--drop-reference",
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

/// A listing counts the open rows a `claim` would refuse, and names `--free`
/// (§4).
///
/// The case is measured, not imagined. A session whose checkout was behind the
/// default branch ran `find -s open`, got thirteen rows of which ten displayed
/// `[finished:… on …]`, and concluded the status filter was broken. Every row
/// was right: `--status` compares the status the file carries, the marker comes
/// from the coordination plane, and the two disagree for exactly as long as a
/// `done` sits on a branch nobody has merged. What was missing was the way out
/// — `--free` already answers the question that reader was asking, and nothing
/// in the listing said so.
///
/// Open tasks alone are counted. A `--status done` listing carries the same
/// marker until `check` prunes the ref, and `--free` keeps no done task, so
/// naming it there would be a hint answering a different question (§7).
#[test]
fn a_listing_counts_the_open_rows_a_claim_would_refuse_and_names_free() {
    let finished = "TASK-a00000000004";
    let held = "TASK-b00000000004";
    let candidate = "TASK-c00000000004";
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task_with(finished, Some("A criterion."), &["ok"]);
    r.seed_task_scoped(held, "crates/**");
    r.seed_task_scoped(candidate, "docs/**");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed tasks"]);

    // Both transitions happen on a branch, so `main` keeps reading `open` for
    // both rows: that is the whole of the case, and a fixture that committed
    // either one onto `main` would be testing nothing.
    r.git(&["checkout", "-q", "-b", "feature"]);
    std::fs::write(r.0.join("work.txt"), "y").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "work"]);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", finished])), 0);
    let out = r.ank("claude-code@ank", &["done"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "done"]);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", held])), 0);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "claimed"]);

    r.git(&["checkout", "-q", "main"]);
    for id in [finished, held] {
        assert!(
            r.task_text(id).contains("status: open"),
            "the fixture is wrong if main already carries the transition of {id}"
        );
    }

    let listing = stdout(&r.ank("someone@ank", &["find", "--status", "open"]));
    assert!(
        listing.contains("[finished:"),
        "the completion marker still stands on an open row: {listing}"
    );
    assert!(
        listing.contains("[claimed:claude-code@ank]"),
        "and so does the claim marker: {listing}"
    );
    assert!(
        listing.contains(&candidate[..9]),
        "the claimable row is listed too, unfiltered: {listing}"
    );
    assert!(
        listing.contains("2 spoken for"),
        "the line counts both rows a claim would refuse: {listing}"
    );
    assert!(
        listing.contains("--free"),
        "and names the flag that drops them: {listing}"
    );
    assert!(
        !listing.contains("hidden"),
        "nothing was dropped, so the hidden count is not borrowed: {listing}"
    );

    // Under the flag the rows are gone, so there is nothing left to say and the
    // line that would point at the flag already in force is absent.
    let free = stdout(&r.ank("someone@ank", &["find", "--free"]));
    assert!(free.contains(&candidate[..9]), "{free}");
    assert!(!free.contains("spoken for"), "{free}");

    // The machine surface is untouched by a sentence written for a reader, and
    // `--quiet` still writes nothing at all.
    let j = stdout(&r.ank("someone@ank", &["find", "--status", "open", "--json"]));
    assert!(!j.contains("spoken for"), "{j}");
    let q = stdout(&r.ank("someone@ank", &["find", "--status", "open", "--quiet"]));
    assert!(q.is_empty(), "{q}");

    // Once the branch lands, the file says `done` and the ref outlives it until
    // `check` prunes: the marker stays, and the line must not follow it there,
    // since `--free` keeps no done task.
    r.git(&["merge", "-q", "--no-ff", "-m", "merge", "feature"]);
    let done = stdout(&r.ank("someone@ank", &["find", "--status", "done"]));
    assert!(
        done.contains("[finished:"),
        "the ref outlives the merge until check prunes: {done}"
    );
    assert!(
        !done.contains("spoken for"),
        "a done listing is not sent to a flag that keeps only open tasks: {done}"
    );
}

/// `find --json` answers about the coordination plane, in the spelling
/// `context --json` already uses (TASK-e8e09606806f).
///
/// The two surfaces of one verb used to disagree about the same row: a reader
/// at a terminal saw `[finished:<sha> on <branch>]`, a script reading `--json`
/// saw `open` and nothing else. A caller filtering on that JSON would schedule
/// work already finished on a branch — exactly the window the completion ref
/// exists to close, reopened for whoever automated the question.
///
/// ADR-0c8ab846d262 is the argument: colour depends on the reader, structure
/// does not, and a marker is not colour. It is the answer, and it reads the
/// same in a pipe.
///
/// The state is asserted against the very string the human listing prints, so
/// a second spelling of one fact fails here rather than shipping.
#[test]
fn find_json_carries_the_coordination_state_the_human_listing_shows() {
    let finished = "TASK-a00000000005";
    let held = "TASK-b00000000005";
    let candidate = "TASK-c00000000005";
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task_with(finished, Some("A criterion."), &["ok"]);
    r.seed_task_scoped(held, "crates/**");
    r.seed_task_scoped(candidate, "docs/**");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed tasks"]);

    // Both transitions happen on a branch, so `main` keeps reading `open` for
    // both rows: the disagreement under test only exists while the file and the
    // ref say different things.
    r.git(&["checkout", "-q", "-b", "feature"]);
    std::fs::write(r.0.join("work.txt"), "y").unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "work"]);
    let finished_at = r.head();
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", finished])), 0);
    let out = r.ank("claude-code@ank", &["done"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "done"]);
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", held])), 0);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "claimed"]);

    r.git(&["checkout", "-q", "main"]);
    for id in [finished, held] {
        assert!(
            r.task_text(id).contains("status: open"),
            "the fixture is wrong if main already carries the transition of {id}"
        );
    }

    // What the reader at a terminal is told about the finished row.
    let marker = format!("[finished:{} on feature]", &finished_at[..7]);
    let listing = stdout(&r.ank("someone@ank", &["find", "--status", "open"]));
    assert!(
        listing.contains(&marker),
        "the fixture is wrong if the human listing does not mark the row: \
         {listing}"
    );

    let j = stdout(&r.ank("someone@ank", &["find", "--status", "open", "--json"]));

    // The same fact, the same words, on the same row — and the stored status
    // still on the key it always had, so nothing a caller reads today moves.
    assert!(
        j.contains(&format!(
            "{{\"id\":\"{finished}\",\"kind\":\"task\",\"status\":\"open\",\
             \"state\":\"{}\"",
            marker.trim_matches(|c| c == '[' || c == ']')
        )),
        "the finished state the listing shows is missing from the JSON row: {j}"
    );
    assert!(
        j.contains(&format!(
            "{{\"id\":\"{held}\",\"kind\":\"task\",\"status\":\"open\",\
             \"state\":\"claimed:claude-code@ank\""
        )),
        "a held row names its holder in the JSON too: {j}"
    );
    assert!(
        j.contains(&format!(
            "{{\"id\":\"{candidate}\",\"kind\":\"task\",\"status\":\"open\",\
             \"state\":\"open\""
        )),
        "a row the plane says nothing about carries its stored status as its \
         state, exactly as context --json spells it: {j}"
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

        // **And once more where the verb can succeed**, for the one verb this
        // fixture can only ever make refuse (TASK-9e63827380a1). `Repo` carries
        // a `.ank/`, which is what `init` exists to produce, so above it always
        // refuses; a refusal leaves stdout empty, and an empty stdout is what
        // `assert_json_only` returns early on. The sweep therefore proved that
        // `init` refuses cleanly and had never once seen it succeed — which is
        // exactly where it printed six lines of prose under `--json`.
        //
        // A sweep that only ever reaches a verb's refusal has a hole the shape
        // of that verb's success.
        if verb == "init" {
            let fresh = fresh_git_dir("sweep-init");
            let out = ank_command()
                .args(["init", "--json"])
                .env("ANK_AGENT", "claude-code@ank")
                .current_dir(&fresh)
                .output()
                .expect("the binary must have been built");
            assert_eq!(code(&out), 0, "init must succeed here: {}", stderr(&out));
            assert_json_only(&out, "`ank init --json` where it succeeds");
            let _ = std::fs::remove_dir_all(&fresh);
        }
    }
}

/// A git repository with no corpus in it, for the verbs whose success needs
/// one. Named by the caller so two of them never collide.
fn fresh_git_dir(what: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "ank-cli-{what}-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(&p).unwrap();
    let out = git_command(&p)
        .args(["init", "-q", "-b", "main"])
        .output()
        .expect("git must be installed: it is a hard dependency");
    assert!(out.status.success(), "git init: {}", stderr(&out));
    p
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
// A dead scope, and where git says the file went (ADR-3094538d831e)
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
/// **The proposed command is run, not merely matched.** ADR-3094538d831e
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
    // ADR-3094538d831e names, and no amount of matching on the string sees it.
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

/// The other death git records, and the note it earns (TASK-ec579d3a566e).
///
/// A deletion is not the silence a move under the similarity threshold is: git
/// names the commit that removed the path as plainly as it names a rename, and
/// the reader can date it and read its message. So the note says so — and
/// proposes nothing, because a deletion names no place a scope could be moved to
/// and a command that fails on the spot is the one thing the error style
/// forbids.
///
/// **The silence still has to exist**, so the second half is the death git
/// cannot name at all: a scope that never named a real file, where the reader
/// really does have nothing.
#[test]
fn a_deleted_file_names_the_commit_that_removed_it_and_proposes_nothing() {
    let deleted = moved_fixture("src/old.rs", None);
    let head = deleted.git(&["rev-parse", "HEAD"]);
    let out = deleted.ank("claude-code@ank", &["check"]);
    assert_eq!(
        code(&out),
        0,
        "a death git records is a signal, and the fault it used to be is one no \
         verb clears: {}",
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(
        text.contains("dead scope 'src/old.rs': no file matches it"),
        "the wording does not move with the severity: {text}"
    );
    let note = proposal(&text).unwrap_or_else(|| panic!("the deletion is named: {text}"));
    assert_eq!(
        note.len(),
        1,
        "a deletion names nowhere to move the scope to, so the note stops after \
         the commit: {note:?}"
    );
    let named = note[0].rsplit(' ').next().unwrap();
    assert!(
        note[0] == format!("git records src/old.rs deleted in {named}") && head.starts_with(named),
        "the note names the commit that removed it, and the commit is {head}: {note:?}"
    );
    assert!(
        !text.contains("renamed"),
        "a deletion went nowhere, and no rename may be invented for it: {text}"
    );

    // And the silence, which must not move. A path git has nothing to say about
    // leaves the reader with nothing, which is what the fault is for.
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/kept.rs"), SIMILAR).unwrap();
    r.seed_adr(DEAD_ADR, "Do not do X.", "src/never.rs");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    let out = r.ank("claude-code@ank", &["check"]);
    let text = stdout(&out);
    assert_eq!(
        code(&out),
        8,
        "a death git cannot name is still a fault: {text}{}",
        stderr(&out)
    );
    assert_eq!(
        proposal(&text),
        None,
        "git has nothing to say about it, and nothing may be invented: {text}"
    );
    for word in ["renamed", "delet", "removed"] {
        assert!(
            !text.contains(word),
            "'{word}' claims to know what became of a file git never knew: {text}"
        );
    }
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
/// Three fixtures differing by one act, because a single one proves the exit
/// code and not the reason for it. **Renamed:** git names the commit and where
/// the path went, so the corpus is outdated in a way the reader can follow
/// rather than broken — a signal. **Deleted:** git names the commit that removed
/// it just as plainly, so the reader can follow that too — a signal
/// (TASK-ec579d3a566e). **Never there:** git has nothing to say, the reader has
/// nothing, and the fault is what says so.
///
/// The third case is what makes the first two mean anything: a walk that stopped
/// asking would lower all three.
///
/// Through the binary because the exit code is the whole claim, and no unit test
/// of the severity reaches it.
#[test]
fn a_finished_tasks_dead_scope_faults_only_when_git_cannot_explain_it() {
    for (act, expected) in [("rename", 0), ("delete", 0), ("never", 8)] {
        let r = Repo::new();
        std::fs::create_dir_all(r.0.join("src")).unwrap();
        std::fs::write(r.0.join("src/old.rs"), SIMILAR).unwrap();
        // The scope of the third fixture names a path no commit ever carried,
        // and the file beside it is what keeps the corpus otherwise ordinary.
        let scope = match act {
            "never" => "src/absent.rs",
            _ => "src/old.rs",
        };
        finished_task_scoped(&r, scope);
        r.git(&["add", "-A"]);
        r.git(&["commit", "-qm", "seed"]);
        match act {
            "rename" => std::fs::rename(r.0.join("src/old.rs"), r.0.join("src/new.rs")).unwrap(),
            "delete" => std::fs::remove_file(r.0.join("src/old.rs")).unwrap(),
            // Nothing dies here: the scope was never alive.
            _ => {}
        }
        r.git(&["add", "-A"]);
        r.git(&["commit", "-qm", "move it", "--allow-empty"]);

        let out = r.ank("claude-code@ank", &["check"]);
        let text = stdout(&out);
        assert_eq!(code(&out), expected, "act={act}: {text}{}", stderr(&out));
        assert!(
            text.contains(&format!("dead scope '{scope}': no file matches it")),
            "the wording does not move with the severity: {text}"
        );
        assert_eq!(
            proposal(&text).is_some(),
            expected == 0,
            "act={act}: the explanation and the severity are the same fact: {text}"
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
/// **No shell is needed for the variable to reach git**, which is the half of
/// the cause a reader is most likely to reason away: `git_command` builds a
/// bare `Command` through `spawn`, with no interpreter in between, so an
/// argument nothing rewrote still comes out refused. `git.exe` reads
/// `MSYS_NO_PATHCONV` itself. Measured against a three-slash form put back for
/// one pair of runs (TASK-5052971b8e9c): with the variable set both
/// shallow-clone tests fail on `fatal: '/C:/...' does not appear to be a git
/// repository`, and with it unset the same two pass.
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
    // nothing, or this would have bought the green by giving up the check. A
    // path no commit ever carried is that case: a deletion is an answer git
    // gives (TASK-ec579d3a566e), and this is the one it cannot.
    let unknown = Repo::new();
    std::fs::create_dir_all(unknown.0.join("src")).unwrap();
    std::fs::write(unknown.0.join("src/kept.rs"), SIMILAR).unwrap();
    unknown.seed_adr(DEAD_ADR, "Do not do X.", "src/absent.rs");
    unknown.git(&["add", "-A"]);
    unknown.git(&["commit", "-qm", "seed"]);
    let whole = clone_of(&unknown, None);
    let out = unknown.ank_at("claude-code@ank", &["check"], &whole);
    assert_eq!(
        code(&out),
        8,
        "a path git never knew is still a fault: {}",
        stdout(&out)
    );
}

/// No repository, no walk, and no line saying so.
///
/// The cost clause of ADR-3094538d831e has a silence clause beside it, and the
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

/// **The cost of `check` stops growing with the corpus** (TASK-1b3d7b61dc8f).
///
/// What is counted is git **processes**, and not seconds. The cost being
/// removed is the process start -- 61 ms each on the machine this was measured
/// on, and 31.5 of the 44 seconds `check` took on this repository -- so a count
/// measures the thing itself. It is also the same number on the three platforms
/// CI runs, where a clock is not: a loaded runner would redden correct code, and
/// a timing test that cries wolf is one people learn to skip.
///
/// Counted with `GIT_TRACE2_EVENT`, which git writes itself, one `start` record
/// per process. No shim on `PATH`: Rust's `Command` appends `.exe` and never
/// consults `PATHEXT`, so a `git.cmd` ahead of the real one is never found on
/// Windows and the test would pass by measuring nothing.
///
/// **Two corpora differing only in how many scopes are dead**, on the same
/// number of entities, so nothing but the property under test moves. Before the
/// walk, each dead scope paid `rev-list -1 HEAD -- <path>` and then `diff-tree`,
/// and one entity with eight dead globs paid sixteen process starts where one
/// with a single dead glob paid two.
#[test]
fn checks_git_cost_does_not_grow_with_the_number_of_dead_scopes() {
    fn dead_scopes(count: usize) -> (Repo, usize) {
        let r = Repo::new();
        r.seed_docs();
        // Real files, committed and then removed, so the scopes are dead the way
        // a corpus's scopes actually die: git has a deletion to name, which is
        // the answer the walk is there to find. A path git never knew would
        // exercise the cheaper half of the question.
        let scopes: Vec<String> = (0..count).map(|i| format!("gone/file-{i}.txt")).collect();
        std::fs::create_dir_all(r.0.join("gone")).unwrap();
        for path in &scopes {
            std::fs::write(r.0.join(path), "content\n").unwrap();
        }
        r.git(&["add", "-A"]);
        r.git(&["commit", "-qm", "the files these scopes name"]);
        let quoted: Vec<&str> = scopes.iter().map(String::as_str).collect();
        let mut args: Vec<&str> = vec!["new", "task", "--title", "Scoped at what is about to go"];
        for path in &quoted {
            args.push("--scope");
            args.push(path);
        }
        args.push("--criteria");
        args.push("A verifiable criterion.");
        let out = r.ank(AGENT, &args);
        assert_eq!(code(&out), 0, "{}", stderr(&out));
        // `done`, because §4 makes a dead scope a fault on a finished task and a
        // signal on an open one: both walk history, and the finished shape is
        // the one a reader meets.
        std::fs::remove_dir_all(r.0.join("gone")).unwrap();
        r.git(&["add", "-A"]);
        r.git(&["commit", "-qm", "and now they are gone"]);

        let trace = r.0.join("trace.json");
        let out = ank_command()
            .args(["check", "--repo"])
            .arg(&r.0)
            .env("ANK_AGENT", AGENT)
            .env("GIT_TRACE2_EVENT", &trace)
            .current_dir(std::env::temp_dir())
            .output()
            .expect("the binary must have been built");
        assert!(code(&out) <= 8, "{}", stderr(&out));
        let text = std::fs::read_to_string(&trace).expect("git must have written the trace");
        let starts = text.matches("\"event\":\"start\"").count();
        assert!(
            starts > 0,
            "the trace records no git process at all, so this test measures \
             nothing: {text:.400}"
        );
        (r, starts)
    }

    let (_one, few) = dead_scopes(1);
    let (_many, lots) = dead_scopes(8);
    assert_eq!(
        few, lots,
        "eight dead scopes cost what one does, or the walk is being made once \
         per scope again: {few} against {lots}"
    );
}

/// **An entity the branch and the tree agree on is read from the tree, and one
/// they disagree on is read from the branch** (TASK-2ba2619b90e2).
///
/// `check` read every entity as the default branch carries it -- 3.9 MB on this
/// repository -- to answer, among other things, whether a task finished there
/// or only here. Where the object names agree the bytes agree, so the branch's
/// copy is the file already on disk, and moving it through the object store
/// answers a question about hashes by shipping the corpus.
///
/// The two directions are what this asserts, because the seeding can break
/// either way. A task `done` in the tree and not on the branch must not read as
/// finished; a task `done` on the branch and edited in the tree must still read
/// as finished. The second is the one seeding gets wrong if it trusts the local
/// file when the names differ.
#[test]
fn a_task_done_on_the_branch_is_read_from_the_branch_whatever_the_tree_says() {
    const ID: &str = "TASK-00000000d0d0";
    let r = Repo::new();
    r.seed_docs();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    // Done on the branch, and committed there.
    let out = r.ank(AGENT, &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let out = r.ank(AGENT, &["done", "--proof", "commit:HEAD"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "done on the branch"]);

    let settled = stdout(&r.ank(AGENT, &["check"]));

    // Now the tree disagrees with the branch about this entity: same id, other
    // bytes, so the object names differ and the branch's copy is the only one
    // that answers what the branch carries.
    let path = r.0.join(".ank/entities").join(format!("{ID}.md"));
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(
        &path,
        text.replace("A verifiable criterion.", "Another one."),
    )
    .unwrap();

    let after = stdout(&r.ank(AGENT, &["check"]));
    assert!(
        after.contains("differ from"),
        "an edited entity is a corpus that differs from the branch: {after}"
    );
    // The freeze finding is what says the criterion moved under a claim that is
    // gone; what matters here is that neither reading invented a task the
    // branch does not carry.
    assert!(
        !after.contains("finished on another branch"),
        "the branch and the tree hold the same task, edited: {after}
before:
{settled}"
    );
}

/// **The cost of `check` stops growing with the corpus itself**
/// (TASK-5f05e0c22f7b).
///
/// The other half of what a per-item question costs. `check` read one file per
/// entity off the default branch and one ref per claim, two process starts
/// each, so the price rose with every task written -- 616 starts on this
/// repository, of which 391 were these two questions asked over and over.
///
/// Two corpora differing only in how many tasks they hold, each task claimed so
/// that it carries a ref: what is varied is exactly what the criterion names,
/// entities and refs under `refs/ank/`.
#[test]
fn checks_git_cost_does_not_grow_with_the_number_of_entities() {
    fn corpus(tasks: usize) -> usize {
        let r = Repo::new();
        r.seed_docs();
        r.git(&["add", "-A"]);
        r.git(&["commit", "-qm", "seed"]);
        for i in 0..tasks {
            let id = format!("TASK-0000000{i:05x}");
            r.seed_task(&id, Some("A verifiable criterion."));
            r.git(&["add", "-A"]);
            r.git(&["commit", "-qm", "one more task"]);
            // Claimed under its own identity, so each task leaves a ref under
            // `refs/ank/` and the one-live-claim rule does not refuse the
            // second. Through the binary, which is also how a real corpus grows
            // its coordination plane.
            let out = r.ank(&format!("claude-code/agent-{i}"), &["claim", &id]);
            assert_eq!(code(&out), 0, "{}", stderr(&out));
        }
        let trace = r.0.join("trace.json");
        let out = ank_command()
            .args(["check", "--repo"])
            .arg(&r.0)
            .env("ANK_AGENT", AGENT)
            .env("GIT_TRACE2_EVENT", &trace)
            .current_dir(std::env::temp_dir())
            .output()
            .expect("the binary must have been built");
        assert!(code(&out) <= 8, "{}", stderr(&out));
        let text = std::fs::read_to_string(&trace).expect("git must have written the trace");
        let starts = text.matches("\"event\":\"start\"").count();
        assert!(starts > 0, "this test measures nothing: {text:.300}");
        starts
    }

    let few = corpus(2);
    let many = corpus(12);
    assert_eq!(
        few, many,
        "twelve claimed tasks cost what two do, or a file or a ref is being read          one at a time again: {few} against {many}"
    );
}

/// **Every ratification signature is verified in one process, whatever the
/// corpus holds** (TASK-1b3d7b61dc8f).
///
/// The other half of the criterion, and it is asked of the real corpus rather
/// than of a fixture: this repository carries the ratifications, the signing key
/// and the allowed-signers file, and seeding a second corpus with several signed
/// ratifications would be rebuilding it. `check` asked `rev-list --format=%G?`
/// once per ratified entity and each call started gpg -- 43 processes and 5.9 of
/// the 31.5 seconds git spent here.
///
/// The assertion is on the shape of the argv rather than on a total, because
/// the total is a property of this corpus on the day it is read, where "at most
/// one call carries the signature format" is the property the batching claims.
#[test]
fn every_ratification_signature_is_verified_in_one_call() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the workspace root is two levels up from this crate");
    let trace = std::env::temp_dir().join(format!("ank-sig-trace-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&trace);
    let out = ank_command()
        .args(["check", "--repo"])
        .arg(root)
        .env("ANK_AGENT", AGENT)
        .env("GIT_TRACE2_EVENT", &trace)
        .current_dir(std::env::temp_dir())
        .output()
        .expect("the binary must have been built");
    assert!(code(&out) <= 8, "{}", stderr(&out));
    let text = std::fs::read_to_string(&trace).expect("git must have written the trace");
    // `%G?` reaches git as the argument that asks for the signature, so a call
    // carrying it is a call that verifies one. The trace quotes the argv, and
    // the question mark is not escaped in JSON.
    let verifying = text.matches("%G?").count();
    assert!(
        verifying <= 1,
        "the signature format reached git {verifying} times, so it is being \
         asked once per ratification again"
    );
    let _ = std::fs::remove_file(&trace);
}

/// **A repository with no signing key ratifies, and the corpus says it did so
/// unsigned** (TASK-507660c3ebc4, ADR-964be4d940b2).
///
/// `accept` passed `-S` unconditionally and exited 9 naming
/// `git config user.signingkey`, so a corpus running the advisory mode §8
/// defines -- no key declared, no signature judged -- could not produce a
/// ratification at all. `check` had a regime `accept` refused to work in.
///
/// Both halves are asserted, because the repair is only honest with the second:
/// the ratification lands, **and** the corpus states the regime rather than
/// letting an unsigned decision read as a verified one.
#[test]
fn a_repository_with_no_signing_key_ratifies_and_says_it_was_unsigned() {
    const ADR: &str = "ADR-00000000c0c0";
    let r = Repo::new();
    r.seed_docs();
    r.seed_adr(ADR, "Do not do X.", "src/**");
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(
        r.0.join("src/main.rs"),
        "fn main() {}
",
    )
    .unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    // No `enable_signing`, no `declare_signing_key`: this repository cannot
    // sign and declares nobody, which is the state that used to be a dead end.
    let out = r.ank(AGENT, &["accept", ADR]);
    assert_eq!(
        code(&out),
        0,
        "a corpus with no key must still be able to ratify: {}",
        stderr(&out)
    );

    // The decision is binding, which is what ratifying is for.
    let shown = stdout(&r.ank(AGENT, &["show", ADR]));
    assert!(shown.contains("accepted"), "{shown}");

    // And the regime is stated rather than implied by silence. The sentence is
    // taken from `check` and matched in `review`, so the two surfaces are
    // compared against each other and never against a string written here.
    let checked = stdout(&r.ank(AGENT, &["check"]));
    let said = checked
        .lines()
        .find(|l| l.contains("no ratification key declared"))
        .unwrap_or_else(|| {
            panic!(
                "check must state the advisory regime:
{checked}"
            )
        })
        .trim()
        .trim_start_matches("signal: allowed_signers:")
        .trim()
        .to_string();
    let reviewed = stdout(&r.ank(AGENT, &["review"]));
    assert!(
        reviewed.contains(&said),
        "review must say what check says:
check: {said}
review:
{reviewed}"
    );
    assert!(
        !checked.contains("ratified by") || !checked.contains("verified"),
        "nothing may read as a verified ratification here: {checked}"
    );
    assert!(code(&out) == 0, "{}", stderr(&out));
}

/// **`review` names who may ratify, and nothing else can** (TASK-8a80b590b356).
///
/// ADR-01b6dd05f0db closes `.ank/` to a direct read by an agent, and
/// `allowed_signers` is not an entity, so `show`, `find` and `config` all pass
/// over it: before this, the one file the format asks a human to edit by hand
/// was the one file no command could show. `review` is where the answer belongs
/// -- §4 calls it the ratification queue, and who may ratify is the standing
/// half of that question.
///
/// Through the binary and on both renderings, because the claim is that a
/// *caller* can learn this without opening the file. Two entries under one
/// principal, which is the shape this repository's own file has: a person with a
/// gpg key and an ssh key is one identity and two rows, so a listing keyed on
/// the principal would silently drop one of them.
#[test]
fn review_names_the_signers_no_other_verb_serves() {
    let r = Repo::new();
    r.seed_docs();
    std::fs::write(
        r.0.join(".ank/allowed_signers"),
        "# a comment, which is not a signer
         marie@laptop gpg 739A603FB05F9F2F7D3C8D50624FCFCC1482554A
         marie@laptop ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIExample
",
    )
    .unwrap();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let text = stdout(&r.ank(AGENT, &["review"]));
    assert!(
        text.contains("MAY RATIFY (2)"),
        "the two rows are two signers, and the comment is not a third: {text}"
    );
    assert!(
        text.contains("marie@laptop") && text.contains("gpg") && text.contains("ssh-ed25519"),
        "the principal and the key type of each are what the caller came for: {text}"
    );

    // Parsed rather than substring-matched: the order of the two rows is the
    // order of the file, and a `contains` would pass on a document that had
    // swapped them or attached the key type to the wrong principal.
    let json = stdout(&r.ank(AGENT, &["review", "--json"]));
    let doc: serde_yaml::Value = serde_yaml::from_str(&json).unwrap();
    let rows = doc["signers"].as_sequence().expect("an array");
    assert_eq!(rows.len(), 2, "{json}");
    assert_eq!(
        rows[0]["principal"].as_str(),
        Some("marie@laptop"),
        "{json}"
    );
    assert_eq!(rows[0]["keytype"].as_str(), Some("gpg"), "{json}");
    assert_eq!(rows[1]["keytype"].as_str(), Some("ssh-ed25519"), "{json}");
}

/// **A corpus that declares no key says so, in the sentence `check` uses.**
///
/// §8 gives that state a defined behaviour: there is no allowlist, so `check`
/// judges no signature at all. An empty section would report the opposite --
/// "declared, and nobody yet" -- to a reader who never opened the file, which is
/// exactly the reader this section exists for.
///
/// The two surfaces are compared against **each other** and never against a
/// string written here: the sentence lives once in the source, and a test that
/// spelled it a fourth time would go on passing the day the two drifted apart.
#[test]
fn no_declared_key_is_said_in_words_and_never_as_an_empty_section() {
    let r = Repo::new();
    r.seed_docs();
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let checked = stdout(&r.ank(AGENT, &["check"]));
    let said = checked
        .lines()
        .find(|l| l.contains("no ratification key declared"))
        .unwrap_or_else(|| {
            panic!(
                "check must report the advisory mode:
{checked}"
            )
        })
        .trim()
        .trim_start_matches("signal: allowed_signers:")
        .trim()
        .to_string();

    let text = stdout(&r.ank(AGENT, &["review"]));
    assert!(
        text.contains(&said),
        "review must say what check says, to the byte:
check: {said}
review:
{text}"
    );
    assert!(
        !text.contains("MAY RATIFY"),
        "a header over nothing is the reading this refuses: {text}"
    );

    let json = stdout(&r.ank(AGENT, &["review", "--json"]));
    assert!(
        json.contains("\"signers\":[]"),
        "the key stays, so a parser reads one shape rather than two: {json}"
    );
}

/// **A proposed spec waits in the queue a human actually reads**
/// (TASK-73e81a8a804d).
///
/// `accept` promotes a spec and an ADR through the same verb, over the same
/// anchor and the same signed commit, so both are documents waiting for a
/// signature. `review` and `status` built their queue by filtering on
/// `EntityKind::Adr`, so a corpus holding a proposed spec was told its queue
/// was empty by the one verb whose stated job is the queue, while `find --type
/// spec --status proposed` named the document and `check` reported it twice.
///
/// Measured on this repository on 2026-08-19, which is what turned a log entry
/// into this test: two specs sat proposed after a supersession, `review` said
/// `nothing proposed for ratification`, and the maintainer had to be handed the
/// two `accept` lines by hand.
///
/// Through the binary, because the claim is about what the two commands print
/// on a corpus: the filter under test is reached by the render, and a unit test
/// on the row set would pass over a surface that never printed it.
///
/// Three corpora, because the criterion asks for three and each one can fail
/// alone: a spec by itself proves the kind is admitted, an ADR by itself proves
/// nothing was traded away for it, and the pair proves the count is a sum
/// rather than a branch.
#[test]
fn a_proposed_spec_alone_waits_where_review_and_status_look() {
    const DOC: &str = "SPEC-00000000e0e0";

    let r = Repo::new();
    r.seed_docs();
    r.seed_spec(DOC, "proposed", &[], None);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let text = stdout(&r.ank("claude-code@ank", &["review"]));
    assert!(
        text.contains("PROPOSED (1)") && text.contains(DOC),
        "a proposed spec is waiting for a signature and review must name it: {text}"
    );
    assert!(
        !text.contains("nothing proposed for ratification"),
        "the queue is not empty: {text}"
    );
    assert!(
        text.contains("LIVE CONSTRAINTS (0)"),
        "a spec declares no constraint and binds nothing, so it has no line \
         among the live ones: {text}"
    );

    let status = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(
        status.contains("queue 1 proposal(s)"),
        "status and review answer one corpus: {status}"
    );

    let out = r.ank("claude-code@ank", &["review", "--json"]);
    assert_json_only(&out, "ank review --json");
    let json = stdout(&out);
    assert!(
        json.contains(&format!("\"proposed\":[{{\"id\":\"{DOC}\"")),
        "a caller parsing review gets the spec as data too: {json}"
    );
    assert!(
        json.contains("\"live\":[]"),
        "and it is not laundered into the constraints: {json}"
    );
}

/// The other kind alone, which is what says the fix widened the queue rather
/// than moving it. This passed before TASK-73e81a8a804d and has to keep
/// passing: a change that admitted specs by dropping ADRs would satisfy the
/// test above and break the verb.
#[test]
fn a_proposed_adr_alone_still_waits_where_it_always_did() {
    const DECISION: &str = "ADR-00000000e1e1";

    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_adr(DECISION, "Do not do X.", "src/**");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let text = stdout(&r.ank("claude-code@ank", &["review"]));
    assert!(
        text.contains("PROPOSED (1)") && text.contains(DECISION),
        "{text}"
    );
    let status = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(status.contains("queue 1 proposal(s)"), "{status}");
}

/// Both kinds in one corpus: the queue is their sum, and ratifying the spec
/// leaves the live section exactly where it was.
///
/// That last assertion is the one keeping this fix inside its perimeter. The
/// criterion says the live section is unchanged, and the way to measure it is
/// not to read the code but to promote the spec and watch the constraints: a
/// spec that had leaked into that section would appear the moment it was
/// accepted, and the count would read 1 instead of 0.
#[test]
fn a_corpus_holding_both_kinds_queues_both_and_keeps_the_live_section() {
    const DECISION: &str = "ADR-00000000e2e2";
    const DOC: &str = "SPEC-00000000e2e2";

    let r = Repo::new();
    r.enable_signing();
    r.seed_docs();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_adr(DECISION, "Do not do X.", "src/**");
    r.seed_spec(DOC, "proposed", &[], None);
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let text = stdout(&r.ank("claude-code@ank", &["review"]));
    assert!(
        text.contains("PROPOSED (2)"),
        "the queue is the sum of both kinds, not one of them: {text}"
    );
    assert!(text.contains(DECISION) && text.contains(DOC), "{text}");
    assert!(
        text.contains("LIVE CONSTRAINTS (0)"),
        "neither is accepted yet: {text}"
    );

    let status = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(status.contains("queue 2 proposal(s)"), "{status}");

    let json = stdout(&r.ank("claude-code@ank", &["review", "--json"]));
    assert!(json.contains(DECISION) && json.contains(DOC), "{json}");

    // Ratify the spec alone. The queue loses it, and the constraints do not
    // gain it: promoting a spec adds nothing to what binds a perimeter.
    let out = r.ank("marie@laptop", &["accept", DOC]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    let text = stdout(&r.ank("claude-code@ank", &["review"]));
    assert!(
        text.contains("PROPOSED (1)") && text.contains(DECISION) && !text.contains(DOC),
        "ratifying takes the spec out of the queue: {text}"
    );
    assert!(
        text.contains("LIVE CONSTRAINTS (0)"),
        "an accepted spec still declares no constraint, so the live section is \
         unchanged by its ratification: {text}"
    );
    let status = stdout(&r.ank("claude-code@ank", &["status"]));
    assert!(status.contains("queue 1 proposal(s)"), "{status}");
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

/// This repository's own `.gitattributes`, so a fixture merges under the rules
/// the corpus actually ships with.
///
/// Copied rather than written, and the reason has outlived the rule it was
/// written for: a test that declared `merge=union` itself proved that git has a
/// union driver, which nobody doubted, and would have gone on passing for years
/// after somebody deleted the line from the file that matters. There is no such
/// line any more -- two entries are two files -- so what this now guards is the
/// opposite claim: the merge below succeeds under whatever the corpus actually
/// ships, and it ships no merge rule for the log at all.
fn repository_gitattributes() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the manifest is two directories under the repository root");
    std::fs::read_to_string(root.join(".gitattributes"))
        .expect("the repository declares its attributes")
}

/// Two branches record an entry about one task, and the merge keeps both.
///
/// **The claim under test is about git, so git is what is made to behave.**
/// Three texts said git's own union resolves two appends with no merge driver;
/// nothing in the repository configured one, and git's three-way merge
/// conflicts on two lines added at the end of one file — the textbook
/// adjacent-change case (TASK-6c0463fb4319). An assertion about the contents of
/// `.gitattributes` would have proved none of that, in either direction.
///
/// That is the case ADR-ff294eff4d1a celebrated most and the one it did not
/// cover: a second party writing to a task's log while the holder writes to it
/// too. ADR-25f977377fa0 removes it rather than resolving it — an entry is an
/// entity, so two concurrent entries are **two new files** and there is nothing
/// for a three-way merge to be three-way about. The test is kept, and it is
/// kept because the failure it caught was a text everybody believed: what makes
/// it evidence is that git actually runs.
#[test]
fn two_branches_recording_an_entry_merge_with_no_conflict() {
    let r = Repo::new();
    std::fs::write(r.0.join(".gitattributes"), repository_gitattributes()).unwrap();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", LOGGED])), 0);

    // A shared ancestor that already has the file: the case is a log two
    // parties append to, not one two parties create.
    assert_eq!(
        code(&r.ank(
            "claude-code@ank",
            &["log", "the entry both sides start from"]
        )),
        0
    );
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "the corpus and its log"]);

    // One branch each, and a real entry written by the binary on each -- a log
    // line forged by the test would be a line no writer produces.
    r.git(&["checkout", "-q", "-b", "reviewer"]);
    assert_eq!(
        code(&r.ank(
            "claude-code@ank",
            &["log", "written on the reviewer branch"]
        )),
        0
    );
    // `add -A` and not `commit -a`: an entry is a **new file**, which is the
    // whole of why the two sides no longer meet, and `-a` stages no such thing.
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "the reviewer's entry"]);

    r.git(&["checkout", "-q", "main"]);
    r.git(&["checkout", "-q", "-b", "holder"]);
    assert_eq!(
        code(&r.ank("claude-code@ank", &["log", "written on the holder branch"])),
        0
    );
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "the holder's entry"]);

    // The merge itself, allowed to fail: `Repo::git` would panic with git's
    // words, and what this test is about is the exit code.
    let merge = git_command(&r.0)
        .args(["merge", "--no-edit", "reviewer"])
        .output()
        .unwrap();
    assert!(
        merge.status.success(),
        "two appends to one log conflicted:\n{}\n{}\n--- the file ---\n{}",
        String::from_utf8_lossy(&merge.stdout),
        String::from_utf8_lossy(&merge.stderr),
        r.log_text(LOGGED)
    );

    // Both entries survived, and the third that was there before them -- as
    // three separate entities, which is what the merge had to keep.
    assert_eq!(r.entry_ids(LOGGED).len(), 3, "one file per entry");
    let log = r.log_text(LOGGED);
    for entry in [
        "the entry both sides start from",
        "written on the reviewer branch",
        "written on the holder branch",
    ] {
        assert!(log.contains(entry), "{entry} was lost:\n{log}");
    }
    for marker in ["<<<<<<<", "=======", ">>>>>>>"] {
        assert!(
            !log.contains(marker),
            "a resolved merge leaves no {marker}:\n{log}"
        );
    }

    // And the corpus reads: the entries parse, so the verbs answer from them
    // and `check` finds nothing.
    let out = r.ank("claude-code@ank", &["log", LOGGED]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let printed = stdout(&out);
    assert!(
        printed.contains("written on the reviewer branch")
            && printed.contains("written on the holder branch"),
        "{printed}"
    );
    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
}

/// The other half, and the one with teeth: when a merge *is* left half done,
/// `check` says so at the severity an entity file gets.
///
/// It used to be a signal. The strict log parser refused the marker line,
/// `check` reported the log as unreadable, and a signal leaves the exit code 0
/// — so an unresolved merge in a log passed CI green while the identical
/// markers in the entity beside it turned it red (TASK-6c0463fb4319).
#[test]
fn a_conflict_marker_in_a_log_is_a_fault_like_one_in_an_entity() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));

    // Seeded in the previous layout, which is what a corpus written by an older
    // build carries and what `check` still reads for one window (§3). No verb
    // writes one any more, so a fixture is the only way to produce the case.
    let path = r.0.join(".ank/log").join(format!("{LOGGED}.md"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        "- 2026-08-15T08:00Z claude-code@ank — an entry\n\
         <<<<<<< HEAD\n- 2026-08-15T09:00Z claude-code@ank — mine\n\
         =======\n- 2026-08-15T09:01Z marie@laptop — theirs\n>>>>>>> reviewer\n",
    )
    .unwrap();

    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(
        code(&out),
        8,
        "a merge left half done is a fault: {}{}",
        stdout(&out),
        stderr(&out)
    );
    let text = stdout(&out);
    assert!(
        text.contains("unresolved git conflict markers"),
        "the same words an entity file gets: {text}"
    );
    assert!(
        text.contains(&format!("log/{LOGGED}.md")),
        "and it names the file: {text}"
    );

    // Restored, and `check` is green again -- so the finding is about the
    // markers and not about the fixture. A corpus still holding the previous
    // log directory is a signal, which leaves the code 0 and names the verb
    // that moves it.
    std::fs::write(&path, "- 2026-08-15T08:00Z claude-code@ank — an entry\n").unwrap();
    let out = r.ank("claude-code@ank", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        stdout(&out).contains("ank migrate"),
        "the signal names the command that moves it: {}",
        stdout(&out)
    );
}

/// `log` and `show` answer under `context_budget` and say what they cut.
///
/// They were the two readers in the tool with no budget and no flags, while
/// `find` and `context` are both bounded — on a corpus where the log is more
/// than a quarter of the bytes and only ever grows. The module header of `find`
/// states the reason it is capped at all: a reader without a cap is a
/// context-explosion vector (TASK-6c0463fb4319).
#[test]
fn log_and_show_cap_the_log_and_announce_what_they_cut() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", LOGGED])), 0);
    for i in 0..12 {
        assert_eq!(
            code(&r.ank(
                "claude-code@ank",
                &[
                    "log",
                    &format!("entry number {i:02} of a log that only ever grows"),
                ]
            )),
            0
        );
    }
    // A budget small enough that the log cannot fit in it. Written after the
    // entries so that the claim above was taken under the shipped default.
    std::fs::write(
        r.0.join(".ank/config.yml"),
        "schema: 1\ncontext_budget: 400\nclaim_ttl_max: 2h\ndefault_branch: main\n",
    )
    .unwrap();

    let out = r.ank("claude-code@ank", &["log", LOGGED]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    let listed = text.lines().filter(|l| l.contains("entry number")).count();
    assert!(
        (1..12).contains(&listed),
        "the cap follows context_budget, and never empties the section:\n{text}"
    );
    assert!(
        text.contains("entry number 11") && !text.contains("entry number 00"),
        "the newest survive and the oldest yield:\n{text}"
    );
    assert!(
        text.contains(&format!("+{} earlier entries", 12 - listed)),
        "announced, never silent:\n{text}"
    );
    assert!(
        text.contains("ank config context_budget "),
        "and it names the command that would print them:\n{text}"
    );

    // The two counts a parser needs, the pair `find --json` already carries.
    let out = r.ank("claude-code@ank", &["log", LOGGED, "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains(&format!("\"total\":12,\"shown\":{listed}")),
        "{}",
        stdout(&out)
    );

    // `show` charges the entity first and never cuts it: a truncated entity is
    // not a short answer but a wrong one (§4). What yields is the log under it.
    let out = r.ank("claude-code@ank", &["show", LOGGED]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let text = stdout(&out);
    assert!(
        text.starts_with(&r.task_text(LOGGED)),
        "the entity is still verbatim:\n{text}"
    );
    let shown = text
        .lines()
        .filter(|l| l.contains("claude-code@ank —"))
        .count();
    assert!(
        (1..12).contains(&shown),
        "the log is what the budget cuts:\n{text}"
    );
    assert!(
        text.contains(&format!("LOG ({shown} of 12)")),
        "kept of total, the header `context` already prints:\n{text}"
    );
    assert!(
        text.contains(&format!(
            "+{} earlier entries, ank log {LOGGED}",
            12 - shown
        )),
        "naming the reader that has more room, not a flag that does not exist:\n{text}"
    );
    assert!(
        shown <= listed,
        "`show` pays for the entity out of the same budget, so it can only \
         print fewer than `log`: {shown} against {listed}"
    );
}

/// Read from wherever it is, by every surface that shows it.
#[test]
fn show_log_and_context_read_the_entries_of_an_entity() {
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
    // `kept of total`, the header `context` already prints for the same
    // section: the log is capped by the budget like every other reader, and one
    // fact does not get two grammars (TASK-6c0463fb4319).
    assert!(text.contains("LOG (1 of 1)"), "{text}");
    assert!(
        !text.contains("earlier entries"),
        "nothing was cut, so nothing is announced: {text}"
    );
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
        stdout(&out).contains("\"log\":[{\"id\":\"LOG-"),
        "{}",
        stdout(&out)
    );
    // The two counts a parser needs to tell a short log from a cut one, the
    // same pair `find --json` carries.
    assert!(
        stdout(&out).contains("\"log_total\":1,\"log_shown\":1"),
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
/// section keeps it exactly as it is, the new entry becomes an entity beside
/// it, and every reader sees both — each exactly once.
///
/// The two sources add up rather than one winning, and that is what the entry
/// kind made correct: nothing appends to the previous layout any more, so the
/// body holds what was written before the move and the corpus holds what came
/// after. Preferring either would leave one half unreachable — the exact
/// failure the schema bump exists to prevent, arriving through the tool rather
/// than through an old reader.
#[test]
fn an_entity_whose_log_is_in_its_body_gains_entries_beside_it() {
    let r = Repo::new();
    r.seed_task_with_body_log(LOGGED, "an entry written before the move");
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", LOGGED])), 0);
    // After the claim, which is a transition and does write the entity. What
    // is under test is the entry below, which must not.
    let before = r.task_text(LOGGED);
    assert_eq!(
        code(&r.ank("claude-code@ank", &["log", "learned something"])),
        0
    );

    assert_eq!(
        r.entry_ids(LOGGED).len(),
        1,
        "the entry is one new entity and nothing else"
    );
    assert_eq!(
        r.task_text(LOGGED),
        before,
        "the entity the entry is about is not opened for writing at all"
    );

    // Every reader sees both, exactly once.
    let out = r.ank("claude-code@ank", &["log", LOGGED]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let listed = stdout(&out);
    assert_eq!(listed.matches("learned something").count(), 1, "{listed}");
    assert_eq!(
        listed.matches("an entry written before the move").count(),
        1,
        "{listed}"
    );

    // `show` prints the body's own section as part of the entity and adds no
    // second copy of it under the fold -- what it adds there is the half the
    // body cannot hold.
    let out = stdout(&r.ank("claude-code@ank", &["show", LOGGED]));
    assert_eq!(out.matches("learned something").count(), 1, "{out}");
    assert_eq!(
        out.matches("an entry written before the move").count(),
        1,
        "the body already carries it: {out}"
    );
    assert!(out.contains("LOG (1 of 1)"), "{out}");
}

// ---------------------------------------------------------------------------
// An entry is an entity (§3, ADR-25f977377fa0, TASK-df9c6d46e8ef)
// ---------------------------------------------------------------------------

/// An entry is a file of its own, and `find` reaches it like anything else.
///
/// **The gap this closes was measured, not supposed**: the previous shape was
/// indexed by nothing, so no question about the log had an answer inside the
/// tool. Through the binary, because what has to be true is a property of the
/// corpus on disk and of the index built from it.
#[test]
fn an_entry_is_an_entity_and_find_reaches_it() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", LOGGED])), 0);
    assert_eq!(
        code(&r.ank(
            "claude-code@ank",
            &["log", "the quicksilver invariant holds"]
        )),
        0
    );

    // One file, in the flat directory, of kind log and naming its subject.
    let ids = r.entry_ids(LOGGED);
    assert_eq!(ids.len(), 1, "one entry, one file: {ids:?}");
    let entry = &ids[0];
    let text = std::fs::read_to_string(r.0.join(".ank/entities").join(format!("{entry}.md")))
        .expect("an entry lives where every entity lives");
    assert!(text.contains("type: log"), "{text}");
    assert!(text.contains(&format!("about: {LOGGED}")), "{text}");
    assert!(
        !text.contains("status:"),
        "an entry has nothing to transition to: {text}"
    );

    // `find` reaches it by its message, and `--type log` narrows to entries.
    let out = r.ank("marie@laptop", &["find", "quicksilver"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("the quicksilver invariant holds"),
        "an entry is reachable by what it says: {}",
        stdout(&out)
    );
    let out = r.ank("marie@laptop", &["find", "quicksilver", "--type", "log"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    // The short form, measured against the corpus like every printed id (§3).
    assert!(stdout(&out).contains(&entry[..8]), "{}", stdout(&out));
    assert!(
        !stdout(&out).contains("[]"),
        "a kind with no lifecycle carries no marker: {}",
        stdout(&out)
    );
    let out = r.ank("marie@laptop", &["find", "quicksilver", "--type", "task"]);
    assert!(
        !stdout(&out).contains(&entry[..8]),
        "the filter is the registry's, not a guess: {}",
        stdout(&out)
    );

    // And `show` prints the entry itself, byte for byte like any other entity.
    let out = r.ank("marie@laptop", &["show", entry]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(stdout(&out).starts_with(&text), "{}", stdout(&out));
}

/// A message no line can hold survives whole, and no lister prints it whole.
///
/// The corpus this task migrates averages 453 characters an entry and its
/// longest is 2105. Both halves matter and neither implies the other: the
/// listing has to stay bounded, and the message has to come back byte for byte.
#[test]
fn a_long_message_is_elided_in_a_listing_and_whole_in_the_entry() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", LOGGED])), 0);

    // One long message with a distinctive head and a distinctive tail, so that
    // "whole" and "elided" are each assertable rather than inferred.
    let long = format!(
        "opening clause of a message far too long for any line, {} and the closing clause",
        "measured and recorded ".repeat(90)
    );
    assert!(long.len() > 2000, "{}", long.len());
    assert_eq!(
        code(&r.ank("claude-code@ank", &["log", &long])),
        0,
        "a long message is not refused"
    );

    // Whole in the corpus: the two fields concatenate back to what went in.
    assert!(
        r.log_text(LOGGED).contains(&long),
        "a message was altered on the way into an entity:\n{}",
        r.log_text(LOGGED)
    );

    // Elided in every listing, and the listing says where the rest is.
    let listed = stdout(&r.ank("marie@laptop", &["log", LOGGED]));
    assert!(listed.contains("opening clause"), "{listed}");
    assert!(
        !listed.contains("and the closing clause"),
        "the line is bounded whatever the message:\n{listed}"
    );
    assert!(listed.contains('\u{2026}'), "and it says so:\n{listed}");
    let widest = listed.lines().map(|l| l.chars().count()).max().unwrap();
    assert!(widest < 200, "{widest} characters on one line:\n{listed}");

    // The entry's own id is on the row, which is what makes the elision
    // recoverable: a command nobody can run is not a way out.
    let entry = &r.entry_ids(LOGGED)[0];
    assert!(
        listed.contains(&entry[..8]),
        "the row names the entry it prints:\n{listed}"
    );
    let shown = stdout(&r.ank("marie@laptop", &["show", entry]));
    assert!(
        shown.contains("and the closing clause"),
        "show prints it whole:\n{shown}"
    );

    // And a parser gets the whole message, because it reads no page.
    let out = r.ank("marie@laptop", &["log", LOGGED, "--json"]);
    assert_json_only(&out, "ank log --json");
    assert!(
        stdout(&out).contains("and the closing clause"),
        "{}",
        stdout(&out)
    );
}

/// The numbers the messages carry, oldest first, out of `ank log`'s newest-first
/// page.
fn logged_order(text: &str) -> Vec<String> {
    let mut n: Vec<String> = text
        .lines()
        .filter_map(|l| l.split(" — entry ").nth(1).map(str::to_string))
        .collect();
    n.reverse();
    n
}

/// **Entries written in one second come back in the order they were written.**
///
/// This is the defect CI caught and the reason the kind carries `seq` at all.
/// `created` has one-second resolution and one `ank log` costs a few hundred
/// milliseconds, so several entries inside one second is the ordinary case;
/// with the identifier as the only tiebreak the order was a hash, and a
/// measured 10 runs in 12 printed four entries wrong. It passed on Windows in
/// CI because that job is four times slower and its writes straddled a second
/// more often, which is luck and not a platform property.
///
/// **Two assertions, and the second is what makes this test worth having on a
/// machine whose clock happens to cooperate.** The order is asserted end to end
/// through the binary; and `seq` is asserted to run 0, 1, 2 … which holds
/// whether or not the writes shared a second, so a slow runner gets a real test
/// rather than a vacuous pass. The timestamps observed are printed on failure,
/// so a red build says which of the two situations it was in.
#[test]
fn entries_written_in_one_second_come_back_in_write_order() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", LOGGED])), 0);
    const N: usize = 8;
    for i in 0..N {
        assert_eq!(
            code(&r.ank("claude-code@ank", &["log", &format!("entry {i}")])),
            0
        );
    }

    let out = r.ank("marie@laptop", &["log", LOGGED, "--json"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let json = stdout(&out);
    let stamps: Vec<&str> = json
        .match_indices("\"timestamp\":\"")
        .map(|(i, m)| {
            let rest = &json[i + m.len()..];
            &rest[..rest.find('"').unwrap()]
        })
        .collect();
    let distinct: std::collections::BTreeSet<&&str> = stamps.iter().collect();

    let page = stdout(&r.ank("marie@laptop", &["log", LOGGED]));
    let expected: Vec<String> = (0..N).map(|i| i.to_string()).collect();
    assert_eq!(
        logged_order(&page),
        expected,
        "{N} entries came back out of the order they were written; \
         {} distinct instants among {}:\n{page}",
        distinct.len(),
        stamps.len()
    );

    // The mechanism, and it holds whatever the clock did: the rank of an entry
    // is one more than the highest its writer could see, so a run of writes is
    // a run of ranks. A machine slow enough to put every write in its own
    // second still tests this.
    let ranks: Vec<String> = r
        .entry_ids(LOGGED)
        .iter()
        .map(|id| {
            let text = std::fs::read_to_string(r.0.join(".ank/entities").join(format!("{id}.md")))
                .unwrap();
            let line = text.lines().find(|l| l.starts_with("seq: ")).expect("seq");
            line["seq: ".len()..].to_string()
        })
        .collect();
    let mut ranks: Vec<u64> = ranks.iter().map(|s| s.parse().unwrap()).collect();
    ranks.sort_unstable();
    assert_eq!(
        ranks,
        (0..N as u64).collect::<Vec<_>>(),
        "the ranks of {N} consecutive writes are 0..{N}"
    );

    // And the answer does not move between two reads of one corpus.
    for _ in 0..3 {
        assert_eq!(stdout(&r.ank("marie@laptop", &["log", LOGGED])), page);
    }
}

/// A corpus migrated from the previous layout keeps the line order of the file
/// it came from, and that is the only order those entries ever had.
///
/// The instants are chosen here, which is the one thing a test writing through
/// `ank log` cannot do: `created` comes from the clock. Two of the four lines
/// share a second, so the file's order is load-bearing for exactly the pair
/// that has no other order — 6 such groups covering 12 entries in this
/// repository's own corpus.
///
/// **Across distinct instants the timestamps decide**, which is why the file is
/// written out of chronological order: one file of the 178 this corpus migrated
/// stores its lines newest-first, and its own timestamps are the better
/// evidence of what happened when (§3).
#[test]
fn a_migrated_log_keeps_its_line_order_inside_one_second() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    std::fs::create_dir_all(r.0.join(".ank/log")).unwrap();
    std::fs::write(
        r.0.join(".ank/log").join(format!("{LOGGED}.md")),
        "- 2026-08-01T09:00:02Z claude-code/1.4.2 — entry 2\n\
         - 2026-08-01T09:00:00Z claude-code/1.4.2 — entry 0\n\
         - 2026-08-01T09:00:03Z claude-code/1.4.2 — entry 3\n\
         - 2026-08-01T09:00:00Z claude-code/1.4.2 — entry 1\n",
    )
    .unwrap();
    assert_eq!(code(&r.ank("marie@laptop", &["migrate"])), 0);

    let page = stdout(&r.ank("marie@laptop", &["log", LOGGED]));
    assert_eq!(
        logged_order(&page),
        vec!["0", "1", "2", "3"],
        "the tied pair keeps the file's order and the rest is chronological:\n{page}"
    );

    // Two reads of one corpus never differ.
    for _ in 0..3 {
        assert_eq!(stdout(&r.ank("marie@laptop", &["log", LOGGED])), page);
    }

    // `show` prints the same set in the opposite direction, and reversing one
    // gives the other -- so both orders come from the entries and not from two
    // independent walks that agree today.
    let mut oldest_first: Vec<String> = stdout(&r.ank("marie@laptop", &["show", LOGGED]))
        .lines()
        .filter_map(|l| l.split(" — entry ").nth(1).map(str::to_string))
        .collect();
    assert_eq!(
        oldest_first,
        vec!["0", "1", "2", "3"],
        "show is oldest first"
    );
    oldest_first.reverse();
    let mut newest_first = logged_order(&page);
    newest_first.reverse();
    assert_eq!(newest_first, oldest_first);
}

/// The migration: every entry moves, the count is equal, and no message
/// changes.
///
/// **Asserted against the corpus and not against a counter the verb kept.** A
/// migration that dropped one entry would be invisible and the loss permanent,
/// which is why the criterion asks for the count and for the messages, and why
/// both are read back out of the files afterwards.
#[test]
fn migrate_moves_every_entry_and_alters_no_message() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    const OTHER: &str = "TASK-000000000002";
    r.seed_task(OTHER, Some("Another verifiable criterion."));

    // Seeded in the previous layout: no verb writes one, so a fixture is what
    // produces a corpus that has not moved yet. Two entries share a second,
    // which is the case the ordering has to survive; one message is far longer
    // than a line, which is the case the split has to survive.
    let long = format!("a message {}", "long enough to be split ".repeat(80));
    let messages = [
        (
            LOGGED,
            "2026-08-01T09:00:00Z",
            "the first thing that happened",
        ),
        (
            LOGGED,
            "2026-08-01T09:00:00Z",
            "and the second, in the same second",
        ),
        (LOGGED, "2026-08-01T10:00:00Z", long.as_str()),
        (
            OTHER,
            "2026-08-02T11:00:00Z",
            "released: the criterion was wrong",
        ),
    ];
    std::fs::create_dir_all(r.0.join(".ank/log")).unwrap();
    for id in [LOGGED, OTHER] {
        let body: String = messages
            .iter()
            .filter(|(subject, _, _)| *subject == id)
            .map(|(_, at, m)| format!("- {at} claude-code/1.4.2 \u{2014} {m}\n"))
            .collect();
        std::fs::write(r.0.join(".ank/log").join(format!("{id}.md")), body).unwrap();
    }

    // `check` names the verb before it is run, which is how a corpus finds out.
    let out = r.ank("marie@laptop", &["check"]);
    assert_eq!(
        code(&out),
        0,
        "a previous layout is a signal: {}",
        stdout(&out)
    );
    assert!(stdout(&out).contains("ank migrate"), "{}", stdout(&out));

    let out = r.ank("marie@laptop", &["migrate"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        stdout(&out).contains("migrated 4 entries from 2 log files"),
        "{}",
        stdout(&out)
    );

    // The count, before and after, from the corpus itself.
    assert_eq!(r.entry_ids(LOGGED).len(), 3);
    assert_eq!(r.entry_ids(OTHER).len(), 1);
    // And no message altered, byte for byte, the long one included.
    for (id, at, message) in messages {
        let rendered = r.log_text(id);
        assert!(
            rendered.contains(&format!("- {at} claude-code/1.4.2 \u{2014} {message}\n")),
            "a message was altered:\n{rendered}"
        );
    }

    // The previous layout is gone, so nothing writes to it and `check` stops
    // naming it -- and the corpus is still sound.
    assert!(
        !r.0.join(".ank/log").exists(),
        "the directory it read is removed"
    );
    let out = r.ank("marie@laptop", &["check"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(!stdout(&out).contains("ank migrate"), "{}", stdout(&out));

    // Running it again on a corpus that has moved is not an error.
    let out = r.ank("marie@laptop", &["migrate"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("nothing to migrate"),
        "{}",
        stdout(&out)
    );
}

/// A subject that **already carries entries** when the migration runs, which is
/// the normal case on a repository that logged before it moved.
///
/// Measured on this repository and not imagined: the first run over its own
/// corpus read 512 lines, found 514 entries afterwards, and reported a failure
/// on a migration that had worked — because the two entries written minutes
/// earlier by a current build were counted against a total that knew nothing
/// about them. The count is per subject and relative to what was there.
#[test]
fn migrate_adds_to_the_entries_a_subject_already_has() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", LOGGED])), 0);
    assert_eq!(
        code(&r.ank(
            "claude-code@ank",
            &["log", "written by a build that had already moved"]
        )),
        0
    );
    assert_eq!(r.entry_ids(LOGGED).len(), 1);

    std::fs::create_dir_all(r.0.join(".ank/log")).unwrap();
    std::fs::write(
        r.0.join(".ank/log").join(format!("{LOGGED}.md")),
        "- 2026-08-01T09:00:00Z claude-code/1.4.2 — written before the move\n",
    )
    .unwrap();

    let out = r.ank("marie@laptop", &["migrate"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        stdout(&out).contains("migrated 1 entries"),
        "the report counts what it read: {}",
        stdout(&out)
    );

    // Both halves of the history, and neither replaced the other.
    assert_eq!(r.entry_ids(LOGGED).len(), 2);
    let rendered = r.log_text(LOGGED);
    assert!(rendered.contains("written before the move"), "{rendered}");
    assert!(
        rendered.contains("written by a build that had already moved"),
        "{rendered}"
    );
}

/// A run interrupted between writing the entries and removing the file it read
/// is recovered by running it again.
///
/// The identifiers are derived from the line, so the second run recognises what
/// the first wrote rather than refusing it as an existing entity or writing a
/// second copy of every entry.
#[test]
fn migrate_run_twice_recognises_what_the_first_run_wrote() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    std::fs::create_dir_all(r.0.join(".ank/log")).unwrap();
    let file = r.0.join(".ank/log").join(format!("{LOGGED}.md"));
    let body = "- 2026-08-01T09:00:00Z claude-code/1.4.2 — the first\n\
                - 2026-08-01T09:00:01Z claude-code/1.4.2 — the second\n";
    std::fs::write(&file, body).unwrap();

    assert_eq!(code(&r.ank("marie@laptop", &["migrate"])), 0);
    let after_first = r.entry_ids(LOGGED);
    assert_eq!(after_first.len(), 2);

    // The interruption: the file is back, the entries are still there.
    std::fs::create_dir_all(r.0.join(".ank/log")).unwrap();
    std::fs::write(&file, body).unwrap();

    let out = r.ank("marie@laptop", &["migrate"]);
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));
    assert!(
        stdout(&out).contains("2 of them already existed"),
        "the recovery is said out loud: {}",
        stdout(&out)
    );
    assert_eq!(
        r.entry_ids(LOGGED),
        after_first,
        "the same identifiers, and no second copy of anything"
    );
    assert!(!file.exists());
}

/// A log file the grammar refuses **stops the migration naming it**, and
/// nothing is written.
///
/// The parser refuses a whole file on its first malformed line, so skipping the
/// file would silently drop every sound entry beside the bad one — which is the
/// invisible, permanent loss the criterion is written around.
#[test]
fn migrate_stops_on_a_log_file_it_cannot_read_and_names_it() {
    let r = Repo::new();
    r.seed_task(LOGGED, Some("A verifiable criterion."));
    const OTHER: &str = "TASK-000000000002";
    r.seed_task(OTHER, Some("Another verifiable criterion."));
    std::fs::create_dir_all(r.0.join(".ank/log")).unwrap();
    std::fs::write(
        r.0.join(".ank/log").join(format!("{LOGGED}.md")),
        "- 2026-08-01T09:00:00Z claude-code/1.4.2 \u{2014} a sound entry\n\
         a line the grammar does not accept\n\
         - 2026-08-01T09:01:00Z claude-code/1.4.2 \u{2014} another sound entry\n",
    )
    .unwrap();
    std::fs::write(
        r.0.join(".ank/log").join(format!("{OTHER}.md")),
        "- 2026-08-02T11:00:00Z claude-code/1.4.2 \u{2014} entirely sound\n",
    )
    .unwrap();

    let out = r.ank("marie@laptop", &["migrate"]);
    assert_eq!(code(&out), 1, "{}{}", stdout(&out), stderr(&out));
    let said = stderr(&out);
    assert!(
        said.contains(&format!("log/{LOGGED}.md")),
        "the file is named: {said}"
    );
    assert!(said.contains("line 2"), "and the line in it: {said}");

    // Nothing was written, on either subject: the whole plan is read before the
    // first entity is created, so a corpus is never left half moved.
    assert!(r.entry_ids(LOGGED).is_empty(), "{:?}", r.entry_ids(LOGGED));
    assert!(r.entry_ids(OTHER).is_empty(), "{:?}", r.entry_ids(OTHER));
    assert!(r.0.join(".ank/log").join(format!("{OTHER}.md")).exists());
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

// ---------------------------------------------------------------------------
// A short id the binary prints is one the binary resolves (§3, TASK-c1f01f301d63)
// ---------------------------------------------------------------------------

// Through the binary, and it could not be anywhere else: the property is about
// what one process writes and another process reads back, over a corpus neither
// of them chose. `short_ids` had unit tests and they were green throughout —
// what a unit test cannot see is a verb that never called it, which is exactly
// the two defects measured here: `check` printing a fixed four-character
// subject, and every listing measuring its short forms against the entities
// that parsed rather than against the ones prefix resolution walks.
//
// The sweep is deliberately blind. It does not name the ids it expects; it
// takes every `TASK-`/`ADR-` token out of what a verb wrote and hands each one
// back to `show`. A test naming the expected short forms would keep passing the
// day a verb starts printing something else, which is the shape both defects
// had.

/// Every identifier-shaped token in a verb's output.
///
/// Deliberately generous: it catches a short form, a full id, and an id inside
/// a `-> ank show <id>` hint alike, because all three are things the tool told
/// the caller they could type. A trailing `.md` is not part of the token, which
/// is what stops the run at the dot.
fn ids_printed(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for word in text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '-')) {
        for kind in ["TASK-", "ADR-"] {
            let Some(hex) = word.strip_prefix(kind) else {
                continue;
            };
            let run: String = hex.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
            let id = format!("{kind}{run}");
            if run.len() >= 4 && !found.contains(&id) {
                found.push(id);
            }
        }
    }
    found
}

/// Runs each invocation and hands every identifier it printed back to `show`.
///
/// The assertion is **code 2** rather than a byte comparison, because code 2 is
/// what §4 gives to both halves of the failure — not found and ambiguous prefix
/// — and it is the code an agent branches on. Any other code is a verb
/// answering about a real entity, which is all this asks.
fn every_printed_id_resolves(r: &Repo, verbs: &[&[&str]]) {
    for args in verbs {
        let out = r.ank("claude-code@ank", args);
        let text = format!("{}{}", stdout(&out), stderr(&out));
        let printed = ids_printed(&text);
        assert!(
            !printed.is_empty(),
            "{args:?} printed no identifier at all, so the sweep asserts nothing: {text}"
        );
        for id in printed {
            let shown = r.ank("claude-code@ank", &["show", &id]);
            assert_ne!(
                code(&shown),
                2,
                "{args:?} printed {id}, which ank show refuses: {}",
                stderr(&shown)
            );
        }
    }
}

/// A corpus that collides on purpose, over every verb that prints an id.
///
/// Two tasks sharing their first four hex characters and two ADRs sharing four
/// of their own, so both halves of the per-kind rule are exercised: the tasks
/// have to lengthen, and the ADRs have to lengthen without the tasks' collision
/// dragging them along.
///
/// `claim` sits **inside** the sweep rather than beside it. Its confirmation
/// line names an identifier too, and the two rows after it — `status`, and
/// `context` in execution mode — are renderers a claimless corpus never
/// reaches.
#[test]
fn every_short_id_the_binary_prints_resolves_to_one_entity() {
    let r = Repo::new();
    r.seed_task_titled("TASK-abcd10000000", "One task");
    r.seed_task_titled("TASK-abcd20000000", "Another task");
    r.seed_adr("ADR-beef10000000", "Do the thing.", "src/**");
    r.seed_adr("ADR-beef20000000", "Do the other thing.", "src/**");
    // An entity written by an agent and read by no human: the one `check`
    // finding whose subject was a fixed four characters where every other
    // finding in the report carries the full id.
    let text = r
        .task_text("TASK-abcd10000000")
        .replace("created: ", "author: claude-code/1\ncreated: ");
    std::fs::write(r.flat_task_path("TASK-abcd10000000"), text).unwrap();

    let taken = r.ank("claude-code@ank", &["claim", "TASK-abcd10000000"]);
    assert_eq!(
        code(&taken),
        0,
        "the fixture's claim must be taken, or status and execution mode are \
         never reached: {}",
        stderr(&taken)
    );

    every_printed_id_resolves(
        &r,
        &[
            &["find"],
            &["graph"],
            &["scope", "src"],
            &["show", "TASK-abcd10000000"],
            &["check"],
            &["status"],
            &["context"],
        ],
    );
}

/// The corpus a short form is measured against is the one **resolution**
/// walks, not the one the index answered with.
///
/// A third task at schema 99: no build of this binary can read it, so every
/// listing leaves it out (TASK-ca7b61b00896) while `store::resolve` walks it
/// anyway — resolution lists `<ID>.md` file names and never asks the index.
/// Measured against the rows a verb returned, `TASK-abcd1` is unambiguous in
/// that answer and ambiguous in the repository, and the process printing it
/// refuses it one command later.
///
/// Kept apart from the sweep above because `claim` refuses a corpus it cannot
/// read whole, which is behaviour of its own and not this rule: putting the
/// unreadable entity in that fixture would have tested a refusal instead.
#[test]
fn a_short_id_is_measured_against_the_entities_this_build_cannot_read() {
    let r = Repo::new();
    r.seed_task_titled("TASK-abcd10000000", "One task");
    r.seed_task_titled("TASK-abcd20000000", "Another task");
    r.seed_task_at_schema("TASK-abcd1fffffff", 99);

    every_printed_id_resolves(
        &r,
        &[
            &["find"],
            &["graph"],
            &["scope", "src"],
            &["show", "TASK-abcd10000000"],
            &["context"],
        ],
    );

    // And the shortening is real rather than a retreat to the full id. Six hex
    // characters is exactly what this corpus forces — `abcd` and `abcd1` both
    // collide, `abcd10`/`abcd1f`/`abcd20` do not — so a seventh would mean the
    // rule stopped measuring, and twelve would mean it gave up measuring.
    let out = r.ank("claude-code@ank", &["find"]);
    assert!(
        stdout(&out).contains("TASK-abcd10  ") && stdout(&out).contains("TASK-abcd20  "),
        "six hex characters is what this corpus forces, no more: {}",
        stdout(&out)
    );
}

// ---------------------------------------------------------------------------
// A criterion proved wrong in part, recorded and never edited (§3)
// ---------------------------------------------------------------------------

/// The record is a log entry, and it changes nothing the freeze verifies.
///
/// Through the binary because that is what the rule is about: `claim` freezes
/// by hash, `log` appends the record, `done` verifies against the very same
/// hash and succeeds, and `check` names the record on the finished task. A unit
/// test on the recognition alone would prove the opening parses and leave every
/// one of those four joins untested.
///
/// The assertion that carries the design is the one in the middle: the entity
/// file is byte for byte what it was before the record was written. The record
/// lives in `.ank/log/<ID>.md`, so the file the freeze exists to make
/// observable is never opened to write it — which is why a field was refused
/// and the log kept.
#[test]
fn a_discrepancy_is_recorded_without_touching_the_frozen_criterion() {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    let sha = r.head();

    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let frozen = r.task_text(ID);

    let record = "discrepancy: the criterion assumes src/main.rs is generated, \
                  and it is written by hand";
    let out = r.ank("claude-code@ank", &["log", ID, record]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));

    assert!(
        r.log_text(ID).contains(record),
        "the record is a line of the log: {}",
        r.log_text(ID)
    );
    assert_eq!(
        r.task_text(ID),
        frozen,
        "recording it writes no byte of the file carrying done_criteria"
    );

    // The criterion still verifies by the hash the claim froze, so the record
    // weakens nothing `done` checks.
    let out = r.ank(
        "claude-code@ank",
        &["done", ID, "--proof", &format!("commit:{sha}")],
    );
    assert_eq!(code(&out), 0, "{}{}", stdout(&out), stderr(&out));

    let out = r.ank("claude-code@ank", &["check"]);
    let said = stdout(&out);
    assert_eq!(
        code(&out),
        0,
        "a judgement somebody wrote down is not a corpus fault: {said}"
    );
    assert!(
        said.contains(&format!("signal: {ID}: discrepancy recorded")),
        "check names the record on the task: {said}"
    );
    assert!(
        said.contains("the criterion assumes src/main.rs is generated"),
        "and lists the entry under the finding: {said}"
    );
}

/// The same record, read by a caller rather than by a reader, and the entry it
/// carries arrives as data rather than as a sentence to split.
#[test]
fn a_recorded_discrepancy_reaches_json_as_a_signal_with_its_entries() {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    assert_eq!(
        code(&r.ank(
            "claude-code@ank",
            &[
                "log",
                ID,
                "discrepancy: one clause of four is unsatisfiable"
            ]
        )),
        0
    );

    let json = stdout(&r.ank("claude-code@ank", &["check", "--json"]));
    assert!(
        json.contains("\"level\":\"signal\"") && json.contains("discrepancy recorded"),
        "{json}"
    );
    assert!(
        json.contains("one clause of four is unsatisfiable"),
        "the entry is a note line and not prose folded into the message: {json}"
    );
}

/// A log the reading cannot parse is said out loud, never counted as no record.
///
/// The failure this guards is the quiet one: a malformed line would make the
/// parse return nothing, and a check reporting no discrepancy because it read
/// no log is a check that stopped looking without saying so.
#[test]
fn a_log_check_cannot_read_is_reported_rather_than_read_as_empty() {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_task(ID, Some("A verifiable criterion."));
    std::fs::create_dir_all(r.0.join(".ank/log")).unwrap();
    std::fs::write(
        r.0.join(".ank/log").join(format!("{ID}.md")),
        "- 2026-07-28T00:00Z claude-code@ank \u{2014} an entry\nnot an entry at all\n",
    )
    .unwrap();

    let out = r.ank("claude-code@ank", &["check"]);
    let said = stdout(&out);
    assert_eq!(
        code(&out),
        0,
        "an unreadable log is not a corpus fault: {said}"
    );
    assert!(
        said.contains(&format!("signal: {ID}: log unreadable")) && said.contains("line 2"),
        "the line is named, because the file grows: {said}"
    );
}

// ---------------------------------------------------------------------------
// The JSON goldens (TASK-2c12b027f805)
// ---------------------------------------------------------------------------

/// Where a caller's parser meets this binary, pinned one file per verb.
///
/// Captured from the process and never from a function. A fixture compared
/// against what an emitter returns proves the emitter, and what §4 promises is
/// what leaves the process — the same distinction this whole file exists for.
///
/// Volatile values are named rather than kept: an instant, a commit, an
/// identifier the binary generated. A golden that changes every run pins
/// nothing, and everything outside those three is compared byte for byte.
///
/// Bless a new or deliberately changed shape with
/// `ANK_BLESS_GOLDEN=1 cargo test --test cli -- json_golden`, and read the diff
/// before committing it: that diff is the contract changing.
const GOLDEN_DIR: &str = "tests/golden-json";

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

/// `dddd-dd-ddTdd:dd:ddZ`, the only instant the format writes.
fn timestamp_len_at(b: &[char], i: usize) -> Option<usize> {
    const SHAPE: &str = "nnnn-nn-nnTnn:nn:nnZ";
    if i + SHAPE.len() > b.len() {
        return None;
    }
    for (k, want) in SHAPE.chars().enumerate() {
        let got = b[i + k];
        let ok = match want {
            'n' => got.is_ascii_digit(),
            c => got == c,
        };
        if !ok {
            return None;
        }
    }
    Some(SHAPE.len())
}

fn redact(s: &str) -> String {
    let b: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if let Some(n) = timestamp_len_at(&b, i) {
            out.push_str("<TIME>");
            i += n;
            continue;
        }
        if b[i].is_ascii_hexdigit() {
            let mut j = i;
            while j < b.len() && b[j].is_ascii_hexdigit() {
                j += 1;
            }
            let word: String = b[i..j].iter().collect();
            let whole =
                (i == 0 || !is_word_char(b[i - 1])) && (j == b.len() || !is_word_char(b[j]));
            // A seeded identifier is deterministic and worth pinning; the
            // zero prefix is what the fixtures in this file use for one. Only
            // what the binary minted itself is named away.
            if whole && word.len() == 40 {
                out.push_str("<SHA>");
            } else if whole && word.len() == 12 && !word.starts_with("0000") {
                out.push_str("<HEX>");
            } else {
                out.push_str(&word);
            }
            i = j;
            continue;
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

fn golden(name: &str, actual: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(GOLDEN_DIR)
        .join(format!("{name}.json"));
    let actual = format!("{}\n", redact(actual.trim_end_matches('\n')));
    if std::env::var_os("ANK_BLESS_GOLDEN").is_some() {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, actual.as_bytes()).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("no golden for {name} at {}: {e}", path.display()))
        .replace("\r\n", "\n");
    assert_eq!(
        actual, expected,
        "the --json document of `{name}` is not what the golden pins.\n\
         If the contract really changed, bless it and read the diff:\n  \
         ANK_BLESS_GOLDEN=1 cargo test --test cli -- json_golden"
    );
}

/// A corpus with one of everything, so that a document has something to carry.
///
/// Seeded with the zero-prefixed identifiers this file uses, and with the fixed
/// instants `seed_*` write, so that what survives redaction is the shape and
/// not the run.
fn golden_repo() -> Repo {
    let r = Repo::new();
    // A budget the constraint above can be measured against. Half of it is the
    // limit an over-constrained perimeter passes, and 400 is what makes one
    // constraint of ordinary length enough: a fixture needs the finding, and a
    // corpus needing five constraints to produce it would be pinning a pile
    // rather than a shape.
    std::fs::write(
        r.0.join(".ank/config.yml"),
        "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\ncontext_budget: 400\n",
    )
    .unwrap();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();
    r.seed_docs();

    // Written here rather than through the shared `seed_*` helpers, and every
    // field is deliberate. This corpus is the sample the conformance test at the
    // end of this section draws from, so an array left empty here is an element
    // shape whose declaration nothing confronts (TASK-e89613d66284) — and those
    // helpers are shared with two hundred other tests, where the same enrichment
    // would be noise.
    //
    // `human:marie` throughout, and it buys two things. ADR-3877fef1d662 makes
    // an entity written by an agent and read by no human a signal, and this
    // corpus has to carry no signal it did not ask for; and the fixtures are
    // read by clients as examples, where an author naming this suite would be an
    // example of nothing.
    r.seed_golden_adr(GOLDEN_ADR);
    // The one `accept`'s own fixture ratifies. Two ADRs and not one, because
    // the corpus needs a ratified decision for `context.constraints` and
    // `accept` needs an unratified one: over the first it would refuse with
    // code 7, which is the right refusal and the wrong fixture.
    r.seed_golden_adr(GOLDEN_ADR_PROPOSED);
    r.seed_golden_spec(GOLDEN_SPEC);
    // The chain is what `show.blocked_by`, `show.unblocks` and `graph.edges`
    // are: one task blocked by a second and blocking a third, so the document
    // `show ID` returns carries a row on either side.
    r.seed_golden_task(GOLDEN_READY, "A task that blocks", &[]);
    r.seed_golden_task(ID, "Example task", &[GOLDEN_READY]);
    r.seed_golden_task(GOLDEN_BLOCKED, "A task that waits", &[ID]);
    r.seed_golden_task(GOLDEN_UNRELATED, "A task apart", &[]);
    r.seed_golden_log(GOLDEN_LOG, ID);
    // Both kinds about one entity, which is what makes the split visible: the
    // work trace answers with one entry and the machinery with the other.
    r.seed_golden_machinery(GOLDEN_EDIT, ID);

    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    // Ratified for real, and not forged. `context.constraints` carries what
    // binds a perimeter, and only an accepted ADR binds one; an `accepted`
    // status over an anchor written by hand is `altered since ratification`,
    // which is a fault and not a fixture. So the corpus declares a key and
    // `accept` makes the commit the anchor names.
    r.forge_detached_proof(ID);
    r.enable_signing();
    declare_signing_key(&r);

    // `human:jean` and not `human:marie`, who wrote them: an entity ratified by
    // its own author is a signal, and this corpus carries no signal it did not
    // ask for — `check.findings[].charge` is the breakdown of an
    // over-constrained perimeter, and it is only exercised if every finding in
    // the fixture is one that carries a breakdown.
    for id in [GOLDEN_ADR, GOLDEN_SPEC] {
        let out = r.ank("human:jean", &["accept", id]);
        assert_eq!(
            code(&out),
            0,
            "the golden corpus needs a real ratification of {id}: {}",
            stderr(&out)
        );
    }
    r
}

/// The identifiers the golden corpus is built from, zero-prefixed so that
/// [`redact`] leaves them alone: they are seeded and deterministic, where an
/// identifier the binary minted is named away.
/// Long enough to be measured against a budget, and that is the point of its
/// length rather than a taste for prose: `check.findings[].charge` is the
/// breakdown of an over-constrained perimeter, and a perimeter is
/// over-constrained when the characters of constraint binding it pass half of
/// `context_budget`. A one-line `Do not do X.` could only produce one against a
/// budget so small that every other fixture would be reading a corpus nobody
/// would configure.
const GOLDEN_CONSTRAINT: &str = concat!(
    "  Nothing under src/ reaches the network at import time. A module that\n",
    "  opens a socket, reads an environment variable naming a host, or resolves\n",
    "  a name while it is being loaded makes the import order a fact about the\n",
    "  machine rather than about the program, and the failure it produces names\n",
    "  the importer instead of the line that reached out.",
);

const GOLDEN_ADR: &str = "ADR-0000000000ab";
const GOLDEN_ADR_PROPOSED: &str = "ADR-0000000000ba";
const GOLDEN_SPEC: &str = "SPEC-0000000000cd";
/// The one task in this corpus nothing blocks, and therefore the one the
/// writing verbs act on: `claim` refuses a task waiting on an open blocker, so
/// the task `show` is interesting about and the task `claim` can take cannot be
/// the same one.
const GOLDEN_READY: &str = "TASK-000000000002";
const GOLDEN_BLOCKED: &str = "TASK-000000000003";
const GOLDEN_LOG: &str = "LOG-0000000000ef";
const GOLDEN_EDIT: &str = "LOG-0000000000fe";
const GOLDEN_UNRELATED: &str = "TASK-000000000004";

const AGENT: &str = "claude-code/1.0.0";

#[test]
fn json_golden_reading_verbs() {
    let r = golden_repo();
    for (name, args) in [
        ("help", &["help", "--json"][..]),
        ("help-verb", &["help", "claim", "--json"][..]),
        ("find", &["find", "Example", "--json"][..]),
        ("show", &["show", ID, "--json"][..]),
        ("graph", &["graph", "src/**", "--json"][..]),
        ("scope", &["scope", "src/main.rs", "--json"][..]),
        ("check", &["check", "--json"][..]),
        ("review", &["review", "src/**", "--json"][..]),
        ("context", &["context", "src/**", "--json"][..]),
        ("config-read", &["config", "claim_ttl_max", "--json"][..]),
        ("log-read", &["log", ID, "--json"][..]),
    ] {
        let out = r.ank(AGENT, args);
        assert!(
            !stdout(&out).is_empty(),
            "{name} emitted no document: {}",
            stderr(&out)
        );
        golden(name, &stdout(&out));
    }
}

/// `status`, which is the one reading verb that needs a corpus of its own.
///
/// `also_held` and `elsewhere` are facts about the coordination plane, and the
/// first of them needs two live claims under the caller's own identity. A claim
/// held by `AGENT` in the shared corpus would put `context` into execution mode
/// and re-value every fixture around a state only this one is about — and the
/// orientation shape, which is what a first reader of `context` needs, would
/// stop being exercised at all.
#[test]
fn json_golden_status() {
    let r = golden_repo();
    r.ank(AGENT, &["claim", GOLDEN_READY]);
    r.forge_claim(GOLDEN_BLOCKED, GOLDEN_READY);
    let out = r.ank("human:marie", &["claim", GOLDEN_UNRELATED]);
    assert_eq!(code(&out), 0, "a claim by somebody else: {}", stderr(&out));
    let out = r.ank(AGENT, &["status", "--json"]);
    assert_eq!(code(&out), 0, "status: {}", stderr(&out));
    golden("status", &stdout(&out));
}

/// The verbs that write. One fixture each: a document that depended on the
/// verb before it would pin the order rather than the shape.
#[test]
fn json_golden_writing_verbs() {
    // new
    let r = golden_repo();
    let out = r.ank(
        AGENT,
        &[
            "new",
            "task",
            "--title",
            "A new task",
            "--scope",
            "src/**",
            "--criteria",
            "It is done when it is done.",
            "--json",
        ],
    );
    golden("new", &stdout(&out));

    // claim, then the verbs that need a live claim
    let r = golden_repo();
    let out = r.ank(AGENT, &["claim", GOLDEN_READY, "--json"]);
    golden("claim", &stdout(&out));
    let out = r.ank(AGENT, &["log", "what I learned", "--json"]);
    golden("log-write", &stdout(&out));
    let out = r.ank(
        AGENT,
        &["release", "--reason", "the criterion is wrong", "--json"],
    );
    golden("release", &stdout(&out));

    // done, over a claim and a proof the caller holds
    let r = golden_repo();
    let head = r.head();
    r.ank(AGENT, &["claim", GOLDEN_READY]);
    let out = r.ank(
        AGENT,
        &["done", "--proof", &format!("commit:{head}"), "--json"],
    );
    golden("done", &stdout(&out));

    // attest, on the task that proof now names
    let out = r.ank(
        AGENT,
        &["attest", GOLDEN_READY, "--proof", "test:12345", "--json"],
    );
    golden("attest", &stdout(&out));

    // amend and close, on a corpus that never claimed. A fourth task, because
    // the corpus already blocks ID on GOLDEN_BLOCKER: a blocker the task
    // already carries is nothing to amend, and the fixture would pin an empty
    // document.
    let r = golden_repo();
    let out = r.ank(
        AGENT,
        &["amend", ID, "--blocked-by", GOLDEN_UNRELATED, "--json"],
    );
    golden("amend", &stdout(&out));
    let out = r.ank(
        AGENT,
        &["close", ID, "--reason", "overtaken by events", "--json"],
    );
    golden("close", &stdout(&out));

    // config, writing
    let r = golden_repo();
    let out = r.ank(AGENT, &["config", "context_budget", "9000", "--json"]);
    golden("config-write", &stdout(&out));
}

/// The four that need an environment of their own: a signature, an editor, a
/// previous layout to migrate, and a directory with no corpus in it yet.
#[test]
fn json_golden_verbs_needing_their_own_environment() {
    // accept, which is the one verb that commits. The corpus already declares a
    // key and already ratified one ADR; this is the other one.
    let r = golden_repo();
    let out = r.ank(AGENT, &["accept", GOLDEN_ADR_PROPOSED, "--json"]);
    assert_eq!(code(&out), 0, "accept: {}", stderr(&out));
    golden("accept", &stdout(&out));

    // edit, with an editor that saves
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let editor = r.editor_saving(EDITED_TASK);
    let out = r.ank_edit(AGENT, &["edit", ID, "--json"], Some(&editor));
    assert_eq!(code(&out), 0, "edit: {}", stderr(&out));
    golden("edit", &stdout(&out));

    // migrate, over the layout that is read and never written
    let r = golden_repo();
    std::fs::create_dir_all(r.0.join(".ank/log")).unwrap();
    std::fs::write(
        r.0.join(".ank/log").join(format!("{ID}.md")),
        "- 2026-07-28T00:00:00Z claude-code@ank \u{2014} an entry\n",
    )
    .unwrap();
    let out = r.ank(AGENT, &["migrate", "--json"]);
    assert_eq!(code(&out), 0, "migrate: {}", stderr(&out));
    golden("migrate", &stdout(&out));

    // init, which refuses --repo by name and so is run from a directory. Two
    // fixtures and not one: the second run is the idempotent case, and the
    // shape it returns is the point — three empty lists and `changed: false`,
    // so a parser reads one document and never two.
    let fresh = fresh_git_dir("golden-init");
    let init_json = |dir: &Path| {
        ank_command()
            .args(["init", "--json"])
            .env("ANK_AGENT", AGENT)
            .current_dir(dir)
            .output()
            .expect("the binary must have been built")
    };
    let out = init_json(&fresh);
    assert_eq!(code(&out), 0, "init: {}", stderr(&out));
    golden("init", &stdout(&out));
    let out = init_json(&fresh);
    assert_eq!(code(&out), 0, "init, again: {}", stderr(&out));
    golden("init-again", &stdout(&out));
    let _ = std::fs::remove_dir_all(&fresh);
}

// ---------------------------------------------------------------------------
// The declared shapes, against what the binary actually printed
// ---------------------------------------------------------------------------

/// Every fixture in `tests/golden-json/`, checked against the shape its verb
/// declares in `ank-contract` (ADR-6fd69efb629c).
///
/// This is what makes the declaration a contract rather than documentation.
/// `help --json` publishes what a client should expect back; without this test
/// nothing would connect that promise to the bytes the binary emits, and the two
/// would drift in the direction that costs the client and not us — a description
/// that is wrong reads exactly like a description that is right.
///
/// **The fixtures are the sample, and their limits are stated rather than
/// papered over.** An array with no rows cannot show the shape of its elements,
/// so a declaration the fixtures reach only through empty arrays goes unverified
/// here; it rests on the builders in the source, and it becomes verified the day
/// one instance carries a row. The limit is named rather than counted, so it
/// cannot quietly grow.
///
/// **The question is about a declaration, not about an instance.** One shape is
/// met once per array the walk descends into — `verbs[].refuses` is one
/// declaration met twenty-two times — and a single row anywhere shows it. So the
/// walk records both sides and subtracts, where it used to record only the empty
/// one and report a shape as unseen because some other instance of it happened
/// to be empty (TASK-fbdf25e30058).
///
/// Parsed with `serde_yaml`, which is already a dependency and reads JSON
/// because YAML 1.2 is a superset of it. §13 spends a dependency only on
/// necessity, and a second parser to read what a parser in the tree already
/// reads is not one.
#[test]
fn every_golden_conforms_to_the_shape_its_verb_declares() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(GOLDEN_DIR);
    let mut checked = 0;
    let mut empty: Vec<String> = Vec::new();
    let mut filled: Vec<String> = Vec::new();

    for entry in std::fs::read_dir(&dir).expect("the goldens must exist") {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        // `config-read`, `help-verb`, `init-again`: the fixture names a call,
        // and the verb is what precedes the dash.
        let verb = stem.split('-').next().unwrap();
        let spec =
            ank_contract::spec_of(verb).unwrap_or_else(|| panic!("{stem}: no verb named {verb}"));

        let text = std::fs::read_to_string(&path).unwrap();
        let doc: serde_yaml::Value = serde_yaml::from_str(&text)
            .unwrap_or_else(|e| panic!("{stem}: the fixture is not readable JSON: {e}"));

        // A verb with several documents conforms if it conforms to one of them.
        // The failure is reported against every candidate, because "it matched
        // none" without saying how is the report that sends a reader guessing.
        let mut failures = Vec::new();
        let mut seen = Arrays::default();
        let matched =
            spec.output
                .iter()
                .any(|shape| match conforms(&doc, shape.fields, "", &mut seen) {
                    Ok(()) => true,
                    Err(e) => {
                        failures.push(format!("  {}: {e}", shape.when.unwrap_or("the document")));
                        false
                    }
                });
        assert!(
            matched,
            "{stem} does not match any shape `{verb}` declares:\n{}",
            failures.join("\n")
        );
        // Named by fixture and with the row indices dropped, so the list says
        // *which* declaration is unexercised rather than which row happened to
        // be empty. `context.log` and `show.log` are two different fields, and a
        // list keyed on the path alone would have hidden one behind the other.
        // Dropping the indices is also what lets the two sides meet:
        // `verbs[0].refuses` and `verbs[7].refuses` are one declaration, and
        // they have to normalise to one key for the second to answer about the
        // first.
        //
        // **The fixture stays in the key, so the subtraction is per document.**
        // A verb with several documents declares several shapes, and a field of
        // one name in two of them is two declarations; `config-read` filling an
        // array would then be read as showing the shape `config-write` declares
        // under that name, which is the conflation the stem is here to prevent.
        // Within one document it is one shape by construction.
        let key = |path: &str| {
            let mut normalised = String::with_capacity(path.len());
            let mut in_index = false;
            for c in path.chars() {
                match c {
                    '[' => {
                        in_index = true;
                        normalised.push('[');
                    }
                    ']' => {
                        in_index = false;
                        normalised.push(']');
                    }
                    _ if in_index => {}
                    _ => normalised.push(c),
                }
            }
            format!("{stem}.{normalised}")
        };
        empty.extend(seen.empty.iter().map(|p| key(p)));
        filled.extend(seen.filled.iter().map(|p| key(p)));
        checked += 1;
    }

    assert_eq!(checked, 26, "one fixture per document the surface returns");
    // **A declaration is unexercised when no instance of it anywhere carries a
    // row**, which is the reading this list is about (TASK-fbdf25e30058). It
    // used to be one instance at a time: a path went on the list every time the
    // walk met it empty, so `help.verbs[].refuses` was reported as unseen
    // because `status` declares no refusal, while the twenty-one other verbs
    // carried rows in that same document and the walk parsed those rows itself
    // one line further down. Subtracting what was filled from what was empty is
    // what turns instances back into shapes.
    //
    // Named rather than counted, and pinned rather than merely reported. A shape
    // still on this list rests on the builders in the source and on nothing this
    // test can see, so writing the list out means a fixture that starts
    // exercising one turns this red and has to be acknowledged, instead of
    // quietly shrinking a number nobody watches.
    //
    // **It is empty, and getting there took two tasks.** Thirteen of the
    // fourteen shapes it once carried were fixture problems, fixed by seeding a
    // golden corpus that holds a blocking chain, a spec and an ADR over the
    // perimeter it asks about, a log entry, a detached proof, two claims under
    // one identity and one under another, and a constraint heavy enough for the
    // budget to charge it (TASK-e89613d66284). The fourteenth was
    // `help.verbs[].refuses`, and it was two faults wearing one symptom: six
    // verbs performed refusals they declared nowhere, which is the table's
    // problem and was fixed there (TASK-106dccc7f71c), and the last empty array
    // is `status`, which refuses on nothing and whose empty declaration is a
    // fact about the verb rather than a gap. Nothing was invented for it; the
    // walk was taught to read shapes.
    empty.sort();
    empty.dedup();
    filled.sort();
    filled.dedup();
    let unverified: Vec<&String> = empty.iter().filter(|p| !filled.contains(p)).collect();
    assert!(
        unverified.is_empty(),
        "the fixtures reach no instance of these declarations, so nothing here \
         verifies them: {unverified:?}"
    );
}

/// Every array path the walk met, split by whether that instance carried a row.
///
/// Two lists and not one, because the question asked of them is about a shape
/// and the walk meets instances: `help.verbs[].refuses` is one declaration met
/// twenty-two times, and it is exercised if any one of those instances has a row
/// in it. Subtracting `filled` from `empty` is what turns instances back into
/// shapes; keeping only `empty` answered a question nobody asked
/// (TASK-fbdf25e30058).
#[derive(Default)]
struct Arrays {
    empty: Vec<String>,
    filled: Vec<String>,
}

/// One document against one declared shape, recursively.
///
/// The keys are compared **in order and in full**: a document that gained a
/// field the shape does not declare is as wrong as one that lost a field it
/// does, because ADR-6fd69efb629c's promise is about both directions — within a
/// version a document may gain a field only by declaring it first.
fn conforms(
    value: &serde_yaml::Value,
    fields: &[ank_contract::shape::Field],
    path: &str,
    arrays: &mut Arrays,
) -> Result<(), String> {
    use ank_contract::shape::Type;
    use serde_yaml::Value;

    let map = match value {
        Value::Mapping(m) => m,
        other => {
            return Err(format!(
                "{path}: expected an object, found {}",
                kind_of(other)
            ))
        }
    };

    // `contract` is on every document and on no declaration: the rendering adds
    // it, so the check adds it too. **Top level only** — the version describes
    // the document, not every object inside it, which is why `Obj::document`
    // seeds it and `Obj::new` does not.
    let expected: Vec<String> = path
        .is_empty()
        .then(|| "contract".to_string())
        .into_iter()
        .chain(fields.iter().map(|f| f.name.to_string()))
        .collect();
    let found: Vec<String> = map
        .keys()
        .map(|k| k.as_str().unwrap_or("<not a string>").to_string())
        .collect();
    if found != expected {
        return Err(format!(
            "{path}: keys are {found:?}, the shape declares {expected:?}"
        ));
    }

    for field in fields {
        let here = match path.is_empty() {
            true => field.name.to_string(),
            false => format!("{path}.{}", field.name),
        };
        let v = &map[field.name];
        if v.is_null() {
            if !field.nullable {
                return Err(format!("{here}: null, and the shape does not allow it"));
            }
            continue;
        }
        match field.ty {
            Type::Str if v.as_str().is_some() => {}
            Type::Num if v.is_number() => {}
            Type::Bool if v.as_bool().is_some() => {}
            Type::Strings | Type::Array(_) if v.as_sequence().is_some() => {}
            Type::Object(inner) => conforms(v, inner, &here, arrays)?,
            _ => {
                return Err(format!(
                    "{here}: declared {}, found {}",
                    field.ty.name(),
                    kind_of(v)
                ))
            }
        }
        if let Type::Strings = field.ty {
            for (i, item) in v.as_sequence().unwrap().iter().enumerate() {
                if item.as_str().is_none() {
                    return Err(format!(
                        "{here}[{i}]: declared string, found {}",
                        kind_of(item)
                    ));
                }
            }
        }
        if let Type::Array(inner) = field.ty {
            let rows = v.as_sequence().unwrap();
            // Both answers are recorded, never only the empty one: the question
            // upstream is about the *shape*, and an instance carrying rows is
            // what shows it. Reporting the empty side alone made an exercised
            // shape read as unexercised because some other instance of it
            // happened to be empty (TASK-fbdf25e30058).
            match rows.is_empty() {
                true => arrays.empty.push(here.clone()),
                false => arrays.filled.push(here.clone()),
            }
            for (i, row) in rows.iter().enumerate() {
                conforms(row, inner, &format!("{here}[{i}]"), arrays)?;
            }
        }
    }
    Ok(())
}

fn kind_of(v: &serde_yaml::Value) -> &'static str {
    match v {
        serde_yaml::Value::Null => "null",
        serde_yaml::Value::Bool(_) => "a boolean",
        serde_yaml::Value::Number(_) => "a number",
        serde_yaml::Value::String(_) => "a string",
        serde_yaml::Value::Sequence(_) => "an array",
        serde_yaml::Value::Mapping(_) => "an object",
        serde_yaml::Value::Tagged(_) => "a tagged value",
    }
}

// ---------------------------------------------------------------------------
// Two roots: the corpus, and the tree it is anchored to (ADR-9e56318631f3)
// ---------------------------------------------------------------------------
//
// Every test below drives the binary with both halves of the address, `--repo`
// naming the corpus and `--worktree` naming the tree it is anchored to, and
// each one pins one of the four questions the decision sorts. They are written
// as pairs wherever the pair is affordable: the anchored run beside the same
// run without the flag, because "the flag moved it" is the claim, and only the
// second half of a pair can establish it.

/// A tree of code with no corpus in it: a git repository, two tracked files,
/// one commit. `only-in-the-code` is the marker a verifier looks for, and its
/// name is the whole of the assertion it carries.
fn code_tree(tag: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let p = std::env::temp_dir().join(format!(
        "ank-cli-code-{}-{}-{}",
        tag,
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&p);
    std::fs::create_dir_all(p.join("src")).unwrap();
    std::fs::write(p.join("src/main.rs"), "fn main() {}\n").unwrap();
    std::fs::write(p.join("only-in-the-code"), "x\n").unwrap();
    for args in [
        vec!["init", "-q", "-b", "main"],
        vec!["config", "user.email", "test@ank.local"],
        vec!["config", "user.name", "Test"],
        vec!["config", "commit.gpgsign", "false"],
        vec!["add", "-A"],
        vec!["commit", "-qm", "code"],
    ] {
        let out = git_command(&p).args(&args).output().unwrap();
        assert!(out.status.success(), "{args:?}: {}", stderr(&out));
    }
    p
}

/// The invocation with both halves of the address, run from neither directory
/// so that nothing about the answer can come from the current one.
fn ank_anchored(corpus: &Path, worktree: &Path, agent: &str, args: &[&str]) -> Output {
    ank_command()
        .args(args)
        .arg("--repo")
        .arg(corpus)
        .arg("--worktree")
        .arg(worktree)
        .env("ANK_AGENT", agent)
        .current_dir(std::env::temp_dir())
        .output()
        .expect("the binary must have been built")
}

fn both_streams(out: &Output) -> String {
    format!("{}{}", stdout(out), stderr(out))
}

/// The glob is confronted with the work tree, and the corpus is not a stand-in
/// for it.
///
/// The pair is the point. Unanchored, `src/**` is held against a directory that
/// holds `.ank/` and nothing else, and `check` reports it as work not started
/// or a typo, which is what a detached corpus would say about every scope it
/// carries. Anchored, the same glob meets the tree that has the file.
#[test]
fn a_scope_is_confronted_with_the_work_tree() {
    let r = Repo::new();
    r.seed_task_scoped(ID, "src/**");
    let code_at = code_tree("scope");

    let alone = r.ank("claude-code@ank", &["check"]);
    assert!(
        both_streams(&alone).contains("matches no file yet"),
        "the corpus has no src/, so unanchored the scope is dead: {}",
        both_streams(&alone)
    );

    let anchored = ank_anchored(&r.0, &code_at, "claude-code@ank", &["check"]);
    assert!(
        !both_streams(&anchored).contains("matches no file yet"),
        "anchored to the tree that holds src/main.rs, the scope is alive: {}",
        both_streams(&anchored)
    );
}

/// An absolute path of the work tree is refused, and named back relative to it.
///
/// The refusal itself does not move: `normalize_path` refuses an absolute path
/// whatever the roots are. What moves is the one thing the caller can act on,
/// the hint, which can only name the relative form if it knows which tree the
/// path was absolute in.
#[test]
fn an_absolute_path_is_named_back_relative_to_the_work_tree() {
    let r = Repo::new();
    r.seed_task_scoped(ID, "src/**");
    let code_at = code_tree("path");
    let abs = code_at.join("src").join("main.rs");
    let abs = abs.to_str().unwrap();

    let anchored = ank_anchored(&r.0, &code_at, "claude-code@ank", &["context", abs]);
    assert_eq!(code(&anchored), 1, "{}", both_streams(&anchored));
    assert!(
        stderr(&anchored).contains("ank context src/main.rs"),
        "the hint names the path relative to the work tree: {}",
        stderr(&anchored)
    );

    let alone = r.ank("claude-code@ank", &["context", abs]);
    assert!(
        stderr(&alone).contains("<inside the repository>"),
        "unanchored, the corpus cannot recognise a path of the code: {}",
        stderr(&alone)
    );
}

/// A verifier runs in the work tree, where the code it tests is.
#[test]
fn a_verifier_runs_in_the_work_tree() {
    let r = Repo::new().with_verifiers("verifiers:\n  here:\n    run: test -f only-in-the-code\n");
    r.seed_task_with(ID, Some("A criterion."), &["here"]);
    let code_at = code_tree("verify");

    let claimed = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&claimed), 0, "{}", both_streams(&claimed));
    let alone = r.ank("claude-code@ank", &["done"]);
    assert_eq!(
        code(&alone),
        5,
        "unanchored the verifier runs in the corpus, where the file is not: {}",
        both_streams(&alone)
    );

    let anchored = ank_anchored(&r.0, &code_at, "claude-code@ank", &["done"]);
    assert_eq!(
        code(&anchored),
        0,
        "anchored, the same verifier runs where the file is: {}",
        both_streams(&anchored)
    );
}

/// A `commit:` proof names a commit of the code, is looked up there, and the
/// ref it produces is written in the corpus.
///
/// The two halves of ADR-9e56318631f3 meet in this one verb and pull in
/// opposite directions: the proof is read from the work tree, the record of it
/// is written to the corpus. `for-each-ref` on both repositories says so.
#[test]
fn a_commit_proof_is_read_in_the_work_tree_and_recorded_in_the_corpus() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));
    // The completion ref records the corpus commit and branch it was written
    // from, which is a fact about the coordination plane and stays there
    // (ADR-9e56318631f3 assigns refs/ank/* to the corpus). A corpus with no
    // commit therefore has nothing to record, so the fixture gives it one.
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "corpus"]);
    let code_at = code_tree("proof");
    let head = String::from_utf8_lossy(
        &git_command(&code_at)
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .trim()
    .to_string();
    let proof = format!("commit:{head}");

    let out = ank_anchored(&r.0, &code_at, "claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    let out = ank_anchored(
        &r.0,
        &code_at,
        "claude-code@ank",
        &["done", "--proof", &proof],
    );
    assert_eq!(
        code(&out),
        0,
        "the commit is in the work tree, which is where it is looked up: {}",
        both_streams(&out)
    );

    assert!(
        r.git(&["for-each-ref", "refs/ank"]).contains(ID),
        "the record of the proof is written in the corpus"
    );
    let in_the_code = String::from_utf8_lossy(
        &git_command(&code_at)
            .args(["for-each-ref", "refs/ank"])
            .output()
            .unwrap()
            .stdout,
    )
    .to_string();
    assert!(
        in_the_code.trim().is_empty(),
        "and the work tree keeps no ank ref at all: {in_the_code}"
    );
}

/// The same proof, unanchored, cannot be looked up at all.
#[test]
fn a_commit_of_the_code_is_unfindable_from_the_corpus_alone() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));
    // The completion ref records the corpus commit and branch it was written
    // from, which is a fact about the coordination plane and stays there
    // (ADR-9e56318631f3 assigns refs/ank/* to the corpus). A corpus with no
    // commit therefore has nothing to record, so the fixture gives it one.
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "corpus"]);
    let code_at = code_tree("unfindable");
    let head = String::from_utf8_lossy(
        &git_command(&code_at)
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .trim()
    .to_string();

    let out = r.ank("claude-code@ank", &["claim", ID]);
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    let out = r.ank(
        "claude-code@ank",
        &["done", "--proof", &format!("commit:{head}")],
    );
    assert_eq!(code(&out), 5, "{}", both_streams(&out));
    assert!(
        stderr(&out).contains("not found in this repository"),
        "{}",
        stderr(&out)
    );
}

/// A work tree that is no git repository is refused by name, and the corpus is
/// never quietly used instead.
#[test]
fn a_work_tree_that_is_no_repository_is_refused_by_name() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));
    let plain = std::env::temp_dir().join(format!("ank-cli-plain-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&plain);
    std::fs::create_dir_all(&plain).unwrap();

    let out = ank_anchored(&r.0, &plain, "claude-code@ank", &["claim", ID]);
    assert_eq!(
        code(&out),
        0,
        "claim reaches only the corpus, and asks nothing of the work tree: {}",
        both_streams(&out)
    );

    let out = ank_anchored(
        &r.0,
        &plain,
        "claude-code@ank",
        &["done", "--proof", "commit:abc"],
    );
    assert_eq!(code(&out), 9, "{}", both_streams(&out));
    let err = stderr(&out);
    assert!(err.contains("work tree"), "{err}");
    let named = plain.file_name().unwrap().to_string_lossy().to_string();
    assert!(
        err.contains(&named),
        "the refusal names which root is the problem: {err}"
    );
    let _ = std::fs::remove_dir_all(&plain);
}

/// `--worktree` naming something that is not a directory is refused before any
/// verb runs.
#[test]
fn a_work_tree_that_is_not_a_directory_is_refused() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));
    let out = ank_anchored(
        &r.0,
        Path::new("no/such/tree"),
        "claude-code@ank",
        &["check"],
    );
    assert_eq!(code(&out), 1, "{}", both_streams(&out));
    assert!(
        stderr(&out).contains("is not a directory"),
        "{}",
        stderr(&out)
    );
}

/// The degenerate case, pinned: naming the corpus as its own work tree is what
/// every invocation without the flag already does, byte for byte.
#[test]
fn the_corpus_named_as_its_own_work_tree_answers_identically() {
    let r = Repo::new();
    r.seed_task_scoped(ID, "src/**");
    let alone = r.ank("claude-code@ank", &["check"]);
    let root = r.0.clone();
    let named = ank_anchored(&r.0, &root, "claude-code@ank", &["check"]);
    assert_eq!(stdout(&alone), stdout(&named));
    assert_eq!(stderr(&alone), stderr(&named));
    assert_eq!(code(&alone), code(&named));
}

// ---------------------------------------------------------------------------
// The work trace and the machinery (ADR-16813b3bcf37, TASK-027a429aad2e)
// ---------------------------------------------------------------------------

/// One entry about `about`, written by hand because no verb writes machinery
/// yet: the verbs that will are TASK-3c12e0ced2c0. `records` absent is a work
/// entry, which is what every entry written before the field existed is.
fn seed_entry(r: &Repo, id: &str, about: &str, seq: u64, title: &str, records: Option<&str>) {
    let records = records
        .map(|v| format!("records: {v}\n"))
        .unwrap_or_default();
    std::fs::write(
        r.0.join(".ank/entities").join(format!("{id}.md")),
        format!(
            "---\nid: {id}\ntype: log\ntitle: {title}\n\
             created: 2026-07-29T00:00:0{seq}Z\nauthor: human:marie\n\
             scope:\n  - src/**\nabout: {about}\n\
             seq: {seq}\n{records}schema: 4\nversion: 1\n---\n\nThe entry itself.\n"
        ),
    )
    .unwrap();
}

/// The work trace holds what a holder wrote, and nothing else.
///
/// The count is half the assertion and not a detail. `ank log` is what an agent
/// reads before repeating what a previous holder already tried, and an entity
/// edited eight times answering "8 of 8" has already told that reader something
/// false about how much there is to learn.
#[test]
fn a_machinery_entry_is_in_neither_the_work_trace_nor_its_count() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));
    seed_entry(
        &r,
        "LOG-00000000ab01",
        ID,
        0,
        "what the last holder learned",
        None,
    );
    seed_entry(
        &r,
        "LOG-00000000ab02",
        ID,
        1,
        "constraint and body rewritten, was 6f1d9c04a7b2",
        Some("edit"),
    );

    let shown = r.ank("claude-code@ank", &["show", ID]);
    assert_eq!(code(&shown), 0, "{}", both_streams(&shown));
    let text = stdout(&shown);
    let (trace, edits) = text
        .split_once("EDITS")
        .expect("the machinery has a section of its own");
    assert!(
        trace.contains("what the last holder learned"),
        "the work trace holds what a holder wrote: {trace}"
    );
    assert!(
        !trace.contains("was 6f1d9c04a7b2"),
        "and does not hold the machinery: {trace}"
    );
    assert!(
        edits.contains("was 6f1d9c04a7b2"),
        "which is printed under it instead: {edits}"
    );
    assert!(
        trace.contains("LOG (1 of 1)"),
        "the count is the trace's, not the corpus's: {trace}"
    );

    let logged = r.ank("claude-code@ank", &["log", ID]);
    assert_eq!(code(&logged), 0, "{}", both_streams(&logged));
    let text = stdout(&logged);
    let (trace, edits) = text
        .split_once("EDITS")
        .expect("the machinery has a section of its own here too");
    assert!(trace.contains("what the last holder learned"), "{trace}");
    assert!(!trace.contains("was 6f1d9c04a7b2"), "{trace}");
    assert!(edits.contains("was 6f1d9c04a7b2"), "{edits}");
}

/// A parser reads it, `check` judges it.
///
/// The direction matters: a value refused at parse time would make the entry
/// disappear from a corpus written by a newer build, which is the failure the
/// reader range exists to prevent. What a reader is told is that this build
/// does not know the word, never that the word is wrong.
#[test]
fn an_unknown_records_value_is_a_finding_and_never_a_parse_error() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));
    seed_entry(
        &r,
        "LOG-00000000ab03",
        ID,
        0,
        "something a later release writes",
        Some("rename"),
    );

    let checked = r.ank("claude-code@ank", &["check"]);
    assert_eq!(
        code(&checked),
        0,
        "an unknown word is a signal and never a fault: {}",
        both_streams(&checked)
    );
    let said = both_streams(&checked);
    assert!(
        said.contains("LOG-00000000ab03 records 'rename'"),
        "the finding names the entry and the word: {said}"
    );

    // Read, not refused: the entry is still there, and it is machinery, because
    // a word this build does not know is still a word.
    let shown = r.ank("claude-code@ank", &["show", ID]);
    assert_eq!(code(&shown), 0, "{}", both_streams(&shown));
    assert!(
        stdout(&shown).contains("something a later release writes"),
        "{}",
        stdout(&shown)
    );
}

/// An entity with no machinery reads exactly as it did before the field
/// existed, section and all.
#[test]
fn an_entity_with_no_machinery_grows_no_section() {
    let r = Repo::new();
    r.seed_task(ID, Some("A criterion."));
    seed_entry(
        &r,
        "LOG-00000000ab04",
        ID,
        0,
        "what the last holder learned",
        None,
    );

    let shown = r.ank("claude-code@ank", &["show", ID]);
    assert_eq!(code(&shown), 0, "{}", both_streams(&shown));
    assert!(!stdout(&shown).contains("EDITS"), "{}", stdout(&shown));
    assert!(
        stdout(&shown).contains("LOG (1 of 1)"),
        "{}",
        stdout(&shown)
    );
}

// ---------------------------------------------------------------------------
// A write of content accounts for the version it moved
// (ADR-16813b3bcf37, TASK-3c12e0ced2c0)
// ---------------------------------------------------------------------------
//
// Through the binary throughout, and the criterion says so for a reason that
// outlives it: what these tests are about is a file written beside another
// file, by a verb, in the order the two writes happen.

/// The hash a machinery entry must carry for a state, computed the way any
/// reader holding that revision would.
///
/// Over the entity and not over the bytes, which is the doctrine every other
/// freeze in this corpus follows: a ratification anchors `constraint` and
/// `scope`, a claim anchors `done_criteria`, and none of them hashes a file.
fn replaced_hash_of(text: &str) -> String {
    ank_core::freeze_hash_short(&ank_core::serialize_entity(
        &ank_core::parse_entity(text).expect("the fixture parses"),
    ))
}

/// The three verbs ADR-16813b3bcf37 names, and the two doors `edit` has.
///
/// One test over the four because the property is one property: whichever door
/// the write came through, the entity ends up accounting for the version it
/// moved, in one grammar. Split four ways it would be four chances to write the
/// message four ways.
#[test]
fn every_door_that_changes_content_writes_one_entry_that_accounts_for_it() {
    // --- edit, on the path that names its field -----------------------------
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let was = r.task_text(ID);

    let out = r.ank_edit(
        "claude-code@ank",
        &["edit", ID, "--title", "A better title"],
        None,
    );
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    let entries = r.machinery_of(ID);
    assert_eq!(entries.len(), 1, "one write, one entry: {entries:?}");
    assert_eq!(
        entries[0],
        format!(
            "title (version 1 to 2, replaced {})",
            replaced_hash_of(&was)
        ),
        "the fields, the transition and the state replaced"
    );

    // --- edit, on the path that opens $EDITOR -------------------------------
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let was = r.task_text(ID);
    let editor = r.editor_saving(EDITED_TASK);

    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some(&editor));
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    let entries = r.machinery_of(ID);
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(
        entries[0],
        format!(
            "title, body (version 1 to 2, replaced {})",
            replaced_hash_of(&was)
        ),
        "the two paths write one grammar, not two"
    );

    // --- amend --------------------------------------------------------------
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let was = r.task_text(ID);

    let out = r.ank("claude-code@ank", &["amend", ID, "--scope", "docs/**"]);
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    let entries = r.machinery_of(ID);
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(
        entries[0],
        format!(
            "+scope docs/** (version 1 to 2, replaced {})",
            replaced_hash_of(&was)
        )
    );
    // And the work trace never sees it, which is the half TASK-027a429aad2e
    // bought: `ank log` is what an agent reads before repeating what a previous
    // holder tried, and an amend is not something a previous holder learned.
    let logged = r.ank("claude-code@ank", &["log", ID]);
    let said = stdout(&logged);
    let (trace, edits) = said.split_once("EDITS").expect("a section of its own");
    assert!(!trace.contains("+scope docs/**"), "{trace}");
    assert!(edits.contains("+scope docs/**"), "{edits}");

    // --- claim --criteria ---------------------------------------------------
    let r = Repo::new();
    r.seed_task(ID, None);
    let was = r.task_text(ID);

    let out = r.ank(
        "claude-code@ank",
        &["claim", ID, "--criteria", "A verifiable criterion."],
    );
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    let entries = r.machinery_of(ID);
    assert_eq!(entries.len(), 1, "{entries:?}");
    assert_eq!(
        entries[0],
        format!(
            "done_criteria, criteria_by (version 1 to 2, replaced {})",
            replaced_hash_of(&was)
        ),
        "the criterion the whole authority model then rests on"
    );
}

/// A claim that writes no criterion moves `status` and nothing else, and a
/// status transition has records of its own.
#[test]
fn a_status_transition_writes_none_of_this() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task(ID, Some("A verifiable criterion."));

    // The claim: a transition, and the claim ref is what records it.
    assert_eq!(code(&r.ank("claude-code@ank", &["claim", ID])), 0);
    assert!(
        r.machinery_of(ID).is_empty(),
        "a plain claim is a transition: {:?}",
        r.machinery_of(ID)
    );

    // And the other direction, which the criterion names: `done` writes a proof
    // and a completion ref, and tracing it again would say nothing the corpus
    // does not already say.
    let head = r.head();
    let out = r.ank(
        "claude-code@ank",
        &["done", "--proof", &format!("commit:{head}")],
    );
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    assert!(
        r.task_text(ID).contains("status: done"),
        "{}",
        r.task_text(ID)
    );
    assert!(
        r.machinery_of(ID).is_empty(),
        "done writes a proof, never machinery: {:?}",
        r.machinery_of(ID)
    );
    // And the work trace is untouched by any of it: `done` still logs what it
    // did, where a holder reads it.
    assert!(
        r.log_text(ID).contains("done, proof commit:"),
        "{}",
        r.log_text(ID)
    );
}

/// Nothing moved, nothing written. A version is what an entry accounts for, so
/// a call that moved none owes none.
#[test]
fn a_verb_that_changes_nothing_writes_no_entry() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let before = r.task_text(ID);

    // The editor saved the file back exactly as it opened it.
    let editor = r.editor_saving(&before);
    let out = r.ank_edit("claude-code@ank", &["edit", ID], Some(&editor));
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    assert!(stdout(&out).starts_with("unchanged"), "{}", stdout(&out));

    // The named path, handed the title the entity already carries.
    let out = r.ank_edit(
        "claude-code@ank",
        &["edit", ID, "--title", "Example task"],
        None,
    );
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    assert!(stdout(&out).starts_with("unchanged"), "{}", stdout(&out));

    // And an amend asking for what is already there is refused outright.
    let out = r.ank("claude-code@ank", &["amend", ID, "--scope", "src/**"]);
    assert_ne!(code(&out), 0, "{}", both_streams(&out));

    assert_eq!(r.task_text(ID), before, "no write, so no version moved");
    assert!(
        r.machinery_of(ID).is_empty(),
        "and nothing to account for: {:?}",
        r.machinery_of(ID)
    );
}

/// The end-to-end clause: created through the binary, edited twice through it,
/// read back through it.
#[test]
fn an_entity_edited_twice_answers_log_with_both_entries_in_order() {
    let r = Repo::new();
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
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    let id = stdout(&out)
        .split_whitespace()
        .nth(1)
        .expect("created <id> <title>")
        .to_string();

    // The state each edit replaces, captured before it happens: the entry has
    // to be checkable by somebody holding that revision, and this is that
    // somebody.
    let at_one = r.task_text(&id);
    let out = r.ank_edit(
        "claude-code@ank",
        &["edit", &id, "--title", "Rotate the secrets"],
        None,
    );
    assert_eq!(code(&out), 0, "{}", both_streams(&out));

    let at_two = r.task_text(&id);
    let out = r.ank_edit(
        "claude-code@ank",
        &["edit", &id, "--body", "A body written second."],
        None,
    );
    assert_eq!(code(&out), 0, "{}", both_streams(&out));

    assert!(
        r.task_text(&id).contains("version: 3"),
        "two writes on top of the creation: {}",
        r.task_text(&id)
    );

    let entries = r.machinery_of(&id);
    assert_eq!(entries.len(), 2, "one per write: {entries:?}");
    assert_eq!(
        entries[0],
        format!(
            "title (version 1 to 2, replaced {})",
            replaced_hash_of(&at_one)
        )
    );
    assert_eq!(
        entries[1],
        format!(
            "body (version 2 to 3, replaced {})",
            replaced_hash_of(&at_two)
        ),
        "the hash is of the state that write replaced, and of no other"
    );

    // Read back through the verb, which is where a reader meets them: a section
    // of their own, in order, each naming its fields and its transition.
    let logged = r.ank("claude-code@ank", &["log", &id]);
    assert_eq!(code(&logged), 0, "{}", both_streams(&logged));
    let said = stdout(&logged);
    let (_, edits) = said.split_once("EDITS").expect("a section of its own");
    let printed: Vec<&str> = edits.lines().filter(|l| l.contains("version ")).collect();
    assert_eq!(printed.len(), 2, "{edits}");
    assert!(printed[0].contains("title (version 1 to 2"), "{edits}");
    assert!(
        printed[1].contains(&format!(
            "body (version 2 to 3, replaced {})",
            replaced_hash_of(&at_two)
        )),
        "{edits}"
    );
}

// ---------------------------------------------------------------------------
// An entity accounts for the versions it carries (ADR-16813b3bcf37,
// TASK-dfe5a1bb0857)
// ---------------------------------------------------------------------------
//
// The falsification is a direct file write, performed rather than described:
// anything less tests that the arithmetic adds up, which was never in doubt,
// rather than that it catches what it exists to catch.

/// The line this rule prints, whichever entity it is about.
const NOT_ACCOUNTED: &str = "and its entries account for";

/// The two counts, and the write that produced the gap is the test's own.
#[test]
fn an_entity_that_cannot_account_for_a_version_is_reported_with_both_counts() {
    let r = Repo::new();
    std::fs::create_dir_all(r.0.join("src")).unwrap();
    std::fs::write(r.0.join("src/main.rs"), "fn main() {}\n").unwrap();

    let out = r.ank(
        "claude-code/1.0",
        &[
            "new",
            "adr",
            "--title",
            "A decision",
            "--scope",
            "src/**",
            "--constraint",
            "Do not do X.",
        ],
    );
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    let id = stdout(&out)
        .split_whitespace()
        .nth(1)
        .expect("created <id> <slug>")
        .to_string();
    for (flag, value) in [("--title", "A better decision"), ("--body", "Rewritten.")] {
        let out = r.ank_edit("claude-code/1.0", &["edit", &id, flag, value], None);
        assert_eq!(code(&out), 0, "{}", both_streams(&out));
    }

    // Before the third write: two entries covering 1 to 2 and 2 to 3, and the
    // entity at version 3. The arithmetic closes and the rule says nothing.
    let text = r.adr_text(&id);
    assert!(text.contains("version: 3"), "{text}");
    let checked = r.ank("claude-code/1.0", &["check"]);
    assert_eq!(code(&checked), 0, "{}", both_streams(&checked));
    assert!(
        !both_streams(&checked).contains(NOT_ACCOUNTED),
        "silent while it adds up: {}",
        both_streams(&checked)
    );

    // The third write, by hand and past the tool: the corpus is writable by
    // anything, which is the premise the whole mechanism rests on.
    std::fs::write(
        r.0.join(".ank/entities").join(format!("{id}.md")),
        text.replace("Rewritten.", "Rewritten again, by hand.")
            .replace("version: 3", "version: 4"),
    )
    .unwrap();

    let checked = r.ank("claude-code/1.0", &["check"]);
    // A signal and never a fault: an entity written outside the CLI is legal,
    // and exiting 8 over it would redden a pipeline on an act ADR-01b6dd05f0db
    // permits a human outright.
    assert_eq!(
        code(&checked),
        0,
        "the arithmetic not closing is a signal: {}",
        both_streams(&checked)
    );
    let said = both_streams(&checked);
    let line = said
        .lines()
        .find(|l| l.contains(NOT_ACCOUNTED))
        .unwrap_or_else(|| panic!("the rule fires: {said}"));
    assert!(line.contains(&id), "the subject is named whole: {line}");
    assert!(
        line.contains("version 4") && line.contains("account for 3"),
        "both counts, so a reader can see the size of the gap: {line}"
    );
    assert!(line.starts_with("signal:"), "{line}");
}

/// The negative test, and it is the one that matters most: the regime opens
/// with an entity's first entry, so a corpus written before any of this existed
/// is silent everywhere.
#[test]
fn a_corpus_that_predates_the_regime_produces_not_one_finding() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.seed_adr("ADR-00000000ab01", "Do not do X.", "src/**");
    r.seed_spec("SPEC-00000000ab02", "accepted", &[], None);
    r.seed_task_with_body_log("TASK-000000000b03", "what the last holder learned");
    seed_entry(
        &r,
        "LOG-00000000ab04",
        ID,
        0,
        "what the last holder learned",
        None,
    );

    let checked = r.ank("claude-code/1.0", &["check"]);
    let said = both_streams(&checked);
    assert!(
        !said.contains(NOT_ACCOUNTED),
        "not one finding from this rule: {said}"
    );
}

/// The transition an ADR does have is counted, and this is the false positive
/// the rule would otherwise fire on every ratified decision in a corpus.
#[test]
fn a_ratification_is_a_write_the_entity_evidences() {
    let r = ready_to_ratify();
    let id = new_adr(&r, "claude-code/1.0", "Do not do X.");
    let out = r.ank_edit(
        "claude-code/1.0",
        &["edit", &id, "--title", "A better decision"],
        None,
    );
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);

    let out = r.ank("human:marie", &["accept", &id]);
    assert_eq!(code(&out), 0, "{}", both_streams(&out));

    // Version 3: created, edited, ratified. One entry accounts for the edit and
    // `ratified` accounts for the rest.
    assert!(
        r.adr_text(&id).contains("version: 3"),
        "{}",
        r.adr_text(&id)
    );
    let checked = r.ank("claude-code/1.0", &["check"]);
    assert!(
        !both_streams(&checked).contains(NOT_ACCOUNTED),
        "a ratification leaves a field behind, so it is counted: {}",
        both_streams(&checked)
    );
}

/// A task carries entries and is never the subject of this rule, and the
/// silence is derived rather than chosen.
///
/// `claim` and `release` each write the file and leave nothing durable behind,
/// so the versions a task owes to transitions cannot be evidenced by any
/// reader. Measured on TASK-3c12e0ced2c0, the first entity in this repository
/// to carry a machinery entry: version 4, one entry covering 2 to 3, the other
/// two versions being the claim and the `done`. Counting those would fire on
/// the rule's own first subject.
#[test]
fn a_task_is_not_the_subject_of_this_rule() {
    let r = Repo::new().with_verifiers("verifiers:\n  ok:\n    run: echo fine\n");
    r.seed_task(ID, Some("A verifiable criterion."));
    // Both scopes matching a file, so the only thing left for this fixture to
    // report is the thing it is about.
    for dir in ["src", "docs"] {
        std::fs::create_dir_all(r.0.join(dir)).unwrap();
        std::fs::write(r.0.join(dir).join("a.md"), "x\n").unwrap();
    }

    assert_eq!(
        code(&r.ank("claude-code/1.0", &["amend", ID, "--scope", "docs/**"])),
        0
    );
    assert_eq!(code(&r.ank("claude-code/1.0", &["claim", ID])), 0);
    let head = r.head();
    let out = r.ank(
        "claude-code/1.0",
        &["done", "--proof", &format!("commit:{head}")],
    );
    // The clause the criterion states outright: no `done` is blocked by any of
    // this, whatever the arithmetic says.
    assert_eq!(code(&out), 0, "{}", both_streams(&out));
    assert!(
        r.task_text(ID).contains("version: 4"),
        "{}",
        r.task_text(ID)
    );

    let checked = r.ank("claude-code/1.0", &["check"]);
    assert_eq!(code(&checked), 0, "{}", both_streams(&checked));
    assert!(
        !both_streams(&checked).contains(NOT_ACCOUNTED),
        "the transitions of a task are not derivable, so nothing is derived: {}",
        both_streams(&checked)
    );
}

/// The same page with the machinery section taken out of it.
///
/// The section is what `show` and `log` grew for these entries, so its going is
/// the display doing what it is for. Everything else on the page is the answer,
/// and the answer is what must not have moved.
fn without_edits(text: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if line.starts_with("EDITS (") {
            // And the blank line that opened the section with it, so what is
            // left reads as a page that never had one.
            kept.pop();
            inside = true;
            continue;
        }
        if inside {
            if !line.is_empty() {
                continue;
            }
            inside = false;
        }
        kept.push(line);
    }
    kept.join("\n").trim_end().to_string()
}

/// A trace and not an anchor: deleting every machinery entry changes no exit
/// code and no answer.
///
/// **Asserted by deletion rather than by a comment**, which is what the
/// criterion asks and what ADR-ff294eff4d1a requires of the log: nothing
/// authoritative is anchored in it. A verb that had quietly started consulting
/// one of these entries would answer differently here, whatever its author
/// believed.
///
/// The two verbs that *print* the entries are compared on what they answer
/// about the entity rather than on the section that holds them: `show` and
/// `log` have one for exactly this, which is the whole of TASK-027a429aad2e.
///
/// **The identity is typed, and that is the one arrangement in the fixture.**
/// An entry is an entity (ADR-25f977377fa0), so `check`'s census of authors
/// counts it like anything else and removing three files moves that number,
/// which is a count of the corpus and not a judgement about the entity the
/// entries are about. Typing the identity takes the census out of the
/// comparison and leaves the judgements in it, which is what the clause is
/// about.
#[test]
fn deleting_every_machinery_entry_changes_no_answer() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    r.seed_adr("ADR-00000000aa01", "Do not do X.", "src/**");
    // A work entry beside them, so that what `ank log` answers is a trace in
    // both readings. Without one the verb would go from printing a section to
    // saying there is no entry at all, which is the display moving and not the
    // answer, and it would drown the assertion this test is making.
    seed_entry(
        &r,
        "LOG-00000000ac01",
        ID,
        0,
        "what the last holder learned",
        None,
    );
    assert_eq!(
        code(&r.ank_edit(
            "claude-code/1.0",
            &["edit", ID, "--title", "A better title"],
            None
        )),
        0
    );
    assert_eq!(
        code(&r.ank("claude-code/1.0", &["amend", ID, "--scope", "docs/**"])),
        0
    );
    assert_eq!(
        code(&r.ank(
            "claude-code/1.0",
            &["amend", "ADR-00000000aa01", "--scope", "docs/**"]
        )),
        0
    );
    assert_eq!(r.machinery_of(ID).len(), 2, "{:?}", r.machinery_of(ID));
    assert_eq!(r.machinery_of("ADR-00000000aa01").len(), 1);

    let asked: [&[&str]; 6] = [
        &["check"],
        &["status"],
        &["context", "src"],
        &["scope", "src"],
        &["graph"],
        &["show", ID],
    ];
    let before: Vec<(i32, String)> = asked
        .iter()
        .map(|a| {
            let o = r.ank("claude-code/1.0", a);
            (code(&o), stdout(&o))
        })
        .collect();
    let logged_before = r.ank("claude-code/1.0", &["log", ID]);

    // Every one of them, by hand: the corpus is writable by anything, and what
    // is under test is what the tool answers about a corpus that lost them.
    let mut deleted = 0;
    for entry in std::fs::read_dir(r.0.join(".ank/entities"))
        .unwrap()
        .flatten()
    {
        let path = entry.path();
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(ank_core::Entity::Log(l)) = ank_core::parse_entity(&text) {
            if l.records.is_some() {
                std::fs::remove_file(&path).unwrap();
                deleted += 1;
            }
        }
    }
    assert_eq!(deleted, 3, "every such entry, and only those");

    for (a, was) in asked.iter().zip(before) {
        let now = r.ank("claude-code/1.0", a);
        assert_eq!(code(&now), was.0, "ank {a:?} moved its exit code");
        assert_eq!(
            without_edits(&stdout(&now)),
            without_edits(&was.1),
            "ank {a:?} moved its answer"
        );
    }

    let logged_now = r.ank("claude-code/1.0", &["log", ID]);
    assert_eq!(code(&logged_now), code(&logged_before));
    assert_eq!(
        without_edits(&stdout(&logged_now)),
        without_edits(&stdout(&logged_before)),
        "the work trace never held them and does not miss them"
    );
}

// ---------------------------------------------------------------------------
// A named edit, and the editor it does not replace (ADR-5bd8257dfeac)
// ---------------------------------------------------------------------------

/// The two paths meet the freeze at the same place, in the same words.
///
/// This is the assertion the whole design rests on. A second entry point that
/// refused differently would be a second surface, and §4 has one; the risk is
/// not the flags but the refusals growing apart behind them, so the test
/// compares them rather than checking each against a sentence written here.
#[test]
fn a_named_edit_and_the_editor_meet_the_freeze_in_the_same_words() {
    let r = ready_to_ratify();
    let id = new_adr(&r, "claude-code/opus-5", "Do not do X.");
    r.git(&["add", "-A"]);
    r.git(&["commit", "-qm", "seed"]);
    let accepted = r.ank("human:marie", &["accept", &id]);
    assert_eq!(code(&accepted), 0, "{}", stderr(&accepted));

    // The same change, expressed twice: once as a field, once as the whole file
    // a person would have saved out of an editor.
    let named = r.ank(
        "claude-code@ank",
        &["edit", &id, "--constraint", "Do not do Y."],
    );
    let saved = entity_text(&r, &id).replace("Do not do X.", "Do not do Y.");
    let editor = r.editor_saving(&saved);
    let typed = r.ank_edit("claude-code@ank", &["edit", &id], Some(&editor));

    assert_eq!(
        code(&named),
        6,
        "a ratified constraint is a transition refusal: {}",
        stderr(&named)
    );
    assert_eq!(
        code(&named),
        code(&typed),
        "the two paths refuse with one code"
    );
    assert!(
        stderr(&named).contains("is ratified: constraint and scope are anchored"),
        "{}",
        stderr(&named)
    );
    // One sentence, and the editor path adds where it kept what was typed.
    // That tail is the one thing the two paths may not share, and it is right
    // that they do not: a refusal after an editor has run must not discard the
    // twenty minutes around the typo, and a flag value is still in the caller's
    // shell. So the refusal is compared as a prefix, and the remainder is
    // asserted to be exactly that note and nothing else.
    let refusal = stderr(&named);
    let refusal = refusal.lines().next().expect("a refusal is a line");
    let typed_line = stderr(&typed);
    let typed_line = typed_line.lines().next().expect("a refusal is a line");
    assert!(
        typed_line.starts_with(refusal),
        "one sentence, differing only in its tail:\n  {refusal}\n  {typed_line}"
    );
    assert!(
        typed_line[refusal.len()..].starts_with(" (the edited text is kept at "),
        "and the tail is where the typed text was kept: {typed_line}"
    );
}

/// A flag the addressed kind has no field for is refused by name.
///
/// Never dropped in silence: a caller who typed `--constraint` on a task
/// believes they changed something, and a verb answering `edited` to that would
/// be lying about the corpus.
#[test]
fn a_field_the_kind_does_not_carry_is_refused_by_name() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    let before = entity_text(&r, ID);

    let out = r.ank(
        "claude-code@ank",
        &["edit", ID, "--constraint", "Do not do X."],
    );
    assert_eq!(code(&out), 1, "{}", stderr(&out));
    assert!(
        stderr(&out).contains("--constraint is not a field of a task"),
        "{}",
        stderr(&out)
    );
    assert_eq!(
        entity_text(&r, ID),
        before,
        "and the entity is left exactly as it was"
    );
}

/// Only what is named is written, and the rest of the file reaches disk
/// untouched.
#[test]
fn a_named_edit_writes_only_the_field_it_names() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    let out = r.ank(
        "claude-code@ank",
        &["edit", ID, "--title", "A sharper title"],
    );
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    assert!(
        stdout(&out).contains("title"),
        "the change is named: {}",
        stdout(&out)
    );
    let after = entity_text(&r, ID);
    assert!(after.contains("title: A sharper title"), "{after}");
    assert!(
        after.contains("A verifiable criterion."),
        "the criterion is untouched: {after}"
    );
    assert!(after.contains("scope:"), "and so is the scope: {after}");
}

/// A named edit that changes nothing is the no-op the editor path reports, and
/// not a version bump.
#[test]
fn a_named_edit_that_changes_nothing_bumps_no_version() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));
    // Through the binary first, so the file on disk is in canonical form: a
    // seeded file is not necessarily, and rewriting it into canonical form is a
    // real change that this test would otherwise mistake for a defect.
    let out = r.ank("claude-code@ank", &["edit", ID, "--title", "Settled"]);
    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let before = entity_text(&r, ID);

    let again = r.ank("claude-code@ank", &["edit", ID, "--title", "Settled"]);
    assert_eq!(code(&again), 0, "{}", stderr(&again));
    assert!(
        stdout(&again).starts_with(&format!("unchanged {ID}")),
        "{}",
        stdout(&again)
    );
    assert_eq!(entity_text(&r, ID), before, "the file did not move");
}

/// A named field never looks for an editor, so an unset `$EDITOR` is not an
/// environment failure for a caller who did not want one.
#[test]
fn a_named_edit_needs_no_editor() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    let out = r.ank_edit(
        "claude-code@ank",
        &["edit", ID, "--title", "A sharper title"],
        None,
    );
    assert_eq!(
        code(&out),
        0,
        "no editor is needed when the field is named: {}",
        stderr(&out)
    );
}

/// The body from stdin, the way `ank new --body -` already reads one.
#[test]
fn a_named_edit_reads_the_body_from_stdin() {
    let r = Repo::new();
    r.seed_task(ID, Some("A verifiable criterion."));

    let mut child = ank_command()
        .args(["edit", ID, "--body", "-"])
        .arg("--repo")
        .arg(&r.0)
        .env("ANK_AGENT", "claude-code@ank")
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the binary must have been built");
    child
        .stdin
        .as_mut()
        .expect("stdin is piped")
        .write_all(b"The reasoning, piped in.\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();

    assert_eq!(code(&out), 0, "{}", stderr(&out));
    let after = entity_text(&r, ID);
    assert!(after.contains("The reasoning, piped in."), "{after}");
    assert!(
        after.contains("A verifiable criterion."),
        "and nothing else moved: {after}"
    );
}

// ---------------------------------------------------------------------------
// Unreadable is not absent (TASK-5c7aae69a4c0)
// ---------------------------------------------------------------------------

/// An entity file one schema past what this build reads, written by hand
/// because no build writes one: what it stands for is a corpus a newer release
/// has already written into, which is the state every installed copy of ank is
/// in between a schema bump landing and a release carrying it.
fn seed_one_schema_ahead(r: &Repo, id: &str, kind: &str, supersedes: Option<&str>) {
    let supersedes = supersedes
        .map(|s| format!("supersedes: {s}\n"))
        .unwrap_or_default();
    let body = match kind {
        "spec" => format!("status: accepted\nscope:\n  - docs/**\nreferences: []\n{supersedes}"),
        "task" => "status: open\nscope:\n  - src/**\nblocked_by: []\n".to_string(),
        other => panic!("no fixture for {other}"),
    };
    std::fs::write(
        r.0.join(".ank/entities").join(format!("{id}.md")),
        format!(
            "---\nid: {id}\ntype: {kind}\nslug: one-ahead\ntitle: One schema ahead\n\
             created: 2026-08-01T00:00:00Z\nauthor: human:marie\n{body}\
             schema: 99\nversion: 1\n---\n\nA document a newer release wrote.\n"
        ),
    )
    .unwrap();
}

/// Nothing that rests on a file this build could not read is reported as
/// missing, and no repair proposes a deletion.
///
/// The measurement this pins: nine unreadable files on the real corpus produced
/// ten extra faults, eight of them `--drop-reference` against citations that
/// were correct. A reader following one leaves the corpus worse than they found
/// it, which is the one thing a finding in this tool may never do.
#[test]
fn an_unreadable_entity_is_never_reported_as_one_that_does_not_exist() {
    let r = Repo::new();
    r.seed_docs();
    // Readable, and pointing at what this build cannot read in all three ways
    // an identifier can point: a citation, a blocker, and a succession.
    seed_one_schema_ahead(&r, "SPEC-00000000aa01", "spec", Some("SPEC-00000000aa02"));
    r.seed_spec(
        "SPEC-00000000aa02",
        "superseded",
        &["SPEC-00000000aa01"],
        None,
    );
    seed_one_schema_ahead(&r, "TASK-00000000aa03", "task", None);
    r.seed_task(ID, Some("A verifiable criterion."));
    r.blocked(ID, &["TASK-00000000aa03"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let said = both_streams(&out);

    assert!(
        !said.contains("--drop-reference"),
        "no repair deletes a citation of something merely unreadable:\n{said}"
    );
    assert!(
        !said.contains("does not exist"),
        "and nothing unreadable is called absent:\n{said}"
    );
    assert!(
        !said.contains("marked superseded but no"),
        "nor is a succession called broken when its successor is the unreadable \
         file:\n{said}"
    );
    // Said once, with the cause, rather than once per consequence.
    assert!(
        said.contains("resolution is incomplete"),
        "the incompleteness is named:\n{said}"
    );
    assert!(
        said.contains("2 entity file(s) could not be read"),
        "and counted:\n{said}"
    );
}

/// The same findings still fire where the target is one this build did read.
///
/// Removing a finding is easy to do too widely, and this is the half that says
/// the guard did not take the honest cases with it.
#[test]
fn a_target_that_is_genuinely_absent_is_still_a_fault() {
    let r = Repo::new();
    r.seed_docs();
    r.seed_spec(
        "SPEC-00000000bb01",
        "accepted",
        &["SPEC-00000000bb99"],
        None,
    );
    r.seed_task(ID, Some("A verifiable criterion."));
    r.blocked(ID, &["TASK-00000000bb98"]);

    let out = r.ank("claude-code@ank", &["check"]);
    let said = both_streams(&out);

    assert_eq!(
        code(&out),
        8,
        "a corpus with dangling ids is faulty: {said}"
    );
    assert!(
        said.contains("references SPEC-00000000bb99, which does not exist"),
        "{said}"
    );
    assert!(
        said.contains("--drop-reference SPEC-00000000bb99"),
        "{said}"
    );
    assert!(
        said.contains("blocked_by names TASK-00000000bb98, which does not exist"),
        "{said}"
    );
    assert!(
        !said.contains("resolution is incomplete"),
        "and nothing was unreadable, so nothing is excused:\n{said}"
    );
}

/// A succession that leads nowhere is still a fault when the whole corpus was
/// read.
#[test]
fn a_succession_with_no_successor_is_still_a_fault_when_everything_was_read() {
    let r = Repo::new();
    r.seed_docs();
    r.seed_spec("SPEC-00000000cc01", "superseded", &[], None);

    let out = r.ank("claude-code@ank", &["check"]);
    let said = both_streams(&out);
    assert_eq!(code(&out), 8, "{said}");
    assert!(
        said.contains("SPEC-00000000cc01: marked superseded but no spec supersedes it"),
        "{said}"
    );
}
