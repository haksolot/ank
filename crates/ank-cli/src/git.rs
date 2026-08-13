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
    // `diff-tree` and not `show` or `log --name-status`: the porcelain pair
    // format their output for a reader, `diff-tree` for a program. Its
    // `--name-status` records have carried the same shape since rename
    // detection existed, and `-z` removes the one part that was ever
    // version-dependent — the C-quoting of an unusual path.
    "diff-tree",
    // The three level 1 needs (§7), each admitted on the criterion rather than
    // on convenience. `push` signals a refused compare-and-swap by its exit
    // code, which is the same contract `update-ref` is trusted for, and
    // `--force-with-lease` is what makes the swap explicit rather than
    // inferred. `ls-remote` prints `<oid>\t<refname>` per line and has since it
    // existed. `fetch` is read for its exit code alone, never parsed.
    "push",
    "fetch",
    "ls-remote",
];

/// The verb an invocation actually runs, skipping the leading `-c <key>=<value>`
/// pairs.
///
/// Those pairs exist for one caller — [`signature_of`], which has to hand git a
/// configuration the repository must not be made to carry permanently — and the
/// plumbing rule is about the command whose output we parse, not about the
/// configuration it runs under. Reading `args[0]` blindly would have made `-c`
/// look like porcelain and fired the assertion on a correct call.
fn verb_of<'a>(args: &'a [&'a str]) -> Option<&'a str> {
    let mut rest = args;
    while let Some(("-c", tail)) = rest.split_first().map(|(h, t)| (*h, t)) {
        // `-c` takes one argument; dropping both is what leaves the verb first.
        rest = tail.get(1..).unwrap_or(&[]);
    }
    rest.first().copied()
}

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
        verb_of(args)
            .map(|a| PLUMBING.contains(&a))
            .unwrap_or(false),
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

// ---------------------------------------------------------------------------
// The remote, and the compare-and-swap that crosses clones (§7, level 1)
// ---------------------------------------------------------------------------

/// The remote claims are arbitrated through, or `None` for a repository that
/// has none.
///
/// **Absence is level 0 and not a failure.** A repository with no remote is the
/// default mode and the only one that ever shipped before this: it must stay
/// silent, or every solo `claim` would carry a warning about a network nobody
/// asked for. What warns is a remote that exists and cannot be reached.
///
/// `origin` by name rather than by discovery. §7 says "any existing remote",
/// and picking one out of several would be a rule nobody declared; `origin` is
/// what `init` writes the `refs/ank/*` refspec for, and it is the name every
/// clone already has.
pub fn remote(cwd: &Path) -> Result<Option<String>> {
    let out = output(cwd, &["config", "--get", "remote.origin.url"])?;
    if !out.status.success() {
        return Ok(None);
    }
    let url = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok((!url.is_empty()).then_some(url))
}

/// What a push of a claim ref did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pushed {
    /// The remote took it: what this clone wrote holds repository-wide.
    Ok,
    /// The remote refused the swap — another clone got there first. The object
    /// it holds instead, when `ls-remote` could name it.
    Refused { holds: Option<String> },
    /// The remote exists and could not be reached. Degrade, do not fail (§2):
    /// the write stands locally and the caller says so out loud.
    Unreachable { reason: String },
}

/// Pushes a ref under `refs/ank/*` with a lease, which is the compare-and-swap
/// of §7 crossing clones.
///
/// Named for the ref and not for the claim: it carries claim records,
/// completion records and detached proofs alike (ADR-493471d64ba0), and a
/// helper called `push_claim` while pushing a proof would be a comment that
/// lies in the one place a reader checks first.
///
/// `expect` is the object the caller read before writing, exactly as it is for
/// the local `update-ref`: `None` means the ref must not exist on the remote.
/// Git spells that as `--force-with-lease=<ref>:<expect>` with an empty
/// expectation, and the check runs server-side and atomically, so two clones
/// racing produce one winner without either trusting the other.
///
/// `new` of `None` is a deletion, which `release` and `close` need: a ref left
/// behind on the remote makes a handed-back task unclaimable everywhere else
/// until its TTL runs out — a state this change would otherwise create.
///
/// **A refused push is distinguished from an unreachable remote by asking, not
/// by reading stderr.** Push says "rejected" in prose written for people, and
/// parsing it would be the fragility ADR-b8884edcebe3 exists to prevent. So a
/// failure is followed by one `ls-remote` of the same ref: an answer means the
/// remote is there and the swap genuinely lost, and no answer at all means the
/// remote is what failed. It costs a round trip on the failing path only.
pub fn push_ref(
    cwd: &Path,
    refname: &str,
    new: Option<&str>,
    expect: Option<&str>,
) -> Result<Pushed> {
    let lease = format!("--force-with-lease={refname}:{}", expect.unwrap_or(""));
    let spec = match new {
        Some(object) => format!("{object}:{refname}"),
        None => format!(":{refname}"),
    };
    let args = ["push", lease.as_str(), "origin", spec.as_str()];
    let out = output(cwd, &args)?;
    if out.status.success() {
        return Ok(Pushed::Ok);
    }
    match ls_remote(cwd, refname) {
        Ok(holds) => Ok(Pushed::Refused { holds }),
        // The remote could not be interrogated either, so the push failure was
        // never about contention.
        Err(_) => Ok(Pushed::Unreachable {
            reason: String::from_utf8_lossy(&out.stderr).trim().to_string(),
        }),
    }
}

/// The object the remote holds at `refname`, or `None` when the ref is absent
/// there. An unreachable remote is an error, which is what tells `push_ref`
/// which kind of failure it just had.
///
/// One line per ref, `<oid>\t<refname>`, which is what makes `ls-remote`
/// admissible under ADR-b8884edcebe3.
pub fn ls_remote(cwd: &Path, refname: &str) -> Result<Option<String>> {
    let args = ["ls-remote", "origin", refname];
    let out = output(cwd, &args)?;
    if !out.status.success() {
        return Err(failed(&args, &out));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    Ok(text
        .lines()
        .find_map(|l| l.split_whitespace().next().map(str::to_string)))
}

/// Brings the remote's view of one `refs/ank/*` ref into this clone.
///
/// `claim` runs it before deciding, so a task held in another clone is refused
/// with the holder named rather than taken and then rolled back. The push is
/// still what arbitrates — this only closes the common case politely. A reader
/// of a detached proof runs it for the other reason: the record it wants was
/// written by a pipeline, in no clone at all (ADR-493471d64ba0).
///
/// Failure is not an error to the caller: an unreachable remote means the
/// answer is the local one, and the push that follows is where that gets said.
/// Read for its exit code alone, never parsed.
pub fn fetch_ref(cwd: &Path, refname: &str) -> Result<bool> {
    let spec = format!("+{refname}:{refname}");
    let out = output(cwd, &["fetch", "--quiet", "origin", spec.as_str()])?;
    Ok(out.status.success())
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
///
/// This is the gate a **coordinating** verb passes through — `claim`, `log`,
/// `done`, `release`, `close`, `accept`, `attest`, `init` — and it is called per
/// verb, never at startup (ADR-9307e5d214a7). It does two things, and both are
/// load-bearing: a repository alone is not enough, because the 2.34 floor is
/// what SSH signing needs, so a gate that only resolved the toplevel would let
/// an old git through to `accept`.
pub fn ensure_usable(cwd: &Path) -> Result<PathBuf> {
    check_version(version()?)?;
    toplevel(cwd)
}

/// Whether a coordinating operation could run here: git present, recent enough,
/// and a repository around `cwd`.
///
/// Never an error, and that is the whole difference with [`ensure_usable`]. The
/// verbs that coordinate call `ensure_usable` and exit 9 naming the command to
/// run; the readers call this to choose which half of their answer they can
/// produce. A probe that could refuse would put the startup gate back one level
/// down, which is the defect ADR-9307e5d214a7 exists to remove.
pub fn usable_here(cwd: &Path) -> bool {
    ensure_usable(cwd).is_ok()
}

/// The **common** git directory of the repository containing `cwd`, absolute,
/// or `None` when `cwd` is not inside a repository at all.
///
/// The common directory and not the toplevel, and the difference is the whole
/// point: a linked worktree has a toplevel of its own and shares `refs/` with
/// the checkout that made it, so comparing toplevels would report two worktrees
/// of one repository as two repositories. The question being asked is where a
/// claim ref lands (§7), and that is the common directory.
///
/// `--path-format=absolute` rather than canonicalising the answer ourselves:
/// `--git-common-dir` is relative when git is run from the repository root, and
/// two paths compared without agreeing on that are two strings that differ for
/// no reason. It has existed since git 2.31 and [`MIN_VERSION`] is 2.34.
///
/// Never an error. Every caller is asking a question whose answer may legibly
/// be "there is no repository here", and a failure to run git is that answer
/// too — this is used to decide whether to say something extra, never to
/// refuse.
pub fn common_dir(cwd: &Path) -> Option<PathBuf> {
    let out = output(
        cwd,
        &["rev-parse", "--path-format=absolute", "--git-common-dir"],
    )
    .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!text.is_empty()).then(|| PathBuf::from(text))
}

/// Whether the resolved corpus and the working directory sit in two different
/// git repositories (TASK-2f01baf94632).
///
/// `discover` walks parents until it finds a `.ank/`, and its stopping
/// condition is the filesystem root — it never consults `.git` and never stops
/// at a repository boundary. From `outer/inner/src` with no `inner/.ank/`, the
/// walk reaches `outer/.ank/` and the verb runs there. Measured: a `claim` made
/// from `inner/src` writes `refs/ank/claims/<id>` into `outer/.git`, and
/// `inner/.git` — the repository holding the code being changed — ends with no
/// ank ref at all. The coordination plane and the code part company, and
/// nothing says so.
///
/// Pure on purpose, like [`resolve_default_branch`]: the two sources are passed
/// in rather than read here, so every combination is testable without building
/// two repositories on disk.
pub fn crosses_repository(here: Option<&Path>, root: Option<&Path>) -> bool {
    match (here, root) {
        // Two repositories, and the claim plane is in the one the caller is not
        // standing in.
        (Some(a), Some(b)) => a != b,
        // No repository here, and one where the corpus was found. Worth saying
        // for the same reason: the refs are somewhere the caller is not.
        (None, Some(_)) => true,
        // The corpus is in no repository at all. This used to be unreachable —
        // `ensure_usable` refused at startup before any caller got here — and
        // git being required per verb makes it ordinary (ADR-9307e5d214a7). It
        // is still not a crossing: there is no second plane for the refs to
        // land in, so there is nothing to warn about being on the wrong side
        // of. `check` is what reports the state, in the one line saying the
        // coordination half was skipped.
        (_, None) => false,
    }
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

/// Where a path went, when the commit that removed it recorded a rename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rename {
    /// The path the file carries now, repository-relative, `/`-separated.
    pub to: String,
    /// The commit that moved it, abbreviated for a reader to paste back.
    pub sha: String,
}

/// The rename that killed `path`, or `None` when git cannot name one
/// (ADR-97beaf55e73a).
///
/// Two plumbing calls and no porcelain. `rev-list -1 HEAD -- <path>` is the
/// last commit that touched the path, and `diff-tree` on that commit is what
/// says whether the touch was a rename.
///
/// **No `--full-history` here, and the difference from [`ratification_at`] is
/// deliberate.** There the target is one specific commit, identified by its
/// subject, and simplification dropping it would lose the anchor. Here the
/// target is the change itself: default simplification walks to the commit that
/// actually made it, where `--full-history` would keep the merges that merely
/// carried it — and a merge is a commit `diff-tree` prints nothing for, so
/// asking for more history would answer less often.
///
/// `None` is the honest answer for everything this cannot explain: a deletion,
/// a move under git's similarity threshold, a path renamed by a merge, a
/// shallow clone, a repository with no `HEAD`. The caller must never render any
/// of them as a claim about what happened to the file.
pub fn rename_of(cwd: &Path, path: &str) -> Result<Option<Rename>> {
    let args = ["rev-list", "-1", "HEAD", "--", path];
    let out = output(cwd, &args)?;
    if !out.status.success() {
        return Ok(None);
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if sha.is_empty() {
        return Ok(None);
    }

    // `-r` to reach into subtrees, `-M` to detect the rename at all,
    // `--no-commit-id` so the first record is a change and not the commit name.
    // No pathspec: it would restrict rename detection to the paths named, and
    // the destination is precisely the path we do not know yet.
    let args = [
        "diff-tree",
        "-M",
        "-r",
        "-z",
        "--name-status",
        "--no-commit-id",
        sha.as_str(),
    ];
    let out = output(cwd, &args)?;
    if !out.status.success() {
        return Ok(None);
    }
    let to = rename_target(&String::from_utf8_lossy(&out.stdout), path);
    Ok(to.map(|to| Rename {
        to,
        sha: sha.chars().take(12).collect(),
    }))
}

/// The destination `text` records for `from`, if any.
///
/// Split out and pure because every branch of it is a shape of git's output,
/// and a shape is cheaper to assert on directly than to stage in a repository.
///
/// `--name-status -z` emits NUL-terminated fields: a status, then one path, or
/// two when the status is a rename or a copy. The letter carries a similarity
/// score (`R100`), which is why the test is on the first byte.
fn rename_target(text: &str, from: &str) -> Option<String> {
    let mut fields = text.split('\0').filter(|f| !f.is_empty());
    while let Some(status) = fields.next() {
        let renamed = status.starts_with('R');
        let Some(src) = fields.next() else {
            return None;
        };
        if !renamed && !status.starts_with('C') {
            continue;
        }
        let Some(dst) = fields.next() else {
            return None;
        };
        if renamed && src == from {
            return Some(dst.to_string());
        }
    }
    None
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
type Memo = OnceLock<Mutex<HashMap<(PathBuf, String), Option<Ratification>>>>;
static RATIFICATIONS: Memo = OnceLock::new();

/// A ratification commit, and the anchor it recorded.
///
/// The commit travels with the hash because the hash alone cannot be trusted:
/// it is only worth as much as the signature on the object carrying it, and
/// checking that signature means knowing which object to ask about
/// (TASK-d31af22248d9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ratification {
    pub sha: String,
    pub anchor: String,
}

/// `paths` are the candidate paths of the entity, canonical first. There is
/// more than one while the previous layout is read (§6), and an ADR ratified
/// before the move has its commit on the path it had then.
///
/// They are tried inside this function rather than by the caller, because the
/// memo below is keyed on the entity and not on the path: a caller looping over
/// paths would cache the first miss and never reach the second candidate. That
/// is not hypothetical — it is what happened the moment the second path
/// appeared, and every ratification in this repository read as unverifiable.
pub fn ratification_at(cwd: &Path, id: &str, paths: &[String]) -> Result<Option<Ratification>> {
    let key = (cwd.to_path_buf(), id.to_string());
    let memo = RATIFICATIONS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(seen) = memo.lock() {
        if let Some(hit) = seen.get(&key) {
            return Ok(hit.clone());
        }
    }
    let mut found = None;
    for path in paths {
        found = ratification_uncached(cwd, id, path)?;
        if found.is_some() {
            break;
        }
    }
    if let Ok(mut seen) = memo.lock() {
        seen.insert(key, found.clone());
    }
    Ok(found)
}

fn ratification_uncached(cwd: &Path, id: &str, path: &str) -> Result<Option<Ratification>> {
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
            .map(|h| Ratification {
                sha: sha.trim().to_string(),
                anchor: h.trim().to_string(),
            }));
    }
    Ok(None)
}

/// What git says about the signature on a commit, before anyone decides what it
/// means.
///
/// Deliberately facts and not a verdict: which keys are allowed to ratify is a
/// question about `.ank/allowed_signers` (§8), and answering it here would put
/// a policy decision inside the plumbing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureFacts {
    /// `%G?`: `G` good, `U` good but untrusted, `E` cannot be checked, `N` no
    /// signature, `B` bad, `X`/`Y` expired signature or key, `R` revoked key.
    pub status: char,
    /// `%GF`: the signing key's fingerprint, empty when there is none to name.
    /// A 40-hex fingerprint under OpenPGP, `SHA256:<base64>` under SSH.
    pub fingerprint: String,
}

/// Reads the signature of `sha`.
///
/// `allowed_signers` is passed to git rather than applied here, and it only
/// bites under `gpg.format = ssh`, which is the only format git checks such a
/// file for. Under OpenPGP git resolves the signature through the keyring and
/// ignores the file entirely — so the caller does that matching itself, which
/// is the whole reason the fingerprint comes back with the status.
///
/// `-c` rather than repository configuration: pointing a repository's
/// `gpg.ssh.allowedSignersFile` at `.ank/` would change how every commit in it
/// verifies, for every tool, to serve one check of ours.
pub fn signature_of(
    cwd: &Path,
    sha: &str,
    allowed_signers: Option<&Path>,
) -> Result<SignatureFacts> {
    // `rev-list` and not `verify-commit`: the exit code of `verify-commit`
    // collapses "no signature" and "cannot check it" into the same failure,
    // and those two are precisely the states this must keep apart. The
    // placeholders are git's documented pretty-format interface.
    let signers = allowed_signers.map(|p| format!("gpg.ssh.allowedSignersFile={}", p.display()));
    let mut args: Vec<&str> = Vec::new();
    if let Some(cfg) = signers.as_deref() {
        args.push("-c");
        args.push(cfg);
    }
    args.extend_from_slice(&["rev-list", "--max-count=1", "--format=%G?%n%GF", sha]);

    let out = output(cwd, &args)?;
    if !out.status.success() {
        return Err(failed(&args, &out));
    }
    let text = String::from_utf8_lossy(&out.stdout);

    // `--format` implies `--pretty`, whose first line is `commit <sha>`. The
    // two placeholders follow, in order, one per line.
    let mut lines = text.lines().skip(1);
    let status = lines
        .next()
        .and_then(|l| l.trim().chars().next())
        // An empty first placeholder is git declining to say anything about a
        // signature, which is the same thing as there being none.
        .unwrap_or('N');
    let fingerprint = lines.next().unwrap_or_default().trim().to_string();
    Ok(SignatureFacts {
        status,
        fingerprint,
    })
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
         -> or ank config default_branch <name>",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    // `no_remote_verb_is_allowed_while_the_spec_says_level_1_is_unimplemented`
    // stood here and did its job: it failed the day level 1 landed, naming what
    // the same change owed the specification (TASK-83d6eefdb36e). §7 no longer
    // says level 1 is unimplemented, so the sentence it guarded is gone and the
    // guard goes with it (TASK-82c3341502c1). A negative test kept past the
    // absence it describes asserts the opposite of what is true.

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
            // Signing off at creation, not at each commit (TASK-40a972e98a9a).
            t.porcelain(&["config", "commit.gpgsign", "false"]);
            t.porcelain(&["config", "tag.gpgsign", "false"]);
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
            self.porcelain(&["commit", "-qm", "step"]);
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
        // The second fix names the command, not the file to open: telling an
        // agent to edit .ank/config.yml is the tool instructing it to do what
        // ADR-01b6dd05f0db forbids (ADR-e64dfaafd578).
        assert!(hint.contains("ank config default_branch <name>"), "{hint}");
        assert!(
            !hint.contains(".ank/config.yml"),
            "the hint still points at the file: {hint}"
        );

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

    /// The four combinations, without building two repositories on disk
    /// (TASK-2f01baf94632).
    ///
    /// Pure for the reason `resolve_default_branch` is: a decision that only
    /// exists inside a fixture is a decision tested under one layout, and the
    /// interesting cases here are the ones nobody sets up by accident.
    #[test]
    fn crossing_a_repository_boundary_is_decided_from_the_two_common_dirs() {
        let a = Path::new("/w/outer/.git");
        let b = Path::new("/w/outer/inner/.git");

        assert!(
            crosses_repository(Some(b), Some(a)),
            "standing in the inner repository with the corpus resolved in the \
             outer one is the whole finding"
        );
        assert!(
            !crosses_repository(Some(a), Some(a)),
            "one repository, whichever subdirectory the caller stands in"
        );
        assert!(
            crosses_repository(None, Some(a)),
            "no repository here and refs over there is the same parting"
        );
        assert!(
            !crosses_repository(Some(a), None),
            "a corpus outside any repository is ensure_usable's code 9, and \
             saying it twice helps nobody"
        );
        assert!(!crosses_repository(None, None));
    }

    /// Two worktrees of one repository are one repository.
    ///
    /// This is why the comparison is on the common directory and not on the
    /// toplevel: a linked worktree has a toplevel of its own, so a toplevel
    /// comparison would warn about a layout where the refs are in fact shared —
    /// and a warning that fires when nothing is wrong is a warning nobody reads
    /// the day something is.
    #[test]
    fn a_linked_worktree_shares_its_common_dir_with_the_checkout_that_made_it() {
        let dir = std::env::temp_dir().join(format!("ank-common-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let main = dir.join("main");
        std::fs::create_dir_all(&main).unwrap();

        let git = |cwd: &Path, args: &[&str]| {
            Command::new("git")
                .current_dir(cwd)
                .args(args)
                .output()
                .expect("git is a hard dependency")
        };
        git(&main, &["init", "-q", "-b", "main"]);
        git(&main, &["config", "user.email", "t@ank.local"]);
        git(&main, &["config", "user.name", "T"]);
        // Signing off at creation, not at each commit (TASK-40a972e98a9a).
        git(&main, &["config", "commit.gpgsign", "false"]);
        git(&main, &["config", "tag.gpgsign", "false"]);
        std::fs::write(main.join("seed.txt"), "x").unwrap();
        git(&main, &["add", "-A"]);
        git(&main, &["commit", "-qm", "s"]);
        let wt = dir.join("wt");
        git(
            &main,
            &["worktree", "add", "-q", wt.to_str().unwrap(), "-b", "side"],
        );

        let from_main = common_dir(&main);
        let from_wt = common_dir(&wt);
        assert!(from_main.is_some() && from_wt.is_some(), "both resolved");
        assert_eq!(
            from_main, from_wt,
            "a linked worktree shares refs/ with the checkout that made it, so \
             it must not read as a second repository"
        );
        assert!(!crosses_repository(
            from_wt.as_deref(),
            from_main.as_deref()
        ));

        // And a repository nested inside another does read as a second one.
        let inner = main.join("inner");
        std::fs::create_dir_all(&inner).unwrap();
        git(&inner, &["init", "-q", "-b", "main"]);
        git(&inner, &["config", "commit.gpgsign", "false"]);
        git(&inner, &["config", "tag.gpgsign", "false"]);
        let from_inner = common_dir(&inner);
        assert!(from_inner.is_some());
        assert_ne!(from_inner, from_main);
        assert!(crosses_repository(
            from_inner.as_deref(),
            from_main.as_deref()
        ));

        let _ = std::fs::remove_dir_all(&dir);
    }

    // -----------------------------------------------------------------------
    // Rename detection (ADR-97beaf55e73a)
    // -----------------------------------------------------------------------

    /// The shapes of `--name-status -z`, asserted on the bytes rather than
    /// staged in a repository.
    ///
    /// Every branch here is a way for the parser to lose its place: a rename it
    /// must skip past to reach the one asked about, a status with one field
    /// where the record before it had two, and a truncated tail that a
    /// `next().unwrap()` would have panicked on.
    #[test]
    fn a_rename_record_is_read_by_its_source_and_the_others_are_stepped_over() {
        let two_field = "M\0a.rs\0R100\0old.rs\0new.rs\0A\0z.rs\0";
        assert_eq!(
            rename_target(two_field, "old.rs"),
            Some("new.rs".to_string()),
            "the rename is found past a status carrying one path"
        );
        assert_eq!(
            rename_target(two_field, "a.rs"),
            None,
            "a path that was modified was not renamed, and saying otherwise \
             would put a file where it never went"
        );
        assert_eq!(
            rename_target("R100\0first.rs\0moved.rs\0R80\0old.rs\0new.rs\0", "old.rs"),
            Some("new.rs".to_string()),
            "a rename record must be stepped over as two fields, not one"
        );
        // A copy is `C` and carries two paths like a rename does. The source
        // survives a copy, so it can never be what killed a scope — but a
        // parser that read it as one field would misalign every record after
        // it, which is how the answer becomes wrong for a different path.
        assert_eq!(
            rename_target("C100\0kept.rs\0copy.rs\0R100\0old.rs\0new.rs\0", "old.rs"),
            Some("new.rs".to_string())
        );
        assert_eq!(rename_target("C100\0kept.rs\0copy.rs\0", "kept.rs"), None);
        assert_eq!(rename_target("", "old.rs"), None);
        assert_eq!(
            rename_target("R100\0old.rs\0", "old.rs"),
            None,
            "output cut mid-record answers nothing, and must not panic"
        );
    }

    #[test]
    fn a_renamed_path_names_where_it_went_and_a_deleted_one_names_nothing() {
        let t = Temp::new_repo();
        t.commit("src/old.rs", "fn main() {}\n// enough body to be similar\n");

        // Through git's own rename detection and not a hand-written record:
        // what is under test is whether `-M` fires on a real commit, which is
        // the one thing a parser test cannot say.
        std::fs::rename(t.0.join("src/old.rs"), t.0.join("src/new.rs")).unwrap();
        t.porcelain(&["add", "-A"]);
        t.porcelain(&["commit", "-qm", "move it"]);
        let sha = run(&t.0, &["rev-parse", "HEAD"]).unwrap();

        let moved = rename_of(&t.0, "src/old.rs").unwrap().expect("git saw it");
        assert_eq!(moved.to, "src/new.rs");
        assert!(
            sha.starts_with(&moved.sha),
            "the commit named must be the one that moved it: {} is not a \
             prefix of {sha}",
            moved.sha
        );

        // A deletion is the case the caller must render as nothing at all.
        t.commit("src/gone.rs", "fn gone() {}\n");
        std::fs::remove_file(t.0.join("src/gone.rs")).unwrap();
        t.porcelain(&["add", "-A"]);
        t.porcelain(&["commit", "-qm", "delete it"]);
        assert_eq!(
            rename_of(&t.0, "src/gone.rs").unwrap(),
            None,
            "a deleted file went nowhere, and no rename may be invented for it"
        );

        // A path that never existed is the third silence, and it must not be an
        // error: a scope can be a typo, and `check` reports that as it is.
        assert_eq!(rename_of(&t.0, "src/never.rs").unwrap(), None);
    }

    /// A repository with no commit at all: `rev-list HEAD` fails, and the
    /// answer is silence rather than the environment error `run` would raise.
    #[test]
    fn a_repository_with_no_head_answers_nothing_rather_than_failing() {
        let t = Temp::new_repo();
        assert_eq!(rename_of(&t.0, "src/old.rs").unwrap(), None);
    }
}
