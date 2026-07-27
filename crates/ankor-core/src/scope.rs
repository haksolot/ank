use crate::error::{Error, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};

/// Valide les globs d'un scope sans construire de matcher.
pub fn validate_globs(globs: &[String]) -> Result<()> {
    for g in globs {
        Glob::new(g).map_err(|_| Error::InvalidGlob(g.clone()))?;
    }
    Ok(())
}

/// Ensemble de globs compile. C'est le seul mecanisme de rattachement
/// entre entites et code : verifiable, contrairement a une etiquette.
pub struct ScopeSet {
    set: GlobSet,
}

impl ScopeSet {
    pub fn new(globs: &[String]) -> Result<Self> {
        let mut b = GlobSetBuilder::new();
        for g in globs {
            b.add(Glob::new(g).map_err(|_| Error::InvalidGlob(g.clone()))?);
        }
        let set = b.build().map_err(|e| Error::InvalidGlob(e.to_string()))?;
        Ok(ScopeSet { set })
    }

    /// Un chemin matche s'il matche un glob, ou s'il est sous un prefixe
    /// repertoire d'un glob (`src/auth/**` matche le repertoire `src/auth/`).
    pub fn matches(&self, path: &str) -> bool {
        self.set.is_match(path.trim_end_matches('/'))
    }

    /// Intersection approximative chemin/scope pour `context <path>` :
    /// vrai si le chemin demande matche, ou si un fichier sous ce chemin
    /// pourrait matcher (le glob commence par le prefixe demande).
    pub fn overlaps_dir(&self, dir: &str, globs: &[String]) -> bool {
        let d = dir.trim_end_matches('/');
        self.matches(d) || globs.iter().any(|g| g.starts_with(&format!("{d}/")))
    }
}
