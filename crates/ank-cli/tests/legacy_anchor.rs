//! The `ratify <id>` commit an `accept` made before ADR-5e47007a75be, made
//! again by hand over what `accept` writes now.
//!
//! `accept` commits nothing since that decision: the ratification is a tag
//! object at `refs/ank/ratify/<id>`. The readers find it there first, and by
//! the commit walk only for an anchor made before the decision, which stays
//! the fallback for every such anchor (TASK-6e3f8f0049e9). So a suite that
//! tests what a reader makes of a commit anchor ratifies with `accept` and
//! then makes the corpus the old `accept` would have left: the commit, whose
//! subject and lines are the tag's message unchanged, no ref, and a file
//! without the `anchor:` line, which no file carried before the decision.

/// Turns the ratification `accept` just wrote for `id` into a commit anchor:
/// commits `.ank/` under the tag's message, signed with `-S` when `sign`
/// holds, after deleting the ref and the file's `anchor:` line. `git` runs
/// one git command in the suite's own environment and returns its standard
/// output.
pub fn commit(git: &dyn Fn(&[&str]) -> String, id: &str, sign: bool) {
    let name = format!("refs/ank/ratify/{id}");
    let tag = git(&["cat-file", "tag", &name]);
    let message = tag
        .split_once("\n\n")
        .map(|(_, m)| m)
        .expect("a tag object has a message")
        .split("-----BEGIN")
        .next()
        .unwrap()
        .to_string();
    // Gone from the origin as well, where there is one: a verb that runs git
    // fetches `refs/ank/*` (ADR-1b03), and would bring the tag back.
    if git(&["remote"]).lines().any(|r| r.trim() == "origin") {
        git(&["push", "-q", "origin", &format!(":{name}")]);
    }
    git(&["update-ref", "-d", &name]);
    let top = std::path::PathBuf::from(git(&["rev-parse", "--show-toplevel"]).trim());
    let file = top.join(".ank/entities").join(format!("{id}.md"));
    let text = std::fs::read_to_string(&file).expect("accept wrote the entity file");
    let kept: String = text
        .split_inclusive('\n')
        .filter(|l| !l.starts_with("anchor: "))
        .collect();
    std::fs::write(&file, kept).unwrap();
    git(&["add", "-A", ".ank"]);
    let mut args = vec!["commit", "-q", "-m", message.as_str()];
    if sign {
        args.insert(1, "-S");
    }
    git(&args);
}
