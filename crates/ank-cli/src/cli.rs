//! Argument parsing, `help` and dispatch (§4, §9, §12).
//!
//! Parsing is done by hand, with no library. The reason is not saving a
//! dependency but character-level control over two surfaces read by agents:
//! the self-correcting errors, which a generic parser would replace with its
//! own messages, and the help, whose cost is paid on every call that triggers
//! it (§12). Neither argument depends on the verb list being frozen, which it
//! is not: hand-written parsing costs once per verb, the verbs share one
//! parser, and what grows is linear and small against a `help` and an error
//! surface that stay exactly as written.
//!
//! `help` lives here rather than in a verb module because it has no data of
//! its own: it is a rendering of [`COMMANDS`], [`GLOBAL_FLAGS`] and [`usage`],
//! which are all in this file. A second, hand-maintained list of the verbs is
//! exactly the drift the `owner_task` field was added to prevent.
//!
//! **The listing is grouped by the moment a verb is reached for, and inside a
//! group the order of [`COMMANDS`] survives untouched** (ADR-91b77f036884). It
//! once grouped verbs under headings named after callers, which was the
//! two-surface model still speaking through the output an agent reads; a
//! heading that sorts callers is a claim about who a verb is for, and there is
//! no such claim left to make. ADR-91b77f036884 removed those headings and left
//! §4's order to carry the structure alone, which works for five verbs and not
//! for twenty-one: the order only says something to a reader who already knows
//! it is meaningful, which is precisely what a first reader does not know. So
//! the headings are back as a second axis laid over that order rather than a
//! replacement for it, saying *when* a verb is used and never *who* may use it.
//! Nothing is trimmed either — every verb stays in the one surface that claims
//! to be complete. What the loop *is* stays in SKILL.md, whose content is
//! frozen and loaded permanently — that is where the token budget is spent, and
//! `help` is loaded on demand precisely so it does not compete.
//!
//! The edge cases of parsing are where hand-written code goes wrong, and they
//! look like business bugs once in production: every one of them is therefore
//! tested, one test per case.

use crate::json::Obj;
use ank_contract::ExitCode;
use std::collections::BTreeMap;
use std::fmt;
use std::io::Write;

// ---------------------------------------------------------------------------
// Error carrying its exit code
// ---------------------------------------------------------------------------

/// A CLI error. The code comes from the table in §4; `hint` carries the exact
/// command to run next. Never generic help: one well-designed error round trip
/// costs less than three blind attempts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliError {
    pub code: ExitCode,
    pub message: String,
    pub hint: Option<String>,
}

impl CliError {
    pub fn new(code: ExitCode, message: impl Into<String>) -> CliError {
        CliError {
            code,
            message: message.into(),
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> CliError {
        self.hint = Some(hint.into());
        self
    }

    /// Terse rendering, `git status` style, on standard error.
    pub fn render(&self) -> String {
        self.render_styled(crate::style::PLAIN)
    }

    /// The same line, with the `error[N]:` tag painted when §4 allows it.
    ///
    /// Only the tag: the message and the hint are the part a reader has to
    /// read, and the hint is a command to copy. Painting either would be
    /// decoration, and the hint is the last thing that should be hard to
    /// select.
    pub fn render_styled(&self, style: crate::style::Style) -> String {
        let tag = style.red(&format!("error[{}]:", self.code));
        match &self.hint {
            Some(h) => format!("{tag} {}\n  -> {}", self.message, h),
            None => format!("{tag} {}", self.message),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CliError {}

impl From<crate::store::StoreError> for CliError {
    fn from(e: crate::store::StoreError) -> CliError {
        let code = e.code();
        let hint = e.hint();
        let mut err = CliError::new(code, e.to_string());
        err.hint = hint;
        err
    }
}

pub type Result<T> = std::result::Result<T, CliError>;

// ---------------------------------------------------------------------------
// The surface itself, which lives in ank-contract (ADR-6fd69efb629c)
// ---------------------------------------------------------------------------

// Re-exported rather than reached through `ank_contract::` at every call site.
// `crate::cli::COMMANDS` is what the parser, the help rendering, the dispatch
// and their tests already name, and a re-export declares nothing — it names
// what the contract crate declares. Keeping the names where they were is what
// makes this a move: had the call sites changed too, the goldens proving the
// output identical would have been proving it of different code.
pub use ank_contract::{
    find_flag, known_flags, long_of, short_of, spec_of, usage, CommandSpec, FlagSpec, COMMANDS,
    GLOBAL_FLAGS, GROUPS,
};

// ---------------------------------------------------------------------------
// Parsed invocation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    pub command: &'static str,
    pub subcommand: Option<String>,
    pub positionals: Vec<String>,
    /// Values per flag. A switch carries an empty list.
    pub flags: BTreeMap<String, Vec<String>>,
    /// How this invocation is allowed to paint (§4).
    ///
    /// Not parsed — [`parse`] has no way to know what the process is attached
    /// to and leaves it [`style::PLAIN`], which is what keeps every unit test
    /// that builds an `Invocation` through the parser uncolored without saying
    /// so. [`dispatch`] is what fills it in, and what forces it back off under
    /// `--json`.
    pub style: crate::style::Style,
}

impl Invocation {
    pub fn has(&self, flag: &str) -> bool {
        self.flags.contains_key(flag)
    }

    pub fn value(&self, flag: &str) -> Option<&str> {
        self.flags.get(flag)?.last().map(|s| s.as_str())
    }

    pub fn values(&self, flag: &str) -> &[String] {
        self.flags.get(flag).map(|v| v.as_slice()).unwrap_or(&[])
    }

    pub fn json(&self) -> bool {
        self.has("--json")
    }

    pub fn quiet(&self) -> bool {
        self.has("--quiet")
    }

    pub fn repo(&self) -> Option<&str> {
        self.value("--repo")
    }

    /// The tree the corpus is anchored to, where the caller named one
    /// (ADR-9e56318631f3). Absent, the corpus anchors to its own directory.
    pub fn worktree(&self) -> Option<&str> {
        self.value("--worktree")
    }

    pub fn style(&self) -> crate::style::Style {
        self.style
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

/// `name=value` split, for both forms: `--status=open` and `-s=open` reach it
/// with the dashes already accounted for by the caller.
fn split_inline(arg: &str) -> (String, Option<String>) {
    match arg.split_once('=') {
        Some((n, v)) => (n.to_string(), Some(v.to_string())),
        None => (arg.to_string(), None),
    }
}

/// A single-dash argument, resolved to the long flag it names (§4).
///
/// Bundling is refused rather than parsed. Accepting `-st` forces a decision
/// about `-sopen` — is that three flags, or `-s` with a value? — and every
/// answer to that is a guess about what the caller meant. The refusal costs one
/// round trip and names exactly what to type instead; a guess costs a wrong
/// answer that looks right.
fn short_flag(spec: &CommandSpec, arg: &str) -> Result<(String, String, Option<String>)> {
    // No flag contains whitespace, so this one is a positional that starts with
    // a dash -- `ank log "-1 rebuilt the index"`. Saying so beats reporting it
    // as a bundle of nine unknown letters, and it is a fact rather than a guess
    // about intent. Single-dash arguments became flags the day short forms did,
    // and that is the one consequence a caller has to be told rather than left
    // to discover.
    if arg.contains(char::is_whitespace) {
        return Err(CliError::new(
            ExitCode::Generic,
            format!("'{arg}' is not a flag: it contains a space"),
        )
        .with_hint(format!("ank {} -- \"{arg}\"", spec.name)));
    }

    let (letters, inline) = split_inline(&arg[1..]);
    let chars: Vec<char> = letters.chars().collect();

    if chars.len() > 1 {
        return Err(bundled(spec, &chars, &letters));
    }

    let unknown = |typed: &str| {
        CliError::new(
            ExitCode::Generic,
            format!("unknown flag '{typed}' for '{}'", spec.name),
        )
        .with_hint(format!("valid flags: {}", known_flags(spec).join(" ")))
    };

    // `-` alone never reaches here, and `-=v` leaves nothing to resolve.
    let Some(c) = chars.first() else {
        return Err(unknown(arg));
    };
    let typed = format!("-{c}");
    let Some(long) = long_of(*c) else {
        return Err(unknown(&typed));
    };
    Ok((typed, long.to_string(), inline))
}

/// `-st`, and the two flags to type instead.
///
/// A letter that names nothing on this verb is reported as itself: naming a
/// flag the caller could type separately would be advice about a command that
/// would refuse too.
fn bundled(spec: &CommandSpec, chars: &[char], letters: &str) -> CliError {
    let mut parts: Vec<String> = Vec::new();
    for c in chars {
        match long_of(*c).and_then(|long| find_flag(spec, long)) {
            Some(fs) if fs.takes_value => parts.push(format!("-{c} <v>")),
            Some(_) => parts.push(format!("-{c}")),
            None => {
                return CliError::new(
                    ExitCode::Generic,
                    format!("unknown flag '-{c}' for '{}'", spec.name),
                )
                .with_hint(format!("valid flags: {}", known_flags(spec).join(" ")))
            }
        }
    }
    CliError::new(
        ExitCode::Generic,
        format!("'-{letters}' bundles short flags"),
    )
    .with_hint(format!("ank {} {}", spec.name, parts.join(" ")))
}

pub fn parse(argv: &[String]) -> Result<Invocation> {
    let Some(first) = argv.first() else {
        return Err(CliError::new(ExitCode::Generic, "no command").with_hint("ank context"));
    };

    let spec = spec_of(first).ok_or_else(|| {
        let names: Vec<&str> = COMMANDS.iter().map(|c| c.name).collect();
        CliError::new(ExitCode::Generic, format!("unknown command '{first}'"))
            .with_hint(format!("ank <{}>", names.join("|")))
    })?;

    let mut rest = &argv[1..];
    let mut subcommand = None;
    if !spec.subcommands.is_empty() {
        let sub = rest.first().ok_or_else(|| {
            CliError::new(
                ExitCode::Generic,
                format!("'{}' expects a subcommand", spec.name),
            )
            .with_hint(format!(
                "ank {} <{}>",
                spec.name,
                spec.subcommands.join("|")
            ))
        })?;
        if !spec.subcommands.contains(&sub.as_str()) {
            return Err(CliError::new(
                ExitCode::Generic,
                format!("unknown subcommand '{sub}' for '{}'", spec.name),
            )
            .with_hint(format!(
                "ank {} <{}>",
                spec.name,
                spec.subcommands.join("|")
            )));
        }
        subcommand = Some(sub.clone());
        rest = &rest[1..];
    }

    let mut positionals: Vec<String> = Vec::new();
    let mut flags: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut i = 0usize;
    let mut terminated = false;

    while i < rest.len() {
        let arg = &rest[i];

        if !terminated && arg == "--" {
            // Everything that follows is positional, verbatim: this is the
            // only way to write a message that starts with a dash.
            terminated = true;
            i += 1;
            continue;
        }

        // `typed` is what the caller wrote and `name` is the long flag it
        // names; they differ only for a short form. Every error below quotes
        // `typed`, because an agent that typed `-s` is helped by an error about
        // `-s` and not about a flag it never wrote.
        let named: Option<(String, String, Option<String>)> = if terminated {
            None
        } else if arg.starts_with("--") {
            let (name, inline) = split_inline(arg);
            Some((name.clone(), name, inline))
        } else if arg.starts_with('-') && arg.len() > 1 {
            Some(short_flag(spec, arg)?)
        } else {
            None
        };

        let Some((typed, name, inline)) = named else {
            positionals.push(arg.clone());
            i += 1;
            continue;
        };

        let Some(fs) = find_flag(spec, &name) else {
            // A letter that names a real flag the verb does not take is a
            // different mistake from a letter that names nothing, and saying
            // which one it is saves the round trip.
            let message = if typed == name {
                format!("unknown flag '{typed}' for '{}'", spec.name)
            } else {
                format!("'{typed}' is {name}, which '{}' does not take", spec.name)
            };
            return Err(CliError::new(ExitCode::Generic, message)
                .with_hint(format!("valid flags: {}", known_flags(spec).join(" "))));
        };

        if !fs.takes_value {
            if inline.is_some() {
                return Err(
                    CliError::new(ExitCode::Generic, format!("'{typed}' takes no value"))
                        .with_hint(format!("ank {} {typed}", spec.name)),
                );
            }
            flags.entry(name).or_default();
            i += 1;
            continue;
        }

        let value = match inline {
            Some(v) => {
                i += 1;
                v
            }
            None => {
                let v = rest.get(i + 1).ok_or_else(|| {
                    CliError::new(ExitCode::Generic, format!("'{typed}' expects a value"))
                        .with_hint(format!("ank {} {typed} <value>", spec.name))
                })?;
                i += 2;
                v.clone()
            }
        };

        let slot = flags.entry(name).or_default();
        if !fs.repeatable {
            slot.clear();
        }
        slot.push(value);
    }

    if positionals.len() > spec.max_positionals {
        let extra = &positionals[spec.max_positionals];
        return Err(CliError::new(
            ExitCode::Generic,
            format!(
                "extra argument '{extra}': '{}' accepts {}",
                spec.name, spec.max_positionals
            ),
        )
        .with_hint(usage(spec)));
    }

    Ok(Invocation {
        command: spec.name,
        subcommand,
        positionals,
        flags,
        style: crate::style::PLAIN,
    })
}

// ---------------------------------------------------------------------------
// help (§9)
// ---------------------------------------------------------------------------

/// A flag as `help` shows it: the name alone says nothing about whether a value
/// follows, and an agent that guesses wrong pays a round trip to find out.
///
/// `with_short` is what separates the two surfaces of §9. `ank help <verb>`
/// shows both forms, since that is the call made to learn one verb precisely.
/// The listing shows neither: it carries a description per verb and sends
/// the flags here, which is the split, and printing a second spelling of every
/// flag in an overview would spend exactly what the split saves.
fn flag_display(f: &FlagSpec, with_short: bool) -> String {
    let name = match short_of(f.name) {
        Some(c) if with_short => format!("-{c}, {}", f.name),
        _ => f.name.to_string(),
    };
    match (f.takes_value, f.repeatable) {
        (false, _) => name,
        (true, false) => format!("{name} <v>"),
        (true, true) => format!("{name} <v>..."),
    }
}

/// The description, folded to the listing's width and indented under itself.
///
/// **One string, printed by both surfaces** (§9). The listing shows
/// `spec.summary` and so does `ank help <verb>`, which is what makes the
/// overview a compression rather than a second text: two strings would be two
/// things to keep true, and the one that drifts is the one nobody reads twice.
/// That drift is exactly how `amend` came to advertise a criterion edit the
/// binary refused always (TASK-84cfad83c308).
///
/// Folded on words and never mid-word, at a width that leaves the whole line
/// inside 100 columns. A description too long to fit is folded rather than cut:
/// truncation would put the clause a verb refuses on — always the tail of the
/// sentence — exactly where it disappears.
fn wrapped_summary(summary: &str, indent: usize, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in summary.split_whitespace() {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
        .into_iter()
        .enumerate()
        .map(|(i, l)| {
            if i == 0 {
                l
            } else {
                format!("{:indent$}{l}", "")
            }
        })
        .collect()
}

/// The flags `help` offers, which is not every flag the parser accepts (§9).
///
/// One filter, read by the listing, the per-verb page and `--json` alike:
/// three renderings of one surface, and a name hidden from one of them and not
/// the others would be the same defect in a quieter place.
fn listed_flags(spec: &CommandSpec) -> Vec<&'static FlagSpec> {
    spec.flags.iter().filter(|f| f.listed).collect()
}

/// The globals that apply to a verb, which is all of them but one (§4).
///
/// Same filter as [`listed_flags`], for the same reason and read by the same
/// three renderings: §9 says a flag the verb rejects by design belongs in the
/// refusals and not in the offer, and being global is not an exemption —
/// `ank help init` offering `--repo` is precisely the offer that let the flag
/// look supported while `init` wrote somewhere else (TASK-b8a12d60686d).
fn globals_of(spec: &CommandSpec) -> Vec<&'static FlagSpec> {
    GLOBAL_FLAGS
        .iter()
        .filter(|f| !spec.refuses_globals.contains(&f.name))
        .collect()
}

/// **The listing passes [`GLOBAL_FLAGS`] whole, and that is deliberate.**
/// It states what the three globals of §4 are, once, for the surface — the
/// exception belongs on the page of the verb that makes it, where a reader
/// asking about `init` is looking. Qualifying it in the listing would put a
/// second structure under headings that already carry one (ADR-91b77f036884),
/// on the three lines the whole surface shares.
fn globals_line(globals: &[&'static FlagSpec], with_short: bool) -> String {
    globals
        .iter()
        .map(|f| flag_display(f, with_short))
        .collect::<Vec<_>>()
        .join(" ")
}

/// The declared shape of a document, flattened to dotted paths.
///
/// Nesting is how a shape is written in `ank-contract`, because that reads well
/// beside the code. A dotted path is how it is published, because `help --json`
/// describes its own output too, and a description that recursed into itself
/// would not terminate. `tasks` is followed by `tasks.id` and `tasks.title`, in
/// the order the document emits them, so a reader walking the list top to bottom
/// walks the document.
fn fields_json(prefix: &str, fields: &[ank_contract::shape::Field]) -> Vec<String> {
    let mut rows = Vec::new();
    for field in fields {
        let name = match prefix.is_empty() {
            true => field.name.to_string(),
            false => format!("{prefix}.{}", field.name),
        };
        rows.push(
            Obj::new()
                .str("name", &name)
                .str("type", field.ty.name())
                .bool("nullable", field.nullable)
                .finish(),
        );
        rows.extend(fields_json(&name, field.ty.fields()));
    }
    rows
}

fn json_of(specs: &[&CommandSpec]) -> String {
    let verbs: Vec<String> = specs
        .iter()
        .map(|spec| {
            let refusals: Vec<String> = spec
                .refuses
                .iter()
                .map(|r| Obj::new().num("code", r.code).str("when", r.when).finish())
                .collect();
            let flags: Vec<String> = listed_flags(spec)
                .into_iter()
                .chain(globals_of(spec))
                .map(|f| {
                    // The short form is here and not only in the human listing:
                    // `--json` is how a script reads the surface, and a mapping
                    // it cannot see is a mapping it cannot use.
                    let short = short_of(f.name).map(|c| format!("-{c}"));
                    Obj::new()
                        .str("name", f.name)
                        .opt_str("short", short.as_deref())
                        .bool("takes_value", f.takes_value)
                        .bool("repeatable", f.repeatable)
                        .finish()
                })
                .collect();
            // `group` is here and not only in the human listing: §4 emits the
            // same structure to everyone and lets only colour depend on the
            // reader, so giving a machine the grouping and withholding it from
            // the caller who scripts against it would be the split this ADR
            // rejected, the other way round (ADR-91b77f036884).
            // What comes back, which is the half this document was missing
            // (ADR-6fd69efb629c). A caller can discover a flag by being refused
            // one; it cannot discover a field by being handed it, because it has
            // to know the name before it can look.
            //
            // `contract` is prepended rather than declared on each verb: it is
            // on every document by construction, and twenty-two copies of one
            // row is twenty-two chances for one of them to be wrong.
            let returns: Vec<String> = spec
                .output
                .iter()
                .map(|shape| {
                    let mut fields = vec![Obj::new()
                        .str("name", "contract")
                        .str("type", "number")
                        .bool("nullable", false)
                        .finish()];
                    fields.extend(fields_json("", shape.fields));
                    Obj::new()
                        .opt_str("when", shape.when)
                        .array("fields", fields)
                        .finish()
                })
                .collect();
            Obj::new()
                .str("name", spec.name)
                .str("usage", &usage(spec))
                .str("summary", spec.summary)
                .str("group", spec.group)
                .array("flags", flags)
                .strings("notes", spec.notes)
                .array("refuses", refusals)
                .array("returns", returns)
                .finish()
        })
        .collect();
    Obj::document().array("verbs", verbs).finish()
}

/// `ank help` and `ank help <verb>` (§9).
///
/// The listing is derived from [`COMMANDS`] on every call, so a verb cannot be
/// added without appearing here. An unknown verb is a **code 2** — "entity not
/// found" in the table of §4, the same code a missing task gets, because the
/// thing being looked up did not exist. It is never a fallback to the general
/// listing: an agent that asked about one verb and received all sixteen has to
/// work out that its question went unanswered, and answering the wrong question
/// silently is worse than refusing the wrong one loudly.
///
/// The listing is grouped by [`GROUPS`], and inside a group the order is
/// [`COMMANDS`]', which is §4's (ADR-91b77f036884). No verb is hidden and there
/// is no second listing to ask for: git shows the common commands and sends the
/// rest to `git help -a`, and hiding a verb from the one surface claiming to be
/// complete is worse than never teaching it.
///
/// `ank help <verb>` gains nothing from any of this. It never had headings and
/// answers about one verb, which is a moment of its own.
pub fn help(inv: &Invocation, out: &mut dyn Write) -> Result<ExitCode> {
    let asked = inv.positionals.first();

    if let Some(name) = asked {
        let spec = spec_of(name).ok_or_else(|| {
            CliError::new(ExitCode::NotFound, format!("no such verb '{name}'"))
                .with_hint("ank help")
        })?;
        if inv.json() {
            let _ = writeln!(out, "{}", json_of(&[spec]));
            return Ok(ExitCode::Ok);
        }
        if inv.quiet() {
            return Ok(ExitCode::Ok);
        }
        let _ = writeln!(out, "{}", usage(spec));
        if !spec.summary.is_empty() {
            let _ = writeln!(out, "  {}", spec.summary);
        }
        let listed = listed_flags(spec);
        if !listed.is_empty() {
            let flags: Vec<String> = listed.iter().map(|f| flag_display(f, true)).collect();
            let _ = writeln!(out, "  flags:    {}", flags.join(" "));
        }
        let _ = writeln!(out, "  global:   {}", globals_line(&globals_of(spec), true));
        for (i, note) in spec.notes.iter().enumerate() {
            let label = if i == 0 { "note:" } else { "" };
            let _ = writeln!(out, "  {label:<9} {note}");
        }
        // What the verb refuses, and the code it comes back with (§9). The
        // question is asked before the call; the error is only available after.
        for (i, r) in spec.refuses.iter().enumerate() {
            let label = if i == 0 { "refuses:" } else { "" };
            let _ = writeln!(out, "  {label:<9} {} ({})", r.when, r.code);
        }
        return Ok(ExitCode::Ok);
    }

    let all: Vec<&CommandSpec> = COMMANDS.iter().collect();
    if inv.json() {
        let _ = writeln!(out, "{}", json_of(&all));
        return Ok(ExitCode::Ok);
    }
    if inv.quiet() {
        return Ok(ExitCode::Ok);
    }

    // One column for the usage, so the descriptions line up and the shape of
    // the surface is readable at a glance rather than one verb at a time.
    //
    // **The description takes the place of the flag names** (§9). They used to
    // sit here, which is the shape that says what none of them does: a bare
    // `--criteria` beside `amend` names a flag without saying what the verb is
    // for, and a caller choosing between twenty-one verbs was choosing on the
    // usage line alone. They are one `ank help <verb>` away, with their value
    // placeholders and the refusals that qualify them, and the trailer below
    // says so rather than leaving the reader to discover it.
    //
    // **The width is computed across all of [`COMMANDS`] and never per group**
    // (ADR-91b77f036884). Five widths would stop the columns lining up between
    // sections, and the listing would read as five tables rather than one thing
    // with five parts — which is the opposite of what the grouping is for.
    let width = COMMANDS.iter().map(|c| usage(c).len()).max().unwrap_or(0);
    let indent = width + 2;
    for (n, group) in GROUPS.iter().enumerate() {
        // A blank line between groups, and none before the first: the heading
        // is what opens the listing, so nothing sits above the first verb but
        // the name of the moment it belongs to.
        if n > 0 {
            let _ = writeln!(out);
        }
        let _ = writeln!(out, "{group}");
        for spec in COMMANDS.iter().filter(|c| c.group == *group) {
            if spec.summary.is_empty() {
                let _ = writeln!(out, "{}", usage(spec));
                continue;
            }
            for (i, line) in wrapped_summary(spec.summary, indent, 98 - indent)
                .into_iter()
                .enumerate()
            {
                if i == 0 {
                    let _ = writeln!(out, "{:width$}  {line}", usage(spec));
                } else {
                    let _ = writeln!(out, "{line}");
                }
            }
        }
    }
    let all: Vec<&FlagSpec> = GLOBAL_FLAGS.iter().collect();
    let _ = writeln!(out, "\nglobal: {}", globals_line(&all, false));
    let _ = writeln!(
        out,
        "ank help <verb> for one verb: its flags, and what it refuses"
    );
    // A trailing pointer, beside the one above it and in the same shape. The
    // three trailer lines are not a group and take no heading: they say where
    // to look next rather than when a verb is used (ADR-91b77f036884). A flag
    // nobody can discover answers nobody's question.
    let _ = writeln!(out, "ank --version for the build");
    Ok(ExitCode::Ok)
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

/// What the binary is, on one line (§4): the crate version, the commit
/// `build.rs` stamped in, and the revision of the `SKILL.md` it was built
/// alongside. `unknown` where there was no checkout, or no `skill/`, to ask.
///
/// The third value is the one a reader can act on with nothing else in hand
/// (TASK-ecda4070354f). The commit places the binary against the repository,
/// which is a thing to go and look at; the skill revision is compared against
/// `metadata.revision` in the file the agent has already loaded, so a stale
/// instruction set announces itself offline instead of costing an
/// investigation.
pub fn version_line() -> String {
    format!(
        "ank {} ({}, skill {})",
        env!("CARGO_PKG_VERSION"),
        env!("ANK_COMMIT"),
        env!("ANK_SKILL")
    )
}

fn not_implemented(spec: &CommandSpec) -> CliError {
    let task = spec.owner_task.unwrap_or("TASK-unknown");
    CliError::new(
        ExitCode::Generic,
        format!("'{}' is not implemented yet", spec.name),
    )
    .with_hint(format!("ank show {task}"))
}

/// `tui`, the terminal reader (ADR-8bd76e8d7c4e, §4).
///
/// Three things happen here and nothing else does: the refusal a caller with no
/// terminal is owed, the resolution of the corpus so that a missing `.ank/` is
/// the refusal it already is rather than a screen with an error on it, and the
/// address the reader spawns this binary with.
///
/// **The order is the refusal's.** The terminal check is first because it is
/// the cheapest and because it is the one an agent that typed `ank tui` by
/// accident needs: `ank` is run by agents far more often than by people, and a
/// process that hung holding a terminal it does not have would be the worst
/// answer of the three.
///
/// **`current_exe` and not a name looked up on `PATH`.** The reader must run
/// *this* build, or the frames it draws come from a different corpus reader
/// than the one the caller invoked, and the version they disagree on would be
/// invisible on the screen.
fn tui(inv: &Invocation, cwd: &std::path::Path, out: &mut dyn std::io::Write) -> Result<ExitCode> {
    if !ank_tui::attached() {
        return Err(refused(ank_tui::no_terminal()));
    }
    let _repo = resolved(inv, cwd)?;
    let exe = std::env::current_exe().map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!("cannot find this binary to run it: {e}"),
        )
        .with_hint(ank_tui::INSTEAD)
    })?;
    let address = ank_tui::Address {
        exe,
        cwd: cwd.to_path_buf(),
        // The caller's own address, passed through rather than the path just
        // resolved: the child then walks exactly as the parent did, so there is
        // one resolution and not two (ADR-9e56318631f3).
        repo: inv.repo().map(str::to_string),
        worktree: inv.worktree().map(str::to_string),
    };
    ank_tui::run(&address, inv.json(), out).map_err(refused)
}

/// `mcp`, the protocol surface (ADR-fd98f4bc6dea, §4).
///
/// The same two things `tui` does above, for the same reasons and minus the
/// terminal. The corpus is resolved here so that a missing `.ank/` is the
/// refusal it already is, printed where a person starting a server will see it,
/// rather than a JSON-RPC error the first client has to decode. And the address
/// is built here, because both halves of it are the caller's foundation and
/// neither is the surface's to guess.
///
/// **`current_exe` and not a search.** The sibling binary had to look for the
/// `ank` it was released with -- beside itself, then `PATH`, with `ANK_BIN` over
/// both -- and a wrong answer there was a server reporting verbs the installed
/// CLI does not have. A verb has no such question: the binary a call runs is the
/// process already running, so the search is gone and the failure with it.
///
/// **What is not folded is the dispatch.** The surface still runs
/// `ank <verb> --repo <corpus> --json` per call rather than calling into this
/// file, so every refusal a client sees is the one the binary gave. Linking the
/// dispatch in would re-derive them, and anything re-derived can differ.
fn mcp(inv: &Invocation, cwd: &std::path::Path, out: &mut dyn std::io::Write) -> Result<ExitCode> {
    let repo = resolved(inv, cwd)?;
    let exe = std::env::current_exe().map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!("cannot find this binary to run it: {e}"),
        )
        .with_hint("every call is a run of ank: start the server by the binary's path on disk")
    })?;
    let address = ank_mcp::Address {
        exe,
        repo: repo.corpus,
        // **The version a client is told, handed over here because the surface
        // cannot honestly work it out** (TASK-ae64d1c5678d). `crates/ank-mcp`
        // must not link this crate (ADR-fd98f4bc6dea), so its own
        // `CARGO_PKG_VERSION` can only ever name the library; this one is the
        // executable's, the same literal `version_line` above prints, and the
        // one the release gates against the tag. `exe` is this very process, so
        // the number handed down is the number that binary answers `--version`
        // with -- by construction and not by anyone keeping two literals in
        // step. The argument in full is on `ank_mcp::Address::version`.
        version: env!("CARGO_PKG_VERSION").to_string(),
    };
    // Standard input is the client's half of the transport and is read nowhere
    // else in this binary, so it is locked here rather than threaded through a
    // dispatch that has no other use for it.
    let stdin = std::io::stdin();
    ank_mcp::serve(&address, &mut stdin.lock(), out);
    Ok(ExitCode::Ok)
}

/// `watch`, the watcher (ADR-a22cd3196529, ADR-1ea31c2f3c5a, §4).
///
/// **It answers no verb, and being started by one does not change that.** The
/// clause a reader meets first in ADR-a22cd3196529 is the easiest one in this
/// tree to read backwards, so it is worth saying in the place the reading
/// happens: what the decision forbids is a *query surface* -- a socket, a
/// protocol, a subset of the CLI answering out of a resident process. It says
/// nothing about how the process is launched, and it never did: a sibling
/// executable was one spelling of a launch and this verb is another. `ank
/// watch` starts something that still serves nobody. The flags below are the
/// four §4 lists and not one of them asks the watcher about a corpus, which is
/// what keeps the clause mechanical rather than a promise.
///
/// **And nothing may come to depend on it.** Every other verb behaves
/// identically with this process absent, its absence is never an error, and no
/// route makes running it a condition of using ank -- which is why this arm is
/// the whole of the dispatch's knowledge of it: nothing above reads a watcher's
/// state, nothing below waits for one.
///
/// **The corpus is not resolved here, and that is the difference from `tui` and
/// `mcp`.** Those two speak for the corpus the caller is standing in. This one
/// speaks for the corpora the reader declared, in a file outside every
/// repository (ADR-96174f1ac2b7) -- so requiring a `.ank/` under the working
/// directory would make starting the watcher depend on where it was started
/// from, and refuse it in the one place a person is most likely to type it.
///
/// **`current_exe` and not a search**, for the reason `mcp` gives above: the
/// binary a warming runs is the process already running, so the sibling's
/// `ANK_BIN` / beside-me / `PATH` ladder is gone and the wrong answer it could
/// have given with it.
fn watch(inv: &Invocation, out: &mut dyn std::io::Write) -> Result<ExitCode> {
    // Refused and never ignored, which is the rule §9 holds `init --repo` to.
    // Both of these address *a* corpus, and this verb is told which corpora to
    // keep warm by the reader's declaration; a caller who typed one would have
    // addressed a tree nothing is watching and been told nothing at all. The
    // declaration is what to correct, so the refusal names the file.
    for named in ["--repo", "--worktree"] {
        if inv.has(named) {
            return Err(CliError::new(
                ExitCode::Generic,
                format!(
                    "{named} addresses one corpus, and the watcher warms the \
                     corpora you declared"
                ),
            )
            .with_hint("ank watch --where names the file that says which"));
        }
    }
    let exe = std::env::current_exe().map_err(|e| {
        CliError::new(
            ExitCode::Environment,
            format!("cannot find this binary to run it: {e}"),
        )
        .with_hint("a warming is a run of ank: start the watcher by the binary's path on disk")
    })?;
    let address = ank_daemon::Address { exe };
    let options = ank_daemon::Options {
        list: inv.has("--list"),
        once: inv.has("--once"),
        location: inv.has("--where"),
        // Carried as the caller typed it. What a number has to be is the
        // watcher's own refusal, raised in its own words, and re-deriving it
        // here would be a second answer to one question.
        interval: inv.value("--interval").map(str::to_string),
    };
    // Standard error is where the log goes, and it is the watcher's ordinary
    // destination: the listing and the declaration path are answers to a person
    // who asked, and everything the loop reports is a line in a log file.
    let mut err = std::io::stderr();
    ank_daemon::run(&address, &options, out, &mut err).map_err(watching)
}

/// A refusal the watcher raised, in the shape this file renders.
///
/// The hint is carried where there is one and left off where there is not:
/// every refusal this crate raises names a next command, and the one that does
/// not is a bug in the watcher rather than a line to invent here.
fn watching(f: ank_daemon::Fail) -> CliError {
    let err = CliError::new(f.code, f.message);
    match f.hint {
        Some(hint) => err.with_hint(hint),
        None => err,
    }
}

/// A refusal the reader raised, in the shape this file renders.
fn refused(r: ank_tui::Refused) -> CliError {
    CliError::new(r.code, r.message).with_hint(r.hint)
}

/// Entry point. Returns the exit code; never calls `exit` itself, so that it
/// stays testable.
///
/// Returns the [`ExitCode`] rather than the integer, so that the one place in
/// the tool that has to hold a bare number is the one place that cannot avoid
/// it: the call to `std::process::exit` in `main`.
pub fn run(
    argv: &[String],
    cwd: &std::path::Path,
    out: &mut dyn std::io::Write,
    style: crate::style::Style,
) -> ExitCode {
    match dispatch(argv, cwd, out, style) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("{}", err.render_styled(style.on_stderr()));
            err.code
        }
    }
}

/// The foundation shared by every verb except `init`, which by definition
/// precedes the existence of the repository.
///
/// The order matters and is not arbitrary: the repository is resolved before
/// git is checked, because a wrong `--repo` must be named as such and not
/// disguised as an environment problem. That resolution happens **before** the
/// rejection of an unimplemented verb: without it, `--repo` would be exercised
/// by no real path while no verb exists, and the foundation would be tested
/// without ever being reached.
struct Startup {
    repo: crate::repo::Repo,
    config: crate::config::Config,
    identity: String,
    /// Carried beside the value rather than asked for again where it is
    /// printed: the identity is resolved once, and a second reading of the
    /// environment would be a second answer free to disagree with the first.
    identity_source: crate::identity::Source,
}

/// Names the corpus when the walk crossed a git repository boundary
/// (TASK-2f01baf94632).
///
/// `discover` stops at the first `.ank/` and at nothing else, so from a
/// checkout nested inside another repository it silently resolves the outer
/// corpus. That is not merely reading the wrong files: claims are git refs
/// (ADR-4e7c), so they land in the resolved root's repository and not in the
/// one holding the code being changed. Measured on a fixture, the inner
/// repository ends with no ank ref at all.
///
/// **Only on the walk.** `--repo` is the caller saying which corpus they mean,
/// and warning about an answer that was asked for by name would fire forever in
/// the one layout this behaviour makes usable — a single `.ank/` above several
/// checkouts, scopes written as `repoA/src/**`. That layout was never designed
/// for and is not forbidden either; naming `--repo` once is what separates it
/// from the accident.
///
/// **On stderr**, and that is not incidental. It is not part of any verb's
/// answer, and §4 requires `--json` to stay byte-for-byte what a caller's
/// parser already reads; a line on stdout would break every one of them to say
/// something no parser asked for. It degrades and never fails (§2): the walk
/// succeeded, and the caller may well have meant it.
fn warn_if_outside_repository(inv: &Invocation, repo: &crate::repo::Repo, cwd: &std::path::Path) {
    if inv.repo().is_some() || inv.quiet() {
        return;
    }
    // **A declared corpus was named, and this warning is about an accident.**
    // It exists for the checkout nested inside another repository, which
    // resolves the outer corpus without anybody choosing it; `--repo` is exempt
    // above because it is the caller saying which corpus they mean, and a
    // declaration keyed on this repository's identity says the same thing once
    // instead of on every command (ADR-96174f1ac2b7). Left in, it would fire
    // forever on the one layout the declaration exists to make usable, which is
    // the failure §6 already names for `--repo`.
    if crate::repo::is_declared(&repo.corpus, cwd) {
        return;
    }
    let here = crate::git::common_dir(cwd);
    let root = crate::git::common_dir(&repo.corpus);
    if !crate::git::crosses_repository(here.as_deref(), root.as_deref()) {
        return;
    }
    let style = inv.style().on_stderr();
    let tag = style.yellow("warning:");
    let root_display = repo.corpus.display();
    match here {
        Some(_) => eprintln!(
            "{tag} .ank/ resolved to {root_display}, outside the git repository holding {}",
            cwd.display()
        ),
        None => eprintln!(
            "{tag} .ank/ resolved to {root_display}; {} is in no git repository",
            cwd.display()
        ),
    }
    eprintln!(
        "  -> claims are refs and land in {root_display}: ank init here, or --repo to confirm"
    );
}

/// Names a corpus written by a binary newer than this one (TASK-ca7b61b00896).
///
/// The one version mismatch the tool can diagnose on its own. A corpus
/// declaring a schema past `SCHEMA_VERSION` is refused entity by entity, deep
/// inside a parse, and every verb that lists then answers as if those entities
/// were not there: `find`, `context`, `graph` print a corpus short of them and
/// say nothing, and `status` reports a fault count without its cause. The
/// reader concludes the corpus is what they see. Two sessions lost time to
/// exactly that, and one nearly filed a regression that did not exist.
///
/// **Not conditional on `--repo`**, unlike [`warn_if_outside_repository`]. That
/// warning is silenced by `--repo` because naming a corpus is the caller saying
/// they meant this one; naming it says nothing about whether the binary can
/// read it.
///
/// **On stderr**, for the reason its neighbour is: it is not part of any verb's
/// answer, and §4 requires `--json` to stay byte-for-byte what a caller's
/// parser reads. It degrades and never fails (§2) — the entities this build
/// does understand are still worth answering with, and a corpus mid-migration
/// is a real state rather than a broken one.
fn warn_if_schema_ahead(inv: &Invocation, repo: &crate::repo::Repo) {
    if inv.quiet() {
        return;
    }
    let Some(ahead) = crate::repo::schema_ahead(repo) else {
        return;
    };
    let style = inv.style().on_stderr();
    let (what, next) = ahead.lines(crate::repo::released_schema());
    eprintln!("{} {what}", style.yellow("warning:"));
    eprintln!("  -> {next}");
}

/// The corpus, and whatever resolving it had to say.
///
/// **The note is printed here and never inside the resolution**, on the split
/// ADR-1f70ce2c3eac draws: what a reader is told is presentation, and a
/// resolver that printed would be a second place deciding what `--quiet` means.
/// Standard error, because stdout is a parser's input (§4) and a declaration
/// answering instead of the tree changes no answer, only where it came from.
fn resolved(inv: &Invocation, cwd: &std::path::Path) -> Result<crate::repo::Repo> {
    let mut notes = Vec::new();
    let repo = crate::repo::resolve(inv.repo(), inv.worktree(), cwd, &mut notes)?;
    if !inv.quiet() {
        let style = inv.style().on_stderr();
        for note in notes {
            eprintln!("{} {note}", style.yellow("warning:"));
        }
    }
    Ok(repo)
}

fn startup(inv: &Invocation, cwd: &std::path::Path) -> Result<Startup> {
    let repo = resolved(inv, cwd)?;
    // git is required by the verbs that coordinate, and never at startup
    // (ADR-9307e5d214a7). The gate used to stand
    // in front of the dispatch rather than in front of the operation, so `show`,
    // `find`, `graph`, `scope`, `new`, `amend` and the whole formal half of
    // `check` refused outside a repository although none of them touches a ref,
    // a commit or a branch.
    //
    // Repository resolution never needed git and does not gain it here:
    // `repo::discover` walks up for `.ank/` exactly as it always has.
    if spec_of(inv.command).is_some_and(|s| s.coordinates) {
        crate::git::ensure_usable(&repo.corpus)?;
    }
    warn_if_outside_repository(inv, &repo, cwd);
    warn_if_schema_ahead(inv, &repo);
    let config = crate::config::load(&repo.config_path())?;
    let (identity, identity_source) = crate::identity::resolved();
    Ok(Startup {
        repo,
        config,
        identity,
        identity_source,
    })
}

/// Routes a parsed invocation to the module that owns the verb.
///
/// Every arm receives the same three things — the repository, the config and
/// the identity — resolved once by [`startup`]. A verb never resolves them
/// itself: the order in which they are established is a property of the
/// foundation, and a module doing it again would be free to get it wrong.
///
/// A verb whose module is still a stub falls through to [`not_implemented`],
/// which names the task that owns it. The arms therefore arrive one per task,
/// and the fall-through is the honest default rather than a placeholder: until
/// TASK-45d18f45de2c the fall-through was total, while six module headers
/// asserted the opposite.
fn dispatch(
    argv: &[String],
    cwd: &std::path::Path,
    out: &mut dyn std::io::Write,
    style: crate::style::Style,
) -> Result<ExitCode> {
    // Before `parse`, and not as a flag on a verb (§4). `--version` replaces the
    // verb rather than modifying one, so the parser — which resolves a command
    // first and would reject this as an unknown one — never sees it. It is also
    // ahead of every check below for the reason `help` is: the caller who needs
    // it is holding an artifact they cannot identify, and a version that
    // demanded a healthy repository would go quiet exactly there.
    if argv.first().is_some_and(|a| a == "--version") {
        let _ = writeln!(out, "{}", version_line());
        return Ok(ExitCode::Ok);
    }
    let mut inv = parse(argv)?;
    let spec = spec_of(inv.command).expect("spec resolved during parsing");

    // The one gate, and the reason it is one. `--json` is never colored (§4),
    // and suppressing color at each printing site would be one chance per site
    // to forget one; suppressing it here makes "no escape sequence under
    // --json" a property of the invocation rather than of the discipline.
    //
    // This comment used to record that three verbs printed an unconditional
    // non-JSON line onto stdout while `--json` was set — `done`'s `running:`,
    // and the takeover warnings of `log` and `amend` — and treated it as a
    // given. It was not: §4 says `--json` stays byte-for-byte what a caller's
    // parser reads, and `ank done --json` emitted its progress line ahead of
    // the JSON document. All three now write to standard error
    // (TASK-2eefcdd80124), and `tests/cli.rs` walks the surface so a fourth
    // cannot arrive quietly.
    inv.style = if inv.json() {
        crate::style::PLAIN
    } else {
        style
    };

    // Three verbs run without the foundation. `init` precedes the existence of
    // the repository. `help` describes the surface rather than acting on it,
    // and the caller most in need of it is the one whose environment is wrong:
    // making `ank help` demand a `.ank/`, a git of 2.34, and a readable
    // `config.yml` would withhold the explanation exactly when it is needed.
    // `config` is the third and the sharpest case of the same reasoning
    // (ADR-e64dfaafd578): `startup` loads `config.yml` for every other verb, so
    // an unreadable one fails all of them — `check` included — and the verb
    // that exists to repair the file would be disabled by exactly the file it
    // repairs. It resolves the repository, because the file it edits lives in
    // one, and nothing else.
    if inv.command == "init" {
        return crate::init::run(&inv, cwd, out);
    }
    if inv.command == "help" {
        return help(&inv, out);
    }
    if inv.command == "config" {
        // Before the corpus is resolved, and that is the point: the reader's
        // declarations live outside every repository, so a scope that asked for
        // one would refuse in the directory where a corpus is being declared
        // (ADR-96174f1ac2b7).
        if inv.has("--user") {
            return crate::config::run_user(&inv, out);
        }
        let repo = resolved(&inv, cwd)?;
        // The same hazard, and it reaches this verb too: a `config` run from a
        // nested checkout edits the outer repository's configuration. `git` is
        // unchecked here, which costs nothing — `common_dir` answers `None`
        // when it cannot run, and two `None`s say nothing.
        warn_if_outside_repository(&inv, &repo, cwd);
        return crate::config::run(&inv, &repo, out);
    }
    // The fourth, and the reason is its own (ADR-8bd76e8d7c4e). `tui` is a
    // reader that reaches the corpus by *running this binary*, so the
    // foundation it needs is the one every child call establishes for itself:
    // loading a config and resolving an identity here would be a second
    // resolution, free to disagree with the twenty the session then makes.
    // What is checked here is what a child cannot check — that there is a
    // terminal to draw on — and that the corpus resolves at all.
    if inv.command == "tui" {
        return tui(&inv, cwd, out);
    }
    // The fifth, and it is the fourth's reasoning exactly (ADR-fd98f4bc6dea).
    // `mcp` is a surface that reaches the corpus by *running this binary*, once
    // per call, so the foundation it needs is the one each of those children
    // establishes for itself. What is checked here is what a child cannot check:
    // that the corpus resolves at all, and that this process can name itself.
    if inv.command == "mcp" {
        return mcp(&inv, cwd, out);
    }
    // The sixth, and it goes one step further than the two above it
    // (ADR-a22cd3196529). `watch` resolves no corpus at all: what it keeps warm
    // is what the reader declared outside every repository, so there is nothing
    // in the working directory for it to need, and a `.ank/` demanded here
    // would refuse the watcher in the one place a person is most likely to
    // start it. It answers no verb once running, and starting it by one is not
    // answering one -- the note on `watch` below says why that reading is the
    // one the decision carries.
    if inv.command == "watch" {
        return watch(&inv, out);
    }

    let s = startup(&inv, cwd)?;
    let code = match inv.command {
        "context" => crate::context::run(&inv, &s.repo, &s.config, &s.identity, out),
        "done" => crate::done::run(&inv, &s.repo, &s.config, &s.identity, out),
        "claim" => crate::claim::run(&inv, &s.repo, &s.config, &s.identity, out),
        "new" => crate::commands::new(&inv, &s.repo, &s.config, &s.identity, out),
        "find" => crate::commands::find(&inv, &s.repo, &s.config, &s.identity, out),
        "status" => crate::status::run(
            &inv,
            &s.repo,
            &s.config,
            &s.identity,
            s.identity_source,
            out,
        ),
        "graph" => crate::graph::run(&inv, &s.repo, out),
        "scope" => crate::commands::scope(&inv, &s.repo, &s.identity, out),
        "log" => crate::commands::log(&inv, &s.repo, &s.config, &s.identity, out),
        "release" => crate::commands::release(&inv, &s.repo, &s.identity, out),
        "check" => crate::human::check(&inv, &s.repo, &s.config, out),
        "migrate" => crate::migrate::run(&inv, &s.repo, out),
        "review" => crate::human::review(&inv, &s.repo, &s.config, out),
        "accept" => crate::human::accept(&inv, &s.repo, &s.config, &s.identity, out),
        "read" => crate::human::read(&inv, &s.repo, &s.identity, out),
        "close" => crate::human::close(&inv, &s.repo, &s.identity, out),
        "attest" => crate::human::attest(&inv, &s.repo, &s.identity, out),
        "edit" => crate::edit::run(&inv, &s.repo, &s.identity, out),
        "amend" => crate::human::amend(&inv, &s.repo, &s.identity, out),
        "show" => crate::human::show(&inv, &s.repo, &s.config, out),
        _ => Err(not_implemented(spec)),
    }?;

    // **The one place the renewal rule is applied** (§3, ADR-0bb7ea8991bc).
    // Working is what keeps the lock, so the lease moves when the verb that just
    // ran was the holder's work on the task it holds. Here rather than at each
    // verb's entry point, because a test scattered across nineteen arms is
    // nineteen chances to answer it differently — what each verb declares is
    // what it is about, and applying that is one line.
    //
    // **After the verb, and only when it succeeded.** A verb that failed did no
    // work; and running before would mean `done` and `release` unwinding a
    // renewal a moment old, which is why they declare `Never` besides.
    //
    // The id the rule needs is the first positional, which is where every verb
    // that names an entity carries it. Verbs whose positional is a path declare
    // `Never`, so no path is ever resolved as an id.
    crate::claim::renew_by_working(
        &s.repo,
        &s.config,
        &s.identity,
        spec.renews,
        inv.positionals.first().map(String::as_str),
    );
    Ok(code)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(s: &[&str]) -> Vec<String> {
        s.iter().map(|x| x.to_string()).collect()
    }

    fn ok(s: &[&str]) -> Invocation {
        parse(&argv(s)).unwrap_or_else(|e| panic!("{s:?} should have parsed: {}", e.render()))
    }

    #[test]
    fn repo_in_both_of_its_forms() {
        let a = ok(&["check", "--repo=/path/to/project"]);
        let b = ok(&["check", "--repo", "/path/to/project"]);
        assert_eq!(a.repo(), Some("/path/to/project"));
        assert_eq!(b.repo(), Some("/path/to/project"));
        assert_eq!(a.flags, b.flags);
    }

    #[test]
    fn the_terminator_passes_the_message_through_intact() {
        // The case that motivates the terminator: a message starting with two
        // dashes, which would otherwise be read as a flag.
        let inv = ok(&["log", "--", "-- message"]);
        assert_eq!(inv.positionals, vec!["-- message".to_string()]);
        assert!(inv.flags.is_empty());

        // After the terminator, nothing is a flag any more.
        let inv = ok(&["log", "--", "--json"]);
        assert_eq!(inv.positionals, vec!["--json".to_string()]);
        assert!(!inv.json(), "--json after -- is text, not a flag");

        // Before the terminator, flags are still flags.
        let inv = ok(&["log", "--json", "--", "--repo"]);
        assert!(inv.json());
        assert_eq!(inv.positionals, vec!["--repo".to_string()]);
    }

    #[test]
    fn an_unknown_flag_names_the_valid_flags() {
        let err = parse(&argv(&["claim", "--tll", "30m"])).unwrap_err();
        assert_eq!(err.code, ExitCode::Generic);
        assert!(err.message.contains("--tll"), "{}", err.message);
        let hint = err.hint.unwrap();
        for expected in ["--criteria", "--ttl", "--json", "--quiet", "--repo"] {
            assert!(hint.contains(expected), "{expected} missing from: {hint}");
        }
    }

    #[test]
    fn a_missing_value_after_a_flag_that_expects_one() {
        let err = parse(&argv(&["claim", "8f3a", "--ttl"])).unwrap_err();
        assert_eq!(err.code, ExitCode::Generic);
        assert!(err.message.contains("--ttl"), "{}", err.message);
        assert_eq!(err.hint.as_deref(), Some("ank claim --ttl <value>"));

        // A global flag is not a special case.
        let err = parse(&argv(&["check", "--repo"])).unwrap_err();
        assert!(err.message.contains("--repo"), "{}", err.message);
    }

    /// `config`'s note lists the key set the verb actually addresses.
    ///
    /// §9 forbids the help surface from making an offer the verb turns down,
    /// and the mirror of that is a key the verb takes and the note never names:
    /// the note is where a caller looks before typing, and `claim_ttl_default`
    /// was one edit away from being addressable and undocumented
    /// (ADR-0bb7ea8991bc).
    ///
    /// **It measures a rendering now rather than arbitrating between two
    /// memories** (TASK-b08d090f699c). Both halves are
    /// `ank_contract::verbs::CONFIG_KEYS` -- the note is `concat!`-ed from it
    /// and `config::KEYS` is it -- so what is left to go wrong is the shape:
    /// the prefix the note is found by, and the split that reads the set back
    /// out of the sentence. A reader that has only the note to go on splits it
    /// exactly this way, so this is the assertion that keeps that road open.
    #[test]
    fn the_config_note_names_every_key_the_verb_addresses() {
        let note = spec_of("config")
            .unwrap()
            .notes
            .iter()
            .find(|n| n.starts_with("keys: "))
            .expect("config's first note is the key set");
        let listed: Vec<&str> = note["keys: ".len()..].split_whitespace().collect();
        assert_eq!(
            listed,
            crate::config::KEYS,
            "the note and the closed key set have drifted apart"
        );
    }

    #[test]
    fn json_is_accepted_on_every_command_without_exception() {
        // Invariant of the specification: full scriptability is not an option.
        // The test walks the table, so a verb added without --json would make
        // it fail.
        for spec in COMMANDS {
            let mut a = vec![spec.name.to_string()];
            if let Some(sub) = spec.subcommands.first() {
                a.push(sub.to_string());
            }
            a.push("--json".to_string());
            let inv = parse(&a)
                .unwrap_or_else(|e| panic!("--json refused on {}: {}", spec.name, e.render()));
            assert!(inv.json(), "--json not retained on {}", spec.name);
        }
        // A count, so that a verb added without a thought about `--json` fails
        // here. What §4 lists and what this table holds is a stronger question
        // and has its own test, in `tests/skill.rs`, which reads §4 rather than
        // counting.
        assert_eq!(
            COMMANDS.len(),
            26,
            "every verb of §4, plus init and help from §9. The surface is \
             complete, so this number moves only when §4 does"
        );
    }

    #[test]
    fn an_extra_positional_is_refused_never_ignored() {
        let err = parse(&argv(&["show", "8f3a", "51c2"])).unwrap_err();
        assert_eq!(err.code, ExitCode::Generic);
        assert!(err.message.contains("51c2"), "{}", err.message);
        assert_eq!(err.hint.as_deref(), Some("ank show <id>"));

        // `log` accepts two: [<id>] <message>.
        assert_eq!(ok(&["log", "8f3a", "message"]).positionals.len(), 2);
        assert!(parse(&argv(&["log", "8f3a", "message", "extra"])).is_err());
    }

    #[test]
    fn an_unknown_command_lists_the_commands() {
        let err = parse(&argv(&["claimm"])).unwrap_err();
        assert!(err.message.contains("claimm"), "{}", err.message);
        let hint = err.hint.unwrap();
        assert!(hint.contains("claim") && hint.contains("context"), "{hint}");
    }

    #[test]
    fn new_requires_its_subcommand() {
        let err = parse(&argv(&["new"])).unwrap_err();
        assert_eq!(err.hint.as_deref(), Some("ank new <task|adr|spec>"));

        let err = parse(&argv(&["new", "epic"])).unwrap_err();
        assert!(err.message.contains("epic"), "{}", err.message);

        let inv = ok(&["new", "task", "--title", "T", "--scope", "src/**"]);
        assert_eq!(inv.subcommand.as_deref(), Some("task"));

        let inv = ok(&["new", "spec", "--title", "T", "--scope", "src/**"]);
        assert_eq!(inv.subcommand.as_deref(), Some("spec"));
    }

    #[test]
    fn scope_is_repeatable_and_the_others_are_not() {
        let inv = ok(&[
            "new", "task", "--scope", "src/**", "--scope", "tests/**", "--title", "A", "--title",
            "B",
        ]);
        assert_eq!(inv.values("--scope"), ["src/**", "tests/**"]);
        // A non-repeatable flag keeps the last value, without accumulating.
        assert_eq!(inv.values("--title"), ["B"]);
    }

    #[test]
    fn a_switch_refuses_an_attached_value() {
        let err = parse(&argv(&["check", "--json=yes"])).unwrap_err();
        assert!(err.message.contains("takes no value"), "{}", err.message);
    }

    #[test]
    fn unimplemented_verbs_name_their_task() {
        for spec in COMMANDS.iter().filter(|c| c.owner_task.is_some()) {
            let err = not_implemented(spec);
            assert_eq!(err.code, ExitCode::Generic);
            let hint = err.hint.unwrap();
            assert!(hint.contains("TASK-"), "{}: {hint}", spec.name);
        }
    }

    #[test]
    fn a_routed_verb_carries_no_owner_task() {
        // `owner_task` is what `not_implemented` names, so leaving it set on a
        // verb dispatch reaches would advertise an implementation as missing
        // while it runs. Exactly the drift this task existed to fix, in the
        // opposite direction: `init`, `claim` and `context` are the verbs
        // routed today, and all must be clear of it.
        for routed in [
            "init", "help", "config", "claim", "context", "done", "log", "release", "new", "find",
            "attest", "amend", "show", "edit", "migrate", "tui", "mcp", "watch",
        ] {
            assert_eq!(
                spec_of(routed).unwrap().owner_task,
                None,
                "{routed} is routed by dispatch"
            );
        }
        // Every verb of the surface is routed now, so none may carry the
        // field any more. The day one is added, it arrives unrouted and this
        // fails until its arm exists.
        assert!(
            COMMANDS.iter().all(|c| c.owner_task.is_none()),
            "an unrouted verb must name its task, a routed one must not"
        );
    }

    #[test]
    fn the_foundation_is_resolved_before_the_verb_runs() {
        // A wrong --repo is named as such and never disguised as a broken
        // environment, which is why `startup` resolves the repository before
        // checking git.
        let mut out = Vec::new();
        let code = run(
            &argv(&["check", "--repo", "/path/that/does/not/exist"]),
            std::path::Path::new("."),
            &mut out,
            crate::style::PLAIN,
        );
        assert_eq!(code, ExitCode::Generic);

        // A valid --repo crosses the foundation and reaches the verb. This
        // used to assert that `check` was unimplemented; every verb of the
        // surface is routed now, so what it asserts instead is that the verb
        // answered — 0 or 8 from `check`, never the 1 of a rejection.
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let mut out = Vec::new();
        let code = run(
            &argv(&["check", "--repo", root.to_str().unwrap(), "--quiet"]),
            std::path::Path::new("."),
            &mut out,
            crate::style::PLAIN,
        );
        assert!(
            code == ExitCode::Ok || code == ExitCode::Findings,
            "check answered with {code}"
        );
    }

    #[test]
    fn init_does_not_go_through_the_foundation() {
        // init precedes the existence of the repository: requiring a `.ank/`
        // beforehand would make the command useless for what it is for.
        let inv = ok(&["init"]);
        assert_eq!(inv.command, "init");
        assert!(spec_of("init").unwrap().owner_task.is_none());
    }

    fn help_out(args: &[&str]) -> String {
        let inv = ok(args);
        let mut out = Vec::new();
        let code = help(&inv, &mut out).unwrap_or_else(|e| panic!("{args:?}: {}", e.render()));
        assert_eq!(code, ExitCode::Ok, "{args:?}");
        String::from_utf8(out).unwrap()
    }

    #[test]
    fn help_lists_every_verb_of_the_table() {
        // Walks COMMANDS rather than a written-out list: a verb added later
        // that the renderer forgets fails here, which is the whole reason the
        // listing is derived from the table instead of maintained beside it.
        //
        // The flags are asserted on the per-verb page and no longer in the
        // listing, which is where they moved when the description took their
        // place (§9, TASK-fe130d2b732c). The guarantee is unchanged — a flag
        // the renderer forgets still fails here — and it now watches the
        // surface that actually carries them, with their value placeholders.
        let text = help_out(&["help"]);
        // Whitespace collapsed, because a description too long for the column
        // is folded across lines: the fold is presentation, and asserting on
        // the text means asserting on the words rather than on where they broke.
        let flowed = text.split_whitespace().collect::<Vec<_>>().join(" ");
        for spec in COMMANDS {
            assert!(
                text.contains(&usage(spec)),
                "{} missing from the listing:\n{text}",
                spec.name
            );
            assert!(
                flowed.contains(
                    &spec
                        .summary
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                "{} is listed without its description:\n{text}",
                spec.name
            );
            let page = help_out(&["help", spec.name]);
            for f in listed_flags(spec) {
                assert!(
                    page.contains(f.name),
                    "{} of {} missing from its page:\n{page}",
                    f.name,
                    spec.name
                );
            }
        }
    }

    #[test]
    fn the_listing_keeps_the_table_order_inside_every_group() {
        // The grouping is a second axis laid over COMMANDS, not a re-sort
        // (ADR-91b77f036884) -- so it is asserted rather than assumed. The test
        // above passes just as well on a renderer that sorts alphabetically,
        // which would bury the loop in the middle of its own group.
        let text = help_out(&["help"]);
        for group in GROUPS {
            let mut at = 0usize;
            for spec in COMMANDS.iter().filter(|c| c.group == *group) {
                let u = usage(spec);
                let found = text[at..].find(&u).unwrap_or_else(|| {
                    panic!(
                        "{} out of order or missing under '{group}':\n{text}",
                        spec.name
                    )
                });
                at += found + u.len();
            }
        }
        // Nothing above the first heading, and the first heading is the moment
        // the tool is entered at: a title would put something between a reader
        // and the loop.
        assert!(
            text.starts_with(&format!("{}\n{}", GROUPS[0], usage(&COMMANDS[0]))),
            "the listing does not open on the first group and its first verb:\n{text}"
        );
    }

    #[test]
    fn every_verb_has_a_group_and_every_group_has_verbs() {
        // The two halves of what stops a twenty-second verb from being added
        // with no home and disappearing off the end of a listing that renders
        // by group (ADR-91b77f036884). The integration suite asserts the same
        // property through the binary, which is where a caller reads it; this
        // one fails at the table, which is where the mistake is made.
        for spec in COMMANDS {
            assert!(
                GROUPS.contains(&spec.group),
                "{} carries the group {:?}, which nothing prints",
                spec.name,
                spec.group
            );
        }
        for group in GROUPS {
            assert!(
                COMMANDS.iter().any(|c| c.group == *group),
                "'{group}' is a heading with no verb under it"
            );
            assert_eq!(
                *group,
                group.to_lowercase(),
                "a heading is a signpost, not a title"
            );
        }
    }

    #[test]
    fn help_for_one_verb_answers_about_that_verb_alone() {
        let text = help_out(&["help", "claim"]);
        assert!(text.contains("ank claim <id>"), "{text}");
        // The flags carry their value placeholder here, which is the detail §9
        // moves out of SKILL.md and into this command.
        assert!(text.contains("--ttl <v>"), "{text}");
        assert!(text.contains("--criteria <v>"), "{text}");
        assert!(
            !text.contains("audience"),
            "the audience line is what ADR-91b77f036884 removes:\n{text}"
        );
        // One verb means one verb: no other usage line rides along.
        assert!(!text.contains("ank accept"), "{text}");

        // A repeatable flag is shown as repeatable.
        let text = help_out(&["help", "new"]);
        assert!(text.contains("--scope <v>..."), "{text}");
    }

    #[test]
    fn an_unknown_verb_passed_to_help_is_a_two_and_never_the_listing() {
        let inv = ok(&["help", "clam"]);
        let mut out = Vec::new();
        let err = help(&inv, &mut out).unwrap_err();
        assert_eq!(
            err.code,
            ExitCode::NotFound,
            "entity not found, per the table of §4"
        );
        assert!(err.message.contains("clam"), "{}", err.message);
        assert_eq!(err.hint.as_deref(), Some("ank help"));
        assert!(
            out.is_empty(),
            "a silent fallback to the general listing: {}",
            String::from_utf8_lossy(&out)
        );
    }

    #[test]
    fn help_speaks_json_in_both_of_its_forms() {
        let all = help_out(&["help", "--json"]);
        // The envelope first and the verbs behind it (ADR-6fd69efb629c): the
        // contract version is the one field a client reads before it knows
        // whether it can read the rest, so it leads.
        assert!(all.starts_with("{\"contract\":1,\"verbs\":["), "{all}");
        for spec in COMMANDS {
            assert!(
                all.contains(&format!("\"name\":\"{}\"", spec.name)),
                "{all}"
            );
        }
        assert!(
            !all.contains("audience"),
            "the audience key carried a grouping by caller into the scripted \
             output, which is the one grouping there is no claim left to \
             make:\n{all}"
        );
        assert!(all.contains("\"takes_value\":false"), "{all}");
        // The moment a verb belongs to reaches a script too: structure is
        // emitted identically to everyone (§4), and only colour depends on the
        // reader.
        for spec in COMMANDS {
            assert!(
                all.contains(&format!("\"group\":\"{}\"", spec.group)),
                "{} is listed without its group:\n{all}",
                spec.name
            );
        }

        let one = help_out(&["help", "claim", "--json"]);
        assert!(one.contains("\"name\":\"claim\""), "{one}");
        assert!(!one.contains("\"name\":\"accept\""), "{one}");
        // --json carries the globals too: they are part of what the verb takes.
        assert!(one.contains("\"name\":\"--repo\""), "{one}");
    }

    #[test]
    fn quiet_help_says_nothing_and_still_exits_zero() {
        assert_eq!(help_out(&["help", "--quiet"]), "");
        assert_eq!(help_out(&["help", "claim", "--quiet"]), "");
    }

    #[test]
    fn error_rendering_follows_the_shape_in_the_spec() {
        let err = CliError::new(ExitCode::Prerequisite, "TASK-51c2 has no done_criteria")
            .with_hint("ank claim 51c2 --criteria \"<verifiable criterion>\"");
        assert_eq!(
            err.render(),
            "error[7]: TASK-51c2 has no done_criteria\n  -> ank claim 51c2 --criteria \"<verifiable criterion>\""
        );
        assert_eq!(
            CliError::new(ExitCode::Generic, "no next step").render(),
            "error[1]: no next step"
        );
    }
}
