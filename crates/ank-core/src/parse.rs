//! Reading and writing the Ank format.
//!
//! Central property: the round-trip is the identity. `serialize(parse(x)) == x`
//! for any canonical file. That is what guarantees that a command which reads
//! then rewrites a file never produces a spurious diff (§12).
//!
//! Ank always writes the canonical form: fields in a fixed order, literal
//! blocks for multi-line fields, flow lists for references. A file written by
//! hand in another valid YAML form is read correctly, and normalised on first
//! rewrite.

use crate::error::{Error, Result};
use crate::id::{EntityId, EntityKind};
use crate::model::*;
use crate::scope::validate_globs;
use serde::Deserialize;
use std::borrow::Cow;

// ---------------------------------------------------------------------------
// Raw frontmatter (serde shapes)
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TaskFm {
    id: String,
    #[serde(rename = "type")]
    entity_type: String,
    slug: Option<String>,
    title: String,
    created: String,
    // Absent in schema 1, so `Option` is what makes an older file readable
    // rather than a parse error. serde treats a missing `Option` as `None`
    // even under `deny_unknown_fields`, which is exactly the shape wanted.
    author: Option<String>,
    status: TaskStatus,
    scope: Vec<String>,
    #[serde(default)]
    blocked_by: Vec<String>,
    done_criteria: Option<String>,
    criteria_by: Option<CriteriaBy>,
    #[serde(default)]
    verify: Vec<String>,
    #[serde(default)]
    proof: Vec<Proof>,
    schema: u32,
    version: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AdrFm {
    id: String,
    #[serde(rename = "type")]
    entity_type: String,
    slug: Option<String>,
    title: String,
    created: String,
    // Absent in schema 1, so `Option` is what makes an older file readable
    // rather than a parse error. serde treats a missing `Option` as `None`
    // even under `deny_unknown_fields`, which is exactly the shape wanted.
    author: Option<String>,
    status: AdrStatus,
    scope: Vec<String>,
    constraint: String,
    see: Option<String>,
    supersedes: Option<String>,
    ratified: Option<String>,
    schema: u32,
    version: u64,
}

#[derive(Deserialize)]
struct TypeProbe {
    #[serde(rename = "type")]
    entity_type: String,
}

// ---------------------------------------------------------------------------
// Splitting
// ---------------------------------------------------------------------------

/// LF-normalised view of the input, borrowed when there is nothing to do.
///
/// CRLF is **read, never written** (§3). Normalising here rather than at each
/// call site is what keeps the rest of the parser free of line-ending cases:
/// everything downstream of this function sees LF, including the body, which
/// is verbatim with respect to the normalised text and not to the bytes on
/// disk. That is the whole of "normalised on first rewrite".
///
/// A lone `\r` is left alone: old Mac line endings are not a case the format
/// claims to support, and silently rewriting them would be a guess.
pub fn normalise_line_endings(input: &str) -> Cow<'_, str> {
    if input.contains("\r\n") {
        Cow::Owned(input.replace("\r\n", "\n"))
    } else {
        Cow::Borrowed(input)
    }
}

/// Does this text carry CRLF line endings? The question `check` asks to tell a
/// file that is merely not canonical from one that is wrong.
pub fn has_crlf(input: &str) -> bool {
    input.contains("\r\n")
}

/// Separates frontmatter from body. The body is kept verbatim, byte for byte,
/// including the newline that follows the closing `---`.
fn split_frontmatter(input: &str) -> Result<(&str, &str)> {
    let rest = input
        .strip_prefix("---\n")
        .ok_or(Error::MissingFrontmatter)?;
    let end = rest.find("\n---\n").ok_or(Error::MissingFrontmatter)?;
    let fm = &rest[..end];
    let body = &rest[end + "\n---\n".len()..];
    Ok((fm, body))
}

// ---------------------------------------------------------------------------
// Parse
// ---------------------------------------------------------------------------

pub fn parse_entity(input: &str) -> Result<Entity> {
    // The single place line endings are dealt with. A `---\r\n` reported as
    // "missing frontmatter" costs the reader an hour down the wrong path (§3),
    // so it never gets as far as the split.
    parse_entity_lf(&normalise_line_endings(input))
}

fn parse_entity_lf(input: &str) -> Result<Entity> {
    let (fm, body) = split_frontmatter(input)?;
    let probe: TypeProbe = serde_yaml::from_str(fm)?;
    match probe.entity_type.as_str() {
        "task" => Ok(Entity::Task(parse_task_fm(fm, body)?)),
        "adr" => Ok(Entity::Adr(parse_adr_fm(fm, body)?)),
        other => Err(Error::TypeMismatch {
            id: "?".into(),
            field_type: other.to_string(),
        }),
    }
}

pub fn parse_task(input: &str) -> Result<Task> {
    match parse_entity(input)? {
        Entity::Task(t) => Ok(t),
        Entity::Adr(a) => Err(Error::TypeMismatch {
            id: a.id.to_string(),
            field_type: "adr".into(),
        }),
    }
}

pub fn parse_adr(input: &str) -> Result<Adr> {
    match parse_entity(input)? {
        Entity::Adr(a) => Ok(a),
        Entity::Task(t) => Err(Error::TypeMismatch {
            id: t.id.to_string(),
            field_type: "task".into(),
        }),
    }
}

fn parse_task_fm(fm: &str, body: &str) -> Result<Task> {
    let raw: TaskFm = serde_yaml::from_str(fm)?;
    let id = EntityId::parse(&raw.id)?;
    if id.kind() != EntityKind::Task || raw.entity_type != "task" {
        return Err(Error::TypeMismatch {
            id: raw.id,
            field_type: raw.entity_type,
        });
    }
    if raw.scope.is_empty() {
        return Err(Error::EmptyScope);
    }
    validate_globs(&raw.scope)?;
    // A range, not an equality (§3). Older parses — every field added after
    // version 1 is optional, so its absence reads as "written before this
    // existed". Newer is refused on the version rather than on the first field
    // it does not recognise, which is the only diagnosis that names the cause.
    if raw.schema < MIN_SCHEMA || raw.schema > SCHEMA_VERSION {
        return Err(Error::UnknownSchema {
            found: raw.schema,
            supported: SCHEMA_VERSION,
        });
    }
    if raw.criteria_by.is_some() && raw.done_criteria.is_none() {
        return Err(Error::CriteriaByWithoutCriteria);
    }
    let blocked_by = raw
        .blocked_by
        .iter()
        .map(|s| EntityId::parse(s))
        .collect::<Result<Vec<_>>>()?;

    Ok(Task {
        id,
        slug: raw.slug,
        title: raw.title,
        created: raw.created,
        author: raw.author,
        status: raw.status,
        scope: raw.scope,
        blocked_by,
        done_criteria: raw.done_criteria,
        criteria_by: raw.criteria_by,
        verify: raw.verify,
        proof: raw.proof,
        schema: raw.schema,
        version: raw.version,
        body: body.to_string(),
    })
}

fn parse_adr_fm(fm: &str, body: &str) -> Result<Adr> {
    let raw: AdrFm = serde_yaml::from_str(fm)?;
    let id = EntityId::parse(&raw.id)?;
    if id.kind() != EntityKind::Adr || raw.entity_type != "adr" {
        return Err(Error::TypeMismatch {
            id: raw.id,
            field_type: raw.entity_type,
        });
    }
    if raw.scope.is_empty() {
        return Err(Error::EmptyScope);
    }
    validate_globs(&raw.scope)?;
    // A range, not an equality (§3). Older parses — every field added after
    // version 1 is optional, so its absence reads as "written before this
    // existed". Newer is refused on the version rather than on the first field
    // it does not recognise, which is the only diagnosis that names the cause.
    if raw.schema < MIN_SCHEMA || raw.schema > SCHEMA_VERSION {
        return Err(Error::UnknownSchema {
            found: raw.schema,
            supported: SCHEMA_VERSION,
        });
    }
    let supersedes = raw.supersedes.as_deref().map(EntityId::parse).transpose()?;

    Ok(Adr {
        id,
        slug: raw.slug,
        title: raw.title,
        created: raw.created,
        author: raw.author,
        status: raw.status,
        scope: raw.scope,
        constraint: raw.constraint,
        see: raw.see,
        supersedes,
        ratified: raw.ratified,
        schema: raw.schema,
        version: raw.version,
        body: body.to_string(),
    })
}

// ---------------------------------------------------------------------------
// Canonical serialisation
// ---------------------------------------------------------------------------

/// A scalar is emitted bare when that is unambiguous, quoted otherwise.
/// Deliberately conservative: when in doubt, quote.
fn emit_scalar(s: &str) -> String {
    let plain_ok = !s.is_empty()
        && !s.contains('\n')
        && !s.contains(": ")
        && !s.ends_with(':')
        && !s.contains(" #")
        && !s.starts_with([
            '-', '?', ':', '#', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`', '[', ']', '{',
            '}', ',', ' ',
        ])
        && !s.ends_with(' ')
        && !matches!(s, "null" | "~" | "true" | "false" | "yes" | "no")
        && s.parse::<f64>().is_err();
    if plain_ok {
        s.to_string()
    } else {
        let escaped = s
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n");
        format!("\"{escaped}\"")
    }
}

fn emit_block(out: &mut String, key: &str, value: &str) {
    // `|` keeps the trailing newline; `|-` marks its absence.
    if value.ends_with('\n') {
        out.push_str(key);
        out.push_str(": |\n");
        for line in value[..value.len() - 1].split('\n') {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
    } else {
        out.push_str(key);
        out.push_str(": |-\n");
        for line in value.split('\n') {
            out.push_str("  ");
            out.push_str(line);
            out.push('\n');
        }
    }
}

fn emit_flow_list(items: &[String]) -> String {
    let inner = items
        .iter()
        .map(|s| emit_scalar(s))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{inner}]")
}

fn emit_scope(out: &mut String, scope: &[String]) {
    out.push_str("scope:\n");
    for g in scope {
        out.push_str("  - ");
        out.push_str(&emit_scalar(g));
        out.push('\n');
    }
}

pub fn serialize_task(t: &Task) -> String {
    let mut o = String::from("---\n");
    o.push_str(&format!("id: {}\n", t.id));
    o.push_str("type: task\n");
    if let Some(slug) = &t.slug {
        o.push_str(&format!("slug: {}\n", emit_scalar(slug)));
    }
    o.push_str(&format!("title: {}\n", emit_scalar(&t.title)));
    o.push_str(&format!("created: {}\n", emit_scalar(&t.created)));
    // Omitted when absent, never emitted empty: that is what lets a schema 1
    // file round-trip byte for byte through a tool that knows the field.
    if let Some(author) = &t.author {
        o.push_str(&format!("author: {}\n", emit_scalar(author)));
    }
    o.push_str(&format!("status: {}\n", t.status.as_str()));
    emit_scope(&mut o, &t.scope);
    let blocked: Vec<String> = t.blocked_by.iter().map(|b| b.to_string()).collect();
    o.push_str(&format!("blocked_by: {}\n", emit_flow_list(&blocked)));
    if let Some(c) = &t.done_criteria {
        emit_block(&mut o, "done_criteria", c);
    }
    if let Some(by) = t.criteria_by {
        o.push_str(&format!("criteria_by: {}\n", by.as_str()));
    }
    if !t.verify.is_empty() {
        o.push_str(&format!("verify: {}\n", emit_flow_list(&t.verify)));
    }
    if !t.proof.is_empty() {
        o.push_str("proof:\n");
        for p in &t.proof {
            o.push_str(&format!("  - type: {}\n", p.proof_type.as_str()));
            o.push_str(&format!("    ref: {}\n", emit_scalar(&p.reference)));
            if let Some(tree) = &p.tree {
                o.push_str(&format!("    tree: {}\n", emit_scalar(tree)));
            }
            if let Some(c) = &p.criteria {
                o.push_str(&format!("    criteria: {}\n", emit_scalar(c)));
            }
            if let Some(v) = &p.verifier {
                o.push_str(&format!("    verifier: {}\n", emit_scalar(v)));
            }
        }
    }
    o.push_str(&format!("schema: {}\n", t.schema));
    o.push_str(&format!("version: {}\n", t.version));
    o.push_str("---\n");
    o.push_str(&t.body);
    o
}

pub fn serialize_adr(a: &Adr) -> String {
    let mut o = String::from("---\n");
    o.push_str(&format!("id: {}\n", a.id));
    o.push_str("type: adr\n");
    if let Some(slug) = &a.slug {
        o.push_str(&format!("slug: {}\n", emit_scalar(slug)));
    }
    o.push_str(&format!("title: {}\n", emit_scalar(&a.title)));
    o.push_str(&format!("created: {}\n", emit_scalar(&a.created)));
    if let Some(author) = &a.author {
        o.push_str(&format!("author: {}\n", emit_scalar(author)));
    }
    o.push_str(&format!("status: {}\n", a.status.as_str()));
    emit_scope(&mut o, &a.scope);
    emit_block(&mut o, "constraint", &a.constraint);
    if let Some(see) = &a.see {
        o.push_str(&format!("see: {}\n", emit_scalar(see)));
    }
    if let Some(sup) = &a.supersedes {
        o.push_str(&format!("supersedes: {}\n", sup));
    }
    if let Some(r) = &a.ratified {
        o.push_str(&format!("ratified: {}\n", emit_scalar(r)));
    }
    o.push_str(&format!("schema: {}\n", a.schema));
    o.push_str(&format!("version: {}\n", a.version));
    o.push_str("---\n");
    o.push_str(&a.body);
    o
}

pub fn serialize_entity(e: &Entity) -> String {
    match e {
        Entity::Task(t) => serialize_task(t),
        Entity::Adr(a) => serialize_adr(a),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A task at a given schema, with `author` present or not. Written out
    /// rather than built from the model, because what is being tested is what a
    /// file on disk does to the parser.
    fn task_text(schema: u32, author: Option<&str>) -> String {
        let author = author.map(|a| format!("author: {a}\n")).unwrap_or_default();
        format!(
            "---\nid: TASK-aaaa11112222\ntype: task\ntitle: A task\n\
             created: 2026-07-25T09:14:00Z\n{author}status: open\nscope:\n  - src/**\n\
             blocked_by: []\nschema: {schema}\nversion: 1\n---\n"
        )
    }

    /// The promise the format keeps in one direction only (§3).
    #[test]
    fn an_older_schema_parses_and_a_newer_one_is_refused_by_version() {
        // Schema 1 predates `author`, and its absence means "written before
        // this existed" rather than "invalid". Without this the task adding the
        // field could not read the repository it is developed in.
        let old = parse_task(&task_text(1, None)).expect("schema 1 must still parse");
        assert_eq!(old.schema, 1);
        assert_eq!(old.author, None);

        let current = parse_task(&task_text(SCHEMA_VERSION, Some("marie@laptop"))).unwrap();
        assert_eq!(current.author.as_deref(), Some("marie@laptop"));

        // One past the newest, which is the boundary the golden suite cannot
        // reach: `bad-schema.md` carries 99 and would pass a check that only
        // rejected absurd values.
        match parse_task(&task_text(SCHEMA_VERSION + 1, None)) {
            Err(Error::UnknownSchema { found, supported }) => {
                assert_eq!(found, SCHEMA_VERSION + 1);
                assert_eq!(supported, SCHEMA_VERSION);
            }
            other => panic!("a newer schema must be refused by version: {other:?}"),
        }
    }

    /// The omission is what makes the asymmetry work. An entity with no author
    /// must serialise without the key at all — emitting `author:` empty would
    /// change every schema 1 file on its first rewrite, and the round-trip is
    /// byte-identical on canonical form (ADR-63b59c5c26f7).
    #[test]
    fn an_absent_author_is_omitted_and_the_round_trip_holds() {
        for (schema, author) in [(1, None), (SCHEMA_VERSION, Some("codex@host-9"))] {
            let text = task_text(schema, author);
            let parsed = parse_task(&text).unwrap();
            let out = serialize_task(&parsed);
            assert_eq!(out, text, "round-trip differs at schema {schema}");
            assert_eq!(out.contains("author:"), author.is_some());
        }
    }

    /// `author` sits between `created` and `status`, and the position is part
    /// of the format rather than a detail: canonical form is a fixed field
    /// order, so a serializer emitting it elsewhere would make every file
    /// carrying it non-canonical.
    #[test]
    fn author_is_emitted_between_created_and_status() {
        let t = parse_task(&task_text(SCHEMA_VERSION, Some("a@h"))).unwrap();
        let out = serialize_task(&t);
        let at = |needle: &str| out.find(needle).unwrap_or_else(|| panic!("{needle}"));
        assert!(at("created:") < at("author:"));
        assert!(at("author:") < at("status:"));
    }
}
