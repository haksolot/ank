//! The tool list, generated from the verb table (ADR-fd98f4bc6dea).
//!
//! **Nothing here names a verb.** The list is [`COMMANDS`] walked, so a verb the
//! table gains reaches this surface with no edit in this crate, and a verb it
//! does not carry is a verb this surface does not have. That is the condition
//! this surface was permitted on (ADR-fd98f4bc6dea), on terms the refusal it
//! replaced had written for itself: not a curated subset, not parity kept by review, generated.
//!
//! What a tool advertises is what the table already knows: the summary as the
//! description, the positionals and flags as the input schema, the refusals with
//! their exit codes so a client can see what a call will refuse *before* making
//! it, and the shape of the document that comes back.

use ank_contract::json::Obj;
use ank_contract::{CommandSpec, COMMANDS};

/// The three global flags, which are the server's business and never the
/// client's.
///
/// This is not a curated subset: no verb is hidden and every verb takes exactly
/// the arguments the table gives it. What is withheld is the three flags that
/// would let a caller contradict the process it is talking to.
///
/// `--repo` because a corpus is named by its identity here and never by a path
/// (ADR-fd98f4bc6dea, [`crate::corpora`]). A server may reach several corpora --
/// the one it was addressed with at startup, and the ones its reader declared --
/// and every one of them is reached by naming a root commit that resolves to a
/// declaration. A caller that could write a path into a flag would reach every
/// corpus on the machine, which is exactly what turns a declared set into a
/// merged one.
/// `--json` because the server always wants the machine document and a client
/// asking for the human one would get a shape nothing describes. `--quiet` means
/// nothing to a caller that reads a return value rather than a terminal.
///
/// **Each with the reason it is withheld for, in the same row**
/// (TASK-308ce062f427). The reason used to be one sentence for all three and
/// the sentence was `--repo`'s, so a caller that passed `"json": true` was told
/// to name a corpus by its identity and never by a path -- an answer to a
/// question it had not asked, which is worse than a bare refusal because a
/// caller acts on it. A row rather than a `match` beside the list: a name added
/// here cannot arrive without a reason, and a reason cannot be written for a
/// name that is not withheld.
pub const SERVER_FLAGS: [(&str, &str); 3] = [
    (
        "--repo",
        // The one sentence this task did not change. `corpus` is spelled here
        // and held to `crate::corpora::ARGUMENT` by the unit test below, which
        // interpolates the constant rather than repeating the word.
        "name a corpus with the corpus argument, by the identity ank status \
         --json prints, never by a path",
    ),
    (
        "--json",
        "a call already comes back as the machine document, and the human one \
         is a shape this schema does not describe",
    ),
    (
        "--quiet",
        "it silences a terminal, and a call reads the document it gets back \
         rather than watching one",
    ),
];

/// The refusal a flag of this server earns, or `None` for a flag the client may
/// pass.
pub fn withheld(flag: &str) -> Option<String> {
    SERVER_FLAGS
        .iter()
        .find(|(name, _)| *name == flag)
        .map(|(name, why)| format!("{name} belongs to the server: {why}"))
}

/// Whether a flag is the client's to pass.
///
/// Derived from [`withheld`], so that "may the caller pass this" and "what do
/// we tell it if not" have exactly one answer between them: a flag the schema
/// hides is a flag the refusal has a reason for, by construction.
pub fn client_flag(name: &str) -> bool {
    withheld(name).is_none()
}

/// A tool name. `ank_<verb>`, because a bare verb collides with every other
/// server a client has loaded, and `ank context` is not a legal tool name.
pub fn tool_name(spec: &CommandSpec) -> String {
    format!("ank_{}", spec.name)
}

/// The verb a tool name refers to, or `None` for a name this surface never
/// advertised.
pub fn verb_of(tool: &str) -> Option<&'static CommandSpec> {
    ank_contract::spec_of(tool.strip_prefix("ank_")?)
}

/// The description a client reads: what the verb does, then what it refuses on
/// and with which code.
///
/// The refusals are in the description because MCP has nowhere else to put them
/// and because they are the half a caller needs first. A client that can see
/// `7: the task is blocked` before calling does not have to learn it from a
/// failure.
fn description(spec: &CommandSpec) -> String {
    let mut out = String::from(spec.summary);
    for note in spec.notes {
        out.push_str("\n\nNote: ");
        out.push_str(note);
    }
    if !spec.refuses.is_empty() {
        out.push_str("\n\nRefuses on state, never on identity:");
        for refusal in spec.refuses {
            out.push_str(&format!("\n  {} (exit {})", refusal.when, refusal.code));
        }
    }
    if let Some(shape) = spec.output.first() {
        out.push_str("\n\nReturns a document carrying: contract");
        for field in shape.fields {
            out.push_str(", ");
            out.push_str(field.name);
        }
        if spec.output.len() > 1 {
            out.push_str(&format!(
                "\n{} different documents, depending on the call; ank help --json describes each.",
                spec.output.len()
            ));
        }
    }
    out
}

/// The JSON Schema of a call, derived from the table's own account of the verb.
///
/// Positionals arrive as `arguments`, an array of strings, because that is what
/// they are on the command line and inventing names for them here would be a
/// second description of the same thing. Flags arrive under their own names,
/// without the leading dashes, since a JSON key beginning with `--` is a key
/// every client would have to quote.
fn input_schema(spec: &CommandSpec) -> String {
    let mut props = Obj::new();
    if spec.max_positionals > 0 || !spec.subcommands.is_empty() {
        let mut items = Obj::new().str("type", "string");
        if !spec.subcommands.is_empty() {
            items = items.raw(
                "enum",
                &ank_contract::json::strings(spec.subcommands.iter().copied()),
            );
        }
        props = props.obj(
            "arguments",
            Obj::new()
                .str("type", "array")
                .obj("items", items)
                .str("description", spec.positional_help),
        );
    }
    for flag in spec.flags.iter().filter(|f| f.listed) {
        let name = flag.name.trim_start_matches('-');
        if !client_flag(flag.name) {
            continue;
        }
        let value = match (flag.takes_value, flag.repeatable) {
            (false, _) => Obj::new().str("type", "boolean"),
            (true, false) => Obj::new().str("type", "string"),
            (true, true) => Obj::new()
                .str("type", "array")
                .obj("items", Obj::new().str("type", "string")),
        };
        props = props.obj(name, value);
    }
    // **Every tool, and never a list of the ones it would suit**
    // (ADR-fd98f4bc6dea). The argument says which corpus a call is addressed to,
    // and a tool that did not carry it would be a verb a multi-corpus client can
    // only reach in one of them -- the curated subset this surface was permitted
    // on condition of not having, arriving through the back door as a curated
    // set of *corpora* per verb.
    //
    // Last, after the verb's own arguments, so that a schema a client already
    // reads is extended rather than rewritten: no property it knew has moved.
    // It is never in `required`, which is the whole of what optional means here
    // -- absent, the call goes where every call went before this existed.
    props = props.obj(
        crate::corpora::ARGUMENT,
        Obj::new()
            .str("type", "string")
            .str("description", crate::corpora::HELP),
    );
    Obj::new()
        .str("type", "object")
        .obj("properties", props)
        .finish()
}

/// Every verb, as a tool.
pub fn list() -> String {
    let tools = COMMANDS.iter().map(|spec| {
        Obj::new()
            .str("name", &tool_name(spec))
            .str("description", &description(spec))
            .raw("inputSchema", &input_schema(spec))
            .finish()
    });
    ank_contract::json::array(tools)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every tool carries the argument, and the table cannot take the name back.
    ///
    /// Two assertions and they are two different failures. The first is the
    /// criterion: a verb the table gains reaches this surface addressable in
    /// every corpus the server can reach, with no edit here. The second is the
    /// collision that would make the first one silently false -- a verb
    /// declaring a `--corpus` flag would put two properties of that name in one
    /// schema, and the surface's own argument is the one a client would stop
    /// being able to pass.
    #[test]
    fn every_tool_carries_the_corpus_argument_and_no_verb_shadows_it() {
        let needle = format!("\"{}\":", crate::corpora::ARGUMENT);
        for spec in COMMANDS {
            let schema = input_schema(spec);
            assert!(
                schema.contains(&needle),
                "{} does not advertise the corpus argument, so a client can \
                 reach it in one corpus only (ADR-fd98f4bc6dea): {schema}",
                spec.name
            );
            assert_eq!(
                schema.matches(&needle).count(),
                1,
                "{} advertises the corpus argument twice: a verb has grown a \
                 --corpus flag and the table now shadows the surface's own \
                 argument",
                spec.name
            );
            assert!(
                ank_contract::find_flag(spec, "--corpus").is_none(),
                "{} declares a --corpus flag: the surface names a corpus by its \
                 identity and a verb naming one by anything else would be a \
                 second answer to one question",
                spec.name
            );
        }
    }

    /// The argument is optional, and optional is a property of the document
    /// rather than of the prose: nothing in a generated schema is required, so a
    /// client that passed no corpus yesterday passes none today.
    #[test]
    fn the_corpus_argument_is_never_required() {
        for spec in COMMANDS {
            let schema = input_schema(spec);
            assert!(
                !schema.contains("\"required\""),
                "{} requires something, and the corpus argument must not be it: \
                 {schema}",
                spec.name
            );
        }
    }

    /// The three global flags stay the server's, `--repo` most of all: it is the
    /// one that takes a path, and a path is how a declared set becomes a merged
    /// one.
    #[test]
    fn the_globals_stay_the_servers() {
        for (flag, _) in SERVER_FLAGS {
            assert!(!client_flag(flag), "{flag} reached the client");
        }
        assert!(client_flag("--json-lines"), "a flag is not a prefix match");
    }

    /// Each withheld flag has its own reason, and the list cannot outgrow them.
    ///
    /// Three assertions and three different failures. The first says the
    /// refusal opens by naming what it is refusing, which is the half a caller
    /// reads first. The second is the defect this task closed: a sentence some
    /// other flag also gets is `--repo`'s reason handed to a caller that asked
    /// about `--json`, and every one of the three was individually a refusal,
    /// so a test that only looked for a refusal was green over it. The third
    /// says a flag that is not withheld earns no reason -- a prefix is not a
    /// match here any more than it is in `client_flag`.
    #[test]
    fn each_withheld_flag_carries_its_own_reason() {
        let mut seen: Vec<String> = Vec::new();
        for (flag, _) in SERVER_FLAGS {
            let refusal = withheld(flag).unwrap_or_else(|| {
                panic!(
                    "{flag} is listed as the server's and withheld() has no \
                     reason for it, so client_flag lets it through"
                )
            });
            assert!(
                refusal.starts_with(&format!("{flag} belongs to the server: ")),
                "the refusal for {flag} does not open by naming it: {refusal}"
            );
            assert!(
                !seen.contains(&refusal),
                "{flag} is refused with a sentence another flag already got, \
                 which is the defect this test exists for: {refusal}"
            );
            seen.push(refusal);
        }
        assert_eq!(seen.len(), SERVER_FLAGS.len());
        assert!(
            withheld("--json-lines").is_none(),
            "a reason is not a prefix match either"
        );
    }

    /// `--repo` keeps the sentence it had, and keeps it anchored on the
    /// argument's own constant rather than on a second spelling of the name.
    #[test]
    fn the_repo_reason_is_the_one_it_always_was() {
        assert_eq!(
            withheld("--repo").as_deref(),
            Some(
                format!(
                    "--repo belongs to the server: name a corpus with the {} \
                     argument, by the identity ank status --json prints, never \
                     by a path",
                    crate::corpora::ARGUMENT
                )
                .as_str()
            )
        );
    }
}
