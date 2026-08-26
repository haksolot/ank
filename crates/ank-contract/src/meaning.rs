//! What a status, a kind or a severity means, as one table (ADR-1f70ce2c3eac).
//!
//! Two surfaces paint this corpus — the CLI in hand-written ANSI, the terminal
//! reader through the library it draws with — and neither may decide for itself
//! what `done` looks like. ADR-1f70ce2c3eac allows the two renderers and forbids
//! the second table, so what is shared is the **meaning**: `done` is an
//! accomplishment, `proposed` is waiting on somebody, a fault is a fault. Roles,
//! never colours.
//!
//! **No escape sequence reaches this crate, and that is the decision rather than
//! an omission.** Shipping SGR codes here would make every consumer carry bytes
//! most of them never emit, to share the one thing the two surfaces genuinely do
//! differently — and it would put the palette where `ank-cli`'s own guard, which
//! reads `ank-cli/src` and no further, cannot see it. The rule is asserted here
//! too, over this crate's own sources, by `the_table_carries_no_escape_sequence`
//! below.
//!
//! **Three families in one table and not three tables.** They are keyed on
//! different names and answer the same question, and one `find` listing prints
//! rows of several kinds side by side: a reader should not have to know which
//! family a row belongs to before knowing what its rendering means. That is the
//! argument SPEC-89070ce7f3b8 already makes for reading a task's statuses and an
//! ADR's out of one table, applied one level up.
//!
//! **Nothing here renders.** A surface reads a [`Role`] and paints it its own
//! way; this crate has no opinion about how, and could not express one.

/// The family of names a row is keyed on.
///
/// Part of the key rather than a comment on it. The families are open — a kind
/// is whatever the registry declares, and a severity is whatever `check`
/// attaches — so two of them are one name apart from colliding, and a lookup
/// that did not say what the name was a name *of* would resolve the collision
/// by declaration order, in silence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Subject {
    /// A status an entity is stored carrying, or the word a ref answers with.
    /// Both, because they are one fact seen twice: `in_progress` is what the
    /// file says and `claimed` is what the claim says, and a reader who saw them
    /// in two renderings would have to learn that they are the same state.
    Status,
    /// An entity kind, as the registry names it.
    Kind,
    /// What `check` attaches to a finding.
    Severity,
}

/// The part a subject plays, which is the whole of what a surface is told.
///
/// Named for what the reader is being told and never for a colour. The CLI
/// paints [`Role::Attention`] yellow; a reader that owns the whole screen may
/// reasonably paint it something else against a background it chose, and a
/// variant called `Yellow` would have made that a contradiction instead of a
/// choice. It is also what keeps this file honest: a role has no rendering to
/// leak into it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Role {
    /// There to be taken.
    Available,
    /// Somebody is on it.
    Underway,
    /// It landed.
    Accomplished,
    /// Given up or retired, and neither is a failure. A release is how the loop
    /// is meant to end when a criterion turns out to be wrong, so this role is
    /// deliberately not [`Role::Fault`]: rendering it as an error would teach an
    /// agent to avoid the honest move.
    Retired,
    /// Legal, complete, and waiting on a human. `accept` is nobody else's act.
    Awaiting,
    /// Worth a look and not a defect: a claim whose lease ran out, a `signal`.
    Attention,
    /// A defect in the corpus.
    Fault,
    /// A name to address the entity by, whatever kind it names.
    ///
    /// One row per kind, all of them landing here, and the repetition is the
    /// point: `find` and `scope` list the kinds side by side and
    /// SPEC-89070ce7f3b8 fixes an identifier's rendering for all of them at
    /// once. A surface that wanted `ADR-` to read differently from `TASK-` has
    /// to remove rows from this table to get it, rather than add a call site
    /// nobody reviews.
    Identifier,
}

/// One row: a name, the family it is a name in, and the part it plays.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Meaning {
    pub subject: Subject,
    pub name: &'static str,
    pub role: Role,
}

const fn row(subject: Subject, name: &'static str, role: Role) -> Meaning {
    Meaning {
        subject,
        name,
        role,
    }
}

/// The qualifier a marker carries when the lease behind it ran out.
///
/// Not a status: it is appended to whichever status the claim expired *from*,
/// which is why [`role_of_status`] tests for it before it looks anything up.
pub const EXPIRED: &str = "expired";

/// The table (ADR-1f70ce2c3eac).
///
/// Grouped by family and written out row by row. A derivation — every kind in
/// the registry mapped to [`Role::Identifier`] by a loop, say — would have been
/// shorter and would have made adding a kind a decision nobody takes: the row is
/// where the question gets asked.
///
/// **`blocked` is absent, and its absence is the same decision.** It is derived
/// from `blocked_by` at read time and no entity is ever stored carrying it
/// (SPEC-89070ce7f3b8), so nothing prints it and there is nothing for a surface
/// to look up. A row here would have invented a status the model does not have.
pub const MEANINGS: &[Meaning] = &[
    // Statuses, and every one the model can hold is here: a task is `open`,
    // `in_progress`, `done` or `closed`; an ADR or a spec is `proposed`,
    // `accepted` or `superseded`.
    row(Subject::Status, "open", Role::Available),
    row(Subject::Status, "in_progress", Role::Underway),
    row(Subject::Status, "claimed", Role::Underway),
    row(Subject::Status, "done", Role::Accomplished),
    row(Subject::Status, "finished", Role::Accomplished),
    row(Subject::Status, "closed", Role::Retired),
    row(Subject::Status, "proposed", Role::Awaiting),
    row(Subject::Status, "accepted", Role::Accomplished),
    row(Subject::Status, "superseded", Role::Retired),
    row(Subject::Status, EXPIRED, Role::Attention),
    // Kinds, by the `type` name the registry gives each of them. Written out
    // rather than read from the registry because this crate has no dependencies
    // and must not gain one for a palette: every surface consumes it, so
    // anything it depends on is depended on by all of them at once.
    row(Subject::Kind, "task", Role::Identifier),
    row(Subject::Kind, "adr", Role::Identifier),
    row(Subject::Kind, "spec", Role::Identifier),
    row(Subject::Kind, "log", Role::Identifier),
    // Severities, as `check` attaches them.
    row(Subject::Severity, "fault", Role::Fault),
    row(Subject::Severity, "signal", Role::Attention),
];

/// The role the table gives a name in one family, or `None` when it declares
/// none.
///
/// `None` is an answer and not a default. It is what a string that is not a
/// name of that family gets, which is how a surface can leave such a string
/// alone instead of rendering it as something it is not.
pub fn role_of(subject: Subject, name: &str) -> Option<Role> {
    MEANINGS
        .iter()
        .find(|m| m.subject == subject && m.name == name)
        .map(|m| m.role)
}

/// The role a status carries, given the word a listing or a ref answers with.
///
/// Two rules stand between the word and the lookup, and both are here rather
/// than at a call site so that two surfaces cannot apply them differently.
///
/// **[`EXPIRED`] wins over whatever it expired from.** The marker is
/// `open expired:who@host` or `done expired:who@host`, and what the reader has
/// to see is the lapse rather than the state it lapsed from.
///
/// **What follows a colon is addressing.** `claimed:who@host` and
/// `finished:abc1234 on main` name a holder and a landing place; the status is
/// the word before the colon, and it is the whole of what this answers about.
pub fn role_of_status(state: &str) -> Option<Role> {
    if state.contains(EXPIRED) {
        return role_of(Subject::Status, EXPIRED);
    }
    role_of(Subject::Status, state.split(':').next().unwrap_or(state))
}

/// The role a bracketed marker carries: `[done]`, `[claimed:who@host]`.
///
/// The same table [`role_of_status`] reads, reached through the brackets a
/// listing prints. `-> done` and `[done]` are the same fact seen twice and §4
/// asks that a reader who has learned one have learned the other, which is only
/// true while one lookup answers both.
pub fn role_of_marker(marker: &str) -> Option<Role> {
    role_of_status(marker.trim_start_matches('[').trim_end_matches(']'))
}

/// The role an entity kind carries, given the `type` the registry names it by.
pub fn role_of_kind(kind: &str) -> Option<Role> {
    role_of(Subject::Kind, kind)
}

/// The role a finding's severity carries: `fault`, `signal`.
pub fn role_of_severity(severity: &str) -> Option<Role> {
    role_of(Subject::Severity, severity)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The clause of ADR-1f70ce2c3eac this crate exists to keep: the meaning
    /// travels and the escape sequences do not.
    ///
    /// Over the whole crate rather than over this file, because the constraint
    /// is about where a palette may live and not about which module holds the
    /// table. `ank-cli` has the mirror of this test in `paint.rs`, walking its
    /// own `src`; between them the two directories that could plausibly grow a
    /// second palette are both covered.
    ///
    /// **The needles are derived from the escape byte and never written out**,
    /// so a grep for an escape sequence over this crate finds nothing at all —
    /// including in the test that forbids it, which is what makes the
    /// criterion's own phrasing checkable by hand. `27` is ESC, written in
    /// decimal for exactly that reason.
    #[test]
    fn the_table_carries_no_escape_sequence() {
        let src = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let byte = char::from(27u8);
        let hex = format!("\\x{:02x}", byte as u32);
        let unicode = format!("\\u{{{:x}}}", byte as u32);
        let mut walked = 0;
        for entry in std::fs::read_dir(&src).expect("src is readable") {
            let path = entry.expect("entry").path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("source is readable");
            for needle in [hex.as_str(), unicode.as_str()] {
                assert!(
                    !text.contains(needle),
                    "{} writes an escape sequence; a role has no rendering and \
                     this crate has no palette",
                    path.display()
                );
            }
            assert!(
                !text.contains(byte),
                "{} carries a raw escape byte",
                path.display()
            );
            walked += 1;
        }
        assert!(
            walked >= 7,
            "the walk read {walked} sources and gave up early"
        );
    }

    /// One name may mean one thing in one family, and the table says which.
    #[test]
    fn no_two_rows_share_a_key() {
        for (i, m) in MEANINGS.iter().enumerate() {
            for other in &MEANINGS[i + 1..] {
                assert!(
                    !(m.subject == other.subject && m.name == other.name),
                    "{:?} {} is declared twice",
                    m.subject,
                    m.name
                );
            }
        }
    }

    /// A name is looked up in the family it belongs to and in no other.
    ///
    /// The property [`Subject`] is in the key for: `open` is a status, and a
    /// surface asking the table what kind of entity an `open` is must be told
    /// nothing rather than told the first row that happens to match.
    #[test]
    fn a_family_answers_only_about_its_own_names() {
        for m in MEANINGS {
            for subject in [Subject::Status, Subject::Kind, Subject::Severity] {
                let answered = role_of(subject, m.name);
                if subject == m.subject {
                    assert_eq!(answered, Some(m.role), "{} lost its row", m.name);
                } else {
                    assert_eq!(
                        answered, None,
                        "{:?} answered about {}, which is a {:?}",
                        subject, m.name, m.subject
                    );
                }
            }
        }
    }

    /// The statuses of SPEC-89070ce7f3b8, enumerated rather than sampled.
    ///
    /// Written out here rather than derived from `MEANINGS`, which would agree
    /// with any table including one nobody meant. `ank-core` is not a dependency
    /// of this crate and must not become one, so what pins these against the
    /// model's own enumerations is the test in `ank-cli/src/style.rs` that walks
    /// `TaskStatus` and `AdrStatus` through a match.
    #[test]
    fn the_statuses_are_the_ones_the_specification_declares() {
        for (state, role) in [
            ("open", Role::Available),
            ("in_progress", Role::Underway),
            ("claimed", Role::Underway),
            ("done", Role::Accomplished),
            ("finished", Role::Accomplished),
            ("closed", Role::Retired),
            ("proposed", Role::Awaiting),
            ("accepted", Role::Accomplished),
            ("superseded", Role::Retired),
        ] {
            assert_eq!(role_of_status(state), Some(role), "{state} moved");
            assert_eq!(
                role_of_marker(&format!("[{state}]")),
                Some(role),
                "the marker of {state} disagrees with the word"
            );
        }
    }

    /// The addressing after a colon names a holder, never a status.
    #[test]
    fn a_payload_is_not_part_of_the_status() {
        assert_eq!(
            role_of_status("claimed:who@host"),
            Some(Role::Underway),
            "a holder changed what claimed means"
        );
        assert_eq!(
            role_of_marker("[finished:abc1234 on main]"),
            Some(Role::Accomplished),
            "a landing place changed what finished means"
        );
    }

    /// The lapse is what the reader has to see, from whichever state it lapsed.
    #[test]
    fn expired_wins_over_the_status_it_expired_from() {
        for marker in [
            "[open expired:who@host]",
            "[done expired:who@host]",
            "[in_progress expired:who@host]",
        ] {
            assert_eq!(
                role_of_marker(marker),
                Some(Role::Attention),
                "{marker} read as something other than a lapse"
            );
        }
    }

    /// What is not a name of the family gets no role, and `blocked` is the case
    /// that matters: SPEC-89070ce7f3b8 struck it because it is derived at read
    /// time and no entity is stored carrying it.
    #[test]
    fn a_string_that_is_not_a_status_has_no_role() {
        for absent in ["blocked", "", "Done", "open_", "who@host"] {
            assert_eq!(role_of_status(absent), None, "{absent:?} found a row");
        }
        assert_eq!(role_of_marker("[blocked]"), None);
    }

    /// Every kind reads as an identifier, which is the decision the rows carry.
    #[test]
    fn every_kind_is_an_identifier() {
        for kind in ["task", "adr", "spec", "log"] {
            assert_eq!(role_of_kind(kind), Some(Role::Identifier), "{kind} moved");
        }
        assert_eq!(role_of_kind("adr "), None);
    }

    /// A fault is a fault, and a signal is worth a look rather than a defect —
    /// which is the line `check`'s exit code already draws.
    #[test]
    fn a_finding_reads_by_its_severity() {
        assert_eq!(role_of_severity("fault"), Some(Role::Fault));
        assert_eq!(role_of_severity("signal"), Some(Role::Attention));
        assert_eq!(role_of_severity("error"), None);
    }
}
