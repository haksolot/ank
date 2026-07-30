use crate::error::{Error, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};

/// Validates a scope's globs without building a matcher.
pub fn validate_globs(globs: &[String]) -> Result<()> {
    for g in globs {
        Glob::new(g).map_err(|_| Error::InvalidGlob(g.clone()))?;
    }
    Ok(())
}

/// A compiled set of globs. This is the only mechanism attaching entities to
/// code: verifiable, unlike a label.
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

    /// A path matches if it matches a glob, or if it sits under a directory
    /// prefix of a glob (`src/auth/**` matches the directory `src/auth/`).
    pub fn matches(&self, path: &str) -> bool {
        self.set.is_match(path.trim_end_matches('/'))
    }

    /// Approximate path/scope intersection for `context <path>`: true if the
    /// requested path matches, or if a file under that path could match (the
    /// glob starts with the requested prefix).
    pub fn overlaps_dir(&self, dir: &str, globs: &[String]) -> bool {
        let d = dir.trim_end_matches('/');
        self.matches(d) || globs.iter().any(|g| g.starts_with(&format!("{d}/")))
    }
}
