//! Prints the config.yml reference page, `docs/config-keys.md`, from the
//! config schema table (ADR-2b62b9a1fe67):
//!
//!     cargo run -q -p ank-core --bin config-keys > docs/config-keys.md
//!
//! The workspace suite runs this binary and fails when the committed page
//! differs from what it prints.

fn main() {
    print!("{}", ank_core::reference::config_keys_page());
}
