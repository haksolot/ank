//! `ankor init` (§9).
//!
//! Perimetre etroit et fixe : creer `.ankor/`, ecrire `config.yml`, ajouter
//! le refspec `refs/ankor/*`, poser un pointeur dans `AGENTS.md`, et ecrire
//! un `.gitattributes`.
//!
//! Ce dernier point n'est pas cosmetique. Sur Windows `core.autocrlf=true`
//! est le defaut, et sans `.gitattributes` git reconvertit en CRLF a chaque
//! checkout ce que l'outil vient d'ecrire en LF — rendant tout clone frais
//! illisible. Le corriger seulement dans le repo d'Ankor l'aurait laisse
//! intact chez chaque utilisateur.

use crate::cli::{CliError, Invocation, Result};
use crate::{config, git, repo};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub const GITATTRIBUTES_LINE: &str = ".ankor/** text eol=lf";
pub const REFSPEC: &str = "+refs/ankor/*:refs/ankor/*";
const AGENTS_POINTER: &str =
    "Ce repo utilise Ankor : les taches et decisions vivent dans `.ankor/`.";

pub fn run(inv: &Invocation, cwd: &Path, out: &mut dyn Write) -> Result<i32> {
    let target = match inv.positionals.first() {
        Some(p) => PathBuf::from(p),
        None => cwd.to_path_buf(),
    };
    // git est une dependance dure (ADR-92b9cda9f6a9) : on echoue avant
    // d'ecrire quoi que ce soit plutot que de laisser un `.ankor/` a moitie
    // pose dans un repertoire qui n'est pas un repo.
    git::ensure_usable(&target)?;
    let report = init_at(&target)?;
    for line in report.lines_terse() {
        let _ = writeln!(out, "{line}");
    }
    Ok(0)
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub created_dirs: bool,
    pub wrote_config: bool,
    pub wrote_gitattributes: bool,
    pub wrote_agents_pointer: bool,
    pub added_refspec: bool,
}

impl Report {
    /// Sortie terse, type `git status` : une ligne par effet reel, rien
    /// pour ce qui etait deja en place.
    pub fn lines_terse(&self) -> Vec<String> {
        let mut v = Vec::new();
        if self.created_dirs {
            v.push("cree .ankor/tasks .ankor/adr".to_string());
        }
        if self.wrote_config {
            v.push("ecrit .ankor/config.yml".to_string());
        }
        if self.wrote_gitattributes {
            v.push("ecrit .gitattributes".to_string());
        }
        if self.wrote_agents_pointer {
            v.push("pointeur ajoute dans AGENTS.md".to_string());
        }
        if self.added_refspec {
            v.push(format!("refspec ajoute : {REFSPEC}"));
        }
        if v.is_empty() {
            v.push("deja initialise, rien a faire".to_string());
        }
        v
    }
}

fn io(path: &Path, e: std::io::Error) -> CliError {
    CliError::new(1, format!("{} : {e}", path.display()))
}

/// Idempotent : reinitialiser un repo deja initialise ne doit rien casser
/// ni rien dupliquer. C'est ce qui permet de le lancer sans reflechir.
pub fn init_at(root: &Path) -> Result<Report> {
    let mut report = Report::default();
    let ankor = root.join(repo::ANKOR_DIR);

    for sub in ["tasks", "adr"] {
        let dir = ankor.join(sub);
        if !dir.is_dir() {
            fs::create_dir_all(&dir).map_err(|e| io(&dir, e))?;
            report.created_dirs = true;
        }
    }

    let cfg = ankor.join("config.yml");
    if !cfg.exists() {
        fs::write(&cfg, config::default_yaml()).map_err(|e| io(&cfg, e))?;
        report.wrote_config = true;
    }

    let ga = root.join(".gitattributes");
    report.wrote_gitattributes = ensure_line(&ga, GITATTRIBUTES_LINE)?;

    let agents = root.join("AGENTS.md");
    report.wrote_agents_pointer = ensure_line(&agents, AGENTS_POINTER)?;

    report.added_refspec = ensure_refspec(root)?;

    Ok(report)
}

/// Ajoute une ligne a un fichier si elle n'y est pas deja. Rend `true` si le
/// fichier a ete touche.
fn ensure_line(path: &Path, line: &str) -> Result<bool> {
    let existing = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(io(path, e)),
    };
    if existing.lines().any(|l| l.trim() == line) {
        return Ok(false);
    }
    let mut next = existing;
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    if !next.is_empty() {
        next.push('\n');
    }
    next.push_str(line);
    next.push('\n');
    fs::write(path, next).map_err(|e| io(path, e))?;
    Ok(true)
}

/// Les hebergeurs ne rapatrient pas les refs non standard d'eux-memes (§7).
/// Le refspec est pose meme sans remote configure : la cle existera quand
/// `origin` sera ajoute, ce qui evite un piege silencieux au premier push.
fn ensure_refspec(root: &Path) -> Result<bool> {
    let existing =
        git::run(root, &["config", "--get-all", "remote.origin.fetch"]).unwrap_or_default();
    if existing.lines().any(|l| l.trim() == REFSPEC) {
        return Ok(false);
    }
    git::run(root, &["config", "--add", "remote.origin.fetch", REFSPEC])?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Temp(PathBuf);

    impl Temp {
        /// Un vrai repo git : `init` refuse d'ecrire hors repo, et c'est
        /// exactement le comportement qu'on veut exercer.
        fn new_repo() -> Temp {
            static SEQ: AtomicU64 = AtomicU64::new(0);
            let p = std::env::temp_dir().join(format!(
                "ankor-init-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&p).unwrap();
            let ok = Command::new("git")
                .current_dir(&p)
                .args(["init", "-q"])
                .status()
                .expect("git doit etre installe : c'est une dependance dure")
                .success();
            assert!(ok, "git init a echoue dans {}", p.display());
            Temp(p)
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn init_pose_les_cinq_effets() {
        let t = Temp::new_repo();
        let r = init_at(&t.0).unwrap();

        assert!(r.created_dirs);
        assert!(t.0.join(".ankor/tasks").is_dir());
        assert!(t.0.join(".ankor/adr").is_dir());

        assert!(r.wrote_config);
        let cfg = fs::read_to_string(t.0.join(".ankor/config.yml")).unwrap();
        assert!(config::parse(&cfg, Path::new("config.yml")).is_ok());

        assert!(r.wrote_gitattributes);
        let ga = fs::read_to_string(t.0.join(".gitattributes")).unwrap();
        assert!(ga.contains(GITATTRIBUTES_LINE), "{ga}");

        assert!(r.wrote_agents_pointer);
        assert!(fs::read_to_string(t.0.join("AGENTS.md"))
            .unwrap()
            .contains(".ankor/"));

        assert!(r.added_refspec);
        let out = git::run(&t.0, &["config", "--get-all", "remote.origin.fetch"]).unwrap();
        assert!(out.contains(REFSPEC), "{out}");
    }

    #[test]
    fn init_est_idempotent() {
        let t = Temp::new_repo();
        init_at(&t.0).unwrap();
        let second = init_at(&t.0).unwrap();
        assert_eq!(second, Report::default(), "rien ne doit etre refait");
        assert_eq!(second.lines_terse(), vec!["deja initialise, rien a faire"]);

        // Le refspec n'est pas duplique.
        let out = git::run(&t.0, &["config", "--get-all", "remote.origin.fetch"]).unwrap();
        assert_eq!(out.lines().filter(|l| l.trim() == REFSPEC).count(), 1);
    }

    #[test]
    fn ensure_line_preserve_le_contenu_existant() {
        let t = Temp::new_repo();
        let f = t.0.join("AGENTS.md");
        fs::write(&f, "# Notes\n\nDeja la.\n").unwrap();
        assert!(ensure_line(&f, AGENTS_POINTER).unwrap());
        let s = fs::read_to_string(&f).unwrap();
        assert!(s.starts_with("# Notes\n\nDeja la.\n"), "{s:?}");
        assert!(s.contains(AGENTS_POINTER));
        assert!(!ensure_line(&f, AGENTS_POINTER).unwrap(), "deuxieme passe");
    }

    /// Le test que le bug d'origine appelait : un repo initialise, commite,
    /// puis clone, doit relire ses entites. Sans le `.gitattributes`, le
    /// clone ressort en CRLF sur Windows et plus rien ne parse.
    #[test]
    fn un_repo_initialise_puis_clone_relit_ses_entites() {
        let t = Temp::new_repo();
        init_at(&t.0).unwrap();

        // Une entite canonique, ecrite en LF comme le fait le store.
        let tache = "---\nid: TASK-000000000001\ntype: task\ntitle: Exemple\n\
created: 2026-07-28T00:00:00Z\nstatus: open\nscope:\n  - src/**\n\
blocked_by: []\nschema: 1\nversion: 1\n---\n\nCorps.\n";
        fs::write(t.0.join(".ankor/tasks/TASK-000000000001.md"), tache).unwrap();

        for args in [
            vec!["config", "user.email", "test@ankor.local"],
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
        run(&["-c", "commit.gpgsign=false", "commit", "-qm", "init"]);

        let clone = t.0.with_extension("clone");
        let st = Command::new("git")
            .args(["clone", "-q"])
            .arg(&t.0)
            .arg(&clone)
            .status()
            .unwrap();
        assert!(st.success(), "clone");

        let relu = fs::read(clone.join(".ankor/tasks/TASK-000000000001.md")).unwrap();
        let _ = fs::remove_dir_all(&clone);

        assert!(
            !relu.contains(&b'\r'),
            "le clone doit ressortir en LF, sinon plus rien ne parse"
        );
        let texte = String::from_utf8(relu).unwrap();
        assert!(
            ankor_core::parse_entity(&texte).is_ok(),
            "l'entite clonee doit parser"
        );
    }
}
