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
use ank_contract::ExitCode;
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
    CliError::new(ExitCode::Environment, "git not found in PATH").with_hint(INSTALL_URL)
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
                CliError::new(
                    ExitCode::Environment,
                    format!("git {}: {e}", args.join(" ")),
                )
            }
        })
}

/// The environment error for a git command that failed for a reason we did not
/// expect. Public alongside [`output`]: a caller reading exit codes itself
/// still needs one single way to say "git broke", stderr included.
pub fn failed(args: &[&str], out: &Output) -> CliError {
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    CliError::new(
        ExitCode::Environment,
        format!("git {} failed: {stderr}", args.join(" ")),
    )
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

/// Every ref the remote holds under `pattern`, with the object each points at.
///
/// The plural of [`ls_remote`], and the reading half of `status --remote`
/// (ADR-47e2ac102f58): a reader that wants the whole claims namespace asks for
/// it once rather than once per task, and it **never fetches** — writing refs
/// into this clone as a side effect of a question would be a reader sanitising
/// the plane underneath everybody else.
///
/// What that costs is exact and worth stating here, where the caller reads it:
/// `ls-remote` carries names and objects, not contents. A claim ref this clone
/// has never fetched has no object here to `cat-file`, so who holds it and until
/// when are unknowable without the fetch. The ref's existence is the answer, and
/// it is the one the question was about.
///
/// An unreachable remote, or none at all, is an **error** and not an empty list:
/// "origin holds no claim" and "origin was not read" are different answers, and
/// a caller that cannot tell them apart would print the first when it means the
/// second.
pub fn ls_remote_refs(cwd: &Path, pattern: &str) -> Result<Vec<AnkRef>> {
    let args = ["ls-remote", "origin", pattern];
    let out = output(cwd, &args)?;
    if !out.status.success() {
        return Err(failed(&args, &out));
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut refs = Vec::new();
    for line in text.lines() {
        // `<oid>\t<refname>`, the same two fields `for-each-ref` is asked for
        // above, and the same tab: it cannot appear in a ref name.
        let Some((object, name)) = line.trim_end().split_once('\t') else {
            continue;
        };
        refs.push(AnkRef {
            name: name.to_string(),
            object: object.to_string(),
        });
    }
    Ok(refs)
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
            CliError::new(ExitCode::Environment, format!("git --version: {e}"))
        }
    })?;
    let text = String::from_utf8_lossy(&out.stdout);
    parse_version(&text).ok_or_else(|| {
        CliError::new(
            ExitCode::Environment,
            format!("unreadable git version: {}", text.trim()),
        )
        .with_hint(INSTALL_URL)
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
            ExitCode::Environment,
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
                CliError::new(ExitCode::Environment, format!("git rev-parse: {e}"))
            }
        })?;
    if !out.status.success() {
        return Err(CliError::new(
            ExitCode::Environment,
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
        let (name, object) = line.split_once('\t').ok_or_else(|| {
            CliError::new(
                ExitCode::Environment,
                format!("unreadable for-each-ref output: {line}"),
            )
        })?;
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
/// an ancestor (ADR-6d8736c04cfa).
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
    // **Asked once per revision and not once per file** (TASK-1b3d7b61dc8f).
    // A branch does not move under a running process, and `check` reads one
    // file per entity from the same one: 99 of the 447 git starts on this
    // repository were this question, re-asked, at 61 ms each. The answer is
    // cached, the error is not -- an unresolvable revision is raised every time
    // it is asked about, so a caller that ignores the first refusal is refused
    // again rather than silently served.
    // Read from the batch when one was loaded for this revision, and asked of
    // git when it was not (TASK-5f05e0c22f7b). An accelerator and never the
    // authority: a path the batch does not carry is a path git is asked about,
    // which is also what keeps a caller outside `check` answering correctly.
    if let Some(hit) = preloaded(cwd, rev, path) {
        return Ok(hit);
    }
    if !resolves(cwd, rev)? {
        return Err(CliError::new(
            ExitCode::Environment,
            format!("branch {rev} not found in this repository"),
        )
        .with_hint(format!("git fetch origin {rev}")));
    }
    let target = format!("{rev}:{path}");
    let out = output(cwd, &["cat-file", "-p", target.as_str()])?;
    if !out.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&out.stdout).to_string()))
}

/// The same call as [`output`], with bytes handed to git on stdin.
///
/// One command reading what another produced is the shape `diff-tree --stdin`
/// wants, and doing it in-process rather than through a pipeline keeps §13's
/// rule that no shell stands between ank and git.
pub fn output_with_stdin(cwd: &Path, args: &[&str], input: &[u8]) -> Result<Output> {
    use std::io::Write as _;
    debug_assert!(
        verb_of(args)
            .map(|a| PLUMBING.contains(&a))
            .unwrap_or(false),
        "porcelain forbidden (ADR-b8884edcebe3): {args:?}"
    );
    let fail = |e: std::io::Error| {
        if e.kind() == std::io::ErrorKind::NotFound {
            env_missing()
        } else {
            CliError::new(
                ExitCode::Environment,
                format!("git {}: {e}", args.join(" ")),
            )
        }
    };
    let mut child = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(fail)?;
    // Written before the output is collected, and the result is deliberately
    // ignored: git closing stdin early is how it says it has read enough, and a
    // broken pipe there is an answer rather than a failure.
    if let Some(mut sink) = child.stdin.take() {
        let _ = sink.write_all(input);
    }
    child.wait_with_output().map_err(fail)
}

/// Reads many objects in one process.
///
/// **One `cat-file --batch` where there was one `cat-file -p` per object**
/// (TASK-5f05e0c22f7b). The coordination plane holds one ref per claim and per
/// proof, and reading them one at a time cost a process each -- 95 of them on
/// this repository, at 61 ms a start, paid by `check`, `find`, `graph`, `scope`
/// and `status` alike.
///
/// **The framing is a size, never a separator**, which is the whole reason this
/// mode exists: git answers `<name> <type> <size>` on one line, then exactly
/// that many bytes, then a newline. A record therefore cannot be confused with
/// content that happens to look like a header, and content containing a newline
/// -- which every record here does -- is read exactly as long as it is. The
/// lesson was paid for in TASK-1b3d7b61dc8f, where a separator a record could
/// contain silently swallowed every ratification after the first commit with no
/// body.
///
/// A name git cannot resolve comes back as `<name> missing` and is simply
/// absent from the answer, which is what every caller already means by a lookup
/// that found nothing.
///
/// Bytes and not text: the size is a byte count, so slicing a lossily decoded
/// string would cut in the wrong place the first time an object carried
/// something that is not UTF-8. The values are decoded one at a time, after the
/// framing has done its work.
pub fn cat_file_batch(cwd: &Path, objects: &[String]) -> Result<HashMap<String, String>> {
    let mut found = HashMap::new();
    if objects.is_empty() {
        return Ok(found);
    }
    let mut input = String::new();
    for o in objects {
        input.push_str(o);
        input.push('\n');
    }
    let out = output_with_stdin(cwd, &["cat-file", "--batch"], input.as_bytes())?;
    if !out.status.success() {
        return Ok(found);
    }
    let body = out.stdout;
    let mut at = 0usize;
    while at < body.len() {
        let Some(eol) = body[at..].iter().position(|b| *b == b'\n') else {
            break;
        };
        let header = String::from_utf8_lossy(&body[at..at + eol]).to_string();
        at += eol + 1;
        let mut parts = header.split_whitespace();
        let Some(name) = parts.next() else { continue };
        let (Some(_kind), Some(size)) = (parts.next(), parts.next()) else {
            // `<name> missing`, and the caller learns it by absence.
            continue;
        };
        let Ok(size) = size.parse::<usize>() else {
            break;
        };
        if at + size > body.len() {
            break;
        }
        let value = String::from_utf8_lossy(&body[at..at + size]).to_string();
        // The size, then git's own trailing newline.
        at += size + 1;
        found.insert(name.to_string(), value);
    }
    Ok(found)
}

/// Every entity file the default branch carries, read in one process and kept
/// for the process.
///
/// **A path lookup cannot be keyed on what git echoes back.** For an object
/// named by sha, `--batch` repeats the name it was given; for `<rev>:<path>` it
/// answers with the blob's own object name, which is not what was asked. So the
/// alignment here is positional, and it is safe for the one reason that makes
/// positional alignment ever safe: git emits **exactly one response per input
/// line, in order**, and a path it cannot resolve still emits one, echoing the
/// input followed by `missing`. The count is asserted rather than assumed.
///
/// Keyed on the revision as well as the repository, because the whole point of
/// reading here is that the default branch and the working tree disagree (§7).
pub fn preload_at(cwd: &Path, rev: &str, paths: &[String]) -> Result<()> {
    if paths.is_empty() {
        return Ok(());
    }
    // **Reloaded on every call, never reused across one.** A branch moves: a
    // test commits between two inspections, `done` writes and reads back. The
    // map is only ever right for the tip it was read at, so keeping it would
    // serve a task as finished on the default branch when it is not -- the one
    // direction this must never be wrong in. Refreshing costs one process per
    // inspection, which is the price this whole task exists to pay once instead
    // of per entity.
    let key = (cwd.to_path_buf(), rev.to_string());
    let memo = PRELOADED.get_or_init(|| Mutex::new(HashMap::new()));
    let names: Vec<String> = paths.iter().map(|p| format!("{rev}:{p}")).collect();
    let answers = cat_file_batch_ordered(cwd, &names)?;
    let mut found: HashMap<String, Option<String>> = HashMap::new();
    // A short answer means the framing was misread, and a half-filled map would
    // report entities as absent from the branch that carries them -- which is a
    // task called unfinished, not a slow tool. Nothing is stored in that case
    // and every caller falls back to asking git itself.
    if answers.len() == paths.len() {
        for (path, answer) in paths.iter().zip(answers) {
            found.insert(path.clone(), answer);
        }
    }
    if let Ok(mut seen) = memo.lock() {
        seen.insert(key, found);
    }
    Ok(())
}

type Preloaded = OnceLock<Mutex<HashMap<(PathBuf, String), HashMap<String, Option<String>>>>>;
static PRELOADED: Preloaded = OnceLock::new();

/// What [`preload_at`] holds for this path, or `None` when it was never asked
/// to hold it.
fn preloaded(cwd: &Path, rev: &str, path: &str) -> Option<Option<String>> {
    let memo = PRELOADED.get()?;
    let seen = memo.lock().ok()?;
    seen.get(&(cwd.to_path_buf(), rev.to_string()))?
        .get(path)
        .cloned()
}

/// The same batch as [`cat_file_batch`], answered in the order it was asked.
///
/// Split out rather than folded in, because the two callers key differently and
/// the difference is not cosmetic: a caller naming objects by sha reads the
/// answer back by name, and a caller naming `<rev>:<path>` cannot.
fn cat_file_batch_ordered(cwd: &Path, names: &[String]) -> Result<Vec<Option<String>>> {
    let mut out = Vec::with_capacity(names.len());
    if names.is_empty() {
        return Ok(out);
    }
    let mut input = String::new();
    for name in names {
        input.push_str(name);
        input.push('\n');
    }
    let answered = output_with_stdin(cwd, &["cat-file", "--batch"], input.as_bytes())?;
    if !answered.status.success() {
        return Ok(out);
    }
    let body = answered.stdout;
    let mut at = 0usize;
    while at < body.len() {
        let Some(eol) = body[at..].iter().position(|b| *b == b'\n') else {
            break;
        };
        let header = String::from_utf8_lossy(&body[at..at + eol]).to_string();
        at += eol + 1;
        let mut parts = header.split_whitespace();
        let _name = parts.next();
        let (Some(_kind), Some(size)) = (parts.next(), parts.next()) else {
            // `<input> missing`, which is a response and keeps the alignment.
            out.push(None);
            continue;
        };
        let Ok(size) = size.parse::<usize>() else {
            break;
        };
        if at + size > body.len() {
            break;
        }
        out.push(Some(
            String::from_utf8_lossy(&body[at..at + size]).to_string(),
        ));
        at += size + 1;
    }
    Ok(out)
}

/// Whether `rev` names a commit in this repository, asked once per revision.
///
/// Split out and memoised rather than inlined: the question is about the
/// repository and the answer cannot change while one process runs, so asking it
/// again is a process start bought for nothing.
fn resolves(cwd: &Path, rev: &str) -> Result<bool> {
    static SEEN: OnceLock<Mutex<HashMap<(PathBuf, String), bool>>> = OnceLock::new();
    let memo = SEEN.get_or_init(|| Mutex::new(HashMap::new()));
    let key = (cwd.to_path_buf(), rev.to_string());
    if let Ok(seen) = memo.lock() {
        if let Some(hit) = seen.get(&key) {
            return Ok(*hit);
        }
    }
    let commit = format!("{rev}^{{commit}}");
    let verified = output(cwd, &["rev-parse", "--verify", "--quiet", commit.as_str()])?
        .status
        .success();
    if let Ok(mut seen) = memo.lock() {
        seen.insert(key, verified);
    }
    Ok(verified)
}

/// Where a path went, when the commit that removed it recorded a rename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rename {
    /// The path the file carries now, repository-relative, `/`-separated.
    pub to: String,
    /// The commit that moved it, abbreviated for a reader to paste back.
    pub sha: String,
}

/// Every commit `HEAD` reaches, newest first, with the records
/// `--name-status` gives for each.
///
/// **One walk answers what used to be two plumbing calls per dead scope**
/// (TASK-1b3d7b61dc8f). `rev-list -1 HEAD -- <path>` followed by `diff-tree`
/// answers one path, and a corpus with a hundred dead scopes paid two hundred
/// process starts for it: 25 of the 44 seconds `check` took on this repository,
/// measured on 2026-08-20, because a process start costs 100 to 200 ms here and
/// each `rev-list` walks history until it finds its path, or all of it when git
/// never knew that path. The same records come out of one `git log` in 339 ms,
/// and the count no longer grows with the corpus.
///
/// The records are kept as the **bytes git emitted** rather than as a parsed
/// structure, so every question below is answered by the same pure readers that
/// read `diff-tree` before: [`records`], [`rename_target`], [`moved_prefix`] and
/// [`deletes`] are untouched, and the alignment rule they encode is not
/// reimplemented here.
#[derive(Debug, Default)]
pub struct History {
    /// The sha and the `--name-status -z` records of each commit, newest first.
    commits: Vec<(String, String)>,
}

/// Reads the whole history in one call.
///
/// `--format=%x00%H` frames it: `-z` terminates the format with a NUL of its
/// own, so a commit arrives as an **empty field followed by its sha**, and an
/// empty field is a frame no path can forge. A path is never empty, where a path
/// of forty hexadecimal characters is perfectly possible.
///
/// **No pathspec and no `--full-history`**, which is what keeps this the same
/// question the per-path calls asked: default simplification walks to the commit
/// that made a change rather than to the merges that carried it, and a merge is
/// a commit `--name-status` prints nothing for.
pub fn history(cwd: &Path) -> Result<History> {
    let listed = output(cwd, &["rev-list", "HEAD"])?;
    if !listed.status.success() {
        return Ok(History::default());
    }
    let shas = String::from_utf8_lossy(&listed.stdout).to_string();
    if shas.trim().is_empty() {
        return Ok(History::default());
    }
    let args = ["diff-tree", "--stdin", "-r", "-M", "-z", "--name-status"];
    let out = output_with_stdin(cwd, &args, shas.as_bytes())?;
    // A repository with no HEAD has no history to read, and that is not an error
    // here: the caller reports the dead scope either way and only loses the
    // explanation, which is what `None` means throughout this file.
    if !out.status.success() {
        return Ok(History::default());
    }
    Ok(History::parse(&String::from_utf8_lossy(&out.stdout)))
}

impl History {
    /// Splits the walk into commits, keeping each one's records as git wrote
    /// them.
    ///
    /// The newline is the one wrinkle: git puts one between the format and the
    /// first record, so the first status arrives as `\nA` and would fail the
    /// `R`/`C` test that decides whether a record carries one path or two,
    /// misaligning every record after it. That is exactly how an answer becomes
    /// wrong for a path nobody asked about.
    fn parse(text: &str) -> History {
        let is_sha = |f: &str| f.len() == 40 && f.bytes().all(|b| b.is_ascii_hexdigit());
        let mut commits: Vec<(String, String)> = Vec::new();
        let mut current: Option<(String, Vec<String>)> = None;
        let mut fields = text.split('\0').filter(|f| !f.is_empty());
        while let Some(field) = fields.next() {
            if is_sha(field) {
                if let Some((sha, records)) = current.take() {
                    commits.push((sha, joined(&records)));
                }
                current = Some((field.to_string(), Vec::new()));
                continue;
            }
            // One record, taken whole: a status carries one path, or two when it
            // is `R` or `C`. Consuming it here rather than field by field is what
            // keeps a path out of the commit test above -- a path is never at a
            // record boundary, which is the whole reason that test is safe.
            let pair = field.starts_with('R') || field.starts_with('C');
            let mut record = vec![field.to_string()];
            for _ in 0..(1 + usize::from(pair)) {
                match fields.next() {
                    Some(path) => record.push(path.to_string()),
                    None => break,
                }
            }
            if let Some((_, records)) = current.as_mut() {
                records.extend(record);
            }
        }
        if let Some((sha, records)) = current.take() {
            commits.push((sha, joined(&records)));
        }
        History { commits }
    }

    /// The newest commit whose records touch `path`, and those records.
    ///
    /// The pathspec rule git applies, stated once: `-- <path>` matches the path
    /// itself and everything beneath it, which is what lets one function serve
    /// both a file and the literal prefix of a glob. Both sides of a rename
    /// count, because a commit that moved a file *to* this path changed it just
    /// as much as one that moved it away.
    fn last_change(&self, path: &str) -> Option<(&str, &str)> {
        let under = format!("{path}/");
        let touches = |p: &str| p == path || p.starts_with(under.as_str());
        self.commits
            .iter()
            .find(|(_, diff)| {
                records(diff).any(|(_, src, dst)| touches(src) || dst.is_some_and(touches))
            })
            .map(|(sha, diff)| (sha.as_str(), diff.as_str()))
    }

    /// The rename that killed `path`, or `None` when git cannot name one
    /// (ADR-97beaf55e73a).
    ///
    /// `None` is the honest answer for everything this cannot explain: a
    /// deletion, a move under git's similarity threshold, a path renamed by a
    /// merge, a shallow clone, a repository with no `HEAD`. The caller must
    /// never render any of them as a claim about what happened to the file.
    pub fn rename_of(&self, path: &str) -> Option<Rename> {
        let (sha, diff) = self.last_change(path)?;
        rename_target(diff, path).map(|to| Rename {
            to,
            sha: sha.chars().take(12).collect(),
        })
    }

    /// Where a *directory* went, when the commit that emptied it recorded its
    /// files as renamed into one other directory.
    ///
    /// What serves a glob, which [`History::rename_of`] cannot: git answers
    /// about a path and has none for `src/**`, so the caller asks about the
    /// literal prefix and this reads the answer back as a directory.
    pub fn directory_rename_of(&self, prefix: &str) -> Option<Rename> {
        let (sha, diff) = self.last_change(prefix)?;
        moved_prefix(diff, prefix).map(|to| Rename {
            to,
            sha: sha.chars().take(12).collect(),
        })
    }

    /// The commit that removed `path`, when the last change git records to it is
    /// a deletion.
    ///
    /// Asked after [`History::rename_of`] and never instead of it: a commit that
    /// removes a file and adds a similar one is recorded as a rename, and the
    /// rename is the better answer because it names a place the reader can
    /// follow.
    pub fn deletion_of(&self, path: &str) -> Option<String> {
        let (sha, diff) = self.last_change(path)?;
        deletes(diff, path).then(|| sha.chars().take(12).collect())
    }

    /// The last commit to touch `prefix`, and the paths under it that commit
    /// deleted.
    ///
    /// The paths are returned rather than reduced to a yes, and that is the
    /// whole difference from [`History::deletion_of`]. A prefix is not the
    /// scope: `src/**/*.rs` asks about `src`, and a commit that deleted
    /// `src/notes.md` there says nothing about the scope that died. Only the
    /// caller holds the glob, so only the caller can say whether a deleted path
    /// is one this scope covered.
    pub fn deletions_under(&self, prefix: &str) -> Option<(String, Vec<String>)> {
        let (sha, diff) = self.last_change(prefix)?;
        let under = format!("{prefix}/");
        let deleted: Vec<String> = records(diff)
            .filter(|(status, src, _)| status.starts_with('D') && src.starts_with(under.as_str()))
            .map(|(_, src, _)| src.to_string())
            .collect();
        Some((sha.chars().take(12).collect(), deleted))
    }
}

/// The records of one commit, back in the shape `diff-tree` handed the readers
/// below: NUL-terminated fields, so nothing downstream has to know which of the
/// two commands produced them.
fn joined(records: &[String]) -> String {
    let mut out = String::new();
    for r in records {
        out.push_str(r);
        out.push('\0');
    }
    out
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
    let Some((sha, diff)) = last_change(cwd, path)? else {
        return Ok(None);
    };
    let to = rename_target(&diff, path);
    Ok(to.map(|to| Rename {
        to,
        sha: sha.chars().take(12).collect(),
    }))
}

/// Where a *directory* went, when the commit that emptied it recorded its files
/// as renamed into one other directory.
///
/// This is what serves a glob, which [`rename_of`] cannot: `rev-list` answers
/// about a path and has no answer for `src/**`, so the caller asks about the
/// literal prefix instead and this reads the answer back as a directory.
///
/// The same two plumbing calls, and deliberately the same ones: a second way of
/// asking git the same question is a second thing to keep in step.
pub fn directory_rename_of(cwd: &Path, prefix: &str) -> Result<Option<Rename>> {
    let Some((sha, diff)) = last_change(cwd, prefix)? else {
        return Ok(None);
    };
    let to = moved_prefix(&diff, prefix);
    Ok(to.map(|to| Rename {
        to,
        sha: sha.chars().take(12).collect(),
    }))
}

/// The commit that removed `path`, when the last change git records to it is a
/// deletion.
///
/// The other half of the question [`rename_of`] asks, put to the same two
/// plumbing calls and read out of the same record. A deletion is as plainly
/// recorded as a rename — the reader can see the commit, date it and read its
/// message — so it is an answer git gives, and it is not the silence
/// [`rename_of`] returns for a move under the similarity threshold, a shallow
/// clone or a path git never knew.
///
/// Asked after `rename_of` and never instead of it: a commit that removes a file
/// and adds a similar one is recorded as a rename, and the rename is the better
/// answer. `None` here keeps its meaning — git has nothing to say — which is
/// what the caller must go on reporting as a fault.
pub fn deletion_of(cwd: &Path, path: &str) -> Result<Option<String>> {
    let Some((sha, diff)) = last_change(cwd, path)? else {
        return Ok(None);
    };
    Ok(deletes(&diff, path).then(|| sha.chars().take(12).collect()))
}

/// The last commit to touch `prefix`, and the paths under it that commit
/// deleted.
///
/// What serves a glob, exactly as [`directory_rename_of`] does and for the same
/// reason: `rev-list` answers about a path and has no answer for `src/**`, so
/// the caller asks about the literal prefix instead.
///
/// The paths are returned rather than reduced to a yes, and that is the whole
/// difference from [`deletion_of`]. A prefix is not the scope: `src/**/*.rs` asks
/// about `src`, and a commit that deleted `src/notes.md` there says nothing about
/// the scope that died. Only the caller holds the glob, so only the caller can
/// say whether a deleted path is one this scope covered — and reporting on the
/// prefix alone would be a claim git did not make.
pub fn deletions_under(cwd: &Path, prefix: &str) -> Result<Option<(String, Vec<String>)>> {
    let Some((sha, diff)) = last_change(cwd, prefix)? else {
        return Ok(None);
    };
    let under = format!("{prefix}/");
    let deleted: Vec<String> = records(&diff)
        .filter(|(status, src, _)| status.starts_with('D') && src.starts_with(under.as_str()))
        .map(|(_, src, _)| src.to_string())
        .collect();
    Ok(Some((sha.chars().take(12).collect(), deleted)))
}

/// The last commit that touched `path`, and what it changed.
///
/// `rev-list -1 HEAD -- <path>` is that commit. `diff-tree` on it is what says
/// whether the touch was a rename.
///
/// **No pathspec on `diff-tree`**: it would restrict rename detection to the
/// paths named, and the destination is precisely the path we do not know yet.
/// `-r` reaches into subtrees, `-M` detects the rename at all, `--no-commit-id`
/// makes the first record a change rather than the commit name.
fn last_change(cwd: &Path, path: &str) -> Result<Option<(String, String)>> {
    let args = ["rev-list", "-1", "HEAD", "--", path];
    let out = output(cwd, &args)?;
    if !out.status.success() {
        return Ok(None);
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if sha.is_empty() {
        return Ok(None);
    }

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
    Ok(Some((
        sha,
        String::from_utf8_lossy(&out.stdout).to_string(),
    )))
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
    for (status, src, dst) in records(text) {
        if status.starts_with('R') && src == from {
            return dst.map(str::to_string);
        }
    }
    None
}

/// The records `--name-status -z` emitted, as a status, a source and the
/// destination a rename or a copy carries.
///
/// One walk for every question asked of a commit, because the alignment is the
/// only difficulty here: a status carries one path, or two when it is `R` or
/// `C`, and a reader that miscounts one record misaligns every record after it —
/// which is how the answer becomes wrong for a path nobody asked about. Output
/// cut mid-record ends the walk, and never panics: git truncated by a broken
/// pipe answers nothing, which is what every caller here already means by
/// `None`.
fn records(text: &str) -> impl Iterator<Item = (&str, &str, Option<&str>)> {
    let mut fields = text.split('\0').filter(|f| !f.is_empty());
    std::iter::from_fn(move || {
        let status = fields.next()?;
        let src = fields.next()?;
        let pair = status.starts_with('R') || status.starts_with('C');
        let dst = match pair {
            true => Some(fields.next()?),
            false => None,
        };
        Some((status, src, dst))
    })
}

/// Whether `text` records `path` as deleted.
///
/// Pure, and split out for the same reason as [`rename_target`]: it is a shape
/// of git's output. `D` is the whole test — a deletion carries one path, and a
/// removal git paired with an addition is an `R` this never sees, because
/// [`rename_of`] is asked first and answers it.
fn deletes(text: &str, path: &str) -> bool {
    records(text).any(|(status, src, _)| status.starts_with('D') && src == path)
}

/// The one directory `text` records every file under `prefix` as moving to.
///
/// Pure, and split out for the same reason as [`rename_target`]: every branch is
/// a shape of git's output.
///
/// **Three ways to answer nothing, and all three are the same answer.** No
/// rename under the prefix at all; sources landing in more than one destination;
/// a rename that also changed a file's name, so the path below the prefix is not
/// carried across. The last is what keeps this from reporting a directory move
/// that did not happen — a commit that moves `a/x` to `b/y` moved a file, and
/// saying `a` became `b` on that evidence would be a claim git did not make.
fn moved_prefix(text: &str, prefix: &str) -> Option<String> {
    let under = format!("{prefix}/");
    let mut found: Option<String> = None;
    for (status, src, dst) in records(text) {
        if !status.starts_with('R') {
            continue;
        }
        let Some(rel) = src.strip_prefix(under.as_str()) else {
            continue;
        };
        let dest = dst
            .and_then(|d| d.strip_suffix(rel))
            .and_then(|d| d.strip_suffix('/'))?;
        match &found {
            None => found = Some(dest.to_string()),
            Some(seen) if seen == dest => {}
            Some(_) => return None,
        }
    }
    found
}

/// Memo for [`is_shallow`], keyed on the working directory.
///
/// The depth of a clone cannot change under a running process, so the question
/// is asked once. Keyed like [`RATIFICATIONS`] and for the same reason: `check`
/// reaches this once per unexplained dead scope, and a corpus with eight of them
/// would otherwise spawn eight processes for an answer that cannot differ.
type ShallowMemo = OnceLock<Mutex<HashMap<PathBuf, bool>>>;
static SHALLOW: ShallowMemo = OnceLock::new();

/// Whether this clone was truncated, so that no history is there to walk.
///
/// **Asked directly, and never inferred from an empty `rev-list`.** An empty
/// result also means a path git genuinely has nothing to say about, and reading
/// the two as one is how "the history is not here" would silently become "the
/// path never moved" — the difference this exists to keep.
///
/// A `rev-parse` that fails is not shallow: the caller is already reporting a
/// dead scope, and inventing a second uncertainty out of a broken git would add
/// a state nobody can act on.
pub fn is_shallow(cwd: &Path) -> bool {
    let memo = SHALLOW.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(seen) = memo.lock() {
        if let Some(hit) = seen.get(cwd) {
            return *hit;
        }
    }
    let answer = output(cwd, &["rev-parse", "--is-shallow-repository"])
        .map(|out| String::from_utf8_lossy(&out.stdout).trim() == "true")
        .unwrap_or(false);
    if let Ok(mut seen) = memo.lock() {
        seen.insert(cwd.to_path_buf(), answer);
    }
    answer
}

/// The commit-message keys an anchor is recorded under, one per kind that
/// carries one.
///
/// The walk, the hash and the signature are one mechanism; what differs is
/// which text the anchor covers, and the key is what says so. An ADR anchors
/// its `constraint`; a spec has no narrower field carrying the authority, so it
/// anchors its body (§3), and a commit claiming `constraint+scope` over a kind
/// that declares no constraint would name a field the file does not have.
pub const ANCHOR_CONSTRAINT: &str = "constraint+scope";
pub const ANCHOR_BODY: &str = "body+scope";

/// The anchor hash a ratification commit recorded for `id`, or `None` when no
/// such commit is reachable.
///
/// `ratified` cannot name the commit: a commit cannot contain its own
/// identifier, so no field written by the single commit `accept` makes could
/// ever hold it (§3). The pointer is the history of the entity's own path
/// instead. Walking back from `HEAD`, the first commit whose subject is
/// `ratify <id>` is the ratification, and the anchor line of its message is the
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
    // `paths` is unused now and kept in the signature deliberately: the walk
    // finds a ratification by the subject its own verb wrote, which is a fact
    // about the commit and not about where the entity file sits, so the layout
    // question the argument answers no longer arises. Removing it would move
    // that reasoning out of the caller's sight.
    // **The walk is an accelerator and never the authority.** It is read once
    // and kept for the process, so a commit made after it -- `accept` ratifying
    // and then reading back, a test staging a second ratification -- is not in
    // it. A miss therefore falls back to the search this replaced, which asks
    // git again and answers from the history as it stands now. The fast path
    // carries every ratification that existed when the process started, which is
    // all of them in the case this was written for.
    let mut found = all_ratifications(cwd)?.get(id).cloned();
    if found.is_none() {
        for path in paths {
            found = ratification_uncached(cwd, id, path)?;
            if found.is_some() {
                break;
            }
        }
    }
    if let Ok(mut seen) = memo.lock() {
        seen.insert(key, found.clone());
    }
    Ok(found)
}

/// Every ratification in the history, read in one walk and kept for the process.
///
/// **This replaced a search per entity** (TASK-1b3d7b61dc8f). Each one ran
/// `rev-list --full-history HEAD -- <path>` and then `cat-file commit` on the
/// commits it returned until a subject matched, so a corpus with thirty
/// ratified decisions paid 58 `rev-list` and 113 `cat-file` process starts,
/// measured on this repository with `GIT_TRACE2_EVENT`. One `git log` carrying
/// the subject and the body answers all of them, and the count stops growing
/// with the corpus.
///
/// **The subject is the key, and it is what `accept` writes.** The old search
/// was restricted to the entity's own paths, which was how it found a
/// ratification made before the flat layout; a subject is independent of the
/// path, so the same commit is found without knowing where the file sat.
///
/// `--full-history` here and nowhere else in this file, for the reason the
/// per-entity search gave: the target is one specific commit identified by its
/// subject, and simplification dropping it would lose the anchor.
fn all_ratifications(cwd: &Path) -> Result<HashMap<String, Ratification>> {
    static WALKED: OnceLock<Mutex<HashMap<PathBuf, HashMap<String, Ratification>>>> =
        OnceLock::new();
    let memo = WALKED.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(seen) = memo.lock() {
        if let Some(hit) = seen.get(cwd) {
            return Ok(hit.clone());
        }
    }
    // NUL between the fields and two NULs between commits: a subject cannot
    // contain one, and a body ending in a newline is left exactly as git wrote
    // it. `%x00` rather than a text marker, which a commit message could forge.
    // `rev-list` and not `log`: the walker is plumbing and the pretty format is
    // the same machinery, where `log` is the porcelain ADR-b8884edcebe3 refuses
    // by name. `--format` makes `rev-list` print a `commit <sha>` line of its
    // own before each record, which is what the reader below steps past.
    let args = [
        "rev-list",
        "--full-history",
        "--format=%H%x00%s%x00%b%x00",
        "HEAD",
    ];
    let out = output(cwd, &args)?;
    let mut found: HashMap<String, Ratification> = HashMap::new();
    if out.status.success() {
        let text = String::from_utf8_lossy(&out.stdout);
        // **Three fields per commit, counted, and never a separator between
        // them.** A double NUL to end each record reads perfectly until a
        // commit has no body: `%b` is then empty, the record ends in three NULs,
        // and the split lands one NUL early -- from there every subject is read
        // as a body and every ratification after the first bodyless commit
        // disappears. Measured, and by a test that already existed: an unsigned
        // ratification came back as no ratification at all, which is the one
        // verdict `check` says nothing about.
        //
        // An empty field is a field, so counting is what a separator could not
        // do.
        let fields: Vec<&str> = text.split('\0').collect();
        for record in fields.chunks_exact(3) {
            let sha = record[0];
            // `--format` implies `--pretty`, whose own `commit <sha>` line
            // precedes the format output. `%H` is the last line of this field
            // and is the same object name said twice; reading the header's
            // instead would rest on the two never disagreeing.
            let sha = sha.lines().next_back().unwrap_or_default().trim();
            let Some(id) = record[1].trim().strip_prefix("ratify ") else {
                continue;
            };
            let body = record[2];
            // Newest first, and the newest wins: `rev-list` returned the same
            // order and the search took the first match, so a decision ratified
            // twice answers with the same commit it answered with before.
            if found.contains_key(id) {
                continue;
            }
            // Either key, and read as a key rather than as a prefix: which one a
            // commit carries is a fact about the kind that was ratified, and a
            // reader that knew only one would report every spec unverifiable.
            let anchor = body.lines().find_map(|l| {
                let (key, hash) = l.trim().split_once(": ")?;
                matches!(key, ANCHOR_CONSTRAINT | ANCHOR_BODY).then(|| hash.trim().to_string())
            });
            if let Some(anchor) = anchor {
                found.insert(
                    id.to_string(),
                    Ratification {
                        sha: sha.to_string(),
                        anchor,
                    },
                );
            }
        }
    }
    if let Ok(mut seen) = memo.lock() {
        seen.insert(cwd.to_path_buf(), found.clone());
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
        // Either key, and read as a key rather than as a prefix: which one a
        // commit carries is a fact about the kind that was ratified, and a
        // reader that knew only one would report every spec unverifiable.
        return Ok(message
            .iter()
            .find_map(|l| {
                let (key, hash) = l.trim().split_once(": ")?;
                matches!(key, ANCHOR_CONSTRAINT | ANCHOR_BODY).then(|| hash.trim().to_string())
            })
            .map(|anchor| Ratification {
                sha: sha.trim().to_string(),
                anchor,
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
    // Forward slashes on Windows. A `-c name=value` pair goes through git's
    // config parser, where a backslash opens an escape sequence, so an absolute
    // Windows path arrives as something other than the path that was meant.
    // Git reads forward slashes on every platform it runs on, and the
    // conversion is guarded because a backslash is an ordinary character in a
    // POSIX filename and rewriting one there would break a path that worked.
    let signers = signers_config(allowed_signers);
    let mut args: Vec<&str> = Vec::new();
    if let Some(cfg) = signers.as_deref() {
        args.push("-c");
        args.push(cfg);
    }
    args.extend_from_slice(&["rev-list", "--max-count=1", "--format=%G?%n%GF", sha]);

    // **Every ratification at once, on the first ask** (TASK-1b3d7b61dc8f).
    // `check` puts this question once per ratified entity, and each call starts
    // git, which starts gpg: 43 processes and 5.9 of the 31.5 seconds git spent
    // on this repository. The shas are known before any of them is asked about
    // -- they are the ratifications the walk above already found -- so one call
    // answers all of them, and what remains is gpg's own work rather than
    // process starts.
    if let Some(facts) = batched(cwd, sha, allowed_signers) {
        return Ok(facts);
    }

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

/// The signature of every ratification, read in one call and kept for the
/// process.
///
/// `None` when the batch cannot answer for this sha, and the caller then asks
/// about it alone: a commit that is not a ratification is a question this map
/// was never built to answer, and inventing a verdict for it is the one thing a
/// signature check may never do.
///
/// The commits are read back from the `commit <sha>` header rather than from the
/// order they were given, because `--no-walk` sorts by commit date unless told
/// otherwise, and an answer aligned by position would be aligned wrongly.
fn batched(cwd: &Path, sha: &str, allowed_signers: Option<&Path>) -> Option<SignatureFacts> {
    type Facts = HashMap<(PathBuf, String), HashMap<String, SignatureFacts>>;
    static SEEN: OnceLock<Mutex<Facts>> = OnceLock::new();
    let memo = SEEN.get_or_init(|| Mutex::new(HashMap::new()));
    let signers = allowed_signers
        .map(|p| p.display().to_string())
        .unwrap_or_default();
    let key = (cwd.to_path_buf(), signers);
    if let Ok(seen) = memo.lock() {
        if let Some(hit) = seen.get(&key) {
            return hit.get(sha).cloned();
        }
    }
    let shas: Vec<String> = all_ratifications(cwd)
        .ok()?
        .values()
        .map(|r| r.sha.clone())
        .collect();
    let mut facts: HashMap<String, SignatureFacts> = HashMap::new();
    if !shas.is_empty() {
        let config = signers_config(allowed_signers);
        let mut args: Vec<&str> = Vec::new();
        if let Some(cfg) = config.as_deref() {
            args.push("-c");
            args.push(cfg);
        }
        args.extend_from_slice(&["rev-list", "--no-walk", "--format=%G?%n%GF"]);
        for sha in &shas {
            args.push(sha.as_str());
        }
        if let Ok(out) = output(cwd, &args) {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);
                let mut lines = text.lines();
                while let Some(header) = lines.next() {
                    let Some(named) = header.strip_prefix("commit ") else {
                        continue;
                    };
                    let status = lines
                        .next()
                        .and_then(|l| l.trim().chars().next())
                        .unwrap_or('N');
                    let fingerprint = lines.next().unwrap_or_default().trim().to_string();
                    facts.insert(
                        named.trim().to_string(),
                        SignatureFacts {
                            status,
                            fingerprint,
                        },
                    );
                }
            }
        }
    }
    let answer = facts.get(sha).cloned();
    if let Ok(mut seen) = memo.lock() {
        seen.insert(key, facts);
    }
    answer
}

/// The `-c` pair that points git at an allowed-signers file, or `None` when
/// there is none to point at.
///
/// One rule for the two callers, and the Windows clause is why it is worth a
/// function rather than a repeated block: a `-c name=value` pair goes through
/// git's config parser, where a backslash opens an escape sequence, so an
/// absolute Windows path arrives as something other than the path that was
/// meant. Git reads forward slashes on every platform it runs on, and the
/// conversion is guarded because a backslash is an ordinary character in a POSIX
/// filename and rewriting one there would break a path that worked.
fn signers_config(allowed_signers: Option<&Path>) -> Option<String> {
    allowed_signers.map(|p| {
        let path = p.display().to_string();
        let path = match cfg!(windows) {
            true => path.replace('\\', "/"),
            false => path,
        };
        format!("gpg.ssh.allowedSignersFile={path}")
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
        ExitCode::Environment,
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
        assert_eq!(
            err.code,
            ExitCode::Environment,
            "an unknown revision is an environment error"
        );
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
            err.code,
            ExitCode::Environment,
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
        assert_eq!(err.code, ExitCode::Environment);
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
        assert_eq!(err.code, ExitCode::Environment);
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
            assert_eq!(err.code, ExitCode::Environment);
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
        assert_eq!(deletion_of(&t.0, "src/old.rs").unwrap(), None);
        assert_eq!(deletions_under(&t.0, "src").unwrap(), None);
    }

    /// The record a deletion is read out of, and the two it must not be read out
    /// of: a modification and the source of a rename both name a path that is
    /// still somewhere.
    #[test]
    fn a_deletion_is_read_by_its_path_and_a_path_still_somewhere_is_not_one() {
        let text = "M\0a.rs\0D\0gone.rs\0R100\0old.rs\0new.rs\0D\0also.rs\0";
        assert!(deletes(text, "gone.rs"));
        assert!(
            deletes(text, "also.rs"),
            "a deletion past a rename must be found, so the two-path record is \
             stepped over as two"
        );
        assert!(!deletes(text, "a.rs"), "a modified file was not deleted");
        assert!(
            !deletes(text, "old.rs"),
            "the source of a rename is somewhere, and calling it deleted would \
             lose the place the reader can follow"
        );
        assert!(!deletes("", "gone.rs"));
        assert!(
            !deletes("D\0", "gone.rs"),
            "output cut mid-record answers nothing, and must not panic"
        );
    }

    /// The commit that removed a path, on a real deletion and on the two
    /// silences: a path git never knew, and a path that moved.
    #[test]
    fn a_deleted_path_names_the_commit_that_removed_it_and_a_moved_one_does_not() {
        let t = Temp::new_repo();
        t.commit("src/gone.rs", "fn gone() {}\n");
        t.commit("src/kept.rs", "fn kept() {}\n");
        std::fs::remove_file(t.0.join("src/gone.rs")).unwrap();
        t.porcelain(&["add", "-A"]);
        t.porcelain(&["commit", "-qm", "delete it"]);
        let sha = run(&t.0, &["rev-parse", "HEAD"]).unwrap();

        let removed = deletion_of(&t.0, "src/gone.rs")
            .unwrap()
            .expect("git records the deletion");
        assert!(
            sha.starts_with(&removed),
            "the commit named must be the one that removed it: {removed} is not \
             a prefix of {sha}"
        );
        assert_eq!(
            deletion_of(&t.0, "src/kept.rs").unwrap(),
            None,
            "a file that is still there was not deleted"
        );
        assert_eq!(
            deletion_of(&t.0, "src/never.rs").unwrap(),
            None,
            "a path git never knew is the silence the caller reports as a fault"
        );

        // A rename is recorded as one record and never as a deletion, so the
        // caller that asks for the rename first is never contradicted here.
        std::fs::rename(t.0.join("src/kept.rs"), t.0.join("src/moved.rs")).unwrap();
        t.porcelain(&["add", "-A"]);
        t.porcelain(&["commit", "-qm", "move it"]);
        assert_eq!(deletion_of(&t.0, "src/kept.rs").unwrap(), None);
    }

    /// The directory question, which serves a glob: the paths a commit deleted
    /// under a prefix, for the caller to confront with the glob it holds.
    #[test]
    fn the_paths_a_commit_deleted_under_a_prefix_are_returned_for_the_caller_to_filter() {
        let t = Temp::new_repo();
        t.commit("tools/hook.sh", "#!/bin/sh\n");
        t.commit("tools/notes.md", "# notes\n");
        std::fs::remove_dir_all(t.0.join("tools")).unwrap();
        t.porcelain(&["add", "-A"]);
        t.porcelain(&["commit", "-qm", "remove the directory"]);
        let sha = run(&t.0, &["rev-parse", "HEAD"]).unwrap();

        let (named, deleted) = deletions_under(&t.0, "tools")
            .unwrap()
            .expect("git records the commit that touched the prefix");
        assert!(sha.starts_with(&named), "{named} is not a prefix of {sha}");
        assert_eq!(
            deleted,
            vec!["tools/hook.sh".to_string(), "tools/notes.md".to_string()],
            "every path deleted under the prefix is returned, because only the \
             caller knows which of them the glob matched"
        );

        // A prefix git never knew, and one whose last commit deleted nothing:
        // the second is the case that would let a claim be made about a
        // directory that is still there.
        assert_eq!(deletions_under(&t.0, "absent").unwrap(), None);
        t.commit("src/lib.rs", "// x\n");
        let (_, none) = deletions_under(&t.0, "src").unwrap().expect("a commit");
        assert!(
            none.is_empty(),
            "a commit that added a file under the prefix deleted nothing: {none:?}"
        );
    }
}
