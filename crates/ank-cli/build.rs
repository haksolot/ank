//! Embeds the commit the binary was built from, and the revision of the
//! `skill/SKILL.md` it was built alongside (§4).
//!
//! `rev-parse` and `symbolic-ref`, and nothing else: both are on the list of
//! commands ADR-9307e5d214a7 allows, their output is a sha or a path and stable
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
//!
//! **Every skill under `skill/` is embedded too** (ADR-e1d750884b82,
//! TASK-544ec9655570): the contract and each sibling, frontmatter and body, read
//! whole from the one source ADR-8b3045cf11db names and handed to the crate as
//! bytes, so `ank skills --install` writes what the tree holds and nothing is
//! committed a second time for it. The whole directory is watched, because a
//! sibling added beside the others is a skill the next build has to carry.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The skill whose revision is stamped in, relative to this package.
const SKILL: &str = "../../skill/SKILL.md";

/// The directory every embedded skill is read from, relative to this package.
const SKILLS: &str = "../../skill";

/// Where `SCHEMA_VERSION` is declared, as git addresses it: repository-relative,
/// which is what `cat-file` takes whatever directory the build runs in.
const MODEL: &str = "crates/ank-core/src/model.rs";

fn main() {
    let dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    println!("cargo:rustc-env=ANK_COMMIT={}", commit(&dir));
    println!("cargo:rustc-env=ANK_SKILL={}", skill_revision(&dir));
    println!(
        "cargo:rustc-env=ANK_RELEASED_SCHEMA={}",
        released_schema(&dir)
    );
    let out = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    std::fs::write(out.join("skills.rs"), embedded_skills(&dir))
        .unwrap_or_else(|e| panic!("{}: {e}", out.join("skills.rs").display()));
    if PathBuf::from(&dir).join(SKILL).is_file() {
        println!("cargo:rerun-if-changed={SKILL}");
    }
    if PathBuf::from(&dir).join(SKILLS).is_dir() {
        println!("cargo:rerun-if-changed={SKILLS}");
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

/// The Rust source `src/skills.rs` includes: one `Embedded` per skill under
/// `skill/`, the contract first and the siblings after it in name order.
///
/// **What is read from the frontmatter is what the file declares**, `name`,
/// `description` and `metadata.revision`, and nothing is recomputed: whether the
/// declared revision is the hash of the body is `tests/skill.rs`'s question, and
/// answering it here too would be a second place for the answer to live. The
/// content itself is `include_bytes!` of the file in the tree, so the bytes the
/// binary carries are the bytes git checked out, line endings included.
///
/// **No `skill/` to read is an empty list, never a failure**, for the reason the
/// revision is `unknown` there: a published crate or a vendored dependency has
/// no tree, and `ank skills` says it carries none. A file that is present and
/// unreadable as a skill fails loudly, as `skill_revision` does.
fn embedded_skills(dir: &str) -> String {
    let root = PathBuf::from(dir).join(SKILLS);
    let mut files: Vec<PathBuf> = Vec::new();
    if root.join("SKILL.md").is_file() {
        files.push(root.join("SKILL.md"));
    }
    if let Ok(entries) = std::fs::read_dir(&root) {
        for entry in entries {
            let path = entry
                .unwrap_or_else(|e| panic!("{}: {e}", root.display()))
                .path();
            if path.is_dir() && path.join("SKILL.md").is_file() {
                files.push(path.join("SKILL.md"));
            }
        }
    }
    let mut skills: Vec<(String, String, String, PathBuf)> = files
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
                .replace("\r\n", "\n");
            let front = text
                .strip_prefix("---\n")
                .and_then(|rest| rest.find("\n---\n").map(|end| rest[..end].to_string()))
                .unwrap_or_else(|| panic!("{}: must open with closed frontmatter", path.display()));
            let field = |key: &str| {
                front
                    .lines()
                    .find_map(|line| line.trim().strip_prefix(key))
                    .map(|v| v.trim().trim_matches('"').to_string())
                    .filter(|v| !v.is_empty())
                    .unwrap_or_else(|| panic!("{}: declares no {key}", path.display()))
            };
            // Absolute already: `CARGO_MANIFEST_DIR` is, and `include_bytes!`
            // resolves a relative path against the generated file instead.
            // Not canonicalised, which on Windows would hand the macro a
            // verbatim `\\?\` path.
            (
                field("name:"),
                field("description:"),
                field("revision:"),
                path,
            )
        })
        .collect();
    skills.sort();
    let mut source = String::from("pub const EMBEDDED: &[Embedded] = &[\n");
    for (name, description, revision, path) in skills {
        source.push_str(&format!(
            "    Embedded {{ name: {name:?}, description: {description:?}, revision: {revision:?}, content: include_bytes!({:?}) }},\n",
            path.display().to_string()
        ));
    }
    source.push_str("];\n");
    source
}

/// The entity schema the newest release reads, or empty where there is nothing
/// to ask (TASK-7a2c9d1b13a0).
///
/// **Derived and never typed**, which is the whole of why it is here rather
/// than a constant in the source. A number kept by hand is true only while
/// somebody remembers to bump it at the tag, and the message it feeds would
/// then be confident and wrong — which is the defect this value exists to
/// repair, reproduced one level down.
///
/// The newest tag by version order, and `SCHEMA_VERSION` as that tag's tree
/// declares it. Both reads are plumbing: `for-each-ref` takes the format it
/// prints, and `cat-file blob` hands back a file.
///
/// **Empty degrades to the safe road.** A tarball, a clone fetched without
/// tags, a spelling of the constant this parser does not know: each answers
/// empty, and the reader is then told to build from the tree rather than told
/// something unverified about a release. Silence about a release is a worse
/// answer than none only when it sends somebody in a circle, and this one does
/// not.
fn released_schema(dir: &str) -> String {
    let Some(tag) = git(
        dir,
        &[
            "for-each-ref",
            "--sort=-v:refname",
            "--count=1",
            "--format=%(refname:short)",
            "refs/tags/v*",
        ],
    ) else {
        return String::new();
    };
    let Some(source) = git(dir, &["cat-file", "blob", &format!("{tag}:{MODEL}")]) else {
        return String::new();
    };
    source
        .lines()
        .find_map(|line| line.trim().strip_prefix("pub const SCHEMA_VERSION: u32 = "))
        .and_then(|value| value.trim().trim_end_matches(';').parse::<u32>().ok())
        .map(|schema| schema.to_string())
        .unwrap_or_default()
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
