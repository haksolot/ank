//! The reference pages the documentation carries, rendered from the tables
//! they describe and never typed (ADR-2b62b9a1fe67).
//!
//! [`entity_fields_page`] is `docs/entity-fields.md`, from [`crate::registry`]
//! and the enums of [`crate::model`]; [`config_keys_page`] is
//! `docs/config-keys.md`, from [`crate::config`]. A binary of this crate prints
//! each, and `tests/reference_pages.rs` fails when a committed page differs
//! from what its binary prints.

use crate::config::{self, Absent};
use crate::id::ID_HEX_LEN;
use crate::model::{
    records_meaning, ProofType, ProofVia, MIN_SCHEMA, RECORDS_KINDS, SCHEMA_VERSION,
};
use crate::registry::{KindSpec, Values, KINDS};

fn row(cells: &[String]) -> String {
    format!("| {} |\n", cells.join(" | "))
}

fn code(s: &str) -> String {
    format!("`{s}`")
}

fn values(kind: &KindSpec, values: Values) -> String {
    match values {
        Values::Any => String::new(),
        Values::Id => code(&format!("{}<{ID_HEX_LEN} hex>", kind.prefix)),
        Values::Kind => format!("always {}", code(kind.name)),
        // `\|` and not `|`: a bare pipe would end the cell.
        Values::OneOf(words) => words()
            .into_iter()
            .map(code)
            .collect::<Vec<_>>()
            .join(" \\| "),
    }
}

/// `docs/entity-fields.md`.
pub fn entity_fields_page() -> String {
    let mut page = String::from(
        "<!-- Generated from crates/ank-core/src/registry.rs and model.rs; do not edit.\n     \
         Regenerate: cargo run -q -p ank-core --bin entity-fields > docs/entity-fields.md -->\n\n\
         # Entity fields\n\n\
         Every kind, with its fields in canonical order: the kind registry the binary \
         reads and writes with, printed. [The file format](format.md) says what the \
         order and the emission forms mean; this page is the table it describes.\n\n",
    );
    page.push_str(&format!(
        "This build writes schema **{SCHEMA_VERSION}**, and reads schema **{MIN_SCHEMA}** \
         through **{SCHEMA_VERSION}**. A file declaring a newer schema is refused on its \
         version, never on the first field it does not recognise.\n"
    ));
    for kind in KINDS {
        page.push_str(&format!(
            "\n## {}\n\n`type: {}`, ids `{}<{ID_HEX_LEN} hex>`.\n\n",
            kind.title, kind.name, kind.prefix
        ));
        page.push_str("| # | Field | Emission | Presence | Values | Notes |\n");
        page.push_str("|---|---|---|---|---|---|\n");
        for (i, field) in kind.fields.iter().enumerate() {
            page.push_str(&row(&[
                (i + 1).to_string(),
                code(field.name),
                field.form.label().to_string(),
                if field.required {
                    "always emitted"
                } else {
                    "omitted when absent"
                }
                .to_string(),
                values(kind, field.values),
                field.note.to_string(),
            ]));
        }
    }

    page.push_str(
        "\n## `records`\n\n\
         What a log entry a verb wrote records. Absent, the entry is work; a value \
         this build does not know is read as machinery and never refused.\n\n\
         | Value | Records |\n|---|---|\n",
    );
    for word in RECORDS_KINDS {
        page.push_str(&row(&[
            code(word),
            records_meaning(word)
                .expect("every known word has a meaning")
                .to_string(),
        ]));
    }

    page.push_str(
        "\n## Proof `type`\n\n\
         What a proof entry's `ref` points at. A weak type anchors nothing outside \
         the agent's reach, and `check` marks it.\n\n\
         | Value | Trust | Meaning |\n|---|---|---|\n",
    );
    for t in ProofType::ALL {
        page.push_str(&row(&[
            code(t.as_str()),
            if t.is_weak() { "weak" } else { "strong" }.to_string(),
            t.meaning().to_string(),
        ]));
    }

    page.push_str(
        "\n## Proof `via`\n\n\
         The route by which a proof entry arrived. Absent means the entry was written \
         before the field existed, never a fourth route; a `test` entry `submitted` by \
         a caller anchors nothing outside the agent's reach.\n\n\
         | Value | Meaning |\n|---|---|\n",
    );
    for v in ProofVia::ALL {
        page.push_str(&row(&[code(v.as_str()), v.meaning().to_string()]));
    }
    page
}

fn absent(a: Absent) -> String {
    match a {
        Absent::Required => "required".to_string(),
        Absent::None => "none".to_string(),
        Absent::Number(n) => code(&n.to_string()),
        Absent::Text(t) => code(t),
    }
}

/// `docs/config-keys.md`.
pub fn config_keys_page() -> String {
    let mut page = String::from(
        "<!-- Generated from crates/ank-core/src/config.rs; do not edit.\n     \
         Regenerate: cargo run -q -p ank-core --bin config-keys > docs/config-keys.md -->\n\n\
         # config.yml keys\n\n\
         Every key `.ank/config.yml` may carry, with the type of its value and what an \
         absent key means. A key not listed here is refused, and so is a file whose \
         `schema` is newer than this build reads.\n\n",
    );
    page.push_str(&format!(
        "The file declares `schema: {}`, the only version this build reads. A duration \
         is `<n><unit>`, the unit one of `s`, `m`, `h` or `d`. `<name>` stands for a key \
         the file chooses; `ank config <key>` reads and writes the scalar keys, and \
         `roles` and `identities` are edited by hand.\n\n",
        config::SCHEMA
    ));
    page.push_str("| Key | Type | Default | Notes |\n|---|---|---|---|\n");
    for key in config::KEYS {
        page.push_str(&row(&[
            code(key.path),
            key.ty.to_string(),
            absent(key.default),
            key.note.to_string(),
        ]));
    }
    page
}
