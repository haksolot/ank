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
use crate::registry::{FieldValue, Fields};
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
    // Absent before schema 3, and `default` rather than `Option` because an
    // empty list and an absent one are the same statement: nobody recorded a
    // reading. The serializer omits it either way.
    #[serde(default)]
    verified: Vec<Verified>,
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
    #[serde(default)]
    verified: Vec<Verified>,
    schema: u32,
    version: u64,
}

/// A spec's frontmatter is an ADR's without `constraint` and without `see`, and
/// the absence is enforced here by `deny_unknown_fields` rather than by a rule
/// of its own: a `constraint:` inside a `spec` is refused naming the field,
/// which is the same refusal a typo earns and the right one — the kind exists
/// because a spec describes where an ADR binds (§3).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpecFm {
    id: String,
    #[serde(rename = "type")]
    entity_type: String,
    slug: Option<String>,
    title: String,
    created: String,
    author: Option<String>,
    status: SpecStatus,
    scope: Vec<String>,
    // Absent before the field existed, and `default` rather than `Option`
    // because an empty list and an absent one are the same statement: this
    // document cites nothing. The serializer omits it either way.
    #[serde(default)]
    references: Vec<String>,
    supersedes: Option<String>,
    ratified: Option<String>,
    #[serde(default)]
    verified: Vec<Verified>,
    schema: u32,
    version: u64,
}

/// A log entry's frontmatter. No `status` — an entry has nothing to transition
/// to — and `about` and `seq` are both required: an entry with no subject is the
/// query the kind exists to make answerable, missing, and an entry with no rank
/// is the order missing, which a timestamp alone cannot supply (§3).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LogFm {
    id: String,
    #[serde(rename = "type")]
    entity_type: String,
    slug: Option<String>,
    title: String,
    created: String,
    author: Option<String>,
    scope: Vec<String>,
    about: String,
    seq: u64,
    #[serde(default)]
    verified: Vec<Verified>,
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
    // The kind is resolved through the registry, and a kind it does not declare
    // is refused **by name** (§3). Not by the id prefix, which would send the
    // reader hunting for a typo in the hex, and not by the first field the kind
    // happens to carry, which would name a symptom.
    let kind = EntityKind::from_type_name(&probe.entity_type).ok_or(Error::UnknownKind {
        kind: probe.entity_type.clone(),
    })?;
    match kind {
        EntityKind::Task => Ok(Entity::Task(parse_task_fm(fm, body)?)),
        EntityKind::Adr => Ok(Entity::Adr(parse_adr_fm(fm, body)?)),
        EntityKind::Spec => Ok(Entity::Spec(parse_spec_fm(fm, body)?)),
        EntityKind::Log => Ok(Entity::Log(parse_log_fm(fm, body)?)),
    }
}

/// The entity was read and is of another kind. The kind it *is* comes from the
/// registry, so the diagnostic names it without a per-kind table of names.
fn wrong_kind(e: &Entity) -> Error {
    Error::TypeMismatch {
        id: e.id().to_string(),
        field_type: e.kind_spec().name.to_string(),
    }
}

pub fn parse_task(input: &str) -> Result<Task> {
    match parse_entity(input)? {
        Entity::Task(t) => Ok(t),
        other => Err(wrong_kind(&other)),
    }
}

pub fn parse_adr(input: &str) -> Result<Adr> {
    match parse_entity(input)? {
        Entity::Adr(a) => Ok(a),
        other => Err(wrong_kind(&other)),
    }
}

pub fn parse_spec(input: &str) -> Result<Spec> {
    match parse_entity(input)? {
        Entity::Spec(s) => Ok(s),
        other => Err(wrong_kind(&other)),
    }
}

/// Named for the entity and not for the line: [`crate::log::parse_log`] reads
/// the `## Log` section of a body, and this reads a file of kind `log`.
pub fn parse_log_entity(input: &str) -> Result<Log> {
    match parse_entity(input)? {
        Entity::Log(l) => Ok(l),
        other => Err(wrong_kind(&other)),
    }
}

/// What every kind owes, checked once rather than once per kind: the id parses
/// and agrees with `type`, the scope is present and its globs are valid, and
/// the schema is inside the range this crate reads. Returns the parsed id,
/// which is the one product of the checks a caller needs afterwards.
///
/// A fourth copy of this block is exactly what ADR-c9f9d0d6f05d says a kind
/// must not cost, and the only thing a fourth copy could do is disagree with
/// the first three.
fn common_fields(
    kind: EntityKind,
    raw_id: &str,
    entity_type: &str,
    scope: &[String],
    schema: u32,
) -> Result<EntityId> {
    let id = EntityId::parse(raw_id)?;
    if id.kind() != kind || entity_type != kind.as_str() {
        return Err(Error::TypeMismatch {
            id: raw_id.to_string(),
            field_type: entity_type.to_string(),
        });
    }
    if scope.is_empty() {
        return Err(Error::EmptyScope);
    }
    validate_globs(scope)?;
    // A range, not an equality (§3). Older parses — every field added after
    // version 1 is optional, so its absence reads as "written before this
    // existed". Newer is refused on the version rather than on the first field
    // it does not recognise, which is the only diagnosis that names the cause.
    if schema < MIN_SCHEMA || schema > SCHEMA_VERSION {
        return Err(Error::UnknownSchema {
            found: schema,
            supported: SCHEMA_VERSION,
        });
    }
    Ok(id)
}

fn parse_task_fm(fm: &str, body: &str) -> Result<Task> {
    let raw: TaskFm = serde_yaml::from_str(fm)?;
    let id = common_fields(
        EntityKind::Task,
        &raw.id,
        &raw.entity_type,
        &raw.scope,
        raw.schema,
    )?;
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
        verified: raw.verified,
        schema: raw.schema,
        version: raw.version,
        body: body.to_string(),
    })
}

fn parse_adr_fm(fm: &str, body: &str) -> Result<Adr> {
    let raw: AdrFm = serde_yaml::from_str(fm)?;
    let id = common_fields(
        EntityKind::Adr,
        &raw.id,
        &raw.entity_type,
        &raw.scope,
        raw.schema,
    )?;
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
        verified: raw.verified,
        schema: raw.schema,
        version: raw.version,
        body: body.to_string(),
    })
}

fn parse_spec_fm(fm: &str, body: &str) -> Result<Spec> {
    let raw: SpecFm = serde_yaml::from_str(fm)?;
    let id = common_fields(
        EntityKind::Spec,
        &raw.id,
        &raw.entity_type,
        &raw.scope,
        raw.schema,
    )?;
    let supersedes = raw.supersedes.as_deref().map(EntityId::parse).transpose()?;
    // Checked here for what it is — an identifier — and for nothing more. Which
    // kinds a specification may cite, and whether the corpus holds the target
    // at all, are `check` findings (§3): a rule enforced at parse time would
    // make one citation cost the whole corpus its readability.
    let references = raw
        .references
        .iter()
        .map(|s| EntityId::parse(s))
        .collect::<Result<Vec<_>>>()?;

    Ok(Spec {
        id,
        slug: raw.slug,
        title: raw.title,
        created: raw.created,
        author: raw.author,
        status: raw.status,
        scope: raw.scope,
        references,
        supersedes,
        ratified: raw.ratified,
        verified: raw.verified,
        schema: raw.schema,
        version: raw.version,
        body: body.to_string(),
    })
}

fn parse_log_fm(fm: &str, body: &str) -> Result<Log> {
    let raw: LogFm = serde_yaml::from_str(fm)?;
    let id = common_fields(
        EntityKind::Log,
        &raw.id,
        &raw.entity_type,
        &raw.scope,
        raw.schema,
    )?;
    // Any kind may be named: a task, an ADR, a spec — and an entry about an
    // entry is not forbidden here either, because the format has no rule that
    // would make it one. What is checked is that the reference is an identifier
    // at all, which is what turns the address into a query that can resolve.
    let about = EntityId::parse(&raw.about)?;

    Ok(Log {
        id,
        slug: raw.slug,
        title: raw.title,
        created: raw.created,
        author: raw.author,
        scope: raw.scope,
        about,
        seq: raw.seq,
        verified: raw.verified,
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

fn emit_seq(out: &mut String, key: &str, items: &[String]) {
    out.push_str(key);
    out.push_str(":\n");
    for item in items {
        out.push_str("  - ");
        out.push_str(&emit_scalar(item));
        out.push('\n');
    }
}

/// Writes one field in the form its value declares. Every emission rule of
/// `docs/format.md` is here and nowhere else.
fn emit_field(o: &mut String, name: &str, value: &FieldValue<'_>) {
    match value {
        FieldValue::Bare(s) => o.push_str(&format!("{name}: {s}\n")),
        FieldValue::Scalar(s) => o.push_str(&format!("{name}: {}\n", emit_scalar(s))),
        FieldValue::Block(s) => emit_block(o, name, s),
        FieldValue::Flow(items) => o.push_str(&format!("{name}: {}\n", emit_flow_list(items))),
        FieldValue::Seq(items) => emit_seq(o, name, items),
        FieldValue::Proofs(proofs) => {
            o.push_str(&format!("{name}:\n"));
            for p in *proofs {
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
                if let Some(via) = p.via {
                    o.push_str(&format!("    via: {}\n", via.as_str()));
                }
            }
        }
        FieldValue::Readings(readings) => {
            o.push_str(&format!("{name}:\n"));
            for v in *readings {
                o.push_str(&format!("  - by: {}\n", emit_scalar(&v.by)));
                o.push_str(&format!("    at: {}\n", emit_scalar(&v.at)));
            }
        }
    }
}

/// **One** serializer, driven by the registry (ADR-c9f9d0d6f05d). There is no
/// per-kind emitter, and the field order is not written here at all: it is the
/// table in [`crate::registry`], which is also the table in `docs/format.md`.
/// That is what makes adding a kind a row rather than a second straight-line
/// function that differs from the first only in which fields it emits.
fn serialize_fields<F: Fields + ?Sized>(e: &F) -> String {
    let spec = e.kind_spec();
    let mut o = String::from("---\n");
    for field in spec.fields {
        match e.field_value(field.name) {
            Some(value) => emit_field(&mut o, field.name, &value),
            // Omitted when absent, never emitted empty — which is what lets a
            // file written before a field existed round-trip unchanged through
            // a tool that knows the field. A *required* field with no value is
            // not that case: it is this crate contradicting its own table.
            None => assert!(
                !field.required,
                "{} is required on {} and the model holds nothing",
                field.name, spec.name
            ),
        }
    }
    o.push_str("---\n");
    o.push_str(e.body());
    o
}

pub fn serialize_task(t: &Task) -> String {
    serialize_fields(t)
}

pub fn serialize_adr(a: &Adr) -> String {
    serialize_fields(a)
}

pub fn serialize_spec(s: &Spec) -> String {
    serialize_fields(s)
}

pub fn serialize_log_entity(l: &Log) -> String {
    serialize_fields(l)
}

pub fn serialize_entity(e: &Entity) -> String {
    serialize_fields(e)
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
