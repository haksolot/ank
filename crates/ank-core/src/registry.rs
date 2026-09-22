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
//! table that `docs/entity-fields.md` is printed from, and nowhere else.
//!
//! What this registry deliberately does not do is make the format permissive.
//! An unknown field inside a known kind is still rejected, and an unknown kind
//! is rejected by name. A kind is cheap to add; nothing else moved.

use crate::model::{
    Adr, AdrStatus, CriteriaBy, Entity, Log, Proof, Spec, Task, TaskStatus, Verified, RECORDS_KINDS,
};

/// One field, at its canonical position.
///
/// Everything the reference page says about a field is a column of this row,
/// so the page is rendered and never typed (ADR-2b62b9a1fe67): the
/// `entity-fields` binary walks [`KINDS`] into `docs/entity-fields.md`.
pub struct FieldSpec {
    pub name: &'static str,
    /// `true` means **always emitted**: `blocked_by` is required and is written
    /// `[]` when empty. `false` means omitted when absent, never emitted empty
    /// — which is what lets a file written before a field existed survive a
    /// rewrite unchanged.
    pub required: bool,
    /// How the value is written. [`FieldValue`] is what the serializer
    /// actually receives; `tests/reference_pages.rs` holds the two to one
    /// answer over every golden fixture.
    pub form: Form,
    /// What the value may be, where the model closes the set.
    pub values: Values,
    /// What a reader needs beyond the columns above, in one line.
    pub note: &'static str,
}

/// The emission form of a field, as the reference page names it. One label
/// per [`FieldValue`] variant, except that an integer is written bare and is
/// named apart because a reader parses it apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Form {
    Bare,
    Integer,
    Scalar,
    Block,
    Flow,
    Seq,
    Maps,
}

impl Form {
    pub fn label(self) -> &'static str {
        match self {
            Form::Bare => "bare",
            Form::Integer => "integer",
            Form::Scalar => "scalar",
            Form::Block => "literal block",
            Form::Flow => "flow list",
            Form::Seq => "block sequence",
            Form::Maps => "block sequence of maps",
        }
    }
}

/// The set a field's value is drawn from, when the format closes one.
#[derive(Clone, Copy)]
pub enum Values {
    /// Free, within its form.
    Any,
    /// The kind's own prefix, then [`crate::id::ID_HEX_LEN`] hex characters.
    Id,
    /// The kind's own name, always.
    Kind,
    /// One of these words, read off the model's enum.
    OneOf(fn() -> Vec<&'static str>),
}

use Form::*;

const fn req(name: &'static str, form: Form, note: &'static str) -> FieldSpec {
    FieldSpec {
        name,
        required: true,
        form,
        values: Values::Any,
        note,
    }
}

const fn opt(name: &'static str, form: Form, note: &'static str) -> FieldSpec {
    FieldSpec {
        name,
        required: false,
        form,
        values: Values::Any,
        note,
    }
}

impl FieldSpec {
    const fn values(mut self, values: Values) -> FieldSpec {
        self.values = values;
        self
    }
}

fn task_statuses() -> Vec<&'static str> {
    TaskStatus::ALL.iter().map(|s| s.as_str()).collect()
}

fn adr_statuses() -> Vec<&'static str> {
    AdrStatus::ALL.iter().map(|s| s.as_str()).collect()
}

fn criteria_by() -> Vec<&'static str> {
    CriteriaBy::ALL.iter().map(|c| c.as_str()).collect()
}

fn records() -> Vec<&'static str> {
    RECORDS_KINDS.to_vec()
}

pub struct KindSpec {
    /// The value of the `type` field.
    pub name: &'static str,
    /// What the reference page calls the kind, as its heading.
    pub title: &'static str,
    /// The id prefix, `TASK-` and the like, trailing dash included.
    pub prefix: &'static str,
    /// Every field, in canonical order.
    pub fields: &'static [FieldSpec],
}

static TASK_FIELDS: &[FieldSpec] = &[
    req("id", Bare, "").values(Values::Id),
    req("type", Bare, "").values(Values::Kind),
    opt("slug", Scalar, "cosmetic, never resolved on"),
    req("title", Scalar, ""),
    req(
        "created",
        Scalar,
        "ISO 8601, always UTC with the `Z` suffix",
    ),
    opt(
        "author",
        Scalar,
        "a typed actor; absent means the entity predates the field",
    ),
    req("status", Bare, "").values(Values::OneOf(task_statuses)),
    req("scope", Seq, "globs, never empty"),
    req("blocked_by", Flow, "task ids, `[]` when empty"),
    opt("done_criteria", Block, "frozen by hash at claim"),
    opt("criteria_by", Bare, "invalid without `done_criteria`").values(Values::OneOf(criteria_by)),
    opt("verify", Flow, "verifier names `config.yml` declares"),
    opt("method", Scalar, "one sibling skill the binary carries"),
    opt(
        "proof",
        Maps,
        "keys in order: `type`, `ref`, `tree`, `criteria`, `verifier`, `via`",
    ),
    opt(
        "verified",
        Maps,
        "readings: `by`, then `at`, both required in an entry",
    ),
    req("schema", Integer, ""),
    req("version", Integer, ""),
];

static ADR_FIELDS: &[FieldSpec] = &[
    req("id", Bare, "").values(Values::Id),
    req("type", Bare, "").values(Values::Kind),
    opt("slug", Scalar, "cosmetic, never resolved on"),
    req("title", Scalar, ""),
    req(
        "created",
        Scalar,
        "ISO 8601, always UTC with the `Z` suffix",
    ),
    opt(
        "author",
        Scalar,
        "a typed actor; absent means the entity predates the field",
    ),
    req("status", Bare, "").values(Values::OneOf(adr_statuses)),
    req("scope", Seq, "globs, never empty"),
    req(
        "constraint",
        Block,
        "binding on every scope it covers once accepted",
    ),
    opt("see", Scalar, "reference code the constraint points at"),
    opt("supersedes", Bare, "an entity id"),
    opt("ratified", Scalar, "the signed commit `accept` wrote"),
    opt(
        "verified",
        Maps,
        "readings: `by`, then `at`, both required in an entry",
    ),
    req("schema", Integer, ""),
    req("version", Integer, ""),
];

/// A spec is an ADR's table without `constraint` and without `see`, and the
/// first absence is the whole justification for the kind (§3): a spec
/// describes, an ADR binds. `see` goes with it — it exists to point at the
/// reference code a positive *constraint* needs, and a kind with no constraint
/// has nothing for it to serve.
///
/// `references` takes the position `blocked_by` takes on a task — immediately
/// after the perimeter, before the succession — because it is the same shape of
/// thing: a declared dependency, resolved locally. It is **optional** where
/// `blocked_by` is required, and the asymmetry is a decision: a task always
/// states whether it has blockers, while a document that cites nothing has
/// nothing to state, and emitting `[]` on every spec written before the field
/// existed would make each of them non-canonical at the release that added it.
static SPEC_FIELDS: &[FieldSpec] = &[
    req("id", Bare, "").values(Values::Id),
    req("type", Bare, "").values(Values::Kind),
    opt("slug", Scalar, "cosmetic, never resolved on"),
    req("title", Scalar, ""),
    req(
        "created",
        Scalar,
        "ISO 8601, always UTC with the `Z` suffix",
    ),
    opt(
        "author",
        Scalar,
        "a typed actor; absent means the entity predates the field",
    ),
    req("status", Bare, "").values(Values::OneOf(adr_statuses)),
    req(
        "scope",
        Seq,
        "globs, never empty; what the document governs",
    ),
    opt("references", Flow, "entity ids"),
    opt("supersedes", Bare, "an entity id"),
    opt(
        "ratified",
        Scalar,
        "the signed commit `accept` wrote, over the body and `scope`",
    ),
    opt(
        "verified",
        Maps,
        "readings: `by`, then `at`, both required in an entry",
    ),
    req("schema", Integer, ""),
    req("version", Integer, ""),
];

/// A log entry carries `about` and `seq` and **no `status`**: an entry is
/// written once and has nothing to transition to, so a status would have one
/// legal value and would only ever be copied (§3). `about` takes the position
/// `status` and `scope` leave, immediately after the scope it is a statement
/// about, and `seq` follows it because it ranks the entry among *that* entity's
/// entries and means nothing without it.
static LOG_FIELDS: &[FieldSpec] = &[
    req("id", Bare, "").values(Values::Id),
    req("type", Bare, "").values(Values::Kind),
    opt("slug", Scalar, "cosmetic, never resolved on"),
    req("title", Scalar, "the message, or its head"),
    req(
        "created",
        Scalar,
        "ISO 8601, always UTC with the `Z` suffix; the instant of the entry",
    ),
    opt("author", Scalar, "a typed actor; who wrote the entry"),
    req("scope", Seq, "the subject's scope as it stood"),
    req("about", Bare, "an entity id of any kind"),
    req("seq", Integer, "rank among that entity's entries, from 0"),
    opt(
        "records",
        Scalar,
        "absent is work; a value unknown to the reader is read as machinery",
    )
    .values(Values::OneOf(records)),
    opt(
        "verified",
        Maps,
        "readings: `by`, then `at`, both required in an entry",
    ),
    req("schema", Integer, ""),
    req("version", Integer, "above 1 means the entry was rewritten"),
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
        title: "Task",
        prefix: "TASK-",
        fields: TASK_FIELDS,
    },
    KindSpec {
        name: "adr",
        title: "ADR",
        prefix: "ADR-",
        fields: ADR_FIELDS,
    },
    KindSpec {
        name: "spec",
        title: "Spec",
        prefix: "SPEC-",
        fields: SPEC_FIELDS,
    },
    KindSpec {
        name: "log",
        title: "Log entry",
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
            // A scalar and not bare: every name a binary carries is written
            // bare by it anyway, and a value no binary would write, arriving by
            // hand, is quoted rather than turned into different YAML.
            "method" => Scalar(self.method.as_deref()?),
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
            // Omitted when empty, never written `[]`: see the table above for
            // why this one does not follow `blocked_by`.
            "references" => {
                if self.references.is_empty() {
                    return None;
                }
                Flow(self.references.iter().map(|r| r.to_string()).collect())
            }
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
            // Absent is a work entry, so an entry that records nothing but work
            // is written exactly as it was before this field existed.
            "records" => Scalar(self.records.as_deref()?),
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
