//! The kind registry: one table, and the only place a kind is declared.
//!
//! A kind used to be stated four times — a closed enum in `id`, a sum type in
//! `model`, a match in `parse`, and a directory on disk — and the only thing a
//! fourth copy can do is disagree with the first three (ADR-c9f9d0d6f05d). The
//! directory went with the flat layout; what is left is here, once:
//!
//! - the name written in `type`,
//! - the id prefix,
//! - the fields, **in canonical order**, each required or optional.
//!
//! Adding a kind is a row in [`KINDS`], a golden fixture and a section of the
//! specification. Never a second serializer, never a second parser branch.
//!
//! The field order is **data**, not control flow. It is what makes the
//! round-trip byte-identical, and it is the single thing most easily lost by
//! rewriting two straight-line emitters as a generic loop, so it lives in a
//! table that reads like the table in `docs/format.md` and nowhere else.
//!
//! What this registry deliberately does not do is make the format permissive.
//! An unknown field inside a known kind is still rejected, and an unknown kind
//! is rejected by name. A kind is cheap to add; nothing else moved.

use crate::model::{Adr, Entity, Log, Proof, Spec, Task, Verified};

/// One field, at its canonical position.
pub struct FieldSpec {
    pub name: &'static str,
    /// `true` means **always emitted**: `blocked_by` is required and is written
    /// `[]` when empty. `false` means omitted when absent, never emitted empty
    /// — which is what lets a file written before a field existed survive a
    /// rewrite unchanged.
    pub required: bool,
}

const fn req(name: &'static str) -> FieldSpec {
    FieldSpec {
        name,
        required: true,
    }
}

const fn opt(name: &'static str) -> FieldSpec {
    FieldSpec {
        name,
        required: false,
    }
}

pub struct KindSpec {
    /// The value of the `type` field.
    pub name: &'static str,
    /// The id prefix, `TASK-` and the like, trailing dash included.
    pub prefix: &'static str,
    /// Every field, in canonical order.
    pub fields: &'static [FieldSpec],
}

static TASK_FIELDS: &[FieldSpec] = &[
    req("id"),
    req("type"),
    opt("slug"),
    req("title"),
    req("created"),
    opt("author"),
    req("status"),
    req("scope"),
    req("blocked_by"),
    opt("done_criteria"),
    opt("criteria_by"),
    opt("verify"),
    opt("proof"),
    opt("verified"),
    req("schema"),
    req("version"),
];

static ADR_FIELDS: &[FieldSpec] = &[
    req("id"),
    req("type"),
    opt("slug"),
    req("title"),
    req("created"),
    opt("author"),
    req("status"),
    req("scope"),
    req("constraint"),
    opt("see"),
    opt("supersedes"),
    opt("ratified"),
    opt("verified"),
    req("schema"),
    req("version"),
];

/// A spec is an ADR's table without `constraint` and without `see`, and the
/// first absence is the whole justification for the kind (§3): a spec
/// describes, an ADR binds. `see` goes with it — it exists to point at the
/// reference code a positive *constraint* needs, and a kind with no constraint
/// has nothing for it to serve.
static SPEC_FIELDS: &[FieldSpec] = &[
    req("id"),
    req("type"),
    opt("slug"),
    req("title"),
    req("created"),
    opt("author"),
    req("status"),
    req("scope"),
    opt("supersedes"),
    opt("ratified"),
    opt("verified"),
    req("schema"),
    req("version"),
];

/// A log entry carries `about` and `seq` and **no `status`**: an entry is
/// written once and has nothing to transition to, so a status would have one
/// legal value and would only ever be copied (§3). `about` takes the position
/// `status` and `scope` leave, immediately after the scope it is a statement
/// about, and `seq` follows it because it ranks the entry among *that* entity's
/// entries and means nothing without it.
static LOG_FIELDS: &[FieldSpec] = &[
    req("id"),
    req("type"),
    opt("slug"),
    req("title"),
    req("created"),
    opt("author"),
    req("scope"),
    req("about"),
    req("seq"),
    opt("verified"),
    req("schema"),
    req("version"),
];

/// The registry. Its order is the order `ank help` and the specification use,
/// and adding a row is the whole cost of a new kind.
///
/// The order is also what [`crate::id::EntityKind`] indexes: its variants are
/// these rows, positionally, and `registry_and_enum_agree` asserts the two
/// agree rather than trusting that they do.
pub static KINDS: &[KindSpec] = &[
    KindSpec {
        name: "task",
        prefix: "TASK-",
        fields: TASK_FIELDS,
    },
    KindSpec {
        name: "adr",
        prefix: "ADR-",
        fields: ADR_FIELDS,
    },
    KindSpec {
        name: "spec",
        prefix: "SPEC-",
        fields: SPEC_FIELDS,
    },
    KindSpec {
        name: "log",
        prefix: "LOG-",
        fields: LOG_FIELDS,
    },
];

/// The kind whose `type` is this string, if the registry declares one.
pub fn by_type_name(name: &str) -> Option<&'static KindSpec> {
    KINDS.iter().find(|k| k.name == name)
}

/// The kind whose id prefix this identifier carries, with the hex that follows.
pub fn by_id_prefix(id: &str) -> Option<(&'static KindSpec, &str)> {
    KINDS
        .iter()
        .find_map(|k| id.strip_prefix(k.prefix).map(|rest| (k, rest)))
}

// ---------------------------------------------------------------------------
// Values, and the form each is written in
// ---------------------------------------------------------------------------

/// A field's value together with **how it is written** in canonical form. The
/// variants are the emission rules of `docs/format.md` and there are no others:
/// a form that is not here is a form no third-party writer has to reproduce.
pub enum FieldValue<'a> {
    /// Written as-is, with no quoting decision to make: identifiers, enum
    /// values, integers.
    Bare(String),
    /// Written bare when that is unambiguous and quoted otherwise.
    Scalar(&'a str),
    /// Literal block, `|` when the value ends in a newline and `|-` when not.
    Block(&'a str),
    /// `[a, b]`, and `[]` when empty.
    Flow(Vec<String>),
    /// Block sequence of scalars.
    Seq(&'a [String]),
    /// Block sequence of maps, with the keys of a proof entry in their order.
    Proofs(&'a [Proof]),
    /// Block sequence of maps: `by` then `at`.
    Readings(&'a [Verified]),
}

/// The value a kind holds under a field name, or `None` when it holds nothing
/// there — which is what "omitted when absent, never emitted empty" means at
/// the point the serializer asks.
///
/// This is a lookup and not an ordering: the order is the table above, and
/// nothing here may be read as stating one.
pub trait Fields {
    fn field_value(&self, name: &str) -> Option<FieldValue<'_>>;
    fn kind_spec(&self) -> &'static KindSpec;
    fn body(&self) -> &str;
}

/// A field named in the table and absent from the model is a bug in this
/// crate, not in the file being written, so it is loud and immediate.
fn no_such_field(kind: &str, name: &str) -> ! {
    unreachable!("the registry declares '{name}' on {kind}, and the model has no such field")
}

impl Fields for Task {
    fn field_value(&self, name: &str) -> Option<FieldValue<'_>> {
        use FieldValue::*;
        Some(match name {
            "id" => Bare(self.id.to_string()),
            "type" => Bare(self.id.kind().as_str().to_string()),
            "slug" => Scalar(self.slug.as_deref()?),
            "title" => Scalar(&self.title),
            "created" => Scalar(&self.created),
            "author" => Scalar(self.author.as_deref()?),
            "status" => Bare(self.status.as_str().to_string()),
            "scope" => Seq(&self.scope),
            // Required, and therefore written `[]` when empty: a task with no
            // blocker says so rather than staying silent about it.
            "blocked_by" => Flow(self.blocked_by.iter().map(|b| b.to_string()).collect()),
            "done_criteria" => Block(self.done_criteria.as_deref()?),
            "criteria_by" => Bare(self.criteria_by?.as_str().to_string()),
            "verify" => {
                if self.verify.is_empty() {
                    return None;
                }
                Flow(self.verify.clone())
            }
            "proof" => {
                if self.proof.is_empty() {
                    return None;
                }
                Proofs(&self.proof)
            }
            "verified" => {
                if self.verified.is_empty() {
                    return None;
                }
                Readings(&self.verified)
            }
            "schema" => Bare(self.schema.to_string()),
            "version" => Bare(self.version.to_string()),
            other => no_such_field("a task", other),
        })
    }

    fn kind_spec(&self) -> &'static KindSpec {
        by_type_name(self.id.kind().as_str()).expect("the id's kind is in the registry")
    }

    fn body(&self) -> &str {
        &self.body
    }
}

impl Fields for Adr {
    fn field_value(&self, name: &str) -> Option<FieldValue<'_>> {
        use FieldValue::*;
        Some(match name {
            "id" => Bare(self.id.to_string()),
            "type" => Bare(self.id.kind().as_str().to_string()),
            "slug" => Scalar(self.slug.as_deref()?),
            "title" => Scalar(&self.title),
            "created" => Scalar(&self.created),
            "author" => Scalar(self.author.as_deref()?),
            "status" => Bare(self.status.as_str().to_string()),
            "scope" => Seq(&self.scope),
            "constraint" => Block(&self.constraint),
            "see" => Scalar(self.see.as_deref()?),
            "supersedes" => Bare(self.supersedes.as_ref()?.to_string()),
            "ratified" => Scalar(self.ratified.as_deref()?),
            "verified" => {
                if self.verified.is_empty() {
                    return None;
                }
                Readings(&self.verified)
            }
            "schema" => Bare(self.schema.to_string()),
            "version" => Bare(self.version.to_string()),
            other => no_such_field("an adr", other),
        })
    }

    fn kind_spec(&self) -> &'static KindSpec {
        by_type_name(self.id.kind().as_str()).expect("the id's kind is in the registry")
    }

    fn body(&self) -> &str {
        &self.body
    }
}

impl Fields for Spec {
    fn field_value(&self, name: &str) -> Option<FieldValue<'_>> {
        use FieldValue::*;
        Some(match name {
            "id" => Bare(self.id.to_string()),
            "type" => Bare(self.id.kind().as_str().to_string()),
            "slug" => Scalar(self.slug.as_deref()?),
            "title" => Scalar(&self.title),
            "created" => Scalar(&self.created),
            "author" => Scalar(self.author.as_deref()?),
            "status" => Bare(self.status.as_str().to_string()),
            "scope" => Seq(&self.scope),
            "supersedes" => Bare(self.supersedes.as_ref()?.to_string()),
            "ratified" => Scalar(self.ratified.as_deref()?),
            "verified" => {
                if self.verified.is_empty() {
                    return None;
                }
                Readings(&self.verified)
            }
            "schema" => Bare(self.schema.to_string()),
            "version" => Bare(self.version.to_string()),
            other => no_such_field("a spec", other),
        })
    }

    fn kind_spec(&self) -> &'static KindSpec {
        by_type_name(self.id.kind().as_str()).expect("the id's kind is in the registry")
    }

    fn body(&self) -> &str {
        &self.body
    }
}

impl Fields for Log {
    fn field_value(&self, name: &str) -> Option<FieldValue<'_>> {
        use FieldValue::*;
        Some(match name {
            "id" => Bare(self.id.to_string()),
            "type" => Bare(self.id.kind().as_str().to_string()),
            "slug" => Scalar(self.slug.as_deref()?),
            "title" => Scalar(&self.title),
            "created" => Scalar(&self.created),
            "author" => Scalar(self.author.as_deref()?),
            "scope" => Seq(&self.scope),
            // Required, and an entry with no subject is not a case the model
            // can hold: it is what turns the address the previous shape
            // computed into something a reader can look up.
            "about" => Bare(self.about.to_string()),
            // Required, and never inferred from a file name or a directory
            // order: a timestamp alone is not a total order over entries, so
            // the rank is a field or it does not exist (§3).
            "seq" => Bare(self.seq.to_string()),
            "verified" => {
                if self.verified.is_empty() {
                    return None;
                }
                Readings(&self.verified)
            }
            "schema" => Bare(self.schema.to_string()),
            "version" => Bare(self.version.to_string()),
            other => no_such_field("a log entry", other),
        })
    }

    fn kind_spec(&self) -> &'static KindSpec {
        by_type_name(self.id.kind().as_str()).expect("the id's kind is in the registry")
    }

    fn body(&self) -> &str {
        &self.body
    }
}

impl Fields for Entity {
    fn field_value(&self, name: &str) -> Option<FieldValue<'_>> {
        match self {
            Entity::Task(t) => t.field_value(name),
            Entity::Adr(a) => a.field_value(name),
            Entity::Spec(s) => s.field_value(name),
            Entity::Log(l) => l.field_value(name),
        }
    }

    fn kind_spec(&self) -> &'static KindSpec {
        match self {
            Entity::Task(t) => t.kind_spec(),
            Entity::Adr(a) => a.kind_spec(),
            Entity::Spec(s) => s.kind_spec(),
            Entity::Log(l) => l.kind_spec(),
        }
    }

    fn body(&self) -> &str {
        match self {
            Entity::Task(t) => t.body(),
            Entity::Adr(a) => a.body(),
            Entity::Spec(s) => s.body(),
            Entity::Log(l) => l.body(),
        }
    }
}
