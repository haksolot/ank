//! `ank init` (§9).
//!
//! A narrow, fixed perimeter: create `.ank/`, write `config.yml`, add the
//! `refs/ank/*` refspec, place a pointer in `AGENTS.md`, and write a
//! `.gitattributes` and a `.gitignore`.
//!
//! Neither git file is cosmetic. On Windows `core.autocrlf=true` is the
//! default, and without `.gitattributes` git converts back to CRLF on every
//! checkout what the tool has just written in LF — making any fresh clone
//! unreadable. Without `.gitignore`, §6 calls the index derived, disposable
//! and gitignored while no repository `init` produces is any of the third:
//! the first `git add -A` commits a binary file that every command rewrites.
//! Fixing either one only in Ank's own repository would have left it broken
//! for every user — which is precisely how the second survived, since this
//! repository carries the ignore line by hand, written before `init` existed
//! to write it.

use crate::cli::{CliError, Invocation, Result};
use crate::store::Store;
use crate::{config, git, repo};
use ank_contract::ExitCode;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const GITATTRIBUTES_LINE: &str = ".ank/** text eol=lf";
/// Root-relative on purpose: a `gitignore` pattern holding a `/` is anchored
/// to the directory of the file that carries it, so this stays correct when
/// `init` runs on a subdirectory rather than on the repository root.
pub const GITIGNORE_LINE: &str = ".ank/index.db";
pub const REFSPEC: &str = "+refs/ank/*:refs/ank/*";
const AGENTS_POINTER: &str = "This repo uses Ank: tasks and decisions live in `.ank/`.";

pub fn run(inv: &Invocation, cwd: &Path, out: &mut dyn Write) -> Result<ExitCode> {
    // **Refused, and refused before anything is written** (§4, §9). `--repo`
    // names a repository that already carries a `.ank/`, which is what this verb
    // is run to produce, so it is the one verb the flag does not apply to — and
    // the target is positional.
    //
    // Not a matter of tidiness: `dispatch` routes `init` ahead of the foundation
    // that resolves `--repo` for every other verb, and rightly so, since `init`
    // precedes the existence of the repository. But routing early is not a
    // reason to drop a global silently, and this one was dropped into a verb
    // that writes. Measured: `ank init --repo <elsewhere>` initialised the
    // *current* repository, appended the pointer paragraph to an `AGENTS.md`
    // nobody was editing, reported `pointer added to AGENTS.md`, and left the
    // named directory empty — noticed only because the next command there
    // answered `no .ank/ found` (TASK-b8a12d60686d).
    if let Some(path) = inv.repo() {
        return Err(CliError::new(
            ExitCode::Generic,
            "--repo names a repository that exists, and init is what makes one",
        )
        .with_hint(format!("ank init {path}")));
    }

    let detached = inv.value("--at");
    if detached.is_some() && inv.positionals.first().is_some() {
        return Err(CliError::new(
            ExitCode::Generic,
            "--at and a positional target name two different directories",
        )
        .with_hint("ank init --at <path>"));
    }
    let named = detached.or(inv.positionals.first().map(String::as_str));
    let target = match named {
        Some(p) => PathBuf::from(p),
        None => cwd.to_path_buf(),
    };
    // git is a hard dependency (ADR-9307e5d214a7): we fail before writing
    // anything rather than leave a half-placed `.ank/` in a directory that is
    // not a repository.
    if detached.is_some() {
        // Everything a detached corpus is refused for, before a byte is
        // written: the identity it will be keyed on, the tree it must stay out
        // of, and the repository whose refs will carry its claims.
        detachable(cwd, &target)?;
    }
    git::ensure_usable(&target)?;
    let mut report = init_at(&target)?;
    if detached.is_some() {
        let identity = repo::identity(cwd).expect("checked by detachable");
        let file = config::declare_corpus(&identity, &target.to_string_lossy())?;
        report.declared = Some(file.to_string_lossy().to_string());
    }
    // `--json` is available on every command without exception (§4), and this
    // verb was the exception: it printed its prose and ignored the flag. The
    // sweep never caught it because a `Repo` fixture already carries a `.ank/`,
    // so `init` refused, stdout was empty, and an empty stdout is what a
    // refusal is supposed to leave (TASK-9e63827380a1).
    if inv.json() {
        let _ = writeln!(out, "{}", report.json());
    } else if !inv.quiet() {
        for line in report.lines_terse() {
            let _ = writeln!(out, "{line}");
        }
    }
    Ok(ExitCode::Ok)
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// The directories this run made, in the order they were made. A list and
    /// not a flag: the flag named both whenever either was missing, so a run
    /// that created one of the two reported creating a directory it found.
    pub created_dirs: Vec<String>,
    /// The declarations file a `--at` run wrote into (ADR-96174f1ac2b7).
    ///
    /// Named rather than silent, and named as a path: it is the one thing this
    /// verb wrote outside the directory the caller pointed at, and a reader who
    /// wants to see it or correct it needs its address.
    pub declared: Option<String>,
    pub wrote_config: bool,
    pub wrote_gitattributes: bool,
    pub wrote_gitignore: bool,
    pub wrote_agents_pointer: bool,
    pub added_refspec: bool,
}

impl Report {
    /// The same six effects as [`lines_terse`], grouped by the verb that
    /// applies to them, and always the same shape: three lists, empty when
    /// there is nothing in them, so a parser reads one document and not two.
    ///
    /// `changed` is the question a script actually asks — whether there is
    /// anything to commit — and it is answered rather than left to be derived
    /// from the emptiness of three lists.
    ///
    /// [`lines_terse`]: Report::lines_terse
    pub fn json(&self) -> String {
        let mut wrote: Vec<&str> = Vec::new();
        if self.wrote_config {
            wrote.push(".ank/config.yml");
        }
        if self.wrote_gitattributes {
            wrote.push(".gitattributes");
        }
        if self.wrote_gitignore {
            wrote.push(".gitignore");
        }
        // Where each was added, not what was added: the caller who wants to
        // inspect the result needs the address, and the two addresses are of
        // different kinds — a file and a git config key.
        let mut added: Vec<&str> = Vec::new();
        if let Some(file) = &self.declared {
            added.push(file.as_str());
        }
        if self.wrote_agents_pointer {
            added.push("AGENTS.md");
        }
        if self.added_refspec {
            added.push("remote.origin.fetch");
        }
        crate::json::Obj::document()
            .strings("created", &self.created_dirs)
            .strings("wrote", &wrote)
            .strings("added", &added)
            .bool("changed", self.changed())
            .finish()
    }

    /// Whether this run had any effect at all.
    pub fn changed(&self) -> bool {
        !self.created_dirs.is_empty()
            || self.wrote_config
            || self.wrote_gitattributes
            || self.wrote_gitignore
            || self.wrote_agents_pointer
            || self.added_refspec
    }

    /// Terse output, `git status` style: one line per real effect, nothing for
    /// what was already in place.
    pub fn lines_terse(&self) -> Vec<String> {
        let mut v = Vec::new();
        if !self.created_dirs.is_empty() {
            v.push(format!("created {}", self.created_dirs.join(" ")));
        }
        if self.wrote_config {
            v.push("wrote .ank/config.yml".to_string());
        }
        if self.wrote_gitattributes {
            v.push("wrote .gitattributes".to_string());
        }
        if self.wrote_gitignore {
            v.push("wrote .gitignore".to_string());
        }
        if self.wrote_agents_pointer {
            v.push("pointer added to AGENTS.md".to_string());
        }
        if self.added_refspec {
            v.push(format!("refspec added: {REFSPEC}"));
        }
        // Last, and never silent: it is the one thing this verb wrote outside
        // the directory the caller pointed at.
        if let Some(file) = &self.declared {
            v.push(format!("declared in {file}"));
        }
        if v.is_empty() {
            v.push("already initialised, nothing to do".to_string());
        }
        v
    }
}

fn io(path: &Path, e: std::io::Error) -> CliError {
    CliError::new(ExitCode::Generic, format!("{}: {e}", path.display()))
}

/// Idempotent: re-initialising an already initialised repository must break
/// nothing and duplicate nothing. That is what makes it safe to run without
/// thinking.
pub fn init_at(root: &Path) -> Result<Report> {
    let mut report = Report::default();
    let ank = root.join(repo::ANK_DIR);

    // One directory for every kind, and one for the logs beside it (§6). The
    // previous layout's `tasks/` and `adr/` are read where they already exist
    // and are never created: a writer does not produce a layout it is only
    // keeping readable.
    for sub in [Store::ENTITIES_DIR, "log"] {
        let dir = ank.join(sub);
        if !dir.is_dir() {
            fs::create_dir_all(&dir).map_err(|e| io(&dir, e))?;
            report.created_dirs.push(format!("{}/{sub}", repo::ANK_DIR));
        }
    }

    let cfg = ank.join("config.yml");
    if !cfg.exists() {
        fs::write(&cfg, config::default_yaml()).map_err(|e| io(&cfg, e))?;
        report.wrote_config = true;
    }

    let ga = root.join(".gitattributes");
    report.wrote_gitattributes = ensure_line(&ga, GITATTRIBUTES_LINE)?;

    let gi = root.join(".gitignore");
    report.wrote_gitignore = ensure_line(&gi, GITIGNORE_LINE)?;

    let agents = root.join("AGENTS.md");
    report.wrote_agents_pointer = ensure_line(&agents, AGENTS_POINTER)?;

    report.added_refspec = ensure_refspec(root)?;

    Ok(report)
}

/// Whether a corpus may be created at `target` for the repository `cwd` sits
/// in, and every reason it may not (ADR-96174f1ac2b7).
///
/// **Inside the tree is refused rather than accepted quietly.** A corpus under
/// the working tree is not a detached corpus, and `--at .ank` would be a long
/// way of writing `ank init` while writing a declaration that promises
/// something it does not deliver. `ank init` is the verb for a corpus that
/// lives beside its code.
///
/// **A tree with no identity is refused too**, because there would be nothing
/// to key the declaration on. A fallback is not a key: every historyless tree
/// on a machine would share one, which is the collision ADR-621a7fd96ce1 chose
/// the root commit to avoid.
///
/// **git is not created here.** The corpus repository is where claims and
/// proofs land (ADR-9e56318631f3), so the target has to be one — but `git init`
/// is not on the plumbing ADR-9307e5d214a7 allows, and running a verb this tool
/// has never run to save a caller one command is not a trade this makes on its
/// own. The refusal names the command, which is what §4 asks of it.
fn detachable(cwd: &Path, target: &Path) -> Result<()> {
    let Some(_) = repo::identity(cwd) else {
        return Err(CliError::new(
            ExitCode::Prerequisite,
            "this tree has no repository identity, so a declaration has nothing to be keyed on",
        )
        .with_hint("git commit: the identity is the root commit"));
    };
    let tree = git::run(cwd, &["rev-parse", "--show-toplevel"])
        .map(PathBuf::from)
        .unwrap_or_else(|_| cwd.to_path_buf());
    // Compared after canonicalisation, for the reason `same_corpus` gives:
    // `..`, symlinks and the case rules of Windows all sit between two spellings
    // of one directory. A target that does not exist yet is canonicalised
    // through its parent, since that is the part that already does.
    let of = |p: &Path| -> PathBuf {
        std::fs::canonicalize(p).unwrap_or_else(|_| match (p.parent(), p.file_name()) {
            (Some(parent), Some(name)) => std::fs::canonicalize(parent)
                .unwrap_or_else(|_| parent.to_path_buf())
                .join(name),
            _ => p.to_path_buf(),
        })
    };
    let (tree, target) = (of(&tree), of(target));
    if target.starts_with(&tree) {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "--at {} is inside {}, which is the tree it is meant to stay out of",
                target.display(),
                tree.display()
            ),
        )
        .with_hint("ank init, for a corpus that lives beside its code"));
    }
    Ok(())
}

/// Adds a line to a file if it is not already there. Returns `true` if the
/// file was touched.
fn ensure_line(path: &Path, line: &str) -> Result<bool> {
    let existing = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(io(path, e)),
    };
    if existing.lines().any(|l| l.trim() == line) {
        return Ok(false);
    }
    let mut next = existing;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    if !next.is_empty() {
        next.push('\n');
    }
    next.push_str(line);
    next.push('\n');
    fs::write(path, next).map_err(|e| io(path, e))?;
    Ok(true)
}

/// Hosts do not fetch non-standard refs on their own (§7). The refspec is set
/// even without a configured remote: the key will exist by the time `origin`
/// is added, which avoids a silent trap at the first push.
fn ensure_refspec(root: &Path) -> Result<bool> {
    let existing =
        git::run(root, &["config", "--get-all", "remote.origin.fetch"]).unwrap_or_default();
    if existing.lines().any(|l| l.trim() == REFSPEC) {
        return Ok(false);
    }
    git::run(root, &["config", "--add", "remote.origin.fetch", REFSPEC])?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(PathBuf);

    impl Temp {
        /// A real git repository: `init` refuses to write outside one, and that
        /// is exactly the behaviour we want to exercise.
        fn new_repo() -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-init-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&p).unwrap();
            let ok = Command::new("git")
                .current_dir(&p)
                .args(["init", "-q"])
                .status()
                .expect("git must be installed: it is a hard dependency")
                .success();
            assert!(ok, "git init failed in {}", p.display());
            let t = Temp(p);
            // Signing off at creation, not at each commit (TASK-40a972e98a9a).
            for args in [
                ["config", "commit.gpgsign", "false"],
                ["config", "tag.gpgsign", "false"],
            ] {
                let st = Command::new("git")
                    .current_dir(&t.0)
                    .args(args)
                    .status()
                    .expect("git must be installed: it is a hard dependency");
                assert!(st.success(), "git {args:?}");
            }
            t
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn init_produces_all_six_effects() {
        let t = Temp::new_repo();
        let r = init_at(&t.0).unwrap();

        assert_eq!(r.created_dirs, vec![".ank/entities", ".ank/log"]);
        assert!(t.0.join(".ank/entities").is_dir());
        assert!(t.0.join(".ank/log").is_dir());

        assert!(r.wrote_config);
        let cfg = fs::read_to_string(t.0.join(".ank/config.yml")).unwrap();
        assert!(config::parse(&cfg, Path::new("config.yml")).is_ok());

        assert!(r.wrote_gitattributes);
        let ga = fs::read_to_string(t.0.join(".gitattributes")).unwrap();
        assert!(ga.contains(GITATTRIBUTES_LINE), "{ga}");

        assert!(r.wrote_gitignore);
        let gi = fs::read_to_string(t.0.join(".gitignore")).unwrap();
        assert!(gi.contains(GITIGNORE_LINE), "{gi}");

        assert!(r.wrote_agents_pointer);
        assert!(fs::read_to_string(t.0.join("AGENTS.md"))
            .unwrap()
            .contains(".ank/"));

        assert!(r.added_refspec);
        let out = git::run(&t.0, &["config", "--get-all", "remote.origin.fetch"]).unwrap();
        assert!(out.contains(REFSPEC), "{out}");
    }

    #[test]
    fn init_is_idempotent() {
        let t = Temp::new_repo();
        init_at(&t.0).unwrap();
        let second = init_at(&t.0).unwrap();
        assert_eq!(second, Report::default(), "nothing should be redone");
        assert_eq!(
            second.lines_terse(),
            vec!["already initialised, nothing to do"]
        );

        // The refspec is not duplicated.
        let out = git::run(&t.0, &["config", "--get-all", "remote.origin.fetch"]).unwrap();
        assert_eq!(out.lines().filter(|l| l.trim() == REFSPEC).count(), 1);
    }

    #[test]
    fn ensure_line_preserves_existing_content() {
        let t = Temp::new_repo();
        let f = t.0.join("AGENTS.md");
        fs::write(&f, "# Notes\n\nAlready here.\n").unwrap();
        assert!(ensure_line(&f, AGENTS_POINTER).unwrap());
        let s = fs::read_to_string(&f).unwrap();
        assert!(s.starts_with("# Notes\n\nAlready here.\n"), "{s:?}");
        assert!(s.contains(AGENTS_POINTER));
        assert!(!ensure_line(&f, AGENTS_POINTER).unwrap(), "second pass");
    }

    /// A `.gitignore` is a file the user curates, far more often than a
    /// `.gitattributes` is. Appending to it must leave every rule already
    /// there intact, and must not grow a second copy of ours on re-init.
    #[test]
    fn an_existing_gitignore_keeps_its_content() {
        let t = Temp::new_repo();
        let gi = t.0.join(".gitignore");
        fs::write(&gi, "/target\nnode_modules/\n").unwrap();

        init_at(&t.0).unwrap();
        let s = fs::read_to_string(&gi).unwrap();
        assert!(s.starts_with("/target\nnode_modules/\n"), "{s:?}");
        assert!(s.contains(GITIGNORE_LINE), "{s:?}");

        let second = init_at(&t.0).unwrap();
        assert!(!second.wrote_gitignore);
        let s = fs::read_to_string(&gi).unwrap();
        assert_eq!(s.lines().filter(|l| l.trim() == GITIGNORE_LINE).count(), 1);
    }

    /// The test the original bug was asking for: a repository initialised,
    /// committed, then cloned must read its entities back. Without the
    /// `.gitattributes`, the clone comes out in CRLF on Windows and nothing
    /// parses any more.
    #[test]
    fn an_initialised_then_cloned_repo_reads_its_entities_back() {
        let t = Temp::new_repo();
        init_at(&t.0).unwrap();

        // A canonical entity, written in LF the way the store does.
        let task = "---\nid: TASK-000000000001\ntype: task\ntitle: Example\n\
created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
blocked_by: []\nschema: 1\nversion: 1\n---\n\nBody.\n";
        fs::write(t.0.join(".ank/entities/TASK-000000000001.md"), task).unwrap();

        for args in [
            vec!["config", "user.email", "test@ank.local"],
            vec!["config", "user.name", "Test"],
        ] {
            git::run(&t.0, &args).unwrap();
        }
        let run = |args: &[&str]| {
            let st = Command::new("git")
                .current_dir(&t.0)
                .args(args)
                .status()
                .unwrap();
            assert!(st.success(), "git {args:?}");
        };
        run(&["add", "-A"]);
        run(&["commit", "-qm", "init"]);

        let clone = t.0.with_extension("clone");
        let st = Command::new("git")
            .args(["clone", "-q"])
            .arg(&t.0)
            .arg(&clone)
            .status()
            .unwrap();
        assert!(st.success(), "clone");

        let read_back = fs::read(clone.join(".ank/entities/TASK-000000000001.md")).unwrap();
        let _ = fs::remove_dir_all(&clone);

        assert!(
            !read_back.contains(&b'\r'),
            "the clone must come out in LF, otherwise nothing parses"
        );
        let text = String::from_utf8(read_back).unwrap();
        assert!(
            ank_core::parse_entity(&text).is_ok(),
            "the cloned entity must parse"
        );
    }
}
