//! The declaration: which corpora this reader wants kept warm
//! (ADR-24e21cb83793).
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
use std::time::Duration;

/// The reader's watch list, under [`user_dir`].
pub const WATCH_FILE: &str = "watch.yml";

/// How often `refs/ank/*` is mirrored when the declaration states nothing.
///
/// A minute, and deliberately three orders of magnitude above the warm poll.
/// The poll is a stat of a local directory and costs nothing; this is a network
/// round trip against somebody's forge, once per watched checkout, forever. A
/// claim is a thirty-minute lease (§3), so a minute is already an order of
/// magnitude finer than the thing it reports on, and anybody who wants it
/// finer, or who is watching a remote that charges for it, states a number.
pub const DEFAULT_FETCH: Duration = Duration::from_secs(60);

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

/// `schema:` alone, read before anything else in the file is looked at.
///
/// **The one shape in this file that tolerates an unknown key**, which is the
/// whole reason it exists. `WatchFileDoc` denies them, so a file one version
/// newer than this build was reported as *unknown field `mirrors`* -- the
/// reader was told a key is wrong when the truth is that the tool is old, and
/// went hunting for a typo in a file that had none. `docs/format.md` already
/// legislates it: "Newer is refused, and refused **on the version rather than
/// on the first field it does not recognise**." The version has to be readable
/// without reading the rest, or the rest refuses first and the version never
/// gets asked.
///
/// Spelled here rather than shared with `ank-cli`'s copy for the reason the
/// rest of this module is: that crate has no library target. The shape is the
/// same one `parse()` and `newer_than_this_ank()` use there, deliberately, so
/// `watch.yml` and `corpora.yml` answer a reader in one sentence.
#[derive(Debug, Deserialize)]
struct SchemaProbe {
    schema: u32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct WatchFileDoc {
    schema: u32,
    /// Seconds between two mirrors of `refs/ank/*`, stated by the reader who
    /// pays for them.
    ///
    /// **Optional, and still schema 1.** A field that may be omitted is
    /// readable by a build that predates it, so the number does not move; a
    /// file that *states* it is refused by an older daemon, which is the
    /// honest outcome of asking for something that build cannot do.
    #[serde(default)]
    fetch: Option<u64>,
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

/// A declaration resolved: what to watch, and how often to pay the network for
/// news about it.
///
/// The interval travels with the list rather than beside it, because it is
/// stated in the same file by the same reader and a second source for it would
/// be a second answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    pub fetch: Duration,
    pub watch: Vec<Watched>,
}

/// Where this reader's declarations live, or `None` where the environment names
/// no home.
///
/// **The same rule `ank config --user` applies to `corpora.yml`**, deliberately
/// and not by coincidence: a reader who declared a corpus in one file and a
/// watch in another, under two different directories, would have two homes and
/// no way to know it.
///
/// It used to be written out here, because `ank-cli` is a binary with no
/// library target and there was nowhere shared to put it. There is now: the
/// change stream of TASK-2f7777a1fdff lands in this same directory and is
/// followed out of it by `ank-tui`, so the rule moved to
/// [`ank_contract::events::user_dir`] and this is the one name the rest of the
/// crate uses. `the_watch_file_sits_beside_the_corpora_file` in this crate's
/// suite still drives both binaries to assert `ank-cli`'s own copy agrees.
pub fn user_dir() -> Option<PathBuf> {
    ank_contract::events::user_dir()
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
) -> Result<Declaration> {
    // **The version first, and the fields afterwards**, which is the rule
    // `corpora.yml` and `.ank/config.yml` already follow. Like the probe in
    // `ank-cli`'s `parse()`, this may only ever *add* a refusal: where it
    // cannot read a version it says nothing and the deserialize below produces
    // exactly the diagnosis it always produced, a missing `schema` and a file
    // that is not a mapping included.
    if let Some(found) = serde_yaml::from_str::<SchemaProbe>(text)
        .ok()
        .map(|p| p.schema)
    {
        if found > SUPPORTED_SCHEMA {
            return Err(Fail::new(
                ExitCode::Environment,
                format!(
                    "{}: schema {found}, this binary reads {SUPPORTED_SCHEMA}: \
                     the file is newer than this ank",
                    path.display()
                ),
            )
            .with_hint("ank --version names the build, npm install -g @haksolot/ank replaces it"));
        }
    }

    let doc: WatchFileDoc = serde_yaml::from_str(text).map_err(|e| {
        Fail::new(ExitCode::Environment, format!("{}: {e}", path.display())).with_hint(
            "schema: 1 and a watch: map of repository identity to the path of a checkout, \
             or to a list of them",
        )
    })?;
    // Below the range rather than above it, exactly as `ank-cli` splits them:
    // no binary ever read schema 0, so there is no older tool to name and the
    // file is simply wrong.
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
    // **A stated zero is refused rather than rounded up.** `fetch: 0` reads as
    // "as often as possible", and as often as possible is a network round trip
    // per poll -- twice a second by default, against somebody's forge. A reader
    // who wants that has mistyped, and a watcher that silently substituted its
    // own number would be watching under a configuration its caller does not
    // have.
    let fetch = match doc.fetch {
        Some(0) => {
            return Err(Fail::new(
                ExitCode::Environment,
                format!(
                    "{}: fetch: 0 would fetch on every look rather than on an interval",
                    path.display()
                ),
            )
            .with_hint(format!(
                "give it a number of seconds, or omit fetch to take the default of {}",
                DEFAULT_FETCH.as_secs()
            )))
        }
        Some(secs) => Duration::from_secs(secs),
        None => DEFAULT_FETCH,
    };

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
    Ok(Declaration {
        fetch,
        watch: watched,
    })
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

    /// **The unknown key is the fixture, not decoration.**
    ///
    /// This test used to feed `schema: 7\nwatch: {}\n` -- a file one version
    /// ahead with no unknown field -- so the deserialize succeeded, the number
    /// check ran, and the ordering the refusal actually depends on was
    /// exercised by nothing. It stayed green for as long as the bug lived
    /// (TASK-56d188a1f8b3). With `mirrors` in the file and the probe removed,
    /// `deny_unknown_fields` answers first and the reader is sent after a typo
    /// in a file that has none.
    #[test]
    fn a_newer_schema_is_refused_as_newer_before_any_field_is_read() {
        let err = resolve(
            "schema: 7\nwatch: {}\nmirrors:\n  - somewhere\n",
            Path::new("watch.yml"),
            &none,
        )
        .unwrap_err();
        assert_eq!(err.code, ExitCode::Environment);
        assert!(err.message.contains("schema 7"), "{err:?}");
        assert!(err.message.contains("reads 1"), "{err:?}");
        assert!(err.message.contains("newer than this ank"), "{err:?}");
        assert!(
            !err.message.contains("mirrors"),
            "the version is the answer, never the key: {err:?}"
        );
    }

    /// Below the range is not "newer", and is not reported as such: no build
    /// ever read schema 0, so there is no older tool for a reader to go and
    /// install.
    #[test]
    fn a_schema_below_the_range_is_refused_by_number() {
        let err = resolve("schema: 0\nwatch: {}\n", Path::new("watch.yml"), &none).unwrap_err();
        assert_eq!(err.code, ExitCode::Environment);
        assert!(err.message.contains("schema 0 is not 1"), "{err:?}");
        assert!(!err.message.contains("newer than this ank"), "{err:?}");
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

    #[test]
    fn a_fetch_interval_of_zero_is_refused_rather_than_run_every_poll() {
        let err = resolve(
            "schema: 1\nfetch: 0\nwatch: {}\n",
            Path::new("watch.yml"),
            &none,
        )
        .unwrap_err();
        assert_eq!(err.code, ExitCode::Environment);
        assert!(err.message.contains("fetch: 0"), "{err:?}");
        // Refused before the emptiness of `watch:` is: a number that cannot be
        // honoured is wrong whatever it is applied to, and reporting the second
        // fault first would send the reader to correct the wrong line.
        assert!(
            !err.message.contains("nothing is declared"),
            "the interval is judged on its own terms: {err:?}"
        );
    }

    #[test]
    fn a_declaration_that_states_no_interval_takes_the_default() {
        // Read through a declaration that resolves, because the interval is a
        // field of the answer rather than a constant a caller reaches for.
        let dir = std::env::temp_dir().join(format!(
            "ank-daemon-declare-{}-{}",
            std::process::id(),
            "default"
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(ANK_DIR)).unwrap();
        let key = "a".repeat(40);
        let found = |_: &Path| Some("a".repeat(40));
        let text = format!("schema: 1\nwatch:\n  {key}: {}\n", dir.display());
        let declared = resolve(&text, Path::new("watch.yml"), &found).unwrap();
        assert_eq!(declared.fetch, DEFAULT_FETCH);

        let stated = format!("schema: 1\nfetch: 5\nwatch:\n  {key}: {}\n", dir.display());
        let declared = resolve(&stated, Path::new("watch.yml"), &found).unwrap();
        assert_eq!(declared.fetch, Duration::from_secs(5));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
