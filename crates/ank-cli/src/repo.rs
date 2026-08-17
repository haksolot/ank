//! Repository discovery (§6).
//!
//! An agent is launched in an arbitrary subdirectory of the tree: resolution
//! walks up to the first `.ank/`, the way git does for `.git`.
//! `--repo <path>` short-circuits the walk without ever contradicting it —
//! the given path must contain a `.ank/`.
//!
//! Resolving the corpus is also where its schema is looked at, because it is
//! the one property of a corpus a binary must know before it reads a single
//! entity: a file written under a schema this build does not know is not read,
//! and every listing verb then answers as if it were not there.

use crate::cli::CliError;
use crate::config::Config;
use crate::store::Store;
use ank_contract::ExitCode;
use ank_core::SCHEMA_VERSION;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

pub const ANK_DIR: &str = ".ank";

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    /// The directory containing `.ank/`.
    pub root: PathBuf,
    /// The `.ank/` directory itself.
    pub ank: PathBuf,
}

impl Repo {
    fn at(root: PathBuf) -> Repo {
        let ank = root.join(ANK_DIR);
        Repo { root, ank }
    }

    pub fn config_path(&self) -> PathBuf {
        self.ank.join("config.yml")
    }
}

fn missing(from: &Path) -> CliError {
    CliError::new(
        ExitCode::Generic,
        format!("no {ANK_DIR}/ found from {}", from.display()),
    )
    .with_hint("ank init")
}

/// Walks up from `start` to the first directory containing `.ank/`.
pub fn discover(start: &Path) -> Result<Repo> {
    let mut cur = Some(start);
    while let Some(dir) = cur {
        if dir.join(ANK_DIR).is_dir() {
            return Ok(Repo::at(dir.to_path_buf()));
        }
        cur = dir.parent();
    }
    Err(missing(start))
}

/// Explicit resolution through `--repo`. Accepts the directory containing
/// `.ank/`, or the `.ank/` itself — being off by one level is the most likely
/// mistake, and refusing it would gain nothing.
pub fn at(path: &Path) -> Result<Repo> {
    if path.join(ANK_DIR).is_dir() {
        return Ok(Repo::at(path.to_path_buf()));
    }
    if path.file_name().and_then(|n| n.to_str()) == Some(ANK_DIR) && path.is_dir() {
        if let Some(parent) = path.parent() {
            return Ok(Repo::at(parent.to_path_buf()));
        }
    }
    Err(missing(path))
}

/// Full resolution: `--repo` if given, otherwise the walk up from the current
/// directory.
pub fn resolve(repo_flag: Option<&str>, cwd: &Path) -> Result<Repo> {
    match repo_flag {
        Some(p) => at(Path::new(p)),
        None => discover(cwd),
    }
}

// ---------------------------------------------------------------------------
// Peer corpora (§7, ADR-a1de673043b4)
// ---------------------------------------------------------------------------
//
// **A peer is declared, never discovered.** Nothing below walks the filesystem
// looking for a sibling `.ank/` and nothing reads a git remote: a peer is a key
// in `config.yml` and only that. Inference is how a corpus starts depending on
// where somebody happened to check something out, which is the absolute scope
// wearing a different hat.
//
// **Reading crosses, writing never does.** Everything here opens a `Store`,
// which reads files and creates none — deliberately not an `Index`, whose first
// act is to write `index.db` into the corpus it was pointed at. That single
// choice is what makes the title of TASK-13e802e46050 true, and the integration
// test asserts it from outside by comparing the peer's bytes before and after.
//
// **Claims do not cross**, and nothing here reads or writes `refs/ank/*` of a
// peer: that is the one lock ADR-a1de673043b4 left standing.

/// Whether `s` can be the name of a peer.
///
/// Two characters at least, so that a scope entry can never be confused with a
/// Windows drive letter: `C:/Users` is a path on a machine, `front:src/**` is a
/// glob under a declared corpus, and one character of difference between the
/// two readings would be a corpus meaning something else on one platform.
pub fn is_peer_name(s: &str) -> bool {
    s.len() >= 2
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// A scope entry that names a peer and a glob under that peer's root, as
/// `<peer>:<glob>` — the form §7 describes, and the only thing that crosses.
///
/// `None` for an ordinary local glob, which is every entry in every corpus that
/// federates with nobody. Callers that match a scope against the checkout's own
/// files use this to leave a cross-corpus entry alone: it names no file here and
/// never will, so treating it as a dead scope would make declaring a peer a
/// corpus fault.
pub fn peer_ref(entry: &str) -> Option<(&str, &str)> {
    let (name, glob) = entry.split_once(':')?;
    (is_peer_name(name) && !glob.is_empty()).then_some((name, glob))
}

/// A declared peer, opened for reading.
pub struct Peer {
    /// The name the declaration gave it, which is the name a scope entry spells.
    pub name: String,
    pub repo: Repo,
    /// The peer's own configuration — needed because a scope entry written
    /// *there* is resolved through the declarations *there* (§7).
    pub config: Config,
}

/// The peer's root, as the declaration names it. A relative path is resolved
/// against the declaring repository's root, which is the form worth reviewing:
/// an absolute one is a fact about one machine.
fn declared_root(from: &Repo, declared: &str) -> PathBuf {
    let p = Path::new(declared);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        from.root.join(p)
    }
}

/// Whether two paths name the same corpus, asked of the filesystem rather than
/// of the strings.
///
/// `../front` from one root and an absolute path from another are the same
/// directory and must compare equal, which no textual comparison delivers —
/// symlinks, `..` and the case rules of Windows all sit in between. A path that
/// cannot be canonicalised does not exist, and two things that do not exist are
/// not the same corpus.
pub fn same_corpus(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Every peer this repository declares, with one warning per peer that could
/// not be opened.
///
/// **Degrade, never fail** (§2). A peer absent from disk, or a corpus this build
/// cannot read, costs one line and the local answer — the rule `status --remote`
/// already follows for an unreachable remote, and the same reason: a reader does
/// not fail because something it was told about is missing. Each warning names
/// the peer and ends with the command that settles it.
pub fn peers_of(from: &Repo, cfg: &Config) -> (Vec<Peer>, Vec<String>) {
    let mut peers = Vec::new();
    let mut warnings = Vec::new();
    for (name, declared) in &cfg.peers {
        if declared.is_empty() {
            warnings.push(format!(
                "peer '{name}' declares no path, answered without it \
                 (ank config peers.{name} <path>)"
            ));
            continue;
        }
        let root = declared_root(from, declared);
        let repo = match at(&root) {
            Ok(repo) => repo,
            Err(_) => {
                warnings.push(format!(
                    "peer '{name}' at {declared} is not a corpus, answered without it \
                     (ank config --unset peers.{name})"
                ));
                continue;
            }
        };
        let config = match crate::config::load(&repo.config_path()) {
            Ok(config) => config,
            Err(_) => {
                warnings.push(format!(
                    "peer '{name}' at {declared} could not be read, answered without it \
                     (ank --repo {declared} config schema)"
                ));
                continue;
            }
        };
        peers.push(Peer {
            name: name.clone(),
            repo,
            config,
        });
    }
    (peers, warnings)
}

impl Peer {
    /// Whether a scope entry written in this peer's corpus binds `reader`.
    ///
    /// The name is resolved through **this peer's** declarations, never the
    /// reader's: that is what makes the entry mean the same thing wherever it is
    /// read, and mean nothing at all where the peer is not declared (§7).
    pub fn binds(&self, name: &str, reader: &Repo) -> bool {
        match self.config.peers.get(name) {
            Some(declared) if !declared.is_empty() => {
                same_corpus(&declared_root(&self.repo, declared), &reader.root)
            }
            _ => false,
        }
    }
}

// ---------------------------------------------------------------------------
// A corpus written by a newer binary
// ---------------------------------------------------------------------------

/// A corpus declaring an entity schema this build does not read
/// (TASK-ca7b61b00896).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SchemaAhead {
    /// The newest schema any entity declares.
    pub found: u32,
    /// The newest this binary reads, which is `SCHEMA_VERSION`.
    pub supported: u32,
    /// How many entities declare a schema past `supported`. Counted, because
    /// it is what a reader compares against the listing that just came up
    /// short.
    pub entities: usize,
}

impl SchemaAhead {
    /// The two lines the caller prints, warning first, next step second.
    ///
    /// Built here rather than at the printing site so that the wording is
    /// asserted where the numbers are, and so that a second caller cannot
    /// phrase the same fact differently.
    pub fn lines(&self) -> (String, String) {
        let entities = if self.entities == 1 {
            "1 entity".to_string()
        } else {
            format!("{} entities", self.entities)
        };
        (
            format!(
                "corpus at schema {}, this binary reads {}: {entities} left out of every listing",
                self.found, self.supported
            ),
            "the binary is older than the corpus: ank --version names the build, \
             npm install -g @haksolot/ank replaces it"
                .to_string(),
        )
    }
}

/// The schema an entity file declares, read from its frontmatter and nothing
/// else.
///
/// Deliberately not a parse: a file one schema ahead is precisely the file
/// `parse_entity` refuses, so asking it would answer the question with the
/// failure the question exists to explain. Reading stops at the closing `---`,
/// so a `schema:` in a body is never mistaken for the field, and only the head
/// of each file is ever touched.
fn declared_schema(path: &Path) -> Option<u32> {
    let mut lines = BufReader::new(std::fs::File::open(path).ok()?).lines();
    // The frontmatter opens on the first line; anything else is not an entity
    // and has no schema to declare.
    if lines.next()?.ok()?.trim_end() != "---" {
        return None;
    }
    for line in lines {
        let line = line.ok()?;
        let line = line.trim_end();
        if line == "---" {
            return None;
        }
        // At column zero, so that an indented `schema:` inside a block scalar
        // is not read as the field.
        if let Some(value) = line.strip_prefix("schema:") {
            return value.trim().parse().ok();
        }
    }
    None
}

/// Whether this corpus declares a schema this build does not know.
///
/// One `read_dir` per layout directory and the head of each entity file — the
/// same walk the index already performs, and cheaper, since nothing here reads
/// a file whole. The index cannot answer instead: it records the hash of a
/// file it failed to parse, so the second invocation over the same corpus sees
/// it unchanged and reports nothing at all.
pub fn schema_ahead(repo: &Repo) -> Option<SchemaAhead> {
    let store = Store::new(&repo.ank);
    let ids = store.list_ids().ok()?;
    let mut found = SCHEMA_VERSION;
    let mut entities = 0;
    for id in ids {
        match declared_schema(&store.read_path_of(&id)) {
            Some(schema) if schema > SCHEMA_VERSION => {
                entities += 1;
                found = found.max(schema);
            }
            _ => {}
        }
    }
    (entities > 0).then_some(SchemaAhead {
        found,
        supported: SCHEMA_VERSION,
        entities,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(PathBuf);

    impl Temp {
        fn new() -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-repo-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(p.join(ANK_DIR).join("tasks")).unwrap();
            Temp(p)
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn walks_up_from_a_deep_subdirectory() {
        let t = Temp::new();
        let deep = t.0.join("crates").join("ank-cli").join("src");
        fs::create_dir_all(&deep).unwrap();

        let repo = discover(&deep).unwrap();
        assert_eq!(repo.root, t.0);
        assert_eq!(repo.ank, t.0.join(ANK_DIR));
    }

    #[test]
    fn a_missing_ank_dir_exits_pointing_at_ank_init() {
        let empty = std::env::temp_dir().join(format!("ank-empty-{}", std::process::id()));
        fs::create_dir_all(&empty).unwrap();
        // The walk up may reach a real Ank repository depending on where temp
        // lives; we only assert when resolution does fail.
        if let Err(err) = discover(&empty) {
            assert_eq!(err.code, ExitCode::Generic);
            assert_eq!(err.hint.as_deref(), Some("ank init"));
        }
        let _ = fs::remove_dir_all(&empty);
    }

    #[test]
    fn explicit_repo_accepts_the_root_and_the_ank_dir() {
        let t = Temp::new();
        assert_eq!(at(&t.0).unwrap().root, t.0);
        assert_eq!(at(&t.0.join(ANK_DIR)).unwrap().root, t.0);
    }

    #[test]
    fn the_repo_flag_short_circuits_the_walk_up() {
        let t = Temp::new();
        let elsewhere = std::env::temp_dir();
        let via_flag = resolve(Some(t.0.to_str().unwrap()), &elsewhere).unwrap();
        assert_eq!(via_flag.root, t.0);
    }

    /// Writes an entity declaring `schema`, with `extra` appended to the body.
    fn seed(t: &Temp, id: &str, schema: u32, extra: &str) {
        let dir = t.0.join(ANK_DIR).join("entities");
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(format!("{id}.md")),
            format!(
                "---\nid: {id}\ntype: task\nslug: example\ntitle: Example\n\
                 created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
                 blocked_by: []\nschema: {schema}\nversion: 1\n---\n\nBody.\n{extra}"
            ),
        )
        .unwrap();
    }

    /// The one form that crosses, and the forms that must not be mistaken for
    /// it (§7).
    #[test]
    fn a_scope_entry_names_a_peer_only_in_the_form_the_spec_gives() {
        assert_eq!(peer_ref("front:src/**"), Some(("front", "src/**")));
        assert_eq!(
            peer_ref("ank-core:crates/**"),
            Some(("ank-core", "crates/**"))
        );

        // An ordinary local glob, which is every entry in every corpus that
        // federates with nobody.
        assert_eq!(peer_ref("src/**"), None);
        assert_eq!(peer_ref("docs/spec.md"), None);
        // A Windows drive letter. This is why a peer name is two characters at
        // minimum: one corpus must not mean two things depending on the machine
        // reading it.
        assert_eq!(peer_ref("C:/Windows/**"), None);
        // A name with nothing under it names no files.
        assert_eq!(peer_ref("front:"), None);
        // A name a declaration could never carry.
        assert_eq!(peer_ref("front end:src/**"), None);
    }

    /// A peer absent from disk is one warning and no peer, never a failure
    /// (§2). The reader answers, which is the whole of "degrade".
    #[test]
    fn an_absent_peer_is_one_warning_and_the_local_answer() {
        let t = Temp::new();
        let repo = Repo::at(t.0.clone());
        let cfg = crate::config::parse(
            "schema: 1\npeers:\n  gone: ../nowhere\n  blank: \"\"\n",
            Path::new("config.yml"),
        )
        .unwrap();

        let (peers, warnings) = peers_of(&repo, &cfg);
        assert!(peers.is_empty());
        assert_eq!(warnings.len(), 2, "{warnings:?}");
        assert!(
            warnings.iter().any(|w| w.contains("peer 'gone'")),
            "{warnings:?}"
        );
        assert!(
            warnings
                .iter()
                .any(|w| w.contains("peer 'blank' declares no path")),
            "{warnings:?}"
        );
    }

    /// A declaration resolves relative to the root that wrote it, and `binds`
    /// answers about the corpus it actually names — not about the string.
    #[test]
    fn a_peer_declaration_resolves_against_the_root_that_wrote_it() {
        let a = Temp::new();
        let b = Temp::new();
        let to_b = format!("../{}", b.0.file_name().unwrap().to_string_lossy());
        let to_a = format!("../{}", a.0.file_name().unwrap().to_string_lossy());
        for (t, peer, path) in [(&a, "bee", &to_b), (&b, "ay", &to_a)] {
            fs::write(
                t.0.join(ANK_DIR).join("config.yml"),
                format!("schema: 1\npeers:\n  {peer}: {path}\n"),
            )
            .unwrap();
        }

        let ra = Repo::at(a.0.clone());
        let rb = Repo::at(b.0.clone());
        let cfg_a = crate::config::load(&ra.config_path()).unwrap();

        let (peers, warnings) = peers_of(&ra, &cfg_a);
        assert!(warnings.is_empty(), "{warnings:?}");
        assert_eq!(peers.len(), 1);
        let bee = &peers[0];
        assert_eq!(bee.name, "bee");
        assert!(same_corpus(&bee.repo.root, &b.0));

        // The name is resolved through the peer's own declarations, so `ay`
        // written in B points back at A -- and a name B does not declare points
        // nowhere at all.
        assert!(bee.binds("ay", &ra));
        assert!(!bee.binds("ay", &rb));
        assert!(!bee.binds("undeclared", &ra));
    }

    #[test]
    fn a_corpus_this_binary_reads_is_not_ahead() {
        let t = Temp::new();
        seed(&t, "TASK-000000000001", SCHEMA_VERSION, "");
        seed(&t, "TASK-000000000002", 1, "");
        assert_eq!(schema_ahead(&Repo::at(t.0.clone())), None);
    }

    #[test]
    fn the_newest_schema_ahead_is_named_and_the_entities_counted() {
        let t = Temp::new();
        seed(&t, "TASK-000000000001", SCHEMA_VERSION + 1, "");
        seed(&t, "TASK-000000000002", SCHEMA_VERSION + 3, "");
        // Readable, and therefore not counted: the number is what the listing
        // is short by, not the size of the corpus.
        seed(&t, "TASK-000000000003", 1, "");

        let ahead = schema_ahead(&Repo::at(t.0.clone())).expect("two entities are ahead");
        assert_eq!(ahead.found, SCHEMA_VERSION + 3);
        assert_eq!(ahead.supported, SCHEMA_VERSION);
        assert_eq!(ahead.entities, 2);
    }

    #[test]
    fn a_schema_line_in_a_body_is_not_the_field() {
        // The reason reading stops at the closing `---`. A body is free prose
        // and may quote anything, this line included.
        let t = Temp::new();
        seed(
            &t,
            "TASK-000000000001",
            1,
            &format!("\nschema: {}\n", SCHEMA_VERSION + 9),
        );
        assert_eq!(schema_ahead(&Repo::at(t.0.clone())), None);
    }

    #[test]
    fn the_message_counts_in_words_the_reader_can_check() {
        let one = SchemaAhead {
            found: 4,
            supported: 3,
            entities: 1,
        };
        let (what, next) = one.lines();
        assert!(
            what.contains("schema 4") && what.contains("reads 3"),
            "{what}"
        );
        assert!(what.contains("1 entity left"), "{what}");
        assert!(next.contains("ank --version"), "{next}");

        let many = SchemaAhead { entities: 2, ..one };
        assert!(
            many.lines().0.contains("2 entities left"),
            "{}",
            many.lines().0
        );
    }
}
