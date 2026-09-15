//! `skills`: the skills this binary carries, and the route that installs them
//! (ADR-e1d750884b82, §4).
//!
//! **The skills arrive inside the executable.** `build.rs` reads every
//! `SKILL.md` under `skill/` whole and hands it to this module as bytes, so a
//! route that carries the binary carries them, and a machine behind a firewall
//! installs the skills of the build it holds rather than whatever a registry
//! serves that day.
//!
//! **`--install` hands a directory to the skills CLI and decides nothing about
//! where a skill goes.** It writes one subdirectory per skill under a directory
//! of its own and runs `npx skills add <that directory>`, which places them for
//! every agent it knows. The flag given is the consent, so nothing here asks,
//! and nothing here reads standard input: the child gets none either, so a
//! question the skills CLI would ask on a cold cache answers itself instead of
//! waiting on a terminal nobody is at. `npm_config_yes` is `npx --yes` spelled
//! as the environment, the same value the installers pass.

use crate::cli::{CliError, Invocation, Result};
use crate::index::Index;
use crate::json::Obj;
use crate::repo::Repo;
use crate::store::Store;
use ank_contract::ExitCode;
use ank_core::{Entity, EntityId, EntityKind};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// One skill as the build read it: what its frontmatter declares, and the file
/// whole.
pub struct Embedded {
    pub name: &'static str,
    pub description: &'static str,
    pub revision: &'static str,
    pub content: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/skills.rs"));

/// The one line a build with no `skill/` to read answers, whether or not
/// `--install` was given: there is nothing to list and nothing to hand over.
pub const NONE: &str =
    "this build carries no skills: there was no skill/ directory to read when it was built";

/// The names a task's `method` may hold: one per sibling skill this binary
/// carries, and never the contract (§3, ADR-a8f9c603a0e7).
///
/// **The name is the sibling's directory under `skill/`**, `tdd` for
/// `skill/tdd/SKILL.md`, and not the `ank-tdd` its frontmatter declares. It is
/// derived here from the frontmatter, which is what [`Embedded`] carries: every
/// sibling is named `ank-<directory>`, which the Agent Skills format and the
/// plugin manifest both hold it to, so stripping the prefix gives the directory
/// back without a second list to keep in step. The contract is `ank` with no
/// prefix to strip, and it is not a method: it is the skill every session
/// loads, and a recommendation that names it recommends nothing.
///
/// One spelling and not two. `ank-tdd` is refused rather than read as `tdd`,
/// because every reader that counts designations would otherwise have to
/// normalise the pair, and the refusal names the spelling to type.
pub fn methods() -> Vec<&'static str> {
    methods_of(EMBEDDED)
}

fn methods_of(skills: &'static [Embedded]) -> Vec<&'static str> {
    skills
        .iter()
        .filter_map(|s| s.name.strip_prefix(SIBLING))
        .collect()
}

/// The prefix every sibling's frontmatter name carries.
const SIBLING: &str = "ank-";

/// A `--method` value, checked against what the binary carries at the moment
/// the task is written, the way `--verify` is checked against `config.yml`: a
/// name misremembered fails here rather than in silence at the one moment the
/// recommendation was for.
pub fn method(raw: &str) -> Result<String> {
    method_among(raw.trim(), &methods())
}

fn method_among(name: &str, carried: &[&str]) -> Result<String> {
    if carried.contains(&name) {
        return Ok(name.to_string());
    }
    if carried.is_empty() {
        return Err(CliError::new(
            ExitCode::Prerequisite,
            format!("no sibling skill named '{name}': this build carries no skills"),
        )
        .with_hint("ank skills"));
    }
    let hint = match name.strip_prefix(SIBLING) {
        Some(short) if carried.contains(&short) => {
            format!("a method is the sibling's short name: --method {short}")
        }
        _ => format!("carried: {}", carried.join(" ")),
    };
    Err(CliError::new(
        ExitCode::Prerequisite,
        format!(
            "no sibling skill named '{name}' in this binary, which carries {}",
            carried.join(", ")
        ),
    )
    .with_hint(hint))
}

pub fn run(inv: &Invocation, cwd: &Path, out: &mut dyn Write) -> Result<ExitCode> {
    if inv.has("--install") {
        // **Refused rather than ignored** (§4, §9). Under `--json` stdout is a
        // document a parser reads and nothing else, and what `--install`
        // produces is a directory and npx's own run on that same stdout.
        if inv.json() {
            return Err(CliError::new(
                ExitCode::Generic,
                "--json: skills --install returns no document, npx writes to the same stdout",
            )
            .with_hint("ank skills --json"));
        }
        return run_over(EMBEDDED, true, out);
    }
    let rates = match corpus(inv, cwd)? {
        Some(repo) => Some(rates(&repo, &methods())?),
        None => None,
    };
    if inv.json() {
        let _ = writeln!(out, "{}", document(EMBEDDED, rates.as_deref()));
        return Ok(ExitCode::Ok);
    }
    run_over(EMBEDDED, false, out)?;
    if let Some(rates) = rates.filter(|r| !r.is_empty()) {
        let _ = write!(
            out,
            "\n{}\n{}",
            inv.style().header("METHODS"),
            report(&rates)
        );
    }
    Ok(ExitCode::Ok)
}

/// The corpus the counts are read from, or `None` where there is none to read.
///
/// **A verb run for the catalogue outside a corpus pays nothing for it**
/// (ADR-f3d1, ADR-a8f9c603a0e7). The installers run this verb in whatever
/// directory the person was in, so a walk that finds no `.ank/`, or a
/// declaration it cannot use, is the catalogue alone and never a refusal. An
/// address the caller wrote is different: `--repo` or `--worktree` naming
/// nothing is the refusal every verb gives it, since answering without the
/// counts would read as a corpus holding none.
fn corpus(inv: &Invocation, cwd: &Path) -> Result<Option<Repo>> {
    let addressed = inv.repo().is_some() || inv.worktree().is_some();
    let mut notes = Vec::new();
    match crate::repo::resolve(inv.repo(), inv.worktree(), cwd, &mut notes) {
        Ok(repo) => {
            if !inv.quiet() {
                let style = inv.style().on_stderr();
                for note in notes {
                    eprintln!("{} {note}", style.yellow("warning:"));
                }
            }
            Ok(Some(repo))
        }
        Err(e) if addressed => Err(e),
        Err(_) => Ok(None),
    }
}

/// One sibling's three counts (ADR-a8f9c603a0e7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rate {
    pub name: &'static str,
    /// The tasks whose `method` names it.
    pub designated: usize,
    /// Those of them carrying at least one of its `method` entries: a task is
    /// counted once however many times the sibling opened on it.
    pub fired: usize,
    /// Its `method` entries on tasks whose `method` names nothing. An entry on
    /// a task designating another sibling counts for neither.
    pub undesignated: usize,
}

/// The counts for every sibling in `names`, read from the corpus once.
///
/// **One pass over each plane, and no git** (ADR-cc659b2b7bd5). The tasks are
/// read whole, because `method` is a field the index does not hold; the entries
/// are narrowed on the index first, by a title naming a sibling, and only those
/// are opened to confirm that `records` says `method`. The title is the name and
/// nothing else, so the narrowing loses nothing: an entry whose title is not a
/// sibling's name is not an entry this report counts.
pub fn rates(repo: &Repo, names: &[&'static str]) -> Result<Vec<Rate>> {
    let store = Store::new(&repo.ank);
    let index = Index::open(&repo.ank)?;

    let mut tasks: HashSet<EntityId> = HashSet::new();
    let mut designated: HashMap<EntityId, String> = HashMap::new();
    for row in index.by_kind(EntityKind::Task)? {
        let Entity::Task(task) = store.load(&row.id)?.entity else {
            continue;
        };
        tasks.insert(task.id.clone());
        if let Some(method) = task.method {
            designated.insert(task.id, method);
        }
    }

    let mut fired: HashSet<(String, EntityId)> = HashSet::new();
    let mut undesignated: HashMap<String, usize> = HashMap::new();
    for row in index.by_kind(EntityKind::Log)? {
        if !names.contains(&row.title.as_str()) {
            continue;
        }
        let Entity::Log(entry) = store.load(&row.id)?.entity else {
            continue;
        };
        if entry.records.as_deref() != Some(ank_core::model::RECORDS_METHOD) {
            continue;
        }
        match designated.get(&entry.about) {
            Some(method) if *method == entry.title => {
                fired.insert((entry.title, entry.about));
            }
            Some(_) => {}
            None if tasks.contains(&entry.about) => {
                *undesignated.entry(entry.title).or_default() += 1;
            }
            None => {}
        }
    }

    Ok(names
        .iter()
        .map(|&name| Rate {
            name,
            designated: designated.values().filter(|m| *m == name).count(),
            fired: fired.iter().filter(|(m, _)| m == name).count(),
            undesignated: undesignated.get(name).copied().unwrap_or(0),
        })
        .collect())
}

/// One line per sibling, each count named beside its number, so a line read
/// alone says what it counts. Names padded to one width, numbers to theirs.
fn report(rates: &[Rate]) -> String {
    let name = rates.iter().map(|r| r.name.len()).max().unwrap_or(0);
    let digits = |count: fn(&Rate) -> usize| {
        rates
            .iter()
            .map(|r| count(r).to_string().len())
            .max()
            .unwrap_or(1)
    };
    let (d, f, u) = (
        digits(|r| r.designated),
        digits(|r| r.fired),
        digits(|r| r.undesignated),
    );
    rates
        .iter()
        .map(|r| {
            format!(
                "{:<name$}  designated {:>d$}  fired {:>f$}  undesignated {:>u$}\n",
                r.name, r.designated, r.fired, r.undesignated
            )
        })
        .collect()
}

/// `skills --json`: the catalogue, and the counts where a corpus answered.
fn document(skills: &[Embedded], rates: Option<&[Rate]>) -> String {
    let catalogue = skills.iter().map(|s| {
        Obj::new()
            .str("name", s.name)
            .str("revision", s.revision)
            .str("description", s.description)
            .finish()
    });
    let methods = rates.unwrap_or_default().iter().map(|r| {
        Obj::new()
            .str("name", r.name)
            .num("designated", r.designated)
            .num("fired", r.fired)
            .num("undesignated", r.undesignated)
            .finish()
    });
    Obj::document()
        .array("skills", catalogue)
        .bool("counted", rates.is_some())
        .array("methods", methods)
        .finish()
}

fn run_over(skills: &[Embedded], install: bool, out: &mut dyn Write) -> Result<ExitCode> {
    if skills.is_empty() {
        let _ = writeln!(out, "{NONE}");
        return Ok(ExitCode::Ok);
    }
    if !install {
        let _ = write!(out, "{}", listing(skills));
        return Ok(ExitCode::Ok);
    }
    let dir = write_all(skills)?;
    let shown = dir.display().to_string();
    let _ = writeln!(out, "wrote {} skills to {shown}", skills.len());

    let Some(npx) = npx_on_path() else {
        // Not a failure, and exit 0 is the decision (ADR-e1d750884b82): the
        // directory is written and stays, so the command printed here is the
        // whole of what is left, runnable later on a machine that has node.
        let _ = writeln!(
            out,
            "npx is not on PATH, so nothing was installed; with node installed, run:"
        );
        let _ = writeln!(out, "  npx skills add {shown}");
        return Ok(ExitCode::Ok);
    };

    let _ = writeln!(out, "running: npx skills add {shown}");
    // The child writes to the same stdout, so what this process has buffered
    // goes first or the two interleave out of order.
    let _ = out.flush();
    let status = Command::new(&npx)
        .args(["skills", "add"])
        .arg(&dir)
        .env("npm_config_yes", "1")
        .stdin(Stdio::null())
        .status()
        .map_err(|e| {
            CliError::new(
                ExitCode::Environment,
                format!("cannot run {}: {e}", npx.display()),
            )
            .with_hint(format!("npx skills add {shown}"))
        })?;
    match status.code() {
        Some(0) => Ok(ExitCode::Ok),
        // **npx failing is this verb failing, and its code is the answer.** The
        // codes of §4 are ank's own and none of them means what npx meant, so
        // the process exits with npx's integer rather than one of them
        // translated: a caller reads the same number it would have read running
        // npx itself, and npx's output above is the reason.
        Some(code) => {
            eprintln!(
                "npx skills add {shown} exited {code}; the skills stay written in that directory"
            );
            let _ = out.flush();
            std::process::exit(code);
        }
        None => Err(CliError::new(
            ExitCode::Generic,
            "npx skills add was stopped by a signal before it exited",
        )
        .with_hint(format!("npx skills add {shown}"))),
    }
}

/// One line per skill: the name, the revision it declares, the description.
/// The names are padded to one width so the revisions read as a column.
fn listing(skills: &[Embedded]) -> String {
    let width = skills.iter().map(|s| s.name.len()).max().unwrap_or(0);
    skills
        .iter()
        .map(|s| format!("{:<width$}  {}  {}\n", s.name, s.revision, s.description))
        .collect()
}

/// A new directory under the platform's temporary directory, holding
/// `<name>/SKILL.md` for every skill.
///
/// Named after the frontmatter, because that is what a harness calls a skill
/// and what the Agent Skills format requires the directory to match. Created
/// fresh rather than reused: a directory that already held files from another
/// build would hand npx a mixture of two.
fn write_all(skills: &[Embedded]) -> Result<PathBuf> {
    let base = std::env::temp_dir();
    let unwritable = |e: std::io::Error, path: &Path| {
        CliError::new(
            ExitCode::Environment,
            format!("cannot write the skills to {}: {e}", path.display()),
        )
        .with_hint("set TMPDIR (TEMP on Windows) to a directory this user can write")
    };
    let mut attempt = 0u32;
    let dir = loop {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let candidate = base.join(format!(
            "ank-skills-{}-{nanos:09}-{attempt}",
            std::process::id()
        ));
        match std::fs::create_dir(&candidate) {
            Ok(()) => break candidate,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists && attempt < 16 => {
                attempt += 1;
            }
            Err(e) => return Err(unwritable(e, &candidate)),
        }
    };
    for skill in skills {
        let sub = dir.join(skill.name);
        std::fs::create_dir(&sub).map_err(|e| unwritable(e, &sub))?;
        let file = sub.join("SKILL.md");
        std::fs::write(&file, skill.content).map_err(|e| unwritable(e, &file))?;
    }
    Ok(dir)
}

/// `npx`, found the way a shell would find it, or `None`.
fn npx_on_path() -> Option<PathBuf> {
    on_path("npx")
}

/// A program found the way a shell would find it, or `None`.
///
/// **By hand, because `Command::new("npx")` does not find it on Windows.** There
/// npx is `npx.cmd`, and the standard library's lookup appends `.exe` alone; the
/// extensionless `npx` beside it is a POSIX script Windows cannot run. So the
/// walk tries the extensions `PATHEXT` names, in its order, restricted to the
/// four a process can be started from. `update` finds `npm` and `powershell`
/// the same way, for the same reason.
pub fn on_path(program: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let names: Vec<String> = if cfg!(windows) {
        let pathext = std::env::var("PATHEXT").unwrap_or_else(|_| ".COM;.EXE;.BAT;.CMD".into());
        pathext
            .split(';')
            .map(str::to_ascii_lowercase)
            .filter(|ext| [".com", ".exe", ".bat", ".cmd"].contains(&ext.as_str()))
            .map(|ext| format!("{program}{ext}"))
            .collect()
    } else {
        vec![program.to_string()]
    };
    std::env::split_paths(&path)
        .flat_map(|dir| names.iter().map(move |name| dir.join(name)))
        .find(|candidate| runnable(candidate))
}

#[cfg(unix)]
fn runnable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn runnable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The build this suite runs from has a `skill/` to read, so the empty case
    /// is reached here, on the function the binary calls with what it carries.
    /// What the binary answers on a build that really had none was measured by
    /// building one, and is on the task's log.
    #[test]
    fn a_build_with_no_skills_answers_one_line() {
        for install in [false, true] {
            let mut out = Vec::new();
            let code = run_over(&[], install, &mut out).expect("nothing to refuse");
            assert_eq!(code, ExitCode::Ok);
            assert_eq!(String::from_utf8(out).unwrap(), format!("{NONE}\n"));
        }
    }

    /// What the binary carries, measured against the tree it was built from:
    /// one method per sibling directory under `skill/`, and the contract none.
    #[test]
    fn the_methods_are_the_sibling_directories_and_never_the_contract() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skill");
        let mut dirs: Vec<String> = std::fs::read_dir(&root)
            .unwrap()
            .map(|e| e.unwrap().path())
            .filter(|p| p.join("SKILL.md").is_file())
            .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        dirs.sort();
        assert!(!dirs.is_empty(), "no sibling under {}", root.display());
        assert_eq!(methods(), dirs);
        assert!(!methods().contains(&"ank"));
    }

    #[test]
    fn a_method_the_binary_does_not_carry_is_refused_naming_the_ones_it_does() {
        let carried = ["diagnose", "tdd"];
        assert_eq!(method_among("tdd", &carried).unwrap(), "tdd");
        for (name, hint) in [
            ("ank", "carried: diagnose tdd"),
            ("", "carried: diagnose tdd"),
            ("ank-tdd", "--method tdd"),
        ] {
            let err = method_among(name, &carried).unwrap_err();
            assert_eq!(err.code, ExitCode::Prerequisite, "{name}");
            assert!(err.message.contains("diagnose, tdd"), "{}", err.message);
            assert!(
                err.hint.as_deref().unwrap().contains(hint),
                "{name}: {:?}",
                err.hint
            );
        }
        let err = method_among("tdd", &[]).unwrap_err();
        assert!(err.message.contains("carries no skills"), "{}", err.message);
    }

    #[test]
    fn the_report_names_each_count_beside_its_number_in_columns() {
        let rates = [
            Rate {
                name: "diagnose",
                designated: 12,
                fired: 3,
                undesignated: 0,
            },
            Rate {
                name: "tdd",
                designated: 1,
                fired: 1,
                undesignated: 10,
            },
        ];
        assert_eq!(
            report(&rates),
            "diagnose  designated 12  fired 3  undesignated  0\n\
             tdd       designated  1  fired 1  undesignated 10\n"
        );
    }

    #[test]
    fn the_document_says_whether_it_counted() {
        let skills = [Embedded {
            name: "ank-tdd",
            description: "The loop.",
            revision: "000000000002",
            content: b"",
        }];
        let catalogue =
            r#""skills":[{"name":"ank-tdd","revision":"000000000002","description":"The loop."}]"#;
        let outside = document(&skills, None);
        assert!(
            outside.ends_with(&format!(r#",{catalogue},"counted":false,"methods":[]}}"#)),
            "{outside}"
        );
        let rates = [Rate {
            name: "tdd",
            designated: 2,
            fired: 1,
            undesignated: 0,
        }];
        let inside = document(&skills, Some(&rates));
        assert!(
            inside.ends_with(&format!(
                r#",{catalogue},"counted":true,"methods":[{{"name":"tdd","designated":2,"fired":1,"undesignated":0}}]}}"#
            )),
            "{inside}"
        );
    }

    #[test]
    fn the_listing_is_one_line_per_skill_with_the_revisions_in_a_column() {
        let skills = [
            Embedded {
                name: "ank",
                description: "The contract.",
                revision: "000000000001",
                content: b"",
            },
            Embedded {
                name: "ank-plan",
                description: "The plan.",
                revision: "000000000002",
                content: b"",
            },
        ];
        assert_eq!(
            listing(&skills),
            "ank       000000000001  The contract.\nank-plan  000000000002  The plan.\n"
        );
    }
}
