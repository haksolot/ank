//! What the reader knows, and where every byte of it came from.
//!
//! [`Snapshot`] is `ank status --json` and `ank find --json` put side by side:
//! who holds what, and every entity of every kind with its status. [`Detail`]
//! is `ank show <id> --json` and one `ank scope <glob> --json` per glob the
//! entity declares: the body whole, and the constraints binding it.
//!
//! **Nothing here derives a fact the CLI did not state.** The one exception is
//! the `scope:` block, which is lifted out of the frontmatter the CLI printed
//! in `content` -- and lifted, not parsed: the round trip is guaranteed on
//! canonical form (ADR-63b59c5c26f7), so the block is a `scope:` line followed
//! by `  - ` lines, and finding it needs no YAML reader. The alternative was to
//! ask a fifth verb for it, and there is none that answers "what does this
//! entity declare".

use crate::ank::{self, Ank, Failed};
use ank_contract::json::Obj;

/// One entity row, as `find` renders one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub id: String,
    pub kind: String,
    pub status: String,
    pub title: String,
}

impl Row {
    fn read(value: &ank::Value) -> Row {
        Row {
            id: ank::text(value, "id"),
            // `scope`'s rows carry `kind`; `find`'s carry it too. A row that
            // carried neither would show as the empty kind rather than as a
            // crash, and the filter would simply never match it.
            kind: ank::text(value, "kind"),
            status: ank::text(value, "status"),
            title: ank::text(value, "title"),
        }
    }

    /// The short form the CLI prints, `TASK-4974`: the kind and four hex.
    pub fn short(&self) -> String {
        short_of(&self.id)
    }
}

/// The short form of an identifier, the way every listing in this tool prints
/// one: everything up to the dash, then four characters.
pub fn short_of(id: &str) -> String {
    match id.split_once('-') {
        Some((kind, rest)) if rest.len() > 4 => format!("{kind}-{}", &rest[..4]),
        _ => id.to_string(),
    }
}

/// A claim, and who holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claim {
    pub id: String,
    pub holder: String,
    pub expires: String,
    /// Whether this is the claim the caller of the reader holds.
    pub mine: bool,
}

/// The corpus as one screen sees it.
#[derive(Debug, Clone)]
pub struct Snapshot {
    pub branch: String,
    pub default_branch: String,
    pub identity: String,
    pub corpus: String,
    pub claims: Vec<Claim>,
    pub entities: Vec<Row>,
    /// What `find` said it withheld, so a screen never implies it saw
    /// everything when it did not (ADR-3e6ce108edcd).
    pub total: u64,
}

impl Snapshot {
    pub fn load(ank: &Ank) -> Result<Snapshot, Failed> {
        let status = ank.json("status", &[])?;
        let found = ank.json("find", &[])?;

        let identity = status
            .get("identity")
            .map(|i| ank::text(i, "value"))
            .unwrap_or_default();

        let mut claims = Vec::new();
        // The caller's own claim first, and marked: "which claim is held by
        // whom" is answered wrongly by a list that does not say which one is
        // the reader's.
        if let Some(held) = status.get("claim").filter(|v| !v.is_null()) {
            claims.push(Claim {
                id: ank::text(held, "id"),
                holder: identity.clone(),
                expires: ank::text(held, "expires"),
                mine: true,
            });
        }
        for also in ank::rows(&status, "also_held") {
            claims.push(Claim {
                id: ank::text(also, "id"),
                holder: identity.clone(),
                expires: ank::text(also, "expires"),
                mine: true,
            });
        }
        for other in ank::rows(&status, "elsewhere") {
            claims.push(Claim {
                id: ank::text(other, "id"),
                holder: ank::text(other, "holder"),
                expires: ank::text(other, "expires"),
                mine: false,
            });
        }

        Ok(Snapshot {
            branch: ank::text(&status, "branch"),
            default_branch: ank::text(&status, "default_branch"),
            identity,
            corpus: ank::text(&status, "corpus"),
            claims,
            entities: ank::rows(&found, "results").iter().map(Row::read).collect(),
            total: ank::count(&found, "total"),
        })
    }

    /// The row for an identifier, whole or abbreviated.
    pub fn find(&self, needle: &str) -> Option<usize> {
        let needle = needle.to_ascii_uppercase();
        self.entities
            .iter()
            .position(|r| r.id.to_ascii_uppercase().starts_with(&needle))
    }

    pub fn row(&self, id: &str) -> Option<&Row> {
        self.entities.iter().find(|r| r.id == id)
    }

    /// The claim held on an entity, if one is.
    pub fn claim_on(&self, id: &str) -> Option<&Claim> {
        self.claims.iter().find(|c| c.id == id)
    }

    /// The opening frame as data, for `ank tui --json` (§4).
    ///
    /// The reader's own answer and not a passthrough: it is what the screen
    /// holds, in the one writer and the one escaper every other document in
    /// this tool goes through (ADR-6fd69efb629c).
    pub fn document(&self) -> String {
        Obj::document()
            .str("corpus", &self.corpus)
            .str("branch", &self.branch)
            .str("default_branch", &self.default_branch)
            .str("identity", &self.identity)
            .num("total", self.total)
            .num("shown", self.entities.len())
            .array(
                "claims",
                self.claims.iter().map(|c| {
                    Obj::new()
                        .str("id", &c.id)
                        .str("short", &short_of(&c.id))
                        .str("holder", &c.holder)
                        .str("expires", &c.expires)
                        .bool("mine", c.mine)
                        .finish()
                }),
            )
            .array(
                "entities",
                self.entities.iter().map(|r| {
                    Obj::new()
                        .str("id", &r.id)
                        .str("short", &r.short())
                        .str("kind", &r.kind)
                        .str("status", &r.status)
                        .str("title", &r.title)
                        .finish()
                }),
            )
            .finish()
    }
}

/// One entity, opened.
#[derive(Debug, Clone)]
pub struct Detail {
    pub id: String,
    /// `claimed by <agent>`, as `show` states it, or nothing.
    pub coordination: Option<String>,
    /// The entity whole, frontmatter and body, byte for byte as `show` printed
    /// it. Never trimmed here: "the body of a selected entity whole" is the
    /// criterion, and paging is the renderer's business.
    pub content: String,
    pub scopes: Vec<String>,
    pub constraints: Vec<Row>,
    pub blocked_by: Vec<Row>,
    pub unblocks: Vec<Row>,
    /// A scope glob that could not be asked about, with what the CLI said.
    /// Shown rather than swallowed: a constraint list silently short by one is
    /// the one wrong answer a reader would act on.
    pub unresolved: Vec<String>,
}

impl Detail {
    pub fn load(ank: &Ank, id: &str) -> Result<Detail, Failed> {
        let shown = ank.json("show", &[id])?;
        let content = ank::text(&shown, "content");
        let scopes = declared_scopes(&content);

        let mut constraints: Vec<Row> = Vec::new();
        let mut unresolved = Vec::new();
        for glob in &scopes {
            match ank.json("scope", &[glob]) {
                Ok(answer) => {
                    for value in ank::rows(&answer, "adr") {
                        let row = Row::read(value);
                        if !constraints.iter().any(|c| c.id == row.id) {
                            constraints.push(row);
                        }
                    }
                }
                Err(failed) => unresolved.push(format!("{glob}: {failed}")),
            }
        }
        constraints.sort_by(|a, b| a.id.cmp(&b.id));

        Ok(Detail {
            id: id.to_string(),
            coordination: ank::maybe(&shown, "coordination"),
            content,
            scopes,
            constraints,
            blocked_by: ank::rows(&shown, "blocked_by")
                .iter()
                .map(Row::read)
                .collect(),
            unblocks: ank::rows(&shown, "unblocks")
                .iter()
                .map(Row::read)
                .collect(),
            unresolved,
        })
    }
}

/// The globs an entity declares, lifted out of the frontmatter `show` printed.
///
/// Canonical form is what the store round-trips (ADR-63b59c5c26f7): the
/// frontmatter is the text between the first two `---` lines, and a list field
/// is the key followed by `  - ` items. That is the whole grammar this needs,
/// and reading it costs no parser -- which matters, because a parser here would
/// be a second implementation of the format, and the crate that holds the first
/// one is exactly the crate this one may not link.
pub fn declared_scopes(content: &str) -> Vec<String> {
    let mut lines = content.lines();
    if lines.next().map(str::trim_end) != Some("---") {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut inside = false;
    for line in lines {
        if line.trim_end() == "---" {
            break;
        }
        if let Some(rest) = line.strip_prefix("scope:") {
            inside = true;
            // The flow form, `scope: [a, b]`, which canonical form does not
            // write for `scope` but which a hand-edited file may carry.
            let rest = rest.trim();
            if let Some(items) = rest.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
                for item in items.split(',') {
                    push_glob(&mut out, item);
                }
                inside = false;
            }
            continue;
        }
        if inside {
            match line.strip_prefix("  - ") {
                Some(item) => push_glob(&mut out, item),
                // Any other line at column zero ends the block; a deeper line
                // that is not an item is not one either.
                None => inside = false,
            }
        }
    }
    out
}

fn push_glob(out: &mut Vec<String>, item: &str) {
    let glob = item.trim().trim_matches(['"', '\'']).trim();
    if !glob.is_empty() && !out.iter().any(|g| g == glob) {
        out.push(glob.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTITY: &str = "---\nid: TASK-49746735127f\ntype: task\ntitle: A title\nscope:\n  - crates/ank-tui/**\n  - crates/ank-cli/src/cli.rs\nblocked_by: [TASK-8108e3771ba0]\nschema: 4\n---\n\nThe body.\n";

    #[test]
    fn the_declared_globs_are_lifted_out_of_the_frontmatter() {
        assert_eq!(
            declared_scopes(ENTITY),
            ["crates/ank-tui/**", "crates/ank-cli/src/cli.rs"]
        );
    }

    #[test]
    fn the_block_ends_where_the_next_field_starts() {
        // `blocked_by` is a list too, and reading past `scope:` would collect
        // its items as globs and then ask `ank scope` about a task id.
        let scopes = declared_scopes(ENTITY);
        assert!(
            !scopes.iter().any(|s| s.contains("TASK-")),
            "the next field leaked into the block: {scopes:?}"
        );
    }

    #[test]
    fn a_body_without_frontmatter_declares_nothing() {
        assert!(declared_scopes("just prose\nscope:\n  - src/**\n").is_empty());
        assert!(declared_scopes("").is_empty());
    }

    #[test]
    fn the_flow_form_is_read_too_and_deduplicated() {
        let content = "---\nid: ADR-0001\nscope: [\"src/**\", 'src/**', docs/**]\n---\n\nbody\n";
        assert_eq!(declared_scopes(content), ["src/**", "docs/**"]);
    }

    #[test]
    fn the_short_form_is_the_one_every_listing_prints() {
        assert_eq!(short_of("TASK-49746735127f"), "TASK-4974");
        assert_eq!(short_of("ADR-8bd76e8d7c4e"), "ADR-8bd7");
        // Too short to abbreviate, and a value that is not an identifier at
        // all: both come back unchanged rather than panicking on a slice.
        assert_eq!(short_of("TASK-abc"), "TASK-abc");
        assert_eq!(short_of("nonsense"), "nonsense");
    }
}
