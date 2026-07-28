//! Decouverte du repo (§6).
//!
//! Un agent est lance dans un sous-repertoire quelconque de l'arbre : la
//! resolution remonte jusqu'au premier `.ankor/`, comme git le fait pour
//! `.git`. `--repo <path>` court-circuite la remontee, sans jamais la
//! contredire — le chemin donne doit contenir un `.ankor/`.

use crate::cli::CliError;
use std::path::{Path, PathBuf};

pub const ANKOR_DIR: &str = ".ankor";

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repo {
    /// Repertoire contenant `.ankor/`.
    pub root: PathBuf,
    /// Le `.ankor/` lui-meme.
    pub ankor: PathBuf,
}

impl Repo {
    fn at(root: PathBuf) -> Repo {
        let ankor = root.join(ANKOR_DIR);
        Repo { root, ankor }
    }

    pub fn config_path(&self) -> PathBuf {
        self.ankor.join("config.yml")
    }
}

fn missing(where_: &Path) -> CliError {
    CliError::new(
        1,
        format!("aucun {ANKOR_DIR}/ trouve depuis {}", where_.display()),
    )
    .with_hint("ankor init")
}

/// Remonte depuis `start` jusqu'au premier repertoire contenant `.ankor/`.
pub fn discover(start: &Path) -> Result<Repo> {
    let mut cur = Some(start);
    while let Some(dir) = cur {
        if dir.join(ANKOR_DIR).is_dir() {
            return Ok(Repo::at(dir.to_path_buf()));
        }
        cur = dir.parent();
    }
    Err(missing(start))
}

/// Resolution explicite par `--repo`. Accepte le repertoire qui contient
/// `.ankor/`, ou le `.ankor/` lui-meme — se tromper d'un cran est l'erreur
/// la plus probable, et la refuser n'apporterait rien.
pub fn at(path: &Path) -> Result<Repo> {
    if path.join(ANKOR_DIR).is_dir() {
        return Ok(Repo::at(path.to_path_buf()));
    }
    if path.file_name().and_then(|n| n.to_str()) == Some(ANKOR_DIR) && path.is_dir() {
        if let Some(parent) = path.parent() {
            return Ok(Repo::at(parent.to_path_buf()));
        }
    }
    Err(missing(path))
}

/// Resolution complete : `--repo` s'il est donne, sinon remontee depuis le
/// repertoire courant.
pub fn resolve(repo_flag: Option<&str>, cwd: &Path) -> Result<Repo> {
    match repo_flag {
        Some(p) => at(Path::new(p)),
        None => discover(cwd),
    }
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
                "ankor-repo-{}-{}",
                std::process::id(),
                SEQ.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(p.join(ANKOR_DIR).join("tasks")).unwrap();
            Temp(p)
        }
    }

    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn remonte_depuis_un_sous_repertoire_profond() {
        let t = Temp::new();
        let profond = t.0.join("crates").join("ankor-cli").join("src");
        fs::create_dir_all(&profond).unwrap();

        let repo = discover(&profond).unwrap();
        assert_eq!(repo.root, t.0);
        assert_eq!(repo.ankor, t.0.join(ANKOR_DIR));
    }

    #[test]
    fn absence_de_ankor_sort_avec_ankor_init() {
        let vide = std::env::temp_dir().join(format!("ankor-vide-{}", std::process::id()));
        fs::create_dir_all(&vide).unwrap();
        // La remontee peut atteindre un vrai repo Ankor selon l'endroit ou
        // vit le temp ; on n'assert que si la resolution echoue bien.
        if let Err(err) = discover(&vide) {
            assert_eq!(err.code, 1);
            assert_eq!(err.hint.as_deref(), Some("ankor init"));
        }
        let _ = fs::remove_dir_all(&vide);
    }

    #[test]
    fn repo_explicite_accepte_la_racine_et_le_ankor() {
        let t = Temp::new();
        assert_eq!(at(&t.0).unwrap().root, t.0);
        assert_eq!(at(&t.0.join(ANKOR_DIR)).unwrap().root, t.0);
    }

    #[test]
    fn le_flag_repo_court_circuite_la_remontee() {
        let t = Temp::new();
        let ailleurs = std::env::temp_dir();
        let via_flag = resolve(Some(t.0.to_str().unwrap()), &ailleurs).unwrap();
        assert_eq!(via_flag.root, t.0);
    }
}
