//! Lecture et ecriture du format Ankor.
//!
//! Propriete centrale : le round-trip est identite. `serialize(parse(x)) == x`
//! pour tout fichier canonique. C'est ce qui garantit qu'une commande qui
//! relit puis reecrit un fichier ne produit jamais de diff parasite (§12).
//!
//! Ankor ecrit toujours la forme canonique : champs dans un ordre fixe,
//! blocs litteraux pour les champs multi-lignes, listes flow pour les
//! references. Un fichier ecrit a la main dans une autre forme YAML valide
//! est lu correctement, et normalise a la premiere reecriture.

use crate::error::{Error, Result};
use crate::id::{EntityId, EntityKind};
use crate::model::*;
use crate::scope::validate_globs;
use serde::Deserialize;

// ---------------------------------------------------------------------------
// Frontmatter brut (formes serde)
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
// Decoupage
// ---------------------------------------------------------------------------

/// Separe frontmatter et corps. Le corps est conserve verbatim, octet pour
/// octet, y compris le saut de ligne qui suit le `---` fermant.
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
    if raw.schema != SCHEMA_VERSION {
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
    if raw.schema != SCHEMA_VERSION {
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
// Serialisation canonique
// ---------------------------------------------------------------------------

/// Un scalaire est emis nu quand c'est sans ambiguite, sinon entre guillemets.
/// Volontairement conservateur : dans le doute, on quote.
fn emit_scalar(s: &str) -> String {
    let plain_ok = !s.is_empty()
        && !s.contains('\n')
        && !s.contains(": ")
        && !s.ends_with(':')
        && !s.contains(" #")
        && !s.starts_with(['-', '?', ':', '#', '&', '*', '!', '|', '>', '\'', '"', '%', '@', '`', '[', ']', '{', '}', ',', ' '])
        && !s.ends_with(' ')
        && !matches!(s, "null" | "~" | "true" | "false" | "yes" | "no")
        && s.parse::<f64>().is_err();
    if plain_ok {
        s.to_string()
    } else {
        let escaped = s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n");
        format!("\"{escaped}\"")
    }
}

fn emit_block(out: &mut String, key: &str, value: &str) {
    // `|` conserve le saut final ; `|-` l'absence de saut final.
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
