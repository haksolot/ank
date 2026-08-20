use crate::error::{Error, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};

/// A perimeter path in the one form globs are written in: `/`-separated,
/// relative to the repository root, no `.` and no `..` left in it.
///
/// This exists because the argument used to reach glob matching verbatim, and a
/// directory then had as many meanings as ways of typing it (TASK-df4c39031583).
/// Measured on the real corpus against `docs/`, which five ADRs bind: `docs` and
/// `docs/` answered five, `docs\` answered **four**, `./docs` and `.\docs\`
/// answered zero. The zeros are survivable because they are obvious. The four is
/// not: it is what Windows tab-completion produces, it looks like a correct
/// answer, and it silently drops a constraint that binds. A backslash survived
/// into the match, where a `**` glob still matched it as an ordinary character
/// and a glob naming a segment did not.
///
/// `None` means the path names nothing inside the repository — it is absolute,
/// or it climbs above the root — and the caller refuses rather than answering
/// about a perimeter it had to invent. An empty result is the repository root
/// itself, which is the whole corpus and what `.` means.
///
/// Case is left alone, deliberately. `DOCS` matches nothing here and matches
/// nothing on Linux; folding it on Windows would give one corpus two meanings
/// depending on the machine reading it, which is worse than the surprise it
/// removes.
pub fn normalize_path(path: &str) -> Option<String> {
    let unified = path.replace('\\', "/");
    // An absolute path names a place on a machine, not a place in the corpus,
    // and `C:/...` is absolute just as `/...` is.
    let drive_letter = {
        let b = unified.as_bytes();
        b.len() >= 2 && b[0].is_ascii_alphabetic() && b[1] == b':'
    };
    if unified.starts_with('/') || drive_letter {
        return None;
    }
    let mut out: Vec<&str> = Vec::new();
    for segment in unified.split('/') {
        match segment {
            // A repeated or trailing separator, and `.`, name the same place.
            "" | "." => {}
            // Resolved lexically rather than refused: `docs/../docs` is `docs`,
            // and answering about a different perimeter was the defect. Popping
            // nothing means climbing out of the repository, which is the one
            // case with no answer to give.
            ".." => {
                out.pop()?;
            }
            s => out.push(s),
        }
    }
    Some(out.join("/"))
}

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

/// Which of `globs` match at least one of `paths`, decided in one pass.
///
/// **One compiled set and one walk of the files, where it was one set per glob
/// and one walk each** (TASK-097883a2c09f). Confronting every glob with every
/// path is the one phase of `check` that grows faster than the corpus: this
/// repository carries 462 scope entries against about 1100 tracked files, and
/// both halves grow together, so a corpus twice the size costs four times as
/// much. A `GlobSet` answers which globs a path matched, so the files are
/// walked once and every glob learns its answer from that walk.
///
/// The two returned lists are aligned with `globs` by index. A glob that does
/// not compile is marked and never silently treated as matching nothing, which
/// would report a typo and a broken pattern the same way: the caller reports
/// them differently and needs to tell them apart.
pub fn live_globs(globs: &[String], paths: &[String]) -> (Vec<bool>, Vec<bool>) {
    let mut alive = vec![false; globs.len()];
    let mut invalid = vec![false; globs.len()];
    let mut b = GlobSetBuilder::new();
    // The index a glob has in the compiled set is not its index here, because
    // the ones that do not compile are absent from it. Kept explicitly rather
    // than derived, since deriving it is exactly the off-by-one that would
    // report one entity's answer against another entity's glob.
    let mut compiled: Vec<usize> = Vec::with_capacity(globs.len());
    for (i, g) in globs.iter().enumerate() {
        match Glob::new(g) {
            Ok(glob) => {
                b.add(glob);
                compiled.push(i);
            }
            Err(_) => invalid[i] = true,
        }
    }
    let Ok(set) = b.build() else {
        // A set that will not build says nothing about any single glob, and
        // answering "nothing is alive" would turn every scope in the corpus
        // dead at once. The caller falls back to asking one glob at a time.
        return (alive, invalid);
    };
    for path in paths {
        for hit in set.matches(path.trim_end_matches('/')) {
            if let Some(&at) = compiled.get(hit) {
                alive[at] = true;
            }
        }
    }
    (alive, invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One pass answers what one set per glob answered, including for a glob
    /// that does not compile (TASK-097883a2c09f).
    ///
    /// The invalid one sits **between** two valid globs on purpose: it is
    /// absent from the compiled set, so every index after it shifts, and a
    /// reader that derived the alignment instead of keeping it would credit
    /// `docs/**`'s answer to `src/**`.
    #[test]
    fn one_pass_answers_every_glob_and_keeps_them_aligned() {
        let globs = vec![
            "src/**".to_string(),
            "src/[".to_string(),
            "docs/**".to_string(),
            "nothing/**".to_string(),
        ];
        let paths = vec!["docs/guide.md".to_string(), "README.md".to_string()];
        let (alive, invalid) = live_globs(&globs, &paths);
        assert_eq!(alive, vec![false, false, true, false], "{alive:?}");
        assert_eq!(invalid, vec![false, true, false, false], "{invalid:?}");
    }

    /// Every glob agrees with what `ScopeSet` answers for it alone, which is
    /// the property the one-pass reading has to preserve.
    #[test]
    fn one_pass_agrees_with_one_set_per_glob() {
        let globs: Vec<String> = ["src/**", "docs/*.md", "README.md", "a/b/**/*.rs"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let paths: Vec<String> = ["src/main.rs", "docs/guide.md", "a/b/c/d.rs", "other.txt"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let (alive, _) = live_globs(&globs, &paths);
        for (i, g) in globs.iter().enumerate() {
            let one = ScopeSet::new(std::slice::from_ref(g)).unwrap();
            let expected = paths.iter().any(|p| one.matches(p));
            assert_eq!(alive[i], expected, "{g}");
        }
    }

    /// The forms a shell actually produces, and the one answer they all name.
    /// `docs\` is the one that mattered: it used to survive into the match and
    /// come back with a partial set rather than an empty one.
    #[test]
    fn every_way_of_writing_a_directory_normalises_to_one() {
        for raw in [
            "docs",
            "docs/",
            "docs\\",
            "./docs",
            ".\\docs\\",
            "docs/../docs",
            "docs//",
            "./docs/./",
        ] {
            assert_eq!(
                normalize_path(raw).as_deref(),
                Some("docs"),
                "{raw} must name the same directory as `docs`"
            );
        }
    }

    /// The root is the whole corpus, and `.` is how it is typed.
    #[test]
    fn the_repository_root_normalises_to_nothing() {
        for raw in [".", "./", ".\\", "", "docs/.."] {
            assert_eq!(normalize_path(raw).as_deref(), Some(""), "{raw}");
        }
    }

    /// No answer rather than a wrong one. A path leaving the repository names a
    /// place on a machine, and the caller refuses instead of inventing a
    /// perimeter.
    #[test]
    fn a_path_outside_the_repository_has_no_normal_form() {
        for raw in [
            "/etc/passwd",
            "C:/Windows",
            "c:\\Windows",
            "..",
            "../sibling",
            "docs/../../elsewhere",
        ] {
            assert_eq!(normalize_path(raw), None, "{raw} is not in the repository");
        }
    }

    /// Case is left alone on purpose: folding it on Windows alone would give one
    /// corpus two meanings depending on the machine reading it.
    #[test]
    fn case_is_not_folded() {
        assert_eq!(normalize_path("DOCS").as_deref(), Some("DOCS"));
    }

    /// The normal form is what the matcher was always meant to receive.
    #[test]
    fn the_normal_form_is_what_the_matcher_matches() {
        let globs = vec!["docs/**".to_string()];
        let set = ScopeSet::new(&globs).unwrap();
        assert!(set.overlaps_dir("docs", &globs));
        // The raw Windows form, which is what used to reach here.
        assert!(!set.overlaps_dir(".\\docs\\", &globs));
        let normal = normalize_path(".\\docs\\").unwrap();
        assert!(set.overlaps_dir(&normal, &globs));
    }
}
