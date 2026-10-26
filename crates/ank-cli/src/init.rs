//! `ank init` (§9).
//!
//! A narrow, fixed perimeter: create `.ank/`, write `config.yml`, add the
//! `refs/ank/*` refspec, place the agent bootstrap in `AGENTS.md` and its
//! import in `CLAUDE.md`, and write a `.gitattributes` and a `.gitignore`.
//!
//! Neither git file is cosmetic. On Windows `core.autocrlf=true` is the
//! default, and without `.gitattributes` git converts back to CRLF on every
//! checkout what the tool has just written in LF — making any fresh clone
//! unreadable. Without `.gitignore`, §6 calls the index derived, disposable
//! and gitignored while no repository `init` produces is any of the third:
//! the first `git add -A` commits a binary file that every command rewrites.
//! Fixing either one only in Ank's own repository would have left it broken
//! for every user — which is precisely how the second survived, since this
//! repository carries the ignore line by hand, written before `init` existed
//! to write it.

use crate::cli::{CliError, Invocation, Result};
use crate::store::Store;
use crate::{config, git, repo};
use ank_contract::ExitCode;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const GITATTRIBUTES_LINE: &str = ".ank/** text eol=lf";
/// Root-relative on purpose: a `gitignore` pattern holding a `/` is anchored
/// to the directory of the file that carries it, so this stays correct when
/// `init` runs on a subdirectory rather than on the repository root.
///
/// **A glob, because the index is not one file.** In WAL mode SQLite keeps
/// `index.db-wal` and `index.db-shm` beside the database and removes them only
/// when the last connection closes cleanly. Measured on 2026-09-22: `ank find`
/// killed mid-run left both, and under the literal `.ank/index.db` `git status
/// --porcelain -uall` offered them for commit (TASK-574dee03ba56). The star
/// covers them and a rollback `-journal` alike, and nothing else `.ank/` holds
/// starts with that name.
pub const GITIGNORE_LINE: &str = ".ank/index.db*";
/// What `init` wrote before the glob, rewritten in place wherever it is found
/// so a repository is never left carrying both.
const LEGACY_GITIGNORE_LINE: &str = ".ank/index.db";
pub const REFSPEC: &str = "+refs/ank/*:refs/ank/*";
/// The command that adds [`REFSPEC`] once origin exists, and the one `init`,
/// `status` and `check` all name (TASK-623d80886c2f). `init` itself, because it
/// is idempotent and adds the refspec from exactly that state: a narrower verb
/// would be a second place the rule lives.
pub const REFSPEC_REPAIR: &str = "ank init";
/// What a remote named `ank` fetches, since it carries the claims and nothing
/// else (ADR-b6fe84504ff2).
pub const CLAIMS_REFSPEC: &str = "+refs/ank/claims/*:refs/ank/claims/*";
/// What origin's fetch carries beside [`REFSPEC`] once `ank` takes the claims:
/// a negative refspec, so that neither `git fetch origin` writes a claim origin
/// still holds nor `git fetch --prune origin` deletes a live claim origin does
/// not (ADR-b6fe84504ff2).
///
/// Measured on git 2.47.3 before relying on it (TASK-df475a664d21): without it
/// a `fetch --prune origin` deleted a claim held locally and on `ank`, and
/// wrote one origin held; with it, neither, while `refs/ank/ratify/*` and
/// `refs/ank/proof/*` came all the same. Negative refspecs are git 2.29 and
/// later, prune included (git's own t5582 carries `fetch --prune with negative
/// refspec`), below the 2.34 floor.
pub const NOT_CLAIMS: &str = "^refs/ank/claims/*";
/// What the run that creates `.ank/` appends to `AGENTS.md` (ADR-5eb0f65c75da).
///
/// It names Ank as a CLI, sends the reader to `ank context <path>` before an
/// edit, forbids reading `.ank/` by hand, names `claim`, `done` and `help`, and
/// links the install page rather than one channel of it. It summarises no
/// constraint: a summary drifts, and the route to them does not. Measured on
/// 2026-10-26 with agents that had no skill, no hook and no memory: the single
/// pointer line it replaces kept 3 runs out of 8 inside an accepted ADR, every
/// one after reading `.ank/` by hand, and a block of this shape kept 6 out of 6.
/// The first line is that pointer, kept verbatim.
const AGENTS_BLOCK: &str = "\
This repo uses Ank: tasks and decisions live in `.ank/`.
Ank is a CLI, `ank`, and `.ank/` is opaque like `.git/`: never read or edit its files by hand, reach them through the CLI.

- Run `ank context <path>` before editing a path: it prints the decisions that bind it, in full, and what is claimable there.
- `ank claim <id>` takes a task and freezes its criterion; `ank done` closes it, running its verifiers itself.
- `ank help` lists every verb, and `ank help <verb>` gives its flags.

Install: https://haksolot.github.io/ank/install.html
";
/// The line that makes Claude Code load `AGENTS.md`, which it does not read
/// on its own (ADR-5eb0f65c75da).
const CLAUDE_IMPORT: &str = "@AGENTS.md";

pub fn run(inv: &Invocation, cwd: &Path, out: &mut dyn Write) -> Result<ExitCode> {
    // **Refused, and refused before anything is written** (§4, §9). `--repo`
    // names a repository that already carries a `.ank/`, which is what this verb
    // is run to produce, so it is the one verb the flag does not apply to — and
    // the target is positional.
    //
    // Not a matter of tidiness: `dispatch` routes `init` ahead of the foundation
    // that resolves `--repo` for every other verb, and rightly so, since `init`
    // precedes the existence of the repository. But routing early is not a
    // reason to drop a global silently, and this one was dropped into a verb
    // that writes. Measured: `ank init --repo <elsewhere>` initialised the
    // *current* repository, appended the pointer paragraph to an `AGENTS.md`
    // nobody was editing, reported `pointer added to AGENTS.md`, and left the
    // named directory empty — noticed only because the next command there
    // answered `no .ank/ found` (TASK-b8a12d60686d).
    if let Some(path) = inv.repo() {
        return Err(CliError::new(
            ExitCode::Generic,
            "--repo names a repository that exists, and init is what makes one",
        )
        .with_hint(format!("ank init {path}")));
    }

    let detached = inv.value("--at");
    if detached.is_some() && inv.positionals.first().is_some() {
        return Err(CliError::new(
            ExitCode::Generic,
            "--at and a positional target name two different directories",
        )
        .with_hint("ank init --at <path>"));
    }
    let named = detached.or(inv.positionals.first().map(String::as_str));
    let target = match named {
        Some(p) => PathBuf::from(p),
        None => cwd.to_path_buf(),
    };
    // git is a hard dependency (ADR-9307e5d214a7): we fail before writing
    // anything rather than leave a half-placed `.ank/` in a directory that is
    // not a repository.
    if detached.is_some() {
        // Everything a detached corpus is refused for, before a byte is
        // written: the identity it will be keyed on, the tree it must stay out
        // of, and the repository whose refs will carry its claims.
        detachable(cwd, &target)?;
    }
    git::ensure_usable(&target)?;
    // **Decided here, written after the corpus, and that ordering is the
    // point** (TASK-0dd151b02854). `detachable` above is documented as taking
    // everything a detached corpus is refused for before a byte is written,
    // and the declaration was the one refusal that escaped it: `init_at` ran
    // first, so a `corpora.yml` this binary cannot read produced a correct
    // refusal *after* the corpus had been created. Measured: the target was
    // left holding `.ank/config.yml`, `.ank/entities`, `.ank/log`, `AGENTS.md`,
    // `.gitattributes`, `.gitignore` and a `+refs/ank/*` refspec written into
    // its own `.git/config`, with no declaration anywhere pointing at any of
    // it.
    //
    // Refused earlier rather than repaired afterwards. Undoing an `ensure_line`
    // append to a file that may have pre-existed, and a refspec written into
    // another repository, is new surface with its own failure modes, and a
    // rollback that fails leaves worse state than the one it was called to
    // clean. `plan_corpus` takes every refusal and touches nothing, so there is
    // nothing left to undo.
    let pending = match detached {
        Some(_) => {
            let identity = repo::identity(cwd).expect("checked by detachable");
            Some(config::plan_corpus(&identity, &target.to_string_lossy())?)
        }
        None => None,
    };
    let mut report = init_at(&target)?;
    if let Some(pending) = pending {
        report.declared = Some(pending.write()?.to_string_lossy().to_string());
    }
    // `--json` is available on every command without exception (§4), and this
    // verb was the exception: it printed its prose and ignored the flag. The
    // sweep never caught it because a `Repo` fixture already carries a `.ank/`,
    // so `init` refused, stdout was empty, and an empty stdout is what a
    // refusal is supposed to leave (TASK-9e63827380a1).
    if inv.json() {
        let _ = writeln!(out, "{}", report.json());
    } else if !inv.quiet() {
        for line in report.lines_terse() {
            let _ = writeln!(out, "{line}");
        }
    }
    Ok(ExitCode::Ok)
}

/// What `init` writes into a corpus it creates: the canonical file, raised to
/// the schema that carries `requires_ank` and declaring the running version
/// (ADR-756e8b0fe416).
///
/// **Beneath `schema:`**, which is where `ank config requires_ank` inserts the
/// key, so a corpus born here and one raised afterwards read the same. The
/// running version and not a release: `init` reaches no network, and the
/// binary that made the corpus is the one known to read it.
fn born_config() -> String {
    let canonical = config::default_yaml();
    let rest = canonical
        .strip_prefix(&format!("schema: {}\n", config::MIN_SCHEMA))
        .expect("default_yaml opens on its schema");
    format!(
        "schema: {}\nrequires_ank: {}\n{rest}",
        config::REQUIRES_ANK_SCHEMA,
        crate::update::Version::running()
    )
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// The directories this run made, in the order they were made. A list and
    /// not a flag, which is what it was before: the flag named every directory
    /// whenever any one of them was missing, so a run that created one of two
    /// reported creating a directory it had found. There is one of them today
    /// (TASK-acaf2c3159dd) and the list stays a list, because `created` is an
    /// array in the machine contract and a reader parses one shape whatever the
    /// count.
    pub created_dirs: Vec<String>,
    /// The declarations file a `--at` run wrote into (ADR-96174f1ac2b7).
    ///
    /// Named rather than silent, and named as a path: it is the one thing this
    /// verb wrote outside the directory the caller pointed at, and a reader who
    /// wants to see it or correct it needs its address.
    pub declared: Option<String>,
    pub wrote_config: bool,
    pub wrote_gitattributes: bool,
    pub wrote_gitignore: bool,
    pub wrote_agents_pointer: bool,
    pub wrote_claude_import: bool,
    pub added_refspec: bool,
    /// The refspec is not there and could not be added, because no remote
    /// named `origin` has a URL yet (TASK-623d80886c2f). Said rather than
    /// skipped in silence: the usual order for a new repository -- `git init`,
    /// `ank init`, then the remote -- lands here every time.
    pub refspec_waits_for_origin: bool,
    /// The lines [`arrange_claims`] wrote, one per refspec it added to or
    /// removed from a remote's fetch.
    pub claims_refspecs: Vec<ClaimsRefspec>,
}

impl Report {
    /// The same six effects as [`lines_terse`], grouped by the verb that
    /// applies to them, and always the same shape: three lists, empty when
    /// there is nothing in them, so a parser reads one document and not two.
    ///
    /// `changed` is the question a script actually asks — whether there is
    /// anything to commit — and it is answered rather than left to be derived
    /// from the emptiness of three lists.
    ///
    /// [`lines_terse`]: Report::lines_terse
    pub fn json(&self) -> String {
        let mut wrote: Vec<&str> = Vec::new();
        if self.wrote_config {
            wrote.push(".ank/config.yml");
        }
        if self.wrote_gitattributes {
            wrote.push(".gitattributes");
        }
        if self.wrote_gitignore {
            wrote.push(".gitignore");
        }
        // Where each was added, not what was added: the caller who wants to
        // inspect the result needs the address, and the two addresses are of
        // different kinds — a file and a git config key.
        let mut added: Vec<&str> = Vec::new();
        if let Some(file) = &self.declared {
            added.push(file.as_str());
        }
        if self.wrote_agents_pointer {
            added.push("AGENTS.md");
        }
        if self.wrote_claude_import {
            added.push("CLAUDE.md");
        }
        if self.added_refspec {
            added.push("remote.origin.fetch");
        }
        for c in &self.claims_refspecs {
            if !added.contains(&c.key()) {
                added.push(c.key());
            }
        }
        crate::json::Obj::document()
            .strings("created", &self.created_dirs)
            .strings("wrote", &wrote)
            .strings("added", &added)
            .bool("changed", self.changed())
            .finish()
    }

    /// Whether this run had any effect at all.
    pub fn changed(&self) -> bool {
        !self.created_dirs.is_empty()
            || self.wrote_config
            || self.wrote_gitattributes
            || self.wrote_gitignore
            || self.wrote_agents_pointer
            || self.wrote_claude_import
            || self.added_refspec
            || !self.claims_refspecs.is_empty()
    }

    /// Terse output, `git status` style: one line per real effect, nothing for
    /// what was already in place.
    pub fn lines_terse(&self) -> Vec<String> {
        let mut v = Vec::new();
        if !self.created_dirs.is_empty() {
            v.push(format!("created {}", self.created_dirs.join(" ")));
        }
        if self.wrote_config {
            v.push("wrote .ank/config.yml".to_string());
        }
        if self.wrote_gitattributes {
            v.push("wrote .gitattributes".to_string());
        }
        if self.wrote_gitignore {
            v.push("wrote .gitignore".to_string());
        }
        if self.wrote_agents_pointer {
            v.push("pointer added to AGENTS.md".to_string());
        }
        if self.wrote_claude_import {
            v.push(format!("{CLAUDE_IMPORT} added to CLAUDE.md"));
        }
        if self.added_refspec {
            v.push(format!("refspec added: {REFSPEC}"));
        }
        v.extend(self.claims_refspecs.iter().map(ClaimsRefspec::line));
        // Last, and never silent: it is the one thing this verb wrote outside
        // the directory the caller pointed at.
        if let Some(file) = &self.declared {
            v.push(format!("declared in {file}"));
        }
        if v.is_empty() {
            v.push("already initialised, nothing to do".to_string());
        }
        // Not an effect, so it neither counts as a change nor displaces the
        // line above: it is what this run could not do, and what will.
        if self.refspec_waits_for_origin {
            v.push(format!(
                "refspec {REFSPEC} not added: no remote named origin yet, \
                 run this once it exists ({REFSPEC_REPAIR})"
            ));
        }
        v
    }
}

fn io(path: &Path, e: std::io::Error) -> CliError {
    CliError::new(ExitCode::Generic, format!("{}: {e}", path.display()))
}

/// Idempotent: re-initialising an already initialised repository must break
/// nothing and duplicate nothing. That is what makes it safe to run without
/// thinking.
pub fn init_at(root: &Path) -> Result<Report> {
    let mut report = Report::default();
    let ank = root.join(repo::ANK_DIR);
    // Read before anything below creates it: the `AGENTS.md` pointer belongs
    // to the run that makes the corpus, and only to that one.
    let creates_corpus = !ank.exists();

    // **One directory, and it is the only one this verb makes** (§6). Every
    // entity of every kind lives in `.ank/entities/`, an entry included since
    // ADR-25f977377fa0 made it an entity like any other.
    //
    // The previous layouts -- `tasks/`, `adr/`, and the `log/` that held one
    // file of lines per entity -- are read where they already exist and are
    // never created. `docs/format.md` puts it in four words, of a reader and a
    // writer respectively: *must accept all of them*, *must never produce
    // them*, and this verb is a writer. `log/` outlived that rule by ten
    // weeks. Measured on ddf11c9: `ank init` in a fresh repository printed
    // `created .ank/entities .ank/log`, and after `new task`, `claim` and
    // `log`, `.ank/log` held zero files while the three entities -- the task,
    // the claim record and the entry -- were all in `.ank/entities/`
    // (TASK-acaf2c3159dd). It survived because `check` counts the *files* in
    // that directory and an empty one is nothing to count, so every corpus this
    // binary made was born with a directory no verb writes into and no verb
    // complains about -- which is what ADR-c9f9d0d6f05d rules out in as many
    // words: no directory means anything, and none is added to make one mean
    // something.
    //
    // Nothing is removed here. A corpus that already carries a `.ank/log/` from
    // an older release keeps it, keeps being read, and is moved by `ank migrate`
    // and never by this verb.
    let dir = ank.join(Store::ENTITIES_DIR);
    if !dir.is_dir() {
        fs::create_dir_all(&dir).map_err(|e| io(&dir, e))?;
        report
            .created_dirs
            .push(format!("{}/{}", repo::ANK_DIR, Store::ENTITIES_DIR));
    }

    let cfg = ank.join("config.yml");
    if !cfg.exists() {
        fs::write(&cfg, born_config()).map_err(|e| io(&cfg, e))?;
        report.wrote_config = true;
    }

    let ga = root.join(".gitattributes");
    report.wrote_gitattributes = ensure_line(&ga, GITATTRIBUTES_LINE)?;

    let gi = root.join(".gitignore");
    report.wrote_gitignore = ensure_gitignore(&gi)?;

    // A bootstrap aid and not an invariant (ADR-1f5a45d43f82). `init` is also
    // the repair verb `status` and `check` name for a missing refspec, and a
    // repair that re-appended the pointer undid every team that dropped it on
    // purpose: observed 2026-10-11 in this repository. On an existing corpus
    // `AGENTS.md` is left as found, created or not, and `CLAUDE.md` with it
    // (ADR-5eb0f65c75da).
    if creates_corpus {
        let agents = root.join("AGENTS.md");
        report.wrote_agents_pointer = ensure_block(&agents, AGENTS_BLOCK)?;
        let claude = root.join("CLAUDE.md");
        report.wrote_claude_import = ensure_import(&claude)?;
    }

    let found = remotes(root);
    match ensure_refspec(root, found.origin)? {
        OriginFetch::Lacks => report.added_refspec = true,
        OriginFetch::NoOrigin => report.refspec_waits_for_origin = true,
        OriginFetch::Fetches => {}
    }
    let origin = match found.origin {
        OriginFetch::Lacks => OriginFetch::Fetches,
        other => other,
    };
    report.claims_refspecs = arrange_claims(root, Remotes { origin, ..found })?;

    Ok(report)
}

/// Whether a corpus may be created at `target` for the repository `cwd` sits
/// in, and every reason it may not (ADR-96174f1ac2b7).
///
/// **Inside the tree is refused rather than accepted quietly.** A corpus under
/// the working tree is not a detached corpus, and `--at .ank` would be a long
/// way of writing `ank init` while writing a declaration that promises
/// something it does not deliver. `ank init` is the verb for a corpus that
/// lives beside its code.
///
/// **A tree with no identity is refused too**, because there would be nothing
/// to key the declaration on. A fallback is not a key: every historyless tree
/// on a machine would share one, which is the collision ADR-621a7fd96ce1 chose
/// the root commit to avoid.
///
/// **git is not created here.** The corpus repository is where claims and
/// proofs land (ADR-9e56318631f3), so the target has to be one — but `git init`
/// is not on the plumbing ADR-9307e5d214a7 allows, and running a verb this tool
/// has never run to save a caller one command is not a trade this makes on its
/// own. The refusal names the command, which is what §4 asks of it.
fn detachable(cwd: &Path, target: &Path) -> Result<()> {
    let Some(_) = repo::identity(cwd) else {
        return Err(CliError::new(
            ExitCode::Prerequisite,
            "this tree has no repository identity, so a declaration has nothing to be keyed on",
        )
        .with_hint("git commit: the identity is the root commit"));
    };
    let tree = git::run(cwd, &["rev-parse", "--show-toplevel"])
        .map(PathBuf::from)
        .unwrap_or_else(|_| cwd.to_path_buf());
    // Compared after canonicalisation, for the reason `same_corpus` gives:
    // `..`, symlinks and the case rules of Windows all sit between two spellings
    // of one directory. A target that does not exist yet is canonicalised
    // through its parent, since that is the part that already does.
    let of = |p: &Path| -> PathBuf {
        std::fs::canonicalize(p).unwrap_or_else(|_| match (p.parent(), p.file_name()) {
            (Some(parent), Some(name)) => std::fs::canonicalize(parent)
                .unwrap_or_else(|_| parent.to_path_buf())
                .join(name),
            _ => p.to_path_buf(),
        })
    };
    let (tree, target) = (of(&tree), of(target));
    if target.starts_with(&tree) {
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "--at {} is inside {}, which is the tree it is meant to stay out of",
                target.display(),
                tree.display()
            ),
        )
        .with_hint("ank init, for a corpus that lives beside its code"));
    }
    Ok(())
}

/// Adds a line to a file if it is not already there. Returns `true` if the
/// file was touched.
fn ensure_line(path: &Path, line: &str) -> Result<bool> {
    let existing = read_or_empty(path)?;
    if existing.lines().any(|l| l.trim() == line) {
        return Ok(false);
    }
    append(path, existing, &format!("{line}\n"))
}

/// Appends `block` to a file unless the file already holds it whole, keeping
/// every byte already there. Returns `true` if the file was touched.
fn ensure_block(path: &Path, block: &str) -> Result<bool> {
    let existing = read_or_empty(path)?;
    if existing.contains(block) {
        return Ok(false);
    }
    append(path, existing, block)
}

/// Appends [`CLAUDE_IMPORT`] to `CLAUDE.md` unless one of its lines already
/// imports `AGENTS.md`, written as `@AGENTS.md` or `@./AGENTS.md` anywhere on
/// the line, which is where Claude Code reads an import. Returns `true` if the
/// file was touched.
fn ensure_import(path: &Path) -> Result<bool> {
    let existing = read_or_empty(path)?;
    let imports = existing.lines().any(|l| {
        l.split_whitespace()
            .any(|w| w == CLAUDE_IMPORT || w == "@./AGENTS.md")
    });
    if imports {
        return Ok(false);
    }
    append(path, existing, &format!("{CLAUDE_IMPORT}\n"))
}

fn read_or_empty(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(s) => Ok(s),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(io(path, e)),
    }
}

/// `existing` followed by `text`, a blank line between them when there was
/// anything before.
fn append(path: &Path, mut existing: String, text: &str) -> Result<bool> {
    if !existing.is_empty() && !existing.ends_with('\n') {
        existing.push('\n');
    }
    if !existing.is_empty() {
        existing.push('\n');
    }
    existing.push_str(text);
    fs::write(path, existing).map_err(|e| io(path, e))?;
    Ok(true)
}

/// [`ensure_line`] for the ignore line, which first brings the previous one
/// up to date: the legacy literal is replaced where it stands, keeping the
/// user's ordering, and dropped outright when the glob is already there.
fn ensure_gitignore(path: &Path) -> Result<bool> {
    let existing = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(io(path, e)),
    };
    if !existing.lines().any(|l| l.trim() == LEGACY_GITIGNORE_LINE) {
        return ensure_line(path, GITIGNORE_LINE);
    }
    let mut placed = existing.lines().any(|l| l.trim() == GITIGNORE_LINE);
    let mut next = String::with_capacity(existing.len() + 1);
    for l in existing.split_inclusive('\n') {
        if l.trim() != LEGACY_GITIGNORE_LINE {
            next.push_str(l);
        } else if !placed {
            next.push_str(GITIGNORE_LINE);
            next.push_str(if l.ends_with("\r\n") { "\r\n" } else { "\n" });
            placed = true;
        }
    }
    fs::write(path, next).map_err(|e| io(path, e))?;
    Ok(true)
}

/// Hosts do not fetch non-standard refs on their own (§7), so `origin` needs
/// `+refs/ank/*` in its fetch refspec. It is written **only once a remote named
/// `origin` has a URL**, and never before.
///
/// Writing it earlier is what TASK-f067ae7c84ff and TASK-0878e19675f4 were
/// both about, and the two attempts are worth stating so neither is tried
/// again. Writing `remote.origin.fetch` into `.git/config` outright made git
/// read `origin` as already configured, so `git remote add origin <url>` exited
/// 3 with `remote origin already exists` -- verbatim the command `ank status
/// --remote` names as the repair -- and `git remote set-url`, the way round it,
/// left `+refs/ank/*` as the only refspec, so a plain `git fetch` brought no
/// branch. Deferring the key into a file included under
/// `includeIf.hasconfig:remote.*.url:**` fixed the common order and not the
/// general case: `remote.*.url` is git's only form of that condition and cannot
/// name `origin`, so any remote with a URL turned the include live and the
/// refusal came back. Measured on git 2.47.3: `remote add upstream` then
/// `remote add origin` exits 3, and worktree scope hides the key no better.
///
/// There is no config placement a plain `git fetch` reads and `git remote add`
/// does not, so the trap is closed by not setting it early rather than by
/// hiding it. What that costs is one more `ank init` after the remote is added,
/// which is the step the guide already describes and which the second branch
/// below completes; what it buys is that the command `status` prints works from
/// every state `init` can leave behind.
///
/// Answers with the state it found: `Lacks` is the one it repaired.
fn ensure_refspec(root: &Path, found: OriginFetch) -> Result<OriginFetch> {
    if found == OriginFetch::Lacks {
        git::run(root, &["config", "--add", "remote.origin.fetch", REFSPEC])?;
    }
    Ok(found)
}

/// What `origin` fetches, repaired when it lacks [`REFSPEC`] (ADR-1b030fe70442),
/// with the line that says so.
///
/// Every clone starts without the refspec, because `git clone` does not carry
/// `remote.origin.fetch`, and until it was added by hand `check` asserted an
/// absence it could not observe: 214 `done with no test proof` signals in a
/// fresh clone of this repository at fb17a26a, every one of them attested on
/// origin. So a verb that already asks this question repairs the answer it
/// gets: the refspec is added through the writer `init` uses, and `refs/ank/*`
/// is fetched once, at the transition. A clone that has the refspec pays
/// nothing beyond the question it was already asking; with no origin nothing
/// is written, for the reason [`ensure_refspec`] gives.
///
/// **A failed fetch is not an error.** The refspec stays written and the next
/// `git fetch` brings the refs; the verb answers on what this clone holds, as
/// it does with the refspec present and origin unreachable. A refspec that
/// could not be written is left as `Lacks`, so `status` and `check` keep
/// saying so.
pub fn repair_origin(root: &Path) -> (OriginFetch, Option<String>) {
    let (remotes, line) = repair_remotes(root);
    (remotes.origin, line)
}

/// [`repair_origin`], answering as well whether a coordination remote named
/// `ank` is in force (ADR-b6fe84504ff2), out of the same process: `status`
/// names it on every call, and a verb read this often does not start a second
/// `git config` to learn it.
///
/// **The claims refspecs are repaired by the same question** (ADR-b6fe84504ff2,
/// TASK-df475a664d21): a clone where `ank` was added after `init` gets what
/// `init` would have written, [`arrange_claims`], and says so on the same
/// stream, one line per refspec. Nothing is fetched for them: claims are read
/// off `ank` by `ls-remote` whatever this clone holds, and the next `git fetch`
/// brings them. A clone with origin alone answers exactly as before.
pub fn repair_remotes(root: &Path) -> (Remotes, Option<String>) {
    let found = remotes(root);
    if found.origin != OriginFetch::Lacks && found.claims_settled() {
        return (found, None);
    }
    let mut now = found;
    let mut lines = Vec::new();
    let mut fetch = false;
    if found.origin == OriginFetch::Lacks && ensure_refspec(root, found.origin).is_ok() {
        now.origin = OriginFetch::Fetches;
        fetch = true;
    }
    match arrange_claims(root, now) {
        Ok(done) => {
            now = remotes_after(now, &done);
            lines.extend(done.iter().map(ClaimsRefspec::line));
        }
        // Left as found, so `status` and `check` keep saying what is missing.
        Err(_) => now = remotes(root),
    }
    if fetch {
        // The refspecs as configured, given on the command line, so a fetch
        // from origin with `ank` in force brings no claim either.
        let mut specs = vec![REFSPEC];
        if now.origin_skips_claims {
            specs.push(NOT_CLAIMS);
        }
        let fetched = git::fetch_refs(root, git::ORIGIN, &specs).unwrap_or(false);
        lines.insert(
            0,
            if fetched {
                format!("refspec added: {REFSPEC}, and refs/ank/* fetched from origin")
            } else {
                format!(
                    "refspec added: {REFSPEC}; refs/ank/* could not be fetched from \
                     origin, the next git fetch brings them"
                )
            },
        );
    }
    let line = (!lines.is_empty()).then(|| lines.join("\n"));
    (now, line)
}

/// One refspec [`arrange_claims`] added to or removed from a remote's fetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimsRefspec {
    /// [`CLAIMS_REFSPEC`] added to `remote.ank.fetch`.
    AnkFetchesClaims,
    /// [`NOT_CLAIMS`] added to `remote.origin.fetch`.
    OriginSkipsClaims,
    /// [`NOT_CLAIMS`] removed from `remote.origin.fetch`, because no remote
    /// named `ank` carries the claims any more and origin does again.
    OriginFetchesClaims,
}

impl ClaimsRefspec {
    /// The git config key it wrote.
    pub fn key(self) -> &'static str {
        match self {
            ClaimsRefspec::AnkFetchesClaims => "remote.ank.fetch",
            _ => "remote.origin.fetch",
        }
    }

    /// What `init` prints for it, and what the repair says on stderr.
    pub fn line(&self) -> String {
        match self {
            ClaimsRefspec::AnkFetchesClaims => {
                format!("refspec added to remote.ank.fetch: {CLAIMS_REFSPEC}")
            }
            ClaimsRefspec::OriginSkipsClaims => {
                format!("refspec added to remote.origin.fetch: {NOT_CLAIMS}")
            }
            ClaimsRefspec::OriginFetchesClaims => format!(
                "refspec removed from remote.origin.fetch: {NOT_CLAIMS}, \
                 no remote named ank carries the claims"
            ),
        }
    }
}

/// Gives `ank` a fetch refspec for the claims and origin the negative one that
/// keeps them out of its fetch and its prune, or takes the negative one back
/// once `ank` is gone (ADR-b6fe84504ff2). Answers what it wrote.
///
/// With origin alone and no negative refspec, the state of every clone that
/// never had `ank`, it writes nothing and starts nothing.
fn arrange_claims(root: &Path, found: Remotes) -> Result<Vec<ClaimsRefspec>> {
    let mut done = Vec::new();
    if found.ank && !found.ank_fetches_claims {
        git::run(
            root,
            &["config", "--add", "remote.ank.fetch", CLAIMS_REFSPEC],
        )?;
        done.push(ClaimsRefspec::AnkFetchesClaims);
    }
    if found.origin_lets_claims_in() {
        git::run(
            root,
            &["config", "--add", "remote.origin.fetch", NOT_CLAIMS],
        )?;
        done.push(ClaimsRefspec::OriginSkipsClaims);
    }
    if !found.ank && found.origin_skips_claims {
        git::run(
            root,
            &[
                "config",
                "--fixed-value",
                "--unset-all",
                "remote.origin.fetch",
                NOT_CLAIMS,
            ],
        )?;
        done.push(ClaimsRefspec::OriginFetchesClaims);
    }
    Ok(done)
}

/// `found` once `done` was written, read from what was written rather than by
/// a second `git config`.
fn remotes_after(found: Remotes, done: &[ClaimsRefspec]) -> Remotes {
    let mut now = found;
    for d in done {
        match d {
            ClaimsRefspec::AnkFetchesClaims => now.ank_fetches_claims = true,
            ClaimsRefspec::OriginSkipsClaims => now.origin_skips_claims = true,
            ClaimsRefspec::OriginFetchesClaims => now.origin_skips_claims = false,
        }
    }
    now
}

/// The two remotes ank reads, as far as `refs/ank/*` is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Remotes {
    /// What `origin` fetches.
    pub origin: OriginFetch,
    /// Whether a remote named `ank` has a URL, and therefore carries
    /// `refs/ank/claims/*` (ADR-b6fe84504ff2).
    pub ank: bool,
    /// Whether `remote.ank.fetch` carries [`CLAIMS_REFSPEC`].
    pub ank_fetches_claims: bool,
    /// Whether `remote.origin.fetch` carries [`NOT_CLAIMS`].
    pub origin_skips_claims: bool,
}

impl Remotes {
    /// `ank` carries the claims and its fetch does not bring them.
    pub fn ank_lacks_claims(&self) -> bool {
        self.ank && !self.ank_fetches_claims
    }

    /// `ank` carries the claims and a fetch from origin would still write
    /// them, or prune them.
    pub fn origin_lets_claims_in(&self) -> bool {
        self.ank && self.origin != OriginFetch::NoOrigin && !self.origin_skips_claims
    }

    /// Nothing for [`arrange_claims`] to write.
    fn claims_settled(&self) -> bool {
        !self.ank_lacks_claims()
            && !self.origin_lets_claims_in()
            && (self.ank || !self.origin_skips_claims)
    }
}

/// What `origin` fetches, as far as `refs/ank/*` is concerned.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OriginFetch {
    /// No remote named `origin` has a URL: nothing to fetch from, nothing
    /// missing.
    NoOrigin,
    /// `origin` carries [`REFSPEC`].
    Fetches,
    /// `origin` has a URL and its fetch refspec does not carry [`REFSPEC`], so
    /// claims and completion refs are never fetched. The state `init` repairs
    /// and `status` and `check` report (TASK-623d80886c2f), and the state every
    /// fresh clone is in, because the refspec lives in `.git/config`.
    Lacks,
}

/// **One predicate, read by `init`, `status` and `check`**, so the verb that
/// repairs and the verbs that report can never disagree about what is missing.
///
/// One process: `--get-regexp` answers the URL and every fetch line together.
/// Exit 1 is "no such key", which is the empty answer, and a git that will not
/// run is taken as no origin -- this is a reader's hint, never a refusal.
pub fn origin_fetch(root: &Path) -> OriginFetch {
    remotes(root).origin
}

/// [`origin_fetch`] and whether `ank` has a URL, in the one process: the
/// pattern names `remote.ank.url` beside origin's two keys, so a clone with
/// origin alone starts what it always started.
pub fn remotes(root: &Path) -> Remotes {
    let said = git::output(
        root,
        &[
            "config",
            "--get-regexp",
            r"^remote\.(origin|ank)\.(url|fetch)$",
        ],
    )
    .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
    .unwrap_or_default();
    let mut url = false;
    let mut fetches = false;
    let mut ank = false;
    let mut ank_fetches_claims = false;
    let mut origin_skips_claims = false;
    for line in said.lines() {
        let (key, value) = line.split_once(' ').unwrap_or((line, ""));
        match key {
            "remote.origin.url" if !value.trim().is_empty() => url = true,
            "remote.origin.fetch" if value.trim() == REFSPEC => fetches = true,
            "remote.origin.fetch" if value.trim() == NOT_CLAIMS => origin_skips_claims = true,
            "remote.ank.url" if !value.trim().is_empty() => ank = true,
            "remote.ank.fetch" if value.trim() == CLAIMS_REFSPEC => ank_fetches_claims = true,
            _ => {}
        }
    }
    let origin = match (fetches, url) {
        (true, _) => OriginFetch::Fetches,
        (false, true) => OriginFetch::Lacks,
        (false, false) => OriginFetch::NoOrigin,
    };
    Remotes {
        origin,
        ank,
        ank_fetches_claims,
        origin_skips_claims,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(PathBuf);

    impl Temp {
        /// A real git repository: `init` refuses to write outside one, and that
        /// is exactly the behaviour we want to exercise.
        fn new_repo() -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ank-init-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&p).unwrap();
            let ok = Command::new("git")
                .current_dir(&p)
                .args(["init", "-q"])
                .status()
                .expect("git must be installed: it is a hard dependency")
                .success();
            assert!(ok, "git init failed in {}", p.display());
            let t = Temp(p);
            // Signing off at creation, not at each commit (TASK-40a972e98a9a),
            // and maintenance off because git is otherwise free to repack a
            // fixture between two reads of it (TASK-fc6bef21e268).
            for args in [
                ["config", "commit.gpgsign", "false"],
                ["config", "tag.gpgsign", "false"],
                ["config", "gc.auto", "0"],
                ["config", "maintenance.auto", "false"],
            ] {
                let st = Command::new("git")
                    .current_dir(&t.0)
                    .args(args)
                    .status()
                    .expect("git must be installed: it is a hard dependency");
                assert!(st.success(), "git {args:?}");
            }
            t
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn init_produces_all_six_effects() {
        let t = Temp::new_repo();
        let r = init_at(&t.0).unwrap();

        assert_eq!(r.created_dirs, vec![".ank/entities"]);
        assert!(t.0.join(".ank/entities").is_dir());
        // The directory of the previous layout, which this verb read for one
        // window and made for ten weeks after it stopped writing into it
        // (TASK-acaf2c3159dd).
        assert!(
            !t.0.join(".ank/log").exists(),
            "init made the previous layout's log directory"
        );

        assert!(r.wrote_config);
        let cfg = fs::read_to_string(t.0.join(".ank/config.yml")).unwrap();
        assert!(config::parse(&cfg, Path::new("config.yml")).is_ok());

        assert!(r.wrote_gitattributes);
        let ga = fs::read_to_string(t.0.join(".gitattributes")).unwrap();
        assert!(ga.contains(GITATTRIBUTES_LINE), "{ga}");

        assert!(r.wrote_gitignore);
        let gi = fs::read_to_string(t.0.join(".gitignore")).unwrap();
        assert!(gi.contains(GITIGNORE_LINE), "{gi}");

        assert!(r.wrote_agents_pointer);
        assert!(fs::read_to_string(t.0.join("AGENTS.md"))
            .unwrap()
            .contains(".ank/"));

        // Nothing about origin while origin has no URL, and nothing that
        // would make git read it as configured (TASK-0878e19675f4).
        assert!(!r.added_refspec);
        let out =
            git::run(&t.0, &["config", "--get-all", "remote.origin.fetch"]).unwrap_or_default();
        assert!(!out.contains(REFSPEC), "written before any remote: {out}");
        let config = fs::read_to_string(t.0.join(".git/config")).unwrap();
        assert!(!config.contains("includeIf"), "{config}");
        assert!(!t.0.join(".git/ank-origin.config").exists());

        // And once origin has a URL, the next init writes it outright.
        git::run(
            &t.0,
            &["config", "remote.origin.url", "https://example.invalid/r"],
        )
        .unwrap();
        assert!(init_at(&t.0).unwrap().added_refspec);
        let out = git::run(&t.0, &["config", "--get-all", "remote.origin.fetch"]).unwrap();
        assert!(out.contains(REFSPEC), "{out}");
    }

    #[test]
    fn init_is_idempotent() {
        let t = Temp::new_repo();
        init_at(&t.0).unwrap();
        let second = init_at(&t.0).unwrap();
        // Nothing is redone, and the refspec origin does not have yet is said
        // again rather than dropped (TASK-623d80886c2f).
        assert_eq!(
            second,
            Report {
                refspec_waits_for_origin: true,
                ..Report::default()
            },
            "nothing should be redone"
        );
        assert!(!second.changed());
        assert_eq!(
            second.lines_terse(),
            vec![
                "already initialised, nothing to do".to_string(),
                format!(
                    "refspec {REFSPEC} not added: no remote named origin yet, \
                     run this once it exists ({REFSPEC_REPAIR})"
                ),
            ]
        );

        // Once origin has a URL the refspec is written once, and a further
        // init adds no second copy.
        git::run(
            &t.0,
            &["config", "remote.origin.url", "https://example.invalid/r"],
        )
        .unwrap();
        assert!(init_at(&t.0).unwrap().added_refspec);
        let out = git::run(&t.0, &["config", "--get-all", "remote.origin.fetch"]).unwrap();
        assert_eq!(out.lines().filter(|l| l.trim() == REFSPEC).count(), 1);
        assert_eq!(init_at(&t.0).unwrap(), Report::default(), "with origin");
    }

    #[test]
    fn ensure_line_preserves_existing_content() {
        let t = Temp::new_repo();
        let f = t.0.join("AGENTS.md");
        fs::write(&f, "# Notes\n\nAlready here.\n").unwrap();
        let line = "One line of ours.";
        assert!(ensure_line(&f, line).unwrap());
        let s = fs::read_to_string(&f).unwrap();
        assert!(s.starts_with("# Notes\n\nAlready here.\n"), "{s:?}");
        assert!(s.contains(line));
        assert!(!ensure_line(&f, line).unwrap(), "second pass");
    }

    /// A `.gitignore` is a file the user curates, far more often than a
    /// `.gitattributes` is. Appending to it must leave every rule already
    /// there intact, and must not grow a second copy of ours on re-init.
    #[test]
    fn an_existing_gitignore_keeps_its_content() {
        let t = Temp::new_repo();
        let gi = t.0.join(".gitignore");
        fs::write(&gi, "/target\nnode_modules/\n").unwrap();

        init_at(&t.0).unwrap();
        let s = fs::read_to_string(&gi).unwrap();
        assert!(s.starts_with("/target\nnode_modules/\n"), "{s:?}");
        assert!(s.contains(GITIGNORE_LINE), "{s:?}");

        let second = init_at(&t.0).unwrap();
        assert!(!second.wrote_gitignore);
        let s = fs::read_to_string(&gi).unwrap();
        assert_eq!(s.lines().filter(|l| l.trim() == GITIGNORE_LINE).count(), 1);
    }

    /// A `.gitignore` carrying both the old literal and the glob, as a hand
    /// edit may leave it, comes out with the glob alone (TASK-574dee03ba56).
    #[test]
    fn a_legacy_ignore_line_beside_the_glob_is_dropped() {
        let t = Temp::new_repo();
        let gi = t.0.join(".gitignore");
        let both = format!("{LEGACY_GITIGNORE_LINE}\r\n/target\r\n{GITIGNORE_LINE}\r\n");
        fs::write(&gi, both).unwrap();
        assert!(ensure_gitignore(&gi).unwrap());
        let s = fs::read_to_string(&gi).unwrap();
        assert_eq!(s, format!("/target\r\n{GITIGNORE_LINE}\r\n"));
        assert!(!ensure_gitignore(&gi).unwrap(), "second pass");
    }

    /// The test the original bug was asking for: a repository initialised,
    /// committed, then cloned must read its entities back. Without the
    /// `.gitattributes`, the clone comes out in CRLF on Windows and nothing
    /// parses any more.
    #[test]
    fn an_initialised_then_cloned_repo_reads_its_entities_back() {
        let t = Temp::new_repo();
        init_at(&t.0).unwrap();

        // A canonical entity, written in LF the way the store does.
        let task = "---\nid: TASK-000000000001\ntype: task\ntitle: Example\n\
created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
blocked_by: []\nschema: 1\nversion: 1\n---\n\nBody.\n";
        fs::write(t.0.join(".ank/entities/TASK-000000000001.md"), task).unwrap();

        for args in [
            vec!["config", "user.email", "test@ank.local"],
            vec!["config", "user.name", "Test"],
        ] {
            git::run(&t.0, &args).unwrap();
        }
        let run = |args: &[&str]| {
            let st = Command::new("git")
                .current_dir(&t.0)
                .args(args)
                .status()
                .unwrap();
            assert!(st.success(), "git {args:?}");
        };
        run(&["add", "-A"]);
        run(&["commit", "-qm", "init"]);

        let clone = t.0.with_extension("clone");
        let st = Command::new("git")
            // `--config` and not two `git config` runs afterwards: the clone is
            // a repository this fixture builds too, and it is unmaintained from
            // the moment it exists rather than shortly after
            // (TASK-fc6bef21e268).
            .args([
                "clone",
                "-q",
                "-c",
                "gc.auto=0",
                "-c",
                "maintenance.auto=false",
            ])
            .arg(&t.0)
            .arg(&clone)
            .status()
            .unwrap();
        assert!(st.success(), "clone");

        let read_back = fs::read(clone.join(".ank/entities/TASK-000000000001.md")).unwrap();
        let _ = fs::remove_dir_all(&clone);

        assert!(
            !read_back.contains(&b'\r'),
            "the clone must come out in LF, otherwise nothing parses"
        );
        let text = String::from_utf8(read_back).unwrap();
        assert!(
            ank_core::parse_entity(&text).is_ok(),
            "the cloned entity must parse"
        );
    }

    /// Every git repository inside a fixture, found rather than listed.
    ///
    /// A directory holding a `HEAD` file and an `objects` directory is one,
    /// whether it is the `.git` beside a working tree or a bare corpus. Found,
    /// because a list would have to be maintained, and what is being guarded
    /// against is exactly a repository nobody remembered to enrol.
    fn repositories_under(root: &std::path::Path) -> Vec<std::path::PathBuf> {
        let mut found = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            if dir.join("HEAD").is_file() && dir.join("objects").is_dir() {
                found.push(dir);
                continue;
            }
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for e in entries.flatten() {
                if e.file_type().is_ok_and(|t| t.is_dir()) {
                    stack.push(e.path());
                }
            }
        }
        found
    }

    /// What git answers for one key of one repository's own configuration, `None`
    /// when the key is unset -- which is the state this asserts against, since an
    /// unset `maintenance.auto` means maintenance is on.
    ///
    /// `--local`, so what comes back is the fixture's answer and never the
    /// machine's: a contributor carrying `gc.auto` in a global configuration would
    /// otherwise read a pass out of a repository that sets nothing.
    fn config_of(git_dir: &std::path::Path, key: &str) -> Option<String> {
        let out = std::process::Command::new("git")
            .arg("--git-dir")
            .arg(git_dir)
            .args(["config", "--local", "--get", key])
            .output()
            .expect("git must be installed: it is a hard dependency");
        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    /// Asserts that every repository under `root` is one git will not maintain.
    ///
    /// Read back out of a freshly built fixture rather than grepped out of this
    /// file: what is under test is the configuration `git init` was actually
    /// followed by, and a grep passes on a comment and fails on a refactor.
    fn assert_unmaintained(root: &std::path::Path) {
        let repos = repositories_under(root);
        assert!(
            !repos.is_empty(),
            "no repository found under {}: this asserts nothing",
            root.display()
        );
        for git_dir in repos {
            let at = git_dir.display();
            assert_eq!(
                config_of(&git_dir, "gc.auto").as_deref(),
                Some("0"),
                "gc.auto at {at}"
            );
            assert_eq!(
                config_of(&git_dir, "maintenance.auto").as_deref(),
                Some("false"),
                "maintenance.auto at {at}"
            );
        }
    }

    /// A fixture repository is not maintained under the test.
    ///
    /// Measured on 2026-08-30 in run 33284185681: git repacked a fixture between
    /// two fingerprints of it -- `objects/maintenance.lock`, a `tmp_pack` and six
    /// loose objects in the first, a multi-pack-index, two packs and `info/refs`
    /// in the second -- and a test asserting that a read writes nothing failed on
    /// one platform of three. Ank had written nothing (TASK-fc6bef21e268).
    ///
    /// The repositories are found by walking the fixture and not named here, so a
    /// second one grown under it later is held to this without anyone remembering
    /// to enrol it.
    #[test]
    fn a_fixture_repository_is_not_maintained_under_the_test() {
        let t = Temp::new_repo();
        assert_unmaintained(&t.0);
    }
}
