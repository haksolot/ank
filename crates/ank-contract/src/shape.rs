//! The shape of the document a verb returns (ADR-6fd69efb629c).
//!
//! `ank help --json` already told a client what it could *send* — the verbs, the
//! flags, the refusals and their codes. What it could not tell it is what comes
//! *back*, and that is the half a typed client needs most: a caller can discover
//! the flags by being refused, and cannot discover a field by being given it,
//! because it has to know the name before it can look.
//!
//! **Declared, and rendered from the declaration.** The description is generated
//! out of this rather than written beside the code, for the reason the verb table
//! is: a description maintained by hand is a description that will disagree with
//! the code, and the disagreement lands on whoever wrote the client, days later,
//! as a bug they cannot see from their side.
//!
//! **A declaration alone would still drift**, so it does not stand alone: the
//! conformance test walks every fixture in `tests/golden-json/` against the
//! declaration for its verb, and a shape that changes without its fixture
//! changing — or a fixture that changes without its declaration — is a failing
//! test. That is ADR-6fd69efb629c's clause, and it is what makes this data a
//! contract rather than documentation.
//!
//! **Deliberately not JSON Schema.** What the documents need is names, types,
//! nullability and nesting; a schema language would also bring validation
//! vocabulary nothing here emits, and a dependency to read it. §13 spends a
//! dependency only on necessity, and this is four `const` types.

/// One document a verb can return.
///
/// A verb usually has one, and three have two — because they genuinely answer
/// two different questions and say so with two different documents, not because
/// a field wanders. `config <key>` reads and `config <key> <value>` writes;
/// `log <id>` reads and `log <id> <message>` appends; `show` over a task carries
/// the `blocked_by` edges and over an ADR does not, since a document carrying
/// them empty would be answering a question nobody asked.
///
/// Declaring the union instead, with the differing fields marked absent-able,
/// would have described a document no call ever returns and left the client to
/// work out which halves go together — which is the discovery this whole
/// description exists to spare it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    /// The call that returns it, `None` when the verb returns only one document.
    ///
    /// Prose rather than a flag pattern: what separates two documents is which
    /// question was asked, and the caller already knows which it asked.
    pub when: Option<&'static str>,
    pub fields: &'static [Field],
}

/// The verb's only document.
pub const fn one(fields: &'static [Field]) -> Shape {
    Shape { when: None, fields }
}

/// One of several documents, named by the call that returns it.
pub const fn when(when: &'static str, fields: &'static [Field]) -> Shape {
    Shape {
        when: Some(when),
        fields,
    }
}

/// One field of a document, by name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Field {
    pub name: &'static str,
    pub ty: Type,
    /// Whether the value may be JSON `null`.
    ///
    /// Its own flag rather than a `Type` variant, because it is orthogonal to
    /// the type and load-bearing on its own: `null` and `""` are different
    /// answers, and §4 already says so — an absent branch on a detached HEAD is
    /// not the empty string. A client that treats a nullable string as a string
    /// crashes on the first detached HEAD it meets.
    pub nullable: bool,
}

/// What a field holds.
///
/// Structural and not semantic: `Str` covers an id, a title and a path alike.
/// A type that said "entity id" would be describing the corpus rather than the
/// document, and the corpus has `ank show` for that.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Str,
    /// Any JSON number. The counts are `usize`, the versions `u64` and the exit
    /// codes `i32`, and no client should have to care which.
    Num,
    Bool,
    /// An array of strings.
    Strings,
    /// A nested document with these fields.
    Object(&'static [Field]),
    /// An array of documents, each with these fields.
    Array(&'static [Field]),
}

impl Type {
    /// The word `help --json` prints for it.
    ///
    /// The nested fields are rendered beside it rather than inside this name, so
    /// that a client reads one vocabulary of six words and finds the structure
    /// where structure belongs.
    pub const fn name(self) -> &'static str {
        match self {
            Type::Str => "string",
            Type::Num => "number",
            Type::Bool => "boolean",
            Type::Strings => "string[]",
            Type::Object(_) => "object",
            Type::Array(_) => "object[]",
        }
    }

    /// The fields of a nested shape, empty for a scalar.
    pub const fn fields(self) -> &'static [Field] {
        match self {
            Type::Object(f) | Type::Array(f) => f,
            _ => &[],
        }
    }
}

/// A required field.
pub const fn f(name: &'static str, ty: Type) -> Field {
    Field {
        name,
        ty,
        nullable: false,
    }
}

/// A field whose value may be `null`.
pub const fn opt(name: &'static str, ty: Type) -> Field {
    Field {
        name,
        ty,
        nullable: true,
    }
}
