//! Regenerates the section of `docs/environment.md` that lists every variable
//! the binary reads, from the table in `src/env.rs` (ADR-2b62b9a1fe67):
//!
//!     cargo run -q -p ank-contract --bin environment -- docs/environment.md
//!
//! Given the page, it rewrites the lines between the page's markers and
//! nothing else; given nothing, it prints the section. The workspace suite runs
//! it bare and fails when the committed section differs from what it prints.

use ank_contract::env;

fn main() {
    let Some(path) = std::env::args_os().nth(1) else {
        print!("{}", env::reference_section());
        return;
    };
    let page = match std::fs::read_to_string(&path) {
        Ok(page) => page.replace("\r\n", "\n"),
        Err(e) => {
            eprintln!("{}: {e}", path.to_string_lossy());
            std::process::exit(1);
        }
    };
    let Some(spliced) = env::splice(&page) else {
        eprintln!(
            "{} carries no {:?} .. {:?} section",
            path.to_string_lossy(),
            env::BEGIN.trim_end(),
            env::END.trim_end()
        );
        std::process::exit(1);
    };
    if let Err(e) = std::fs::write(&path, spliced) {
        eprintln!("{}: {e}", path.to_string_lossy());
        std::process::exit(1);
    }
}
