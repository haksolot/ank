//! The declaration: which corpora this reader wants kept warm
//! (ADR-a22cd3196529).
//!
//! **Declared, never discovered.** ADR-621a7fd96ce1 says it and
//! ADR-96174f1ac2b7 repeats it: nothing walks a filesystem looking for a
//! corpus. A daemon that scanned a home directory for `.ank/` would be the
//! single most natural implementation of this crate and the one the corpus
//! refused in writing before it was proposed, so there is no scan here, no
//! fallback to one, and no "if the declared path is wrong, try nearby". A
//! declaration that names a directory carrying no corpus is a refusal.
//!
//! **Keyed on the repository identity, never on a path.** Two worktrees of one
//! repository have two paths and one identity; two clones of two repositories
//! can sit at one path on two machines. So the key is the root commit of
//! ADR-621a7fd96ce1 and the value is where to look -- one place, or several
//! when the reader keeps several checkouts. Several paths under one key are one
//! watched corpus, which is the whole reason the key is not the path.
//!
//! **Held outside every repository**, in the reader's own configuration
//! directory, on the rule ADR-96174f1ac2b7 fixed for `corpora.yml`: a pointer
//! committed in a code repository is exactly the trace that decision exists to
//! refuse, and a watcher's list of what somebody is working on is a worse trace
//! than a corpus location.

use crate::fail::{Fail, Result};
use ank_contract::ExitCode;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// The reader's watch list, under [`user_dir`].
pub const WATCH_FILE: &str = "watch.yml";

/// The only schema this build reads. A file declaring anything else is refused
/// by number rather than read optimistically, for the reason `corpora.yml` is:
/// a watcher that guessed at a format it does not know would be watching
/// something nobody declared.
const SUPPORTED_SCHEMA: u32 = 1;

/// The corpus directory under a declared tree.
///
/// Spelled here rather than imported, because `ank-cli` has no library target:
/// see the note in [`crate::warm`] about why this crate reaches the corpus
/// through the binary and never through the code that reads it.
pub const ANK_DIR: &str = ".ank";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WatchFileDoc {
    schema: u32,
    #[serde(default)]
    watch: BTreeMap<String, Trees>,
}

/// One path, or several. A reader with one checkout writes a string; a reader
/// with worktrees writes a list, and gets one watched corpus either way.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Trees {
    One(String),
    Many(Vec<String>),
}

impl Trees {
    fn paths(&self) -> Vec<&str> {
        match self {
            Trees::One(p) => vec![p.as_str()],
            Trees::Many(ps) => ps.iter().map(String::as_str).collect(),
        }
    }
}

/// One corpus this daemon watches: an identity, and every checkout of it the
/// reader declared.
///
/// **One entry per identity and never per path.** Two worktrees of one
/// repository are two `roots` here, not two corpora -- each carries its own
/// `.ank/index.db`, because each is its own checkout of the same committed
/// files, and both are caches of the one corpus this names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Watched {
    pub identity: String,
    pub roots: Vec<PathBuf>,
}

/// Where this reader's declarations live, or `None` where the environment names
/// no home.
///
/// **The same rule `ank config --user` applies to `corpora.yml`**, deliberately
/// and not by coincidence: a reader who declared a corpus in one file and a
/// watch in another, under two different directories, would have two homes and
/// no way to know it. `%APPDATA%\ank` on Windows; `$XDG_CONFIG_HOME/ank`
/// elsewhere, falling back to `$HOME/.config/ank`. An empty value counts as
/// unset, because a shell that exports a variable to nothing has said nothing
/// and joining onto it would name a relative path under the current directory.
///
/// The rule is three lines of platform difference and the environment, so it is
/// written here rather than depended on: `ank-cli` is a binary with no library
/// target, and `the_watch_file_sits_beside_the_corpora_file` in this crate's
/// suite drives both binaries to assert the two agree.
pub fn user_dir() -> Option<PathBuf> {
    let var = |key: &str| {
        std::env::var_os(key)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    if cfg!(windows) {
        return var("APPDATA").map(|p| p.join("ank"));
    }
    if let Some(xdg) = var("XDG_CONFIG_HOME") {
        return Some(xdg.join("ank"));
    }
    var("HOME").map(|p| p.join(".config").join("ank"))
}

/// The declaration file, wherever this reader's home is.
pub fn watch_path() -> Result<PathBuf> {
    user_dir().map(|d| d.join(WATCH_FILE)).ok_or_else(|| {
        Fail::new(
            ExitCode::Environment,
            "no home directory in the environment, so there is nowhere to declare a corpus",
        )
        .with_hint(if cfg!(windows) {
            "set APPDATA"
        } else {
            "set XDG_CONFIG_HOME or HOME"
        })
    })
}

/// A repository identity as ADR-621a7fd96ce1 defines it: the root commit, and
/// therefore forty lowercase hex characters.
fn is_identity(key: &str) -> bool {
    key.len() == 40
        && key
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// What the reader declared, resolved against the filesystem and against git.
///
/// Every refusal below is a refusal to *start*. A watcher that started on a
/// declaration it could not honour would be warming nothing while reporting
/// that it watches something, and the reader would find out when a listing was
/// slow rather than when the file was wrong.
pub fn resolve(
    text: &str,
    path: &Path,
    identity_of: &dyn Fn(&Path) -> Option<String>,
) -> Result<Vec<Watched>> {
    let doc: WatchFileDoc = serde_yaml::from_str(text).map_err(|e| {
        Fail::new(ExitCode::Environment, format!("{}: {e}", path.display())).with_hint(
            "schema: 1 and a watch: map of repository identity to the path of a checkout, \
             or to a list of them",
        )
    })?;
    if doc.schema != SUPPORTED_SCHEMA {
        return Err(Fail::new(
            ExitCode::Environment,
            format!(
                "{}: schema {} is not {SUPPORTED_SCHEMA}",
                path.display(),
                doc.schema
            ),
        ));
    }
    if doc.watch.is_empty() {
        return Err(Fail::new(
            ExitCode::Environment,
            format!(
                "{}: nothing is declared, so there is nothing to watch",
                path.display()
            ),
        )
        .with_hint(
            "a key is the root commit, which ank status --json prints under \"corpus\", \
             and its value is the path of a checkout",
        ));
    }

    let mut watched = Vec::new();
    for (key, trees) in &doc.watch {
        // **A key that is not an identity is refused by name.** The mistake
        // this catches is the one ADR-96174f1ac2b7 names: a slug, a remote URL
        // or a path typed where the root commit belongs. Each of those keys
        // differently for one clone over ssh and one over https, for a fork and
        // its upstream, and for a repository renamed on the forge -- and a
        // wrong key that merely matched nothing would leave the corpus quietly
        // unwatched, which is the one outcome worse than a refusal.
        if !is_identity(key) {
            return Err(Fail::new(
                ExitCode::Environment,
                format!("{}: '{key}' is not a repository identity", path.display()),
            )
            .with_hint(
                "a key is the root commit, never a path, a remote or a slug: \
                 ank status --json prints it under \"corpus\"",
            ));
        }
        let declared = trees.paths();
        if declared.is_empty() {
            return Err(Fail::new(
                ExitCode::Environment,
                format!("{}: {key} declares no path", path.display()),
            )
            .with_hint("give it the path of a checkout, or remove the key"));
        }
        let mut roots: Vec<PathBuf> = Vec::new();
        for raw in declared {
            let root = PathBuf::from(raw);
            // **No search, in either direction.** The walk up that `ank`
            // performs from a working directory is a resolution the caller
            // asked for; here the reader named a directory, and a directory
            // that carries no corpus is a declaration to correct rather than a
            // starting point for a hunt.
            if !root.join(ANK_DIR).is_dir() {
                return Err(Fail::new(
                    ExitCode::Environment,
                    format!("{raw} carries no {ANK_DIR}/, and nothing looks for one elsewhere"),
                )
                .with_hint(format!(
                    "name the directory that contains {ANK_DIR}/, or ank init {raw}"
                )));
            }
            // **The key is checked against the repository, not trusted.** This
            // is what makes "keyed on the identity" a property rather than a
            // label: without it, two checkouts of two different repositories
            // filed under one key would be reported as one corpus, which is
            // precisely the confusion a path key produces and this key exists
            // to remove.
            match identity_of(&root) {
                Some(found) if found == *key => {}
                Some(found) => {
                    return Err(Fail::new(
                        ExitCode::Environment,
                        format!("{raw} is repository {found}, declared under {key}"),
                    )
                    .with_hint("file a checkout under its own root commit, or correct the key"));
                }
                None => {
                    return Err(Fail::new(
                        ExitCode::Environment,
                        format!("{raw} has no root commit, so it has no identity to be keyed on"),
                    )
                    .with_hint(
                        "a repository with no history cannot be keyed: commit, or drop the entry",
                    ));
                }
            }
            if !roots
                .iter()
                .any(|r: &PathBuf| same_dir(r.as_path(), root.as_path()))
            {
                roots.push(root);
            }
        }
        watched.push(Watched {
            identity: key.clone(),
            roots,
        });
    }
    Ok(watched)
}

/// Whether two paths name one directory, asked of the filesystem rather than of
/// the strings.
///
/// One checkout declared twice -- once relative, once absolute, or once through
/// a symlink -- is one root, and no textual comparison delivers that. A path
/// that cannot be canonicalised does not exist, and by the time this is asked
/// both have been shown to carry a `.ank/`.
fn same_dir(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => a == b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none(_: &Path) -> Option<String> {
        None
    }

    #[test]
    fn a_key_that_is_not_a_root_commit_is_refused_by_name() {
        let err = resolve(
            "schema: 1\nwatch:\n  git@github.com:me/ank.git: /tmp\n",
            Path::new("watch.yml"),
            &none,
        )
        .unwrap_err();
        assert_eq!(err.code, ExitCode::Environment);
        assert!(
            err.message.contains("is not a repository identity"),
            "{err:?}"
        );
    }

    #[test]
    fn an_unknown_schema_is_refused_by_number() {
        let err = resolve("schema: 7\nwatch: {}\n", Path::new("watch.yml"), &none).unwrap_err();
        assert!(err.message.contains("schema 7 is not 1"), "{err:?}");
    }

    #[test]
    fn an_empty_declaration_is_refused_rather_than_watched() {
        let err = resolve("schema: 1\nwatch: {}\n", Path::new("watch.yml"), &none).unwrap_err();
        assert!(err.message.contains("nothing is declared"), "{err:?}");
    }

    #[test]
    fn an_unknown_field_is_refused() {
        let err = resolve("schema: 1\nscan: true\n", Path::new("watch.yml"), &none).unwrap_err();
        assert_eq!(err.code, ExitCode::Environment);
    }
}
