//! Minimal git runner (§12).
//!
//! Ank calls the git binary, never a library: `accept` and `check` rest on
//! signing, and `git commit -S` / `git verify-commit` are three lines where a
//! cryptographic reimplementation would be a project. The counterpart is the
//! discipline imposed by ADR-b8884edcebe3: **plumbing only**, never porcelain,
//! whose output offers no stability contract across versions.
//!
//! A broken git environment is not a failure of the agent's work: absence, too
//! old a version and a directory outside a repository all exit with code 9,
//! with the exact command to run.

use crate::cli::CliError;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::{Mutex, OnceLock};

/// Floor imposed by SSH signing and `gpg.ssh.allowedSignersFile`.
pub const MIN_VERSION: (u32, u32) = (2, 34);

const INSTALL_URL: &str = "https://git-scm.com/downloads";

/// Allowed plumbing subcommands. The rule is the criterion carried by
/// ADR-b8884edcebe3 — a command is usable only if its output is stable by
/// contract across git versions — and this list is its application, kept
/// explicit so that reaching for porcelain is a visible act in review rather
/// than an oversight. A closed list goes stale at every new need; the
/// criterion is what decides.
const PLUMBING: &[&str] = &[
    "update-ref",
    "rev-parse",
    // `rev-list` and not `log`: the porcelain is `log`, and its output carries
    // no stability contract. `rev-list` prints one object name per line and has
    // since it was git's original commit walker.
    "rev-list",
    "symbolic-ref",
    "merge-base",
    "verify-commit",
    "hash-object",
    "cat-file",
    "for-each-ref",
    "config",
    "--version",
];

pub type Result<T> = std::result::Result<T, CliError>;

fn env_missing() -> CliError {
    CliError::new(9, "git not found in PATH").with_hint(INSTALL_URL)
}

/// Runs git in `cwd` and hands back the raw outcome, exit code included.
///
/// Several primitives below read a non-zero code as an answer rather than a
/// failure — `merge-base --is-ancestor` says "no" with a 1, `symbolic-ref`
/// says "absent" the same way — so the success check cannot live here.
///
/// Public for the same reason: `update-ref` signals a lost compare-and-swap by
/// its exit code, and that signal is what `claim`'s code 4 rests on (§7).
/// Reaching it through [`run`] would mean reading the distinction back out of
/// stderr, which is exactly the fragility the plumbing rule exists to avoid.
pub fn output(cwd: &Path, args: &[&str]) -> Result<Output> {
    debug_assert!(
        args.first().map(|a| PLUMBING.contains(a)).unwrap_or(false),
        "porcelain forbidden (ADR-b8884edcebe3): {args:?}"
    );
    Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                env_missing()
            } else {
                CliError::new(9, format!("git {}: {e}", args.join(" ")))
            }
        })
}

/// The environment error for a git command that failed for a reason we did not
/// expect. Public alongside [`output`]: a caller reading exit codes itself
/// still needs one single way to say "git broke", stderr included.
pub fn failed(args: &[&str], out: &Output) -> CliError {
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    CliError::new(9, format!("git {} failed: {stderr}", args.join(" ")))
}

/// Runs git in `cwd`. Returns standard output with trailing whitespace
/// trimmed. A non-zero exit code yields the error along with stderr.
pub fn run(cwd: &Path, args: &[&str]) -> Result<String> {
    let out = output(cwd, args)?;
    if !out.status.success() {
        return Err(failed(args, &out));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

/// The installed version, as `(major, minor)`.
pub fn version() -> Result<(u32, u32)> {
    let out = Command::new("git").arg("--version").output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            env_missing()
        } else {
            CliError::new(9, format!("git --version: {e}"))
        }
    })?;
    let text = String::from_utf8_lossy(&out.stdout);
    parse_version(&text).ok_or_else(|| {
        CliError::new(9, format!("unreadable git version: {}", text.trim())).with_hint(INSTALL_URL)
    })
}

/// `git version 2.43.0.windows.1` -> `(2, 43)`. Tolerates distribution
/// suffixes, which vary between platforms.
pub fn parse_version(text: &str) -> Option<(u32, u32)> {
    let rest = text.split_whitespace().find(|w| {
        w.chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && w.contains('.')
    })?;
    let mut parts = rest.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

pub fn check_version(found: (u32, u32)) -> Result<()> {
    if found < MIN_VERSION {
        return Err(CliError::new(
            9,
            format!(
                "git {}.{} too old, {}.{} required for SSH signing",
                found.0, found.1, MIN_VERSION.0, MIN_VERSION.1
            ),
        )
        .with_hint(INSTALL_URL));
    }
    Ok(())
}

/// Root of the git repository containing `cwd`.
pub fn toplevel(cwd: &Path) -> Result<PathBuf> {
    let out = Command::new("git")
        .current_dir(cwd)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                env_missing()
            } else {
                CliError::new(9, format!("git rev-parse: {e}"))
            }
        })?;
    if !out.status.success() {
        return Err(CliError::new(
            9,
            format!("{} is not inside a git repository", cwd.display()),
        )
        .with_hint("git init"));
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(PathBuf::from(path))
}

/// Checks the whole git environment: presence, version, and repository.
pub fn ensure_usable(cwd: &Path) -> Result<PathBuf> {
    check_version(version()?)?;
    toplevel(cwd)
}

// ---------------------------------------------------------------------------
// Refs, branches, reachability (§7, §12)
// ---------------------------------------------------------------------------

const HEADS_PREFIX: &str = "refs/heads/";
const ANK_NAMESPACE: &str = "refs/ank/";
const ORIGIN_HEAD: &str = "refs/remotes/origin/HEAD";
const ORIGIN_PREFIX: &str = "refs/remotes/origin/";

/// A ref of the `refs/ank/*` namespace and the object it points at.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnkRef {
    pub name: String,
    pub object: String,
}

/// Enumerates `refs/ank/*`, which is what pruning rests on — pruning without
/// enumeration is not implementable, and that gap predated the ADR that fixed
/// the plumbing list.
///
/// A repository carrying no ank ref yields an **empty list, never an error**:
/// that is the nominal state of a fresh repository, and maintenance has to be
/// able to run there.
pub fn ank_refs(cwd: &Path) -> Result<Vec<AnkRef>> {
    // A tab separates the two fields: it cannot appear in a ref name, where a
    // space can be ambiguous to read back.
    let out = run(
        cwd,
        &[
            "for-each-ref",
            "--format=%(refname)%09%(objectname)",
            ANK_NAMESPACE,
        ],
    )?;
    let mut refs = Vec::new();
    for line in out.lines() {
        let line = line.trim_end();
        if line.is_empty() {
            continue;
        }
        let (name, object) = line
            .split_once('\t')
            .ok_or_else(|| CliError::new(9, format!("unreadable for-each-ref output: {line}")))?;
        refs.push(AnkRef {
            name: name.to_string(),
            object: object.to_string(),
        });
    }
    Ok(refs)
}

/// Reads a symbolic ref in full form. `symbolic-ref --short` is avoided on
/// purpose: its shortening depends on the other refs present, where the full
/// name is a fixed prefix to strip. A ref that is absent or not symbolic is
/// not an error — it is the answer `None`.
fn symbolic_ref(cwd: &Path, name: &str) -> Result<Option<String>> {
    let out = output(cwd, &["symbolic-ref", "--quiet", name])?;
    if !out.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(if text.is_empty() { None } else { Some(text) })
}

fn without_prefix(full: &str, prefix: &str) -> String {
    full.strip_prefix(prefix).unwrap_or(full).to_string()
}

/// The branch HEAD points at, which the completion ref records (§7). A
/// detached HEAD yields `None`: working detached is legitimate, and the record
/// simply carries no branch.
pub fn current_branch(cwd: &Path) -> Result<Option<String>> {
    Ok(symbolic_ref(cwd, "HEAD")?.map(|r| without_prefix(&r, HEADS_PREFIX)))
}

/// The branch `refs/remotes/origin/HEAD` designates, the fallback source for
/// the default branch. Absent — which is the case at level 0, and after some
/// clones — yields `None`, and it is [`resolve_default_branch`] that decides
/// what to make of it.
pub fn origin_head(cwd: &Path) -> Result<Option<String>> {
    Ok(symbolic_ref(cwd, ORIGIN_HEAD)?.map(|r| without_prefix(&r, ORIGIN_PREFIX)))
}

/// Whether `ancestor` is reachable from `descendant`.
///
/// Serves the diagnostic and never the pruning decision: a completion ref is
/// pruned on what the task file says on the default branch, because `done`
/// writes to the working tree and the commit it records is frequently already
/// an ancestor (ADR-bcf222a31525).
pub fn is_ancestor(cwd: &Path, ancestor: &str, descendant: &str) -> Result<bool> {
    let args = ["merge-base", "--is-ancestor", ancestor, descendant];
    let out = output(cwd, &args)?;
    match out.status.code() {
        Some(0) => Ok(true),
        Some(1) => Ok(false),
        // 128 for an unknown revision, no code at all if a signal killed it.
        // The realistic cause is a commit produced in another clone.
        _ => Err(failed(&args, &out).with_hint("git fetch origin")),
    }
}

/// A file as it appears on `rev`, not as the working tree holds it. That
/// distinction is the whole point: the pruning predicate asks what the default
/// branch carries, and `done` writes only to the tree (§7, §12).
///
/// `path` is repository-relative and uses `/`, which is git's own syntax on
/// the three platforms. A path absent from that revision yields `None`; an
/// unresolvable `rev` is an environment error, so that a mistyped
/// `default_branch` cannot read as "no task ever finished".
pub fn file_at(cwd: &Path, rev: &str, path: &str) -> Result<Option<String>> {
    // The revision is checked first because git answers both cases with the
    // same exit code, and telling them apart from stderr would be exactly the
    // fragility the plumbing rule exists to avoid.
    let commit = format!("{rev}^{{commit}}");
    let verify = ["rev-parse", "--verify", "--quiet", commit.as_str()];
    if !output(cwd, &verify)?.status.success() {
        return Err(
            CliError::new(9, format!("branch {rev} not found in this repository"))
                .with_hint(format!("git fetch origin {rev}")),
        );
    }
    let target = format!("{rev}:{path}");
    let out = output(cwd, &["cat-file", "-p", target.as_str()])?;
    if !out.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&out.stdout).to_string()))
}

/// The `constraint`+`scope` hash a ratification commit recorded for `id`, or
/// `None` when no such commit is reachable.
///
/// `ratified` cannot name the commit: a commit cannot contain its own
/// identifier, so no field written by the single commit `accept` makes could
/// ever hold it (§3). The pointer is the history of the ADR's own path instead.
/// Walking back from `HEAD`, the first commit whose subject is `ratify <id>` is
/// the ratification, and the `constraint+scope:` line of its message is the
/// anchor — the copy that matters, because the one in the file is written by
/// whoever writes the file.
///
/// `--full-history` and not the default: path simplification exists to explain
/// a tree's final state and is free to drop a commit that a merge made
/// redundant. Dropping the ratification commit would report a perfectly frozen
/// ADR as unverifiable.
///
/// `None` covers a shallow clone, a rewritten history, a corpus moved between
/// repositories — and a repository with no `HEAD` at all. The caller reports
/// that as unverifiable, never as a divergence.
/// Memo over the lookup below, and it is not premature. Git history does not
/// change under a running process, so the answer is fixed for the life of the
/// invocation — while `check` asks the same question once per ADR and again for
/// every task that ADR bears on. Measured on this repository before it existed:
/// hundreds of `git` spawns and 4.3s for what takes 0.9s without them.
type Memo = OnceLock<Mutex<HashMap<(PathBuf, String), Option<String>>>>;
static RATIFICATIONS: Memo = OnceLock::new();

pub fn ratification_anchor_at(cwd: &Path, id: &str, path: &str) -> Result<Option<String>> {
    let key = (cwd.to_path_buf(), id.to_string());
    let memo = RATIFICATIONS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(seen) = memo.lock() {
        if let Some(hit) = seen.get(&key) {
            return Ok(hit.clone());
        }
    }
    let found = ratification_anchor_uncached(cwd, id, path)?;
    if let Ok(mut seen) = memo.lock() {
        seen.insert(key, found.clone());
    }
    Ok(found)
}

fn ratification_anchor_uncached(cwd: &Path, id: &str, path: &str) -> Result<Option<String>> {
    let subject = format!("ratify {id}");
    let args = ["rev-list", "--full-history", "HEAD", "--", path];
    let out = output(cwd, &args)?;
    if !out.status.success() {
        return Ok(None);
    }
    for sha in String::from_utf8_lossy(&out.stdout).lines() {
        let object = output(cwd, &["cat-file", "commit", sha.trim()])?;
        if !object.status.success() {
            continue;
        }
        let text = String::from_utf8_lossy(&object.stdout);

        // Headers, one empty line, then the message. The blank line inside an
        // armoured signature is not that separator: every continuation line of
        // a header carries a leading space, so it never reads as empty here.
        let mut lines = text.lines().skip_while(|l| !l.is_empty());
        if lines.next().is_none() {
            continue;
        }
        let message: Vec<&str> = lines.collect();
        if message.first().map(|l| l.trim()) != Some(subject.as_str()) {
            continue;
        }
        return Ok(message
            .iter()
            .find_map(|l| l.trim().strip_prefix("constraint+scope: "))
            .map(|h| h.trim().to_string()));
    }
    Ok(None)
}

/// Resolves the default branch (§7): the config value first, then
/// `refs/remotes/origin/HEAD`. If neither answers there is nothing to invent —
/// assuming `main` would be exactly the guess the tool refuses everywhere
/// else — and the error names both missing sources with both fix commands.
///
/// Pure on purpose. The two sources are passed in rather than read here, so
/// the four combinations are testable without a repository and without git. A
/// branch that only exists under some environments is a branch only tested
/// under some environments, which is the hole TASK-dc87e0ecfb6c fell into.
pub fn resolve_default_branch(
    configured: Option<&str>,
    origin_head: Option<&str>,
) -> Result<String> {
    fn named(v: Option<&str>) -> Option<&str> {
        v.map(str::trim).filter(|s| !s.is_empty())
    }
    if let Some(branch) = named(configured).or_else(|| named(origin_head)) {
        return Ok(branch.to_string());
    }
    Err(CliError::new(
        9,
        "default branch indeterminable (default_branch absent from .ank/config.yml, \
         refs/remotes/origin/HEAD absent)",
    )
    .with_hint(
        "git remote set-head origin -a\n  \
         -> or add \"default_branch: <name>\" to .ank/config.yml",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(PathBuf);

    impl Temp {
        /// A real repository on a known branch. Git's default branch name
        /// varies with the version and the user's config, so a test that
        /// relies on it passes or fails depending on who runs it; `-b` has
        /// existed since 2.28, well below our floor.
        fn new_repo() -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-git-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&p).unwrap();
            let t = Temp(p);
            t.porcelain(&["init", "-q", "-b", "main"]);
            // The CI runners carry no identity, and commit refuses without
            // one. autocrlf is pinned so that the content read back on Windows
            // is the content committed, whatever the machine's global config.
            t.porcelain(&["config", "user.email", "test@ank.local"]);
            t.porcelain(&["config", "user.name", "Test"]);
            t.porcelain(&["config", "core.autocrlf", "false"]);
            t
        }

        /// Porcelain is forbidden to the tool (ADR-b8884edcebe3), not to the
        /// harness that builds the fixture: `init` and `commit` have no
        /// plumbing equivalent worth rewriting here.
        fn porcelain(&self, args: &[&str]) {
            let st = Command::new("git")
                .current_dir(&self.0)
                .args(args)
                .status()
                .expect("git must be installed: it is a hard dependency");
            assert!(st.success(), "git {args:?}");
        }

        /// Writes a file, commits it, and returns the commit.
        fn commit(&self, path: &str, content: &str) -> String {
            let full = self.0.join(path);
            std::fs::create_dir_all(full.parent().unwrap()).unwrap();
            std::fs::write(&full, content).unwrap();
            self.porcelain(&["add", "-A"]);
            self.porcelain(&["-c", "commit.gpgsign=false", "commit", "-qm", "step"]);
            run(&self.0, &["rev-parse", "HEAD"]).unwrap()
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn ank_refs_are_enumerated_and_their_absence_is_an_empty_list() {
        let t = Temp::new_repo();
        assert_eq!(
            ank_refs(&t.0).unwrap(),
            vec![],
            "a repository with no ank ref is the nominal case, not an error"
        );

        let sha = t.commit(".ank/tasks/TASK-000000000001.md", "task\n");
        run(
            &t.0,
            &["update-ref", "refs/ank/claims/TASK-000000000001", &sha],
        )
        .unwrap();

        let refs = ank_refs(&t.0).unwrap();
        assert_eq!(refs.len(), 1, "{refs:?}");
        assert_eq!(refs[0].name, "refs/ank/claims/TASK-000000000001");
        assert_eq!(refs[0].object, sha);

        // A ref outside the namespace is not swept up with them.
        run(&t.0, &["update-ref", "refs/heads/other", &sha]).unwrap();
        assert_eq!(ank_refs(&t.0).unwrap().len(), 1);
    }

    #[test]
    fn the_current_branch_is_read_and_a_detached_head_answers_none() {
        let t = Temp::new_repo();
        assert_eq!(
            current_branch(&t.0).unwrap().as_deref(),
            Some("main"),
            "readable before the first commit: HEAD is symbolic from the start"
        );

        let sha = t.commit("a.txt", "a\n");
        run(&t.0, &["update-ref", "--no-deref", "HEAD", &sha]).unwrap();
        assert_eq!(
            current_branch(&t.0).unwrap(),
            None,
            "detached HEAD is an answer, not a failure"
        );
    }

    #[test]
    fn origin_head_is_read_and_its_absence_answers_none() {
        let t = Temp::new_repo();
        assert_eq!(
            origin_head(&t.0).unwrap(),
            None,
            "level 0 has no remote at all"
        );

        let sha = t.commit("a.txt", "a\n");
        run(&t.0, &["update-ref", "refs/remotes/origin/main", &sha]).unwrap();
        run(
            &t.0,
            &["symbolic-ref", ORIGIN_HEAD, "refs/remotes/origin/main"],
        )
        .unwrap();
        assert_eq!(origin_head(&t.0).unwrap().as_deref(), Some("main"));
    }

    #[test]
    fn reachability_reads_the_two_answers_of_merge_base() {
        let t = Temp::new_repo();
        let first = t.commit("a.txt", "a\n");
        let second = t.commit("a.txt", "b\n");

        assert!(is_ancestor(&t.0, &first, &second).unwrap());
        assert!(
            !is_ancestor(&t.0, &second, &first).unwrap(),
            "exit 1 is the answer no, not a failure"
        );

        let err = is_ancestor(&t.0, "no-such-rev", &second).unwrap_err();
        assert_eq!(err.code, 9, "an unknown revision is an environment error");
        assert!(err.hint.is_some());
    }

    #[test]
    fn a_file_is_read_as_the_branch_carries_it_not_as_the_tree_holds_it() {
        let t = Temp::new_repo();
        let path = ".ank/tasks/TASK-000000000001.md";
        t.commit(path, "status: done\n");

        // The working tree moves on; the branch does not. This is the whole
        // point of the primitive: the pruning predicate asks the branch.
        std::fs::write(t.0.join(path), "status: open\n").unwrap();
        assert_eq!(
            file_at(&t.0, "main", path).unwrap().as_deref(),
            Some("status: done\n")
        );

        assert_eq!(
            file_at(&t.0, "main", ".ank/tasks/TASK-000000000002.md").unwrap(),
            None,
            "a path absent from the revision is an absence, not an error"
        );

        let err = file_at(&t.0, "no-such-branch", path).unwrap_err();
        assert_eq!(
            err.code, 9,
            "an unresolvable branch must not read as an absent file"
        );
        assert!(err.hint.is_some());
    }

    #[test]
    fn the_default_branch_resolves_from_two_sources_and_invents_nothing() {
        assert_eq!(
            resolve_default_branch(Some("trunk"), None).unwrap(),
            "trunk"
        );
        assert_eq!(resolve_default_branch(None, Some("main")).unwrap(), "main");
        assert_eq!(
            resolve_default_branch(Some("trunk"), Some("main")).unwrap(),
            "trunk",
            "the config wins: it is the explicit statement"
        );
        assert_eq!(
            resolve_default_branch(Some("  "), Some("main")).unwrap(),
            "main",
            "a blank value is an absence, not a branch"
        );

        let err = resolve_default_branch(None, None).unwrap_err();
        assert_eq!(err.code, 9);
        assert!(err.message.contains("default_branch"), "{}", err.message);
        assert!(
            err.message.contains("refs/remotes/origin/HEAD"),
            "{}",
            err.message
        );

        let hint = err.hint.clone().unwrap();
        assert!(hint.contains("git remote set-head origin -a"), "{hint}");
        assert!(hint.contains("\"default_branch: <name>\""), "{hint}");

        // Rendered, the two fixes come out as two arrow lines, as in §7.
        let rendered = err.render();
        assert_eq!(
            rendered
                .lines()
                .filter(|l| l.trim_start().starts_with("->"))
                .count(),
            2,
            "{rendered}"
        );
    }

    #[test]
    fn version_is_parsed_with_its_distribution_suffixes() {
        assert_eq!(parse_version("git version 2.43.0"), Some((2, 43)));
        assert_eq!(parse_version("git version 2.43.0.windows.1"), Some((2, 43)));
        assert_eq!(
            parse_version("git version 2.34.1 (Apple Git-141)"),
            Some((2, 34))
        );
        assert_eq!(parse_version("git version 3.0.0"), Some((3, 0)));
        assert_eq!(parse_version("not a version"), None);
    }

    #[test]
    fn the_version_floor_is_refused_with_code_9_and_the_link() {
        let err = check_version((2, 33)).unwrap_err();
        assert_eq!(err.code, 9);
        assert!(err.message.contains("2.33"), "{}", err.message);
        assert!(err.message.contains("2.34"), "{}", err.message);
        assert_eq!(err.hint.as_deref(), Some(INSTALL_URL));

        assert!(check_version((2, 34)).is_ok(), "the floor is inclusive");
        assert!(check_version((2, 45)).is_ok());
    }

    #[test]
    fn outside_a_git_repository_exits_with_code_9_and_git_init() {
        let dir = std::env::temp_dir().join(format!("ank-nogit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // A temporary directory may itself sit under a git repository; we only
        // assert when it does not, otherwise the assertion would be wrong.
        if toplevel(&dir).is_err() {
            let err = toplevel(&dir).unwrap_err();
            assert_eq!(err.code, 9);
            assert_eq!(err.hint.as_deref(), Some("git init"));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
