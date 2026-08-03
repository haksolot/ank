//! Embeds the commit the binary was built from (§4).
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

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    println!("cargo:rustc-env=ANK_COMMIT={}", commit(&dir));
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
