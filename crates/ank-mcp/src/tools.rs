//! The tool list, generated from the verb table (ADR-372b82af1ec7).
//!
//! **Nothing here names a verb.** The list is [`COMMANDS`] walked, so a verb the
//! table gains reaches this surface with no edit in this crate, and a verb it
//! does not carry is a verb this surface does not have. That is the condition
//! ADR-1713af205186 wrote for itself and ADR-372b82af1ec7 answered: not a
//! curated subset, not parity kept by review, generated.
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
/// `--repo` because the server speaks for **exactly one corpus**, addressed once
/// at startup: a per-call repository would make one process several corpora,
/// which is the merged claim space ADR-372b82af1ec7 forbids in as many words.
/// `--json` because the server always wants the machine document and a client
/// asking for the human one would get a shape nothing describes. `--quiet` means
/// nothing to a caller that reads a return value rather than a terminal.
pub const SERVER_FLAGS: [&str; 3] = ["--repo", "--json", "--quiet"];

/// Whether a flag is the client's to pass.
pub fn client_flag(name: &str) -> bool {
    !SERVER_FLAGS.contains(&name)
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
