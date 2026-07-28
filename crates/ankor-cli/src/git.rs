//! Runner git minimal (§12).
//!
//! Ankor appelle le binaire git, jamais une bibliotheque : `accept` et
//! `check` reposent sur la signature, et `git commit -S` / `git verify-commit`
//! sont trois lignes la ou une reimplementation cryptographique serait un
//! chantier. La contrepartie est la discipline imposee par ADR-92b9cda9f6a9 :
//! **plomberie uniquement**, jamais de porcelaine, dont la sortie n'offre
//! aucun contrat de stabilite entre versions.
//!
//! Un environnement git defaillant n'est pas un echec de la tache de
//! l'agent : absence, version trop ancienne et repertoire hors repo sortent
//! tous en code 9, avec la commande exacte a executer.

use crate::cli::CliError;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Plancher impose par la signature SSH et `gpg.ssh.allowedSignersFile`.
pub const MIN_VERSION: (u32, u32) = (2, 34);

const INSTALL_URL: &str = "https://git-scm.com/downloads";

/// Sous-commandes de plomberie autorisees. La liste est fermee pour que
/// l'ajout d'une porcelaine soit un acte visible en revue, pas un oubli.
const PLUMBING: &[&str] = &[
    "update-ref",
    "rev-parse",
    "verify-commit",
    "hash-object",
    "cat-file",
    "for-each-ref",
    "config",
    "--version",
];

pub type Result<T> = std::result::Result<T, CliError>;

fn env_missing() -> CliError {
    CliError::new(9, "git introuvable dans le PATH").with_hint(INSTALL_URL)
}

/// Execute git dans `cwd`. Rend la sortie standard, ecourtee des blancs de
/// fin. Un code de sortie non nul rend l'erreur avec la sortie d'erreur.
pub fn run(cwd: &Path, args: &[&str]) -> Result<String> {
    debug_assert!(
        args.first().map(|a| PLUMBING.contains(a)).unwrap_or(false),
        "porcelaine interdite (ADR-92b9cda9f6a9) : {args:?}"
    );
    let out = Command::new("git")
        .current_dir(cwd)
        .args(args)
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                env_missing()
            } else {
                CliError::new(9, format!("git {} : {e}", args.join(" ")))
            }
        })?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
        return Err(CliError::new(
            9,
            format!("git {} a echoue : {stderr}", args.join(" ")),
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}

/// Version installee, sous forme `(majeur, mineur)`.
pub fn version() -> Result<(u32, u32)> {
    let out = Command::new("git").arg("--version").output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            env_missing()
        } else {
            CliError::new(9, format!("git --version : {e}"))
        }
    })?;
    let text = String::from_utf8_lossy(&out.stdout);
    parse_version(&text).ok_or_else(|| {
        CliError::new(9, format!("version de git illisible : {}", text.trim()))
            .with_hint(INSTALL_URL)
    })
}

/// `git version 2.43.0.windows.1` -> `(2, 43)`. Tolere les suffixes de
/// distribution, qui varient entre plateformes.
pub fn parse_version(text: &str) -> Option<(u32, u32)> {
    let rest = text.split_whitespace().find(|w| {
        w.chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && w.contains('.')
    })?;
    let mut parts = rest.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

pub fn check_version(found: (u32, u32)) -> Result<()> {
    if found < MIN_VERSION {
        return Err(CliError::new(
            9,
            format!(
                "git {}.{} trop ancien, {}.{} requis pour la signature SSH",
                found.0, found.1, MIN_VERSION.0, MIN_VERSION.1
            ),
        )
        .with_hint(INSTALL_URL));
    }
    Ok(())
}

/// Racine du repo git contenant `cwd`.
pub fn toplevel(cwd: &Path) -> Result<PathBuf> {
    let out = Command::new("git")
        .current_dir(cwd)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                env_missing()
            } else {
                CliError::new(9, format!("git rev-parse : {e}"))
            }
        })?;
    if !out.status.success() {
        return Err(
            CliError::new(9, format!("{} n'est pas dans un repo git", cwd.display()))
                .with_hint("git init"),
        );
    }
    let path = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok(PathBuf::from(path))
}

/// Verifie l'environnement git complet : presence, version, et repo.
pub fn ensure_usable(cwd: &Path) -> Result<PathBuf> {
    check_version(version()?)?;
    toplevel(cwd)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_parsee_avec_ses_suffixes_de_distribution() {
        assert_eq!(parse_version("git version 2.43.0"), Some((2, 43)));
        assert_eq!(parse_version("git version 2.43.0.windows.1"), Some((2, 43)));
        assert_eq!(
            parse_version("git version 2.34.1 (Apple Git-141)"),
            Some((2, 34))
        );
        assert_eq!(parse_version("git version 3.0.0"), Some((3, 0)));
        assert_eq!(parse_version("pas une version"), None);
    }

    #[test]
    fn plancher_de_version_refuse_en_9_avec_le_lien() {
        let err = check_version((2, 33)).unwrap_err();
        assert_eq!(err.code, 9);
        assert!(err.message.contains("2.33"), "{}", err.message);
        assert!(err.message.contains("2.34"), "{}", err.message);
        assert_eq!(err.hint.as_deref(), Some(INSTALL_URL));

        assert!(check_version((2, 34)).is_ok(), "le plancher est inclusif");
        assert!(check_version((2, 45)).is_ok());
    }

    #[test]
    fn hors_repo_git_sort_en_9_avec_git_init() {
        let dir = std::env::temp_dir().join(format!("ankor-nogit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        // Un repertoire temporaire peut se trouver sous un repo git ; on ne
        // teste que si ce n'est pas le cas, sans quoi l'assertion est fausse.
        if toplevel(&dir).is_err() {
            let err = toplevel(&dir).unwrap_err();
            assert_eq!(err.code, 9);
            assert_eq!(err.hint.as_deref(), Some("git init"));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
