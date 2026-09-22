//! Every environment variable the binary reads at run time, and what it
//! changes (ADR-2b62b9a1fe67, TASK-28290de150ac).
//!
//! **One table, and the call sites read their names from it.** A name typed at
//! the read and again on the page is two copies that drift apart without a
//! sound; here the read names a constant of this module and the page's section
//! is rendered from [`VARIABLES`] by the `environment` binary, so a variable
//! the binary starts reading is a row the page gains, and
//! `tests/environment_page.rs` fails on a page that was not regenerated or on a
//! read that spells its name as a literal somewhere else.
//!
//! What is not here: `ANK_COMMIT`, `ANK_SKILL` and `ANK_RELEASED_SCHEMA` are
//! read by the build through `env!`, never at run time, and a variable ank
//! only *sets* for a child process it starts is the child's to document.

/// One variable the binary reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Variable {
    /// The name, exactly as the environment spells it.
    pub name: &'static str,
    /// Which surface reads it, in the words a reader would look for.
    pub read_by: &'static str,
    /// What setting it changes, and what its absence means.
    pub changes: &'static str,
    /// `false` for an instrument the test suite sets to observe the binary: a
    /// release may change or remove it without notice.
    pub interface: bool,
}

pub const ANK_AGENT: &str = "ANK_AGENT";
pub const USERNAME: &str = "USERNAME";
pub const USER: &str = "USER";
pub const LOGNAME: &str = "LOGNAME";
pub const COMPUTERNAME: &str = "COMPUTERNAME";
pub const HOSTNAME: &str = "HOSTNAME";
pub const ANK_UPDATE_REPOSITORY: &str = "ANK_UPDATE_REPOSITORY";
pub const NO_COLOR: &str = "NO_COLOR";
pub const TERM: &str = "TERM";
pub const WT_SESSION: &str = "WT_SESSION";
pub const TERM_PROGRAM: &str = "TERM_PROGRAM";
pub const CON_EMU_ANSI: &str = "ConEmuANSI";
pub const ANSICON: &str = "ANSICON";
pub const EDITOR: &str = "EDITOR";
pub const APPDATA: &str = "APPDATA";
pub const XDG_CONFIG_HOME: &str = "XDG_CONFIG_HOME";
pub const HOME: &str = "HOME";
pub const PATH: &str = "PATH";
pub const PATHEXT: &str = "PATHEXT";
pub const ANK_INDEX_BUSY_MS: &str = "ANK_INDEX_BUSY_MS";
pub const ANK_INDEX_STEPS: &str = "ANK_INDEX_STEPS";
pub const ANK_INDEX_REFRESHED: &str = "ANK_INDEX_REFRESHED";
pub const ANK_TRACE_READS: &str = "ANK_TRACE_READS";

/// The names the fallback identity takes its `<user>` from, first set wins.
pub const USER_NAMES: [&str; 3] = [USERNAME, USER, LOGNAME];

/// The names the fallback identity takes its `<hostname>` from, first set wins.
pub const HOST_NAMES: [&str; 2] = [COMPUTERNAME, HOSTNAME];

/// On Windows, any one of these set says the console renders escape sequences.
pub const WINDOWS_TERMINALS: [&str; 5] = [WT_SESSION, TERM, TERM_PROGRAM, CON_EMU_ANSI, ANSICON];

/// The table, in the order the page shows it.
pub const VARIABLES: &[Variable] = &[
    Variable {
        name: ANK_AGENT,
        read_by: "every verb, and `ank mcp`",
        changes: "the identity this session acts as: who holds a claim, who wrote an entity, \
                  who ran `done`. Unset or blank, `<user>@<hostname>`, and `ank mcp` writes \
                  under `ank-mcp/<version>`",
        interface: true,
    },
    Variable {
        name: USERNAME,
        read_by: "every verb, with `ANK_AGENT` unset",
        changes: "the `<user>` of the fallback identity; the first of `USERNAME`, `USER`, \
                  `LOGNAME` set and not blank wins, and none gives `unknown`",
        interface: true,
    },
    Variable {
        name: USER,
        read_by: "every verb, with `ANK_AGENT` unset",
        changes: "the `<user>` of the fallback identity, when `USERNAME` gives none",
        interface: true,
    },
    Variable {
        name: LOGNAME,
        read_by: "every verb, with `ANK_AGENT` unset",
        changes: "the `<user>` of the fallback identity, when `USERNAME` and `USER` give none",
        interface: true,
    },
    Variable {
        name: COMPUTERNAME,
        read_by: "every verb, with `ANK_AGENT` unset",
        changes: "the `<hostname>` of the fallback identity, cut at its first dot and \
                  lowercased; the first of `COMPUTERNAME`, `HOSTNAME` set wins, and none \
                  asks the `hostname` program, then says `localhost`",
        interface: true,
    },
    Variable {
        name: HOSTNAME,
        read_by: "every verb, with `ANK_AGENT` unset",
        changes: "the `<hostname>` of the fallback identity, when `COMPUTERNAME` gives none",
        interface: true,
    },
    Variable {
        name: ANK_UPDATE_REPOSITORY,
        read_by: "`ank update`",
        changes: "the repository release tags are read from, in place of \
                  `https://github.com/haksolot/ank`; empty counts as unset",
        interface: true,
    },
    Variable {
        name: NO_COLOR,
        read_by: "every verb at a terminal, and `ank tui`",
        changes: "set and not empty, takes the colour and nothing else; the empty value is \
                  not an opt-out",
        interface: true,
    },
    Variable {
        name: TERM,
        read_by: "every verb at a terminal, and `ank tui`",
        changes: "`dumb` takes the colour, as `NO_COLOR=1` does, and draws the structure of \
                  `ank tui` in ASCII; on Windows, set at all, it says the console renders \
                  escape sequences",
        interface: true,
    },
    Variable {
        name: WT_SESSION,
        read_by: "every verb at a Windows terminal",
        changes: "set, says the console renders escape sequences; with none of `WT_SESSION`, \
                  `TERM`, `TERM_PROGRAM`, `ConEmuANSI`, `ANSICON` set, the output is plain",
        interface: true,
    },
    Variable {
        name: TERM_PROGRAM,
        read_by: "every verb at a Windows terminal",
        changes: "set, says the console renders escape sequences",
        interface: true,
    },
    Variable {
        name: CON_EMU_ANSI,
        read_by: "every verb at a Windows terminal",
        changes: "set, says the console renders escape sequences",
        interface: true,
    },
    Variable {
        name: ANSICON,
        read_by: "every verb at a Windows terminal",
        changes: "set, says the console renders escape sequences",
        interface: true,
    },
    Variable {
        name: EDITOR,
        read_by: "`ank edit`",
        changes: "the editor `ank edit <id>` opens when given no field to change; unset or \
                  blank, the verb refuses at exit 9",
        interface: true,
    },
    Variable {
        name: APPDATA,
        read_by: "`ank config --user`, `--repo`, `ank mcp`, `ank watch`, `ank tui`",
        changes: "on Windows, the reader's configuration directory is `%APPDATA%\\ank`; \
                  unset, a verb that needs it refuses at exit 9",
        interface: true,
    },
    Variable {
        name: XDG_CONFIG_HOME,
        read_by: "`ank config --user`, `--repo`, `ank mcp`, `ank watch`, `ank tui`",
        changes: "elsewhere than Windows, the reader's configuration directory is \
                  `$XDG_CONFIG_HOME/ank`; empty counts as unset",
        interface: true,
    },
    Variable {
        name: HOME,
        read_by: "`ank config --user`, `--repo`, `ank mcp`, `ank watch`, `ank tui`",
        changes: "with `XDG_CONFIG_HOME` unset, the reader's configuration directory is \
                  `$HOME/.config/ank`; neither set, a verb that needs it refuses at exit 9",
        interface: true,
    },
    Variable {
        name: PATH,
        read_by: "`ank done`, `ank skills --install`, `ank update`",
        changes: "where `sh` and `git`, `npx`, and `npm`, `powershell`, `pwsh` and `curl` \
                  are looked for; a program found on none of its directories is refused by \
                  name",
        interface: true,
    },
    Variable {
        name: PATHEXT,
        read_by: "`ank skills --install`, `ank update`, on Windows",
        changes: "the extensions tried on `PATH`, in its order, of `.COM`, `.EXE`, `.BAT`, \
                  `.CMD`; unset, those four",
        interface: true,
    },
    Variable {
        name: ANK_INDEX_BUSY_MS,
        read_by: "every verb that opens the index",
        changes: "how long, in milliseconds, a connection waits on an index another process \
                  holds locked; unset, five seconds",
        interface: false,
    },
    Variable {
        name: ANK_INDEX_STEPS,
        read_by: "every verb that writes the index",
        changes: "a file the SQLite steps a refresh executed are written to",
        interface: false,
    },
    Variable {
        name: ANK_INDEX_REFRESHED,
        read_by: "every verb that opens the index",
        changes: "a file every refresh appends what it hashed and reindexed to",
        interface: false,
    },
    Variable {
        name: ANK_TRACE_READS,
        read_by: "every verb that reads the corpus",
        changes: "an absolute path every entity parse and every index opening appends a line to",
        interface: false,
    },
];

/// The generated section of `docs/environment.md`, the lines between its
/// markers.
pub fn reference_section() -> String {
    let mut out = String::from(
        "<!-- Generated from crates/ank-contract/src/env.rs; do not edit between the markers. -->\n\n\
         ## Every variable\n\n\
         | Variable | Read by | What it changes |\n\
         |---|---|---|\n",
    );
    let row = |v: &Variable| format!("| `{}` | {} | {} |\n", v.name, v.read_by, v.changes);
    for v in VARIABLES.iter().filter(|v| v.interface) {
        out.push_str(&row(v));
    }
    out.push_str(
        "\nThe test suite sets these to observe the binary. They are not an interface, and a \
         release may change or remove any of them without notice:\n\n\
         | Variable | Read by | What it changes |\n\
         |---|---|---|\n",
    );
    for v in VARIABLES.iter().filter(|v| !v.interface) {
        out.push_str(&row(v));
    }
    out
}

/// The page with its generated section replaced by [`reference_section`], or
/// `None` when the page carries no pair of markers.
pub fn splice(page: &str) -> Option<String> {
    let begin = page.find(BEGIN)? + BEGIN.len();
    let end = page[begin..].find(END)? + begin;
    Some(format!(
        "{}{}{}",
        &page[..begin],
        reference_section(),
        &page[end..]
    ))
}

/// The line opening the generated section of the page.
pub const BEGIN: &str = "<!-- BEGIN environment -->\n";
/// The line closing it.
pub const END: &str = "<!-- END environment -->\n";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splice_replaces_what_lies_between_the_markers_and_nothing_else() {
        let page = format!("head\n{BEGIN}stale\n{END}tail\n");
        assert_eq!(
            splice(&page).unwrap(),
            format!("head\n{BEGIN}{}{END}tail\n", reference_section())
        );
        assert_eq!(splice("no markers\n"), None);
    }

    #[test]
    fn every_name_appears_once() {
        let mut names: Vec<&str> = VARIABLES.iter().map(|v| v.name).collect();
        names.sort_unstable();
        let before = names.len();
        names.dedup();
        assert_eq!(before, names.len());
    }
}
