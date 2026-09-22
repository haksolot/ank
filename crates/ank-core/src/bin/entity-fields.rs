//! Prints the entity field reference page, `docs/entity-fields.md`, from the
//! kind registry and the model (ADR-2b62b9a1fe67):
//!
//!     cargo run -q -p ank-core --bin entity-fields > docs/entity-fields.md
//!
//! The workspace suite runs this binary and fails when the committed page
//! differs from what it prints.

fn main() {
    print!("{}", ank_core::reference::entity_fields_page());
}
