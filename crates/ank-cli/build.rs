//! Embeds the commit the binary was built from (§4).
//!
//! `rev-parse` and nothing else: it is on the list of commands ADR-b8884edcebe3
//! allows, its output is a sha and stable by contract, and no porcelain is
//! parsed here any more than in the binary itself.
//!
//! No `rerun-if-changed` is emitted, and that is the deliberate choice rather
//! than an omission. Naming a file would pin the rebuild to it: `.git/HEAD`
//! misses a `packed-refs` update, a linked worktree keeps its HEAD elsewhere,
//! and pinning to `build.rs` alone would freeze the commit forever — the exact
//! staleness this whole flag exists to expose. With nothing named, Cargo falls
//! back to watching the whole package, so any source change refreshes the
//! stamp. What that still cannot catch is a commit whose tree is identical to
//! the last build's, an amend or a checkout across equal trees; the honest
//! answer there is that no build script sees it, and a release is built from a
//! clean checkout anyway.

use std::process::Command;

fn main() {
    println!("cargo:rustc-env=ANK_COMMIT={}", commit());
}

/// The short sha, or `unknown` when there is no checkout to ask — a source
/// tarball, a vendored crate, a build in a container with no `.git`. Naming the
/// absence beats inventing a value: a version string that quietly claims a
/// commit it does not know is worse than one that admits it.
fn commit() -> String {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let out = Command::new("git")
        .current_dir(dir)
        .args(["rev-parse", "--short", "HEAD"])
        .output();
    match out {
        Ok(o) if o.status.success() => {
            let sha = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if sha.is_empty() {
                "unknown".into()
            } else {
                sha
            }
        }
        _ => "unknown".into(),
    }
}
