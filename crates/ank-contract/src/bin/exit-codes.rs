//! Prints the exit-code reference page, `docs/exit-codes.md`, from the table in
//! `src/exit.rs` (ADR-2b62b9a1fe67):
//!
//!     cargo run -q -p ank-contract --bin exit-codes > docs/exit-codes.md
//!
//! The workspace suite runs this binary and fails when the committed page
//! differs from what it prints.

fn main() {
    print!("{}", ank_contract::exit::reference_page());
}
