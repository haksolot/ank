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

pub fn run(inv: &Invocation, cwd: &Path, out: &mut dyn Write) -> Result<i32> {
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
            1,
            "--repo names a repository that exists, and init is what makes one",
        )
        .with_hint(format!("ank init {path}")));
    }

    let target = match inv.positionals.first() {
        Some(p) => PathBuf::from(p),
        None => cwd.to_path_buf(),
    };
    // git is a hard dependency (ADR-b8884edcebe3): we fail before writing
    // anything rather than leave a half-placed `.ank/` in a directory that is
    // not a repository.
    git::ensure_usable(&target)?;
    let report = init_at(&target)?;
    for line in report.lines_terse() {
        let _ = writeln!(out, "{line}");
    }
    Ok(0)
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub created_dirs: bool,
    pub wrote_config: bool,
    pub wrote_gitattributes: bool,
    pub wrote_gitignore: bool,
    pub wrote_agents_pointer: bool,
    pub added_refspec: bool,
}

impl Report {
    /// Terse output, `git status` style: one line per real effect, nothing for
    /// what was already in place.
    pub fn lines_terse(&self) -> Vec<String> {
        let mut v = Vec::new();
        if self.created_dirs {
            v.push("created .ank/entities .ank/log".to_string());
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
        if v.is_empty() {
            v.push("already initialised, nothing to do".to_string());
        }
        v
    }
}

fn io(path: &Path, e: std::io::Error) -> CliError {
    CliError::new(1, format!("{}: {e}", path.display()))
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
            report.created_dirs = true;
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

        assert!(r.created_dirs);
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
