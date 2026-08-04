//! Embeds the commit the binary was built from, and the revision of the
//! `skill/SKILL.md` it was built alongside (§4).
//!
//! `rev-parse` and `symbolic-ref`, and nothing else: both are on the list of
//! commands ADR-b8884edcebe3 allows, their output is a sha or a path and stable
//! by contract, and no porcelain is parsed here any more than in the binary
//! itself.
//!
//! **What the script watches is the point** (TASK-0b26c8b5bfc5). It first
//! emitted no `rerun-if-changed` at all, on the argument that naming
//! `.git/HEAD` misses a `packed-refs` update and that a linked worktree keeps
//! its HEAD elsewhere. Both objections were true and the conclusion did not
//! follow: with nothing named, Cargo watches the package instead, and a commit
//! that changes no file in it never reruns the script — so the stamp lagged,
//! which is the exact staleness this flag exists to expose. Measured on the
//! build that shipped it: `HEAD 3110392`, stamp `aeb1841`.
//!
//! Asking git for the paths answers both objections at once. `rev-parse
//! --git-path` resolves `HEAD` to the *current* worktree's HEAD, linked or not,
//! and names `packed-refs` wherever it lives; `symbolic-ref` gives the branch
//! file to watch alongside it, and fails on a detached HEAD, where the HEAD file
//! carries the sha itself and is already watched. Losing the package-wide watch
//! costs nothing: a source edit with no commit leaves the stamp correct, because
//! the commit has not moved.
//!
//! With no git and no checkout, nothing is emitted and Cargo falls back to
//! watching the package — the right behaviour for a tarball, where the answer is
//! `unknown` however often it is recomputed.
//!
//! The skill revision is watched the same way and for the same reason: the file
//! it is derived from is outside this package, so nothing Cargo watches by
//! default would notice it moving (TASK-ecda4070354f).

use std::path::{Path, PathBuf};
use std::process::Command;

/// The skill whose revision is stamped in, relative to this package.
const SKILL: &str = "../../skill/SKILL.md";

fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    println!("cargo:rustc-env=ANK_COMMIT={}", commit(&dir));
    println!("cargo:rustc-env=ANK_SKILL={}", skill_revision(&dir));
    if PathBuf::from(&dir).join(SKILL).is_file() {
        println!("cargo:rerun-if-changed={SKILL}");
    }
    for path in watched(&dir) {
        println!("cargo:rerun-if-changed={path}");
    }
}

fn git(dir: &str, args: &[&str]) -> Option<String> {
    let out = Command::new("git")
        .current_dir(dir)
        .args(args)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
    (!text.is_empty()).then_some(text)
}

/// The short sha, or `unknown` when there is no checkout to ask — a source
/// tarball, a vendored crate, a build in a container with no `.git`. Naming the
/// absence beats inventing a value: a version string that quietly claims a
/// commit it does not know is worse than one that admits it.
fn commit(dir: &str) -> String {
    git(dir, &["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".into())
}

/// The revision of the skill this binary was built alongside: the short freeze
/// hash of `SKILL.md`'s body, which is the value the file declares under
/// `metadata.revision` (§4).
///
/// Derived here rather than typed anywhere, which is the whole point: a number
/// kept by hand drifts the first time somebody edits the body and forgets it,
/// and a stale marker is worse than none because it answers confidently.
///
/// The body, not the whole file — the frontmatter carries the revision itself,
/// and hashing it would make the value an input to its own computation. The
/// split is the entity format's own delimiters, on text with line endings
/// unified first: `.gitattributes` covers `.ank/**` and not `skill/`, so a
/// Windows checkout of this file can legitimately be CRLF.
///
/// `unknown` when there is no `skill/` to read — a published crate, a vendored
/// dependency — for the same reason the commit is. A file that is present and
/// unreadable as a skill is a different case and fails loudly: it is a defect in
/// the tree, not a build without a checkout.
fn skill_revision(dir: &str) -> String {
    let path = PathBuf::from(dir).join(SKILL);
    if !path.is_file() {
        return "unknown".into();
    }
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
        .replace("\r\n", "\n");
    let rest = text
        .strip_prefix("---\n")
        .unwrap_or_else(|| panic!("{}: must open with frontmatter", path.display()));
    let end = rest
        .find("\n---\n")
        .unwrap_or_else(|| panic!("{}: frontmatter must be closed", path.display()));
    ank_core::freeze_hash_short(&rest[end + "\n---\n".len()..])
}

/// The files whose change can change the answer, and no others.
///
/// Only paths that exist are emitted. A `rerun-if-changed` on a missing file
/// reruns the script on every build, which would recompile the crate every time
/// for nothing; and emitting none at all is the deliberate fallback for a tree
/// with no git.
fn watched(dir: &str) -> Vec<String> {
    let mut refs = vec!["HEAD".to_string(), "packed-refs".to_string()];
    // The branch file, when there is a branch. On a detached HEAD this fails and
    // there is nothing to add: the sha lives in HEAD, already listed above.
    if let Some(head_ref) = git(dir, &["symbolic-ref", "-q", "HEAD"]) {
        refs.push(head_ref);
    }
    refs.iter()
        .filter_map(|r| git(dir, &["rev-parse", "--git-path", r]))
        // `--git-path` answers relative to the working directory, which is the
        // package root here, and Cargo reads a relative `rerun-if-changed` the
        // same way. A linked worktree answers absolute, which both accept.
        .filter(|p| PathBuf::from(dir).join(Path::new(p)).is_file())
        .collect()
}
