//! Loading `.ank/config.yml` (§4, §8).
//!
//! The file is controlled by the repository, and therefore reviewed like any
//! code change — that is what makes it acceptable for `done` to run its
//! verifiers. An unknown `schema` is cleanly refused rather than misread:
//! that is the counterpart of "the format is the specification".

use crate::cli::{CliError, Invocation};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::io::Write;
use std::ops::Range;
use std::path::Path;
use std::time::Duration;

pub const SUPPORTED_SCHEMA: u32 = 1;
pub const DEFAULT_CONTEXT_BUDGET: usize = 8000;
pub const DEFAULT_CLAIM_TTL_MAX: &str = "2h";
/// What `claim` grants without `--ttl` (§3). The same thirty minutes
/// [`crate::claim::DEFAULT_TTL`] has always been, spelled the way the file
/// spells a duration; a unit test below pins the two to one value, because two
/// spellings of one number is exactly how they start to disagree.
pub const DEFAULT_CLAIM_TTL: &str = "30m";
pub const DEFAULT_VERIFIER_TIMEOUT: &str = "10m";

pub type Result<T> = std::result::Result<T, CliError>;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigFile {
    schema: u32,
    #[serde(default = "default_budget")]
    context_budget: usize,
    #[serde(default = "default_ttl_max")]
    claim_ttl_max: String,
    #[serde(default = "default_ttl")]
    claim_ttl_default: String,
    #[serde(default)]
    default_branch: Option<String>,
    /// The corpora this repository reads, by name (§7, ADR-a1de673043b4). A key
    /// per peer rather than a list, so `ank config peers.<name> <path>` reaches
    /// exactly one declaration -- and so that adding federation adds a key
    /// instead of relaxing the `deny_unknown_fields` above.
    #[serde(default)]
    peers: BTreeMap<String, String>,
    #[serde(default)]
    verifiers: BTreeMap<String, VerifierFile>,
    #[serde(default)]
    roles: BTreeMap<String, Role>,
    #[serde(default)]
    identities: BTreeMap<String, String>,
}

fn default_budget() -> usize {
    DEFAULT_CONTEXT_BUDGET
}

fn default_ttl_max() -> String {
    DEFAULT_CLAIM_TTL_MAX.to_string()
}

fn default_ttl() -> String {
    DEFAULT_CLAIM_TTL.to_string()
}

fn default_timeout() -> String {
    DEFAULT_VERIFIER_TIMEOUT.to_string()
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VerifierFile {
    run: String,
    #[serde(default = "default_timeout")]
    timeout: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Role {
    #[serde(default)]
    pub can: Vec<String>,
    #[serde(default)]
    pub cannot: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verifier {
    pub run: String,
    pub timeout: Duration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub schema: u32,
    pub context_budget: usize,
    pub claim_ttl_max: Duration,
    /// What a claim is granted when `--ttl` says nothing (§3). Capped by
    /// [`Config::claim_ttl_max`] at claim time and not here: the file may state
    /// a default above the cap, and the refusal §3 wants is the cap binding
    /// what is granted, not a configuration that will not load.
    pub claim_ttl_default: Duration,
    /// Branch carrying the reference durable state (§7). Optional: absent, the
    /// resolution falls back to `refs/remotes/origin/HEAD`, and fails rather
    /// than guessing if that is absent too — see
    /// [`crate::git::resolve_default_branch`].
    pub default_branch: Option<String>,
    /// Peer corpora, by the name a scope entry uses to reach them (§7). The
    /// value is the path to the peer's root, resolved against this repository's
    /// root when it is relative — never discovered, never inferred from a
    /// remote, because inference is how a corpus starts depending on where
    /// somebody checked something out.
    pub peers: BTreeMap<String, String>,
    pub verifiers: BTreeMap<String, Verifier>,
    pub roles: BTreeMap<String, Role>,
    pub identities: BTreeMap<String, String>,
}

impl Config {
    pub fn verifier(&self, name: &str) -> Option<&Verifier> {
        self.verifiers.get(name)
    }
}

/// Durations of the form `<n><unit>`, units `s`, `m`, `h`, `d`. Deliberately
/// narrow: `claim_ttl_max: 2h` must read without documentation, and a richer
/// grammar would only add forms nobody should write.
pub fn parse_duration(text: &str) -> std::result::Result<Duration, String> {
    let t = text.trim();
    if t.is_empty() {
        return Err("empty duration".to_string());
    }
    let (digits, unit) = t.split_at(
        t.find(|c: char| !c.is_ascii_digit())
            .unwrap_or_else(|| t.len()),
    );
    if digits.is_empty() {
        return Err(format!("duration '{text}': digit expected before the unit"));
    }
    let n: u64 = digits
        .parse()
        .map_err(|_| format!("duration '{text}': unreadable number"))?;
    let secs = match unit {
        "s" => n,
        "m" => n * 60,
        "h" => n * 3600,
        "d" => n * 86_400,
        "" => return Err(format!("duration '{text}': missing unit (s, m, h, d)")),
        other => {
            return Err(format!(
                "duration '{text}': unknown unit '{other}' (s, m, h, d)"
            ))
        }
    };
    Ok(Duration::from_secs(secs))
}

pub fn parse(text: &str, path: &Path) -> Result<Config> {
    let raw: ConfigFile = serde_yaml::from_str(text)
        .map_err(|e| CliError::new(1, format!("{}: {e}", path.display())))?;

    if raw.schema != SUPPORTED_SCHEMA {
        return Err(CliError::new(
            1,
            format!(
                "{}: unknown schema {} (supported: {SUPPORTED_SCHEMA})",
                path.display(),
                raw.schema
            ),
        )
        .with_hint("update ank, or fix the schema field"));
    }

    let dur = |v: &str, field: &str| -> Result<Duration> {
        parse_duration(v).map_err(|e| CliError::new(1, format!("{}: {field}: {e}", path.display())))
    };

    let claim_ttl_max = dur(&raw.claim_ttl_max, "claim_ttl_max")?;
    let claim_ttl_default = dur(&raw.claim_ttl_default, "claim_ttl_default")?;
    let mut verifiers = BTreeMap::new();
    for (name, v) in raw.verifiers {
        let timeout = dur(&v.timeout, &format!("verifiers.{name}.timeout"))?;
        verifiers.insert(
            name,
            Verifier {
                run: v.run,
                timeout,
            },
        );
    }

    // A key present but blank names no branch: it is an absence written out,
    // not a branch called "".
    let default_branch = raw
        .default_branch
        .map(|b| b.trim().to_string())
        .filter(|b| !b.is_empty());

    Ok(Config {
        schema: raw.schema,
        context_budget: raw.context_budget,
        claim_ttl_max,
        claim_ttl_default,
        default_branch,
        // A declaration whose path is blank names no corpus. Kept rather than
        // dropped: resolution warns about it by name, and a peer silently
        // removed at parse time is a declaration nobody can see failing.
        peers: raw
            .peers
            .into_iter()
            .map(|(name, path)| (name, path.trim().to_string()))
            .collect(),
        verifiers,
        roles: raw.roles,
        identities: raw.identities,
    })
}

pub fn load(path: &Path) -> Result<Config> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CliError::new(1, format!("{} not found", path.display())).with_hint("ank init")
        } else {
            CliError::new(1, format!("{}: {e}", path.display()))
        }
    })?;
    parse(&text, path)
}

/// Content written by `ank init`. The canonical form of the file.
///
/// `default_branch` is deliberately absent: `init` runs where the reference
/// branch is not known yet, and writing `main` here would be exactly the guess
/// §7 refuses. Detection through `refs/remotes/origin/HEAD` covers the case,
/// and the error names the key to add when it does not.
pub fn default_yaml() -> String {
    "\
schema: 1
context_budget: 8000
claim_ttl_max: 2h
verifiers: {}
roles:
  agent:
    can: [context, find, claim, log, done, new:task, new:adr:proposed]
    cannot: [adr:accept, adr:edit-constraint, task:close, delete]
  human:
    can: [\"*\"]
identities: {}
"
    .to_string()
}

// ---------------------------------------------------------------------------
// `ank config` (§4, ADR-e64dfaafd578)
// ---------------------------------------------------------------------------
//
// Text surgery, and never a round-trip through a serializer. `Config` derives
// `Deserialize` and nothing else, and building the other half would be the
// wrong move rather than a shortcut avoided: a parse-mutate-serialize writer
// drops every comment and blank line, returns `verifiers`, `roles` and
// `identities` alphabetised because they are `BTreeMap`s, and -- the reason
// this is not a matter of taste -- writes out every field carrying a serde
// default. An unset key means "follows the tool"; a written one means "pinned
// here". So a round-trip turns absence into assertion, and every repository
// that ever ran one `ank config` silently holds the old value the day a default
// moves.
//
// The writer therefore finds the line, replaces the span the value occupies,
// and leaves every other byte alone.

/// The keys this verb addresses, spelled the way §4's table spells them.
pub const KEYS: &[&str] = &[
    "schema",
    "context_budget",
    "claim_ttl_max",
    "claim_ttl_default",
    "default_branch",
    "peers.<name>",
    "verifiers.<name>.run",
    "verifiers.<name>.timeout",
];

/// A dotted path, resolved against the closed key set.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Key {
    /// A scalar at the root of the document.
    Top {
        name: &'static str,
        /// Emitted verbatim: the field holds a number, and quoting one would
        /// store a string the parser refuses.
        numeric: bool,
        default: Option<String>,
    },
    /// `verifiers.<name>.run` or `.timeout`.
    Field {
        verifier: String,
        field: &'static str,
        default: Option<String>,
    },
    /// `verifiers.<name>`: legal for `--unset` alone, which is what makes
    /// declaring a verifier reversible.
    Block { verifier: String },
    /// `peers.<name>`: one scalar under one mapping, and how a peer corpus is
    /// declared (§7). One level shallower than a verifier, so the whole key is
    /// the declaration and `--unset` on it removes the peer outright.
    Peer { name: String },
}

fn unknown_key(path: &str) -> CliError {
    CliError::new(1, format!("unknown key '{path}'")).with_hint(format!("keys: {}", KEYS.join(" ")))
}

fn structured(name: &str) -> CliError {
    CliError::new(
        1,
        format!("'{name}' has a structured value, which ank config does not address"),
    )
    .with_hint(format!(
        "edit .ank/config.yml by hand: {name} is a mapping of lists, not a scalar"
    ))
}

/// A `run` written as a block or folded scalar. Refused rather than rewritten:
/// its resolved string depends on line structure a one-line replacement would
/// flatten, and `verify::definition_hash` is taken over the resolved value, so
/// flattening it moves a hash that anchors historical proofs.
fn blocky(name: &str) -> CliError {
    CliError::new(
        1,
        format!("'{name}' is a block or folded scalar, which ank config does not rewrite"),
    )
    .with_hint(
        "edit .ank/config.yml by hand: flattening it would move the verifier's definition hash",
    )
}

fn flow_mapping(name: &str) -> CliError {
    CliError::new(
        1,
        format!("'{name}' is written as a non-empty flow mapping, which ank config does not edit"),
    )
    .with_hint("edit .ank/config.yml by hand, or write it as a block mapping")
}

fn undeclared(verifier: &str) -> CliError {
    CliError::new(7, format!("verifier '{verifier}' is not declared"))
        .with_hint(format!("ank config verifiers.{verifier}.run \"<command>\""))
}

fn whole_block(verifier: &str) -> CliError {
    CliError::new(
        1,
        format!("'verifiers.{verifier}' is a whole verifier, not a value: --unset removes it"),
    )
    .with_hint(format!("ank config verifiers.{verifier}.run"))
}

fn resolve_key(path: &str) -> Result<Key> {
    let segs: Vec<&str> = path.split('.').collect();
    if segs.iter().any(|s| s.is_empty()) {
        return Err(unknown_key(path));
    }
    match segs.as_slice() {
        ["schema"] => Ok(Key::Top {
            name: "schema",
            numeric: true,
            default: None,
        }),
        ["context_budget"] => Ok(Key::Top {
            name: "context_budget",
            numeric: true,
            default: Some(DEFAULT_CONTEXT_BUDGET.to_string()),
        }),
        ["claim_ttl_max"] => Ok(Key::Top {
            name: "claim_ttl_max",
            numeric: false,
            default: Some(DEFAULT_CLAIM_TTL_MAX.to_string()),
        }),
        ["claim_ttl_default"] => Ok(Key::Top {
            name: "claim_ttl_default",
            numeric: false,
            default: Some(DEFAULT_CLAIM_TTL.to_string()),
        }),
        // No default: absent, the resolution falls back to
        // refs/remotes/origin/HEAD and fails rather than guessing (§7).
        ["default_branch"] => Ok(Key::Top {
            name: "default_branch",
            numeric: false,
            default: None,
        }),
        ["peers"] => Err(
            CliError::new(1, "'peers' is a mapping: address one by name")
                .with_hint("ank config peers.<name> <path>"),
        ),
        ["peers", name] => {
            // A name a scope entry could never spell is a declaration nothing
            // can reach, and writing it would be a silent no-op rather than a
            // configuration. Refused here, where the caller can still type
            // another one.
            if !crate::repo::is_peer_name(name) {
                return Err(CliError::new(
                    1,
                    format!(
                        "peer name '{name}' cannot be named by a scope: \
                         two or more of a-z, A-Z, 0-9, '-' and '_'"
                    ),
                )
                .with_hint("ank config peers.<name> <path>"));
            }
            Ok(Key::Peer {
                name: (*name).to_string(),
            })
        }
        ["peers", _, ..] => Err(CliError::new(
            1,
            format!("'{path}': a peer is one path, not a block"),
        )
        .with_hint("ank config peers.<name> <path>")),
        ["roles", ..] => Err(structured("roles")),
        ["identities", ..] => Err(structured("identities")),
        ["verifiers"] => Err(
            CliError::new(1, "'verifiers' is a mapping: address one by name")
                .with_hint("ank config verifiers.<name>.run \"<command>\""),
        ),
        ["verifiers", name] => Ok(Key::Block {
            verifier: (*name).to_string(),
        }),
        ["verifiers", name, "run"] => Ok(Key::Field {
            verifier: (*name).to_string(),
            field: "run",
            default: None,
        }),
        ["verifiers", name, "timeout"] => Ok(Key::Field {
            verifier: (*name).to_string(),
            field: "timeout",
            default: Some(DEFAULT_VERIFIER_TIMEOUT.to_string()),
        }),
        ["verifiers", _, other] => Err(CliError::new(
            1,
            format!("unknown verifier field '{other}'"),
        )
        .with_hint("ank config verifiers.<name>.run   or   ank config verifiers.<name>.timeout")),
        ["verifiers", _, _, _, ..] => Err(CliError::new(
            1,
            format!(
                "'{path}': a verifier whose name contains '.' cannot be addressed by a dotted path"
            ),
        )
        .with_hint("edit .ank/config.yml by hand, or rename the verifier")),
        _ => Err(unknown_key(path)),
    }
}

// ---------------------------------------------------------------------------
// Lines, kept exactly as they were read
// ---------------------------------------------------------------------------

/// One physical line and the terminator it carried.
///
/// Each line keeps its own, rather than the file keeping one: §3 forbids ank to
/// *write* CRLF and says nothing about rewriting a CRLF checkout in place, and
/// a writer that normalised terminators would rewrite every line of such a file
/// to change one key.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Line {
    text: String,
    eol: String,
}

fn split_lines(text: &str) -> Vec<Line> {
    let mut out = Vec::new();
    let mut rest = text;
    loop {
        match rest.find('\n') {
            Some(i) => {
                let raw = &rest[..i];
                let (body, eol) = match raw.strip_suffix('\r') {
                    Some(b) => (b, "\r\n"),
                    None => (raw, "\n"),
                };
                out.push(Line {
                    text: body.to_string(),
                    eol: eol.to_string(),
                });
                rest = &rest[i + 1..];
            }
            None => {
                if !rest.is_empty() {
                    out.push(Line {
                        text: rest.to_string(),
                        eol: String::new(),
                    });
                }
                return out;
            }
        }
    }
}

fn join_lines(lines: &[Line]) -> String {
    let mut s = String::new();
    for l in lines {
        s.push_str(&l.text);
        s.push_str(&l.eol);
    }
    s
}

/// The terminator a line appended to this file should carry.
fn dominant_eol(lines: &[Line]) -> String {
    if lines.iter().any(|l| l.eol == "\r\n") {
        "\r\n".to_string()
    } else {
        "\n".to_string()
    }
}

/// Gives the last line a terminator, so that appending after it does not join
/// the two. Only the last line can be unterminated.
fn terminate_last(lines: &mut [Line], eol: &str) {
    if let Some(last) = lines.last_mut() {
        if last.eol.is_empty() {
            last.eol = eol.to_string();
        }
    }
}

/// Removes whole lines, terminator included.
///
/// Removing the last line of a file that had no trailing newline leaves the
/// previous line's terminator in place, and that is the faithful reading rather
/// than an oversight: that byte was in the file, the caller asked for the key
/// below it, and clearing it would delete a byte nobody named. The absence of a
/// trailing newline is only preserved when the line carrying it survives.
fn remove_lines(lines: &mut Vec<Line>, r: Range<usize>) {
    lines.drain(r);
}

// ---------------------------------------------------------------------------
// Reading the structure: enough YAML to find a line, and no more
// ---------------------------------------------------------------------------

/// The indentation of a line that carries content.
///
/// `None` for a blank line or one holding only a comment. Neither belongs to a
/// block, and reporting them at indent 0 would close every block a comment
/// happens to sit in.
fn content_indent(text: &str) -> Option<usize> {
    let n = text.len() - text.trim_start_matches(' ').len();
    let rest = &text[n..];
    if rest.trim().is_empty() || rest.starts_with('#') {
        return None;
    }
    Some(n)
}

/// A quoted scalar at the head of `s`: its value, and the bytes it occupies.
fn read_quoted(s: &str, quote: char) -> Option<(String, usize)> {
    let bytes = s.as_bytes();
    let mut out = String::new();
    let mut i = quote.len_utf8();
    while i < bytes.len() {
        let b = bytes[i];
        if quote == '"' && b == b'\\' {
            let n = *bytes.get(i + 1)? as char;
            out.push(match n {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                other => other,
            });
            i += 2;
            continue;
        }
        if quote == '"' && b == b'"' {
            return Some((out, i + 1));
        }
        if quote == '\'' && b == b'\'' {
            if bytes.get(i + 1) == Some(&b'\'') {
                out.push('\'');
                i += 2;
                continue;
            }
            return Some((out, i + 1));
        }
        let ch = s[i..].chars().next()?;
        out.push(ch);
        i += ch.len_utf8();
    }
    None
}

/// The key a line declares, and the byte offset just past its colon.
///
/// Quoted keys are read as keys: `identities` carries `"marie@laptop": human`,
/// and a scan stopping at the first colon would find one inside the quotes.
fn key_of(text: &str) -> Option<(String, usize)> {
    let indent = text.len() - text.trim_start_matches(' ').len();
    let rest = &text[indent..];
    let first = rest.chars().next()?;
    let (name, mut at) = if first == '"' || first == '\'' {
        let (v, len) = read_quoted(rest, first)?;
        (v, indent + len)
    } else {
        let i = rest.find(':')?;
        (rest[..i].trim_end().to_string(), indent + i)
    };
    let bytes = text.as_bytes();
    while bytes.get(at) == Some(&b' ') {
        at += 1;
    }
    if bytes.get(at) != Some(&b':') {
        return None;
    }
    // `key: value` and `key:`, never `key:value` -- YAML requires the space.
    match bytes.get(at + 1) {
        None | Some(b' ') => Some((name, at + 1)),
        _ => None,
    }
}

/// What a value occupies on its line, as a byte range into it.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ValueSpan {
    start: usize,
    end: usize,
    /// The quote the value already carried, so a rewrite keeps its style.
    quote: Option<char>,
    /// `|` or `>`.
    blocky: bool,
    value: String,
}

fn value_span(text: &str, after_colon: usize) -> ValueSpan {
    let bytes = text.as_bytes();
    let mut start = after_colon;
    while bytes.get(start) == Some(&b' ') {
        start += 1;
    }
    let empty = ValueSpan {
        start,
        end: start,
        quote: None,
        blocky: false,
        value: String::new(),
    };
    let rest = &text[start..];
    let Some(first) = rest.chars().next() else {
        return empty;
    };
    if first == '#' {
        return empty;
    }
    if first == '|' || first == '>' {
        return ValueSpan {
            start,
            end: text.len(),
            quote: None,
            blocky: true,
            value: String::new(),
        };
    }
    if first == '"' || first == '\'' {
        if let Some((v, len)) = read_quoted(rest, first) {
            return ValueSpan {
                start,
                end: start + len,
                quote: Some(first),
                blocky: false,
                value: v,
            };
        }
    }
    // A plain scalar ends where the comment after it begins: in YAML a `#`
    // preceded by whitespace opens one, and a `#` that is not part of the
    // value.
    let end_rel = rest.find(" #").unwrap_or(rest.len());
    let plain = rest[..end_rel].trim_end();
    ValueSpan {
        start,
        end: start + plain.len(),
        quote: None,
        blocky: false,
        value: plain.to_string(),
    }
}

/// The block hanging under the key on line `parent`.
///
/// `range` ends one past the block's last line **of content**, which is the
/// distinction insertion needs: a comment between the last verifier and the
/// next top-level key conventionally introduces what follows it, so a new entry
/// goes above the comment and not below it.
struct Block {
    range: Range<usize>,
    /// The indentation the children use; `None` when the block is empty.
    indent: Option<usize>,
}

fn block_under(lines: &[Line], parent: usize, parent_indent: usize) -> Block {
    let mut indent: Option<usize> = None;
    let mut last_content = parent;
    for (i, line) in lines.iter().enumerate().skip(parent + 1) {
        let Some(n) = content_indent(&line.text) else {
            continue;
        };
        match indent {
            None => {
                if n <= parent_indent {
                    break;
                }
                indent = Some(n);
            }
            Some(c) if n < c => break,
            Some(_) => {}
        }
        last_content = i;
    }
    Block {
        range: parent + 1..last_content + 1,
        indent,
    }
}

/// The line declaring `name` at exactly `indent`, within `range`.
fn find_key(lines: &[Line], range: Range<usize>, indent: usize, name: &str) -> Option<usize> {
    range.into_iter().find(|&i| {
        content_indent(&lines[i].text) == Some(indent)
            && key_of(&lines[i].text).is_some_and(|(k, _)| k == name)
    })
}

/// The `verifiers:` line, the block under it, and the indentation of the
/// entries it holds.
fn verifiers_block(lines: &[Line]) -> Option<(usize, Block)> {
    let vi = find_key(lines, 0..lines.len(), 0, "verifiers")?;
    let block = block_under(lines, vi, 0);
    Some((vi, block))
}

/// The `peers:` line and the block under it.
fn peers_block(lines: &[Line]) -> Option<(usize, Block)> {
    let i = find_key(lines, 0..lines.len(), 0, "peers")?;
    let block = block_under(lines, i, 0);
    Some((i, block))
}

/// The line declaring peer `name`.
fn locate_peer(lines: &[Line], name: &str) -> Option<usize> {
    let (_, block) = peers_block(lines)?;
    let child = block.indent?;
    find_key(lines, block.range, child, name)
}

/// The line declaring verifier `name`, and the block of its own fields.
fn locate_verifier(lines: &[Line], name: &str) -> Option<(usize, usize, Block)> {
    let (_, block) = verifiers_block(lines)?;
    let child = block.indent?;
    let i = find_key(lines, block.range.clone(), child, name)?;
    Some((i, child, block_under(lines, i, child)))
}

// ---------------------------------------------------------------------------
// Rendering a value back
// ---------------------------------------------------------------------------

/// Plain scalars YAML resolves to something other than a string. A branch
/// called `no` written plain would be stored as a boolean, and the point of the
/// verb is that the value the caller typed is the value stored.
fn resolves_to_non_string(s: &str) -> bool {
    const WORDS: [&str; 10] = [
        "true", "false", "yes", "no", "on", "off", "null", "~", "y", "n",
    ];
    let lower = s.to_ascii_lowercase();
    WORDS.contains(&lower.as_str()) || s.parse::<i64>().is_ok() || s.parse::<f64>().is_ok()
}

fn plain_safe(s: &str) -> bool {
    const INDICATORS: &str = "-?:,[]{}#&*!|>'\"%@`";
    !s.is_empty()
        && s.trim() == s
        && !s.chars().any(char::is_control)
        && !s.starts_with(|c| INDICATORS.contains(c))
        && !s.contains(": ")
        && !s.ends_with(':')
        && !s.contains(" #")
        && !resolves_to_non_string(s)
}

fn double_quoted(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The value as it will be written. A string key keeps the quoting style it
/// already had, which is what makes "quoting style survives a write" true of
/// the edited key as well as of its neighbours.
fn render_value(value: &str, numeric: bool, keep: Option<char>) -> String {
    if numeric {
        return value.to_string();
    }
    match keep {
        Some('"') => double_quoted(value),
        Some('\'') if !value.contains('\n') => format!("'{}'", value.replace('\'', "''")),
        _ if plain_safe(value) => value.to_string(),
        _ => double_quoted(value),
    }
}

/// Puts `rendered` where `span` was, and nowhere else on the line.
fn splice(line: &mut Line, span: &ValueSpan, rendered: &str) {
    let head = line.text[..span.start].to_string();
    let tail = line.text[span.end..].to_string();
    // `key:` carries no space before an absent value, and one has to appear.
    let sep = if span.start == span.end && !head.ends_with(' ') {
        " "
    } else {
        ""
    };
    line.text = format!("{head}{sep}{rendered}{tail}");
}

// ---------------------------------------------------------------------------
// Reading and writing one key
// ---------------------------------------------------------------------------

/// The value in effect, in the shape both surfaces print.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Value {
    /// Carried by the file.
    Set(String),
    /// Absent, and the tool resolves one.
    Default(String),
    /// Absent, with nothing to resolve.
    Unset,
}

impl Value {
    fn display(&self) -> String {
        match self {
            Value::Set(v) => v.clone(),
            Value::Default(v) => format!("{v} (default)"),
            Value::Unset => "(unset)".to_string(),
        }
    }

    fn source(&self) -> &'static str {
        match self {
            Value::Set(_) => "file",
            Value::Default(_) => "default",
            Value::Unset => "unset",
        }
    }

    fn json(&self) -> String {
        match self {
            Value::Set(v) | Value::Default(v) => crate::json::string(v),
            Value::Unset => "null".to_string(),
        }
    }
}

fn from_default(d: &Option<String>) -> Value {
    match d {
        Some(v) => Value::Default(v.clone()),
        None => Value::Unset,
    }
}

fn scalar_at(line: &Line, name: &str) -> Result<Value> {
    let (_, after) = key_of(&line.text).expect("the line was found by its key");
    let span = value_span(&line.text, after);
    if span.blocky {
        return Err(blocky(name));
    }
    if span.start == span.end {
        return Ok(Value::Unset);
    }
    Ok(Value::Set(span.value))
}

/// Read from the text and not from a parsed [`Config`]: this verb runs on a
/// file that does not parse, and a reader that needed the parse would go silent
/// exactly where it is needed (§4).
fn read_key(lines: &[Line], key: &Key) -> Result<Value> {
    match key {
        Key::Block { verifier } => Err(whole_block(verifier)),
        Key::Top { name, default, .. } => match find_key(lines, 0..lines.len(), 0, name) {
            Some(i) => scalar_at(&lines[i], name),
            None => Ok(from_default(default)),
        },
        // No default: a peer nobody declared is a peer that does not exist, and
        // there is nothing for the tool to resolve in its place.
        Key::Peer { name } => match locate_peer(lines, name) {
            Some(i) => scalar_at(&lines[i], &format!("peers.{name}")),
            None => Ok(Value::Unset),
        },
        Key::Field {
            verifier,
            field,
            default,
        } => {
            // A timeout resolved for a verifier that is not declared would be a
            // value in effect for something that will never run.
            let Some((_, _, inner)) = locate_verifier(lines, verifier) else {
                return Ok(Value::Unset);
            };
            match inner
                .indent
                .and_then(|g| find_key(lines, inner.range.clone(), g, field))
            {
                Some(j) => scalar_at(&lines[j], &format!("verifiers.{verifier}.{field}")),
                None => Ok(from_default(default)),
            }
        }
    }
}

/// What a write reports as the state before and after it.
///
/// Not [`read_key`], because a whole verifier has no value to read and is still
/// the thing `--unset` removes: reporting `declared -> (unset)` is the honest
/// answer, and refusing to observe it would make the removal unreportable.
fn observe(lines: &[Line], key: &Key) -> Result<Value> {
    match key {
        Key::Block { verifier } => Ok(match locate_verifier(lines, verifier) {
            Some(_) => Value::Set("declared".to_string()),
            None => Value::Unset,
        }),
        other => read_key(lines, other),
    }
}

fn write_key(lines: &mut Vec<Line>, key: &Key, value: &str) -> Result<()> {
    match key {
        Key::Block { verifier } => Err(whole_block(verifier)),
        Key::Top { name, numeric, .. } => match find_key(lines, 0..lines.len(), 0, name) {
            Some(i) => {
                let (_, after) = key_of(&lines[i].text).expect("found by its key");
                let span = value_span(&lines[i].text, after);
                if span.blocky {
                    return Err(blocky(name));
                }
                let rendered = render_value(value, *numeric, span.quote);
                splice(&mut lines[i], &span, &rendered);
                Ok(())
            }
            None => {
                let eol = dominant_eol(lines);
                terminate_last(lines, &eol);
                let rendered = render_value(value, *numeric, None);
                lines.push(Line {
                    text: format!("{name}: {rendered}"),
                    eol,
                });
                Ok(())
            }
        },
        Key::Field {
            verifier, field, ..
        } => write_field(lines, verifier, field, value),
        Key::Peer { name } => write_peer(lines, name, value),
    }
}

/// One scalar under `peers:`, written the way [`write_field`] writes one under a
/// verifier and one level shallower.
///
/// The `{}` promotion is the same byte and the same reason: a mapping written
/// empty cannot receive a child, and the parent of the key being written is the
/// one byte outside the line that a write is allowed to move (§4).
fn write_peer(lines: &mut Vec<Line>, name: &str, value: &str) -> Result<()> {
    let eol = dominant_eol(lines);
    let rendered = render_value(value, false, None);

    let Some(pi) = find_key(lines, 0..lines.len(), 0, "peers") else {
        terminate_last(lines, &eol);
        lines.push(Line {
            text: "peers:".to_string(),
            eol: eol.clone(),
        });
        lines.push(Line {
            text: format!("  {name}: {rendered}"),
            eol,
        });
        return Ok(());
    };

    let (_, after) = key_of(&lines[pi].text).expect("found by its key");
    let span = value_span(&lines[pi].text, after);
    if span.blocky {
        return Err(blocky("peers"));
    }
    if !span.value.is_empty() && span.value != "{}" {
        return Err(flow_mapping("peers"));
    }

    if span.value == "{}" {
        let head = lines[pi].text[..span.start].trim_end().to_string();
        let tail = lines[pi].text[span.end..].to_string();
        lines[pi].text = format!("{head}{tail}");
        lines.insert(
            pi + 1,
            Line {
                text: format!("  {name}: {rendered}"),
                eol,
            },
        );
        return Ok(());
    }

    let block = block_under(lines, pi, 0);
    let child = block.indent.unwrap_or(2);
    match find_key(lines, block.range.clone(), child, name) {
        Some(i) => {
            let (_, after) = key_of(&lines[i].text).expect("found by its key");
            let span = value_span(&lines[i].text, after);
            if span.blocky {
                return Err(blocky(&format!("peers.{name}")));
            }
            let rendered = render_value(value, false, span.quote);
            splice(&mut lines[i], &span, &rendered);
            Ok(())
        }
        None => {
            let at = block.range.end;
            if at == lines.len() {
                terminate_last(lines, &eol);
            }
            lines.insert(
                at,
                Line {
                    text: format!("{}{name}: {rendered}", " ".repeat(child)),
                    eol,
                },
            );
            Ok(())
        }
    }
}

fn write_field(lines: &mut Vec<Line>, verifier: &str, field: &str, value: &str) -> Result<()> {
    let eol = dominant_eol(lines);
    let rendered = render_value(value, false, None);

    let Some(vi) = find_key(lines, 0..lines.len(), 0, "verifiers") else {
        // Only `run` declares a verifier: a timeout with no command names one
        // that cannot run, and the file would not parse.
        if field != "run" {
            return Err(undeclared(verifier));
        }
        terminate_last(lines, &eol);
        lines.push(Line {
            text: "verifiers:".to_string(),
            eol: eol.clone(),
        });
        lines.push(Line {
            text: format!("  {verifier}:"),
            eol: eol.clone(),
        });
        lines.push(Line {
            text: format!("    run: {rendered}"),
            eol,
        });
        return Ok(());
    };

    let (_, after) = key_of(&lines[vi].text).expect("found by its key");
    let span = value_span(&lines[vi].text, after);
    if span.blocky {
        return Err(blocky("verifiers"));
    }
    if !span.value.is_empty() && span.value != "{}" {
        return Err(flow_mapping("verifiers"));
    }

    // `verifiers: {}` is what `init` writes, and the first verifier declared
    // into it turns the key into a block mapping. That byte is the parent of
    // the key being written, not a byte beside it (§4) -- and without it the
    // file `init` produces could never receive a verifier at all.
    if span.value == "{}" {
        if field != "run" {
            return Err(undeclared(verifier));
        }
        let head = lines[vi].text[..span.start].trim_end().to_string();
        let tail = lines[vi].text[span.end..].to_string();
        lines[vi].text = format!("{head}{tail}");
        lines.insert(
            vi + 1,
            Line {
                text: format!("  {verifier}:"),
                eol: eol.clone(),
            },
        );
        lines.insert(
            vi + 2,
            Line {
                text: format!("    run: {rendered}"),
                eol,
            },
        );
        return Ok(());
    }

    let block = block_under(lines, vi, 0);
    let child = block.indent.unwrap_or(2);
    match find_key(lines, block.range.clone(), child, verifier) {
        Some(i) => {
            let inner = block_under(lines, i, child);
            let grand = inner.indent.unwrap_or(child + 2);
            match find_key(lines, inner.range.clone(), grand, field) {
                Some(j) => {
                    let (_, after) = key_of(&lines[j].text).expect("found by its key");
                    let span = value_span(&lines[j].text, after);
                    if span.blocky {
                        return Err(blocky(&format!("verifiers.{verifier}.{field}")));
                    }
                    let rendered = render_value(value, false, span.quote);
                    splice(&mut lines[j], &span, &rendered);
                    Ok(())
                }
                None => {
                    let at = inner.range.end;
                    if at == lines.len() {
                        terminate_last(lines, &eol);
                    }
                    lines.insert(
                        at,
                        Line {
                            text: format!("{}{field}: {rendered}", " ".repeat(grand)),
                            eol,
                        },
                    );
                    Ok(())
                }
            }
        }
        None => {
            if field != "run" {
                return Err(undeclared(verifier));
            }
            // A new verifier goes at the end of the block, indented the way the
            // ones already there are rather than the way this file would have
            // been written from scratch.
            let grand = first_grandchild_indent(lines, &block, child).unwrap_or(child + 2);
            let at = block.range.end;
            if at == lines.len() {
                terminate_last(lines, &eol);
            }
            lines.insert(
                at,
                Line {
                    text: format!("{}{verifier}:", " ".repeat(child)),
                    eol: eol.clone(),
                },
            );
            lines.insert(
                at + 1,
                Line {
                    text: format!("{}run: {rendered}", " ".repeat(grand)),
                    eol,
                },
            );
            Ok(())
        }
    }
}

fn first_grandchild_indent(lines: &[Line], block: &Block, child: usize) -> Option<usize> {
    for i in block.range.clone() {
        if content_indent(&lines[i].text) == Some(child) && key_of(&lines[i].text).is_some() {
            return block_under(lines, i, child).indent;
        }
    }
    None
}

fn unset_key(lines: &mut Vec<Line>, key: &Key) -> Result<()> {
    match key {
        Key::Top { name, .. } => {
            if let Some(i) = find_key(lines, 0..lines.len(), 0, name) {
                remove_lines(lines, i..i + 1);
            }
            Ok(())
        }
        Key::Field {
            verifier, field, ..
        } => {
            if *field == "run" {
                return Err(CliError::new(
                    1,
                    format!("verifiers.{verifier}.run is required: a verifier with no command is not one"),
                )
                .with_hint(format!("ank config --unset verifiers.{verifier}")));
            }
            let Some((_, _, inner)) = locate_verifier(lines, verifier) else {
                return Ok(());
            };
            if let Some(j) = inner
                .indent
                .and_then(|g| find_key(lines, inner.range.clone(), g, field))
            {
                remove_lines(lines, j..j + 1);
            }
            Ok(())
        }
        Key::Peer { name } => {
            let Some((pi, _)) = peers_block(lines) else {
                return Ok(());
            };
            let Some(i) = locate_peer(lines, name) else {
                return Ok(());
            };
            remove_lines(lines, i..i + 1);
            // Same counterpart as `verifiers` below: the last peer removed
            // leaves `peers:` with no children, which is a parse error and not
            // an empty map.
            if block_under(lines, pi, 0).indent.is_none() {
                let (_, after) = key_of(&lines[pi].text).expect("found by its key");
                let span = value_span(&lines[pi].text, after);
                splice(&mut lines[pi], &span, "{}");
            }
            Ok(())
        }
        Key::Block { verifier } => {
            let Some((vi, block)) = verifiers_block(lines) else {
                return Ok(());
            };
            let Some(child) = block.indent else {
                return Ok(());
            };
            let Some(i) = find_key(lines, block.range.clone(), child, verifier) else {
                return Ok(());
            };
            let inner = block_under(lines, i, child);
            remove_lines(lines, i..inner.range.end.max(i + 1));

            // The last verifier removed leaves `verifiers:` with no children,
            // which is not an empty map but a parse error -- so the key goes
            // back to the `{}` `init` writes. The counterpart of the promotion
            // above, and forced by the same rule.
            if block_under(lines, vi, 0).indent.is_none() {
                let (_, after) = key_of(&lines[vi].text).expect("found by its key");
                let span = value_span(&lines[vi].text, after);
                splice(&mut lines[vi], &span, "{}");
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// The verb
// ---------------------------------------------------------------------------

/// `ank config <key> [<value>] [--unset]` (§4).
///
/// Runs without the foundation, as `init` and `help` do: `startup` loads
/// `config.yml` for every other verb, so a file that does not parse fails all
/// of them, `check` included. A verb that exists to repair the file and is
/// disabled by exactly the file it repairs is not a verb.
pub fn run(inv: &Invocation, repo: &crate::repo::Repo, out: &mut dyn Write) -> Result<i32> {
    let path = repo.config_path();
    let unset = inv.has("--unset");

    let Some(raw_key) = inv.positionals.first() else {
        return Err(
            CliError::new(1, "config expects a key").with_hint(format!("keys: {}", KEYS.join(" ")))
        );
    };
    let key = resolve_key(raw_key)?;
    let value = inv.positionals.get(1);

    if unset && value.is_some() {
        return Err(CliError::new(1, "--unset takes no value")
            .with_hint(format!("ank config --unset {raw_key}")));
    }

    let text = std::fs::read_to_string(&path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            CliError::new(1, format!("{} not found", path.display())).with_hint("ank init")
        } else {
            CliError::new(1, format!("{}: {e}", path.display()))
        }
    })?;
    let lines = split_lines(&text);

    if !unset && value.is_none() {
        // `read_key` is what refuses a whole verifier here: reading one as a
        // value is not a thing, and only `--unset` addresses it.
        let before = read_key(&lines, &key)?;
        if inv.json() {
            let doc = crate::json::Obj::new()
                .str("key", raw_key)
                .raw("value", &before.json())
                .str("source", before.source())
                .finish();
            let _ = writeln!(out, "{doc}");
        } else if !inv.quiet() {
            let _ = writeln!(out, "{}", before.display());
        }
        return Ok(0);
    }

    let before = observe(&lines, &key)?;
    let mut edited = lines.clone();
    match value {
        Some(v) => write_key(&mut edited, &key, v)?,
        None => unset_key(&mut edited, &key)?,
    }
    let after_text = join_lines(&edited);
    let after = observe(&edited, &key)?;

    // Differential, and it has to be (§4): the write is refused when it
    // *introduces* a parse failure, never when it is performed on a file that
    // already had one -- which is the file this verb exists for.
    let mut warning = None;
    if let Err(e) = parse(&after_text, &path) {
        if parse(&text, &path).is_ok() {
            return Err(CliError::new(
                1,
                format!(
                    "refused: the write would leave {} unreadable",
                    path.display()
                ),
            )
            .with_hint(format!("{}\n  -> ank config {raw_key}", e.message)));
        }
        warning = Some(e.message);
    }

    let changed = after_text != text;
    if changed {
        std::fs::write(&path, &after_text)
            .map_err(|e| CliError::new(1, format!("{}: {e}", path.display())))?;
    }

    if inv.json() {
        let doc = crate::json::Obj::new()
            .str("key", raw_key)
            .raw("previous", &before.json())
            .raw("value", &after.json())
            .bool("changed", changed)
            .finish();
        let _ = writeln!(out, "{doc}");
    } else if !inv.quiet() {
        if changed {
            let _ = writeln!(out, "{raw_key} {} -> {}", before.display(), after.display());
        } else {
            let _ = writeln!(out, "{raw_key} {} (unchanged)", after.display());
        }
        if let Some(w) = warning {
            let _ = writeln!(
                out,
                "{} {} still does not parse: {w}",
                inv.style().yellow("warning:"),
                path.display()
            );
        }
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p() -> &'static Path {
        Path::new(".ank/config.yml")
    }

    #[test]
    fn durations_in_all_four_units() {
        assert_eq!(parse_duration("30s").unwrap(), Duration::from_secs(30));
        assert_eq!(parse_duration("30m").unwrap(), Duration::from_secs(1800));
        assert_eq!(parse_duration("2h").unwrap(), Duration::from_secs(7200));
        assert_eq!(parse_duration("1d").unwrap(), Duration::from_secs(86_400));
        assert_eq!(parse_duration(" 10m ").unwrap(), Duration::from_secs(600));
    }

    #[test]
    fn invalid_durations_are_named_precisely() {
        assert!(parse_duration("30").unwrap_err().contains("missing unit"));
        assert!(parse_duration("30w").unwrap_err().contains("unknown unit"));
        assert!(parse_duration("h").unwrap_err().contains("digit expected"));
        assert!(parse_duration("").unwrap_err().contains("empty"));
    }

    #[test]
    fn an_unknown_schema_is_refused_with_the_next_step() {
        let err = parse("schema: 2\n", p()).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.contains("unknown schema 2"), "{}", err.message);
        assert!(err.hint.is_some());
    }

    #[test]
    fn an_unknown_field_is_refused_rather_than_ignored() {
        let err = parse("schema: 1\nbudget_context: 10\n", p()).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.contains("budget_context"), "{}", err.message);
    }

    #[test]
    fn the_five_fields_of_the_spec_are_read() {
        let cfg = parse(
            "\
schema: 1
context_budget: 4000
claim_ttl_max: 90m
verifiers:
  cargo-test:
    run: cargo test --workspace -q
    timeout: 10m
  fmt-check:
    run: cargo fmt --check
roles:
  agent:
    can: [context, claim]
    cannot: [delete]
  human:
    can: [\"*\"]
identities:
  \"marie@laptop\": human
",
            p(),
        )
        .unwrap();

        assert_eq!(cfg.context_budget, 4000);
        assert_eq!(cfg.claim_ttl_max, Duration::from_secs(5400));
        assert_eq!(
            cfg.verifier("cargo-test").unwrap().run,
            "cargo test --workspace -q"
        );
        // The timeout default is the one from the spec, not zero.
        assert_eq!(
            cfg.verifier("fmt-check").unwrap().timeout,
            Duration::from_secs(600)
        );
        assert_eq!(cfg.roles["agent"].cannot, vec!["delete".to_string()]);
        assert_eq!(cfg.identities["marie@laptop"], "human");
    }

    /// The repository states its own rhythm, and the tool's value is what an
    /// absent key resolves to (§3, ADR-0bb7ea8991bc).
    ///
    /// The last assertion is the one worth having: `claim_ttl_default` and
    /// [`crate::claim::DEFAULT_TTL`] are thirty minutes written twice, once as
    /// a duration this file can carry and once as a constant the claim record
    /// falls back to, and two spellings of one number is how they start to
    /// disagree. Nothing else pins them together.
    #[test]
    fn claim_ttl_default_is_read_and_falls_back_to_the_tools_value() {
        let cfg = parse("schema: 1\nclaim_ttl_default: 90m\n", p()).unwrap();
        assert_eq!(cfg.claim_ttl_default, Duration::from_secs(5400));

        let cfg = parse("schema: 1\n", p()).unwrap();
        assert_eq!(cfg.claim_ttl_default, Duration::from_secs(30 * 60));

        // A value the file cannot mean is named with its key, like every other
        // duration here.
        let err = parse("schema: 1\nclaim_ttl_default: 30w\n", p()).unwrap_err();
        assert!(err.message.contains("claim_ttl_default"), "{}", err.message);

        assert_eq!(
            parse_duration(DEFAULT_CLAIM_TTL).unwrap(),
            crate::claim::DEFAULT_TTL,
            "the file's default and the record's fallback are the same thirty \
             minutes, and only this assertion says so"
        );
    }

    #[test]
    fn default_branch_is_optional_and_read_when_present() {
        let cfg = parse("schema: 1\ndefault_branch: trunk\n", p()).unwrap();
        assert_eq!(cfg.default_branch.as_deref(), Some("trunk"));

        let cfg = parse("schema: 1\n", p()).unwrap();
        assert_eq!(cfg.default_branch, None, "the key is optional");

        // A blank value is an absence written out, not a branch named "": it
        // must reach the resolution as absent, otherwise the error naming the
        // two missing sources would never fire.
        let cfg = parse("schema: 1\ndefault_branch: \"  \"\n", p()).unwrap();
        assert_eq!(cfg.default_branch, None);
    }

    #[test]
    fn this_repositorys_own_config_loads() {
        // Dogfooding: the config that drives this repository must pass the
        // parser we just wrote, otherwise one of the two is lying.
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../.ank/config.yml")
            .canonicalize()
            .unwrap();
        let cfg = load(&path).unwrap();
        assert_eq!(cfg.claim_ttl_max, Duration::from_secs(7200));
        assert_eq!(cfg.default_branch.as_deref(), Some("main"));
        assert!(cfg.verifier("cargo-test").is_some());
        assert!(cfg.verifier("check-repo").is_some());
    }

    #[test]
    fn the_default_yaml_written_by_init_reads_back() {
        let cfg = parse(&default_yaml(), p()).unwrap();
        assert_eq!(cfg.schema, SUPPORTED_SCHEMA);
        assert_eq!(cfg.context_budget, DEFAULT_CONTEXT_BUDGET);
        assert_eq!(cfg.claim_ttl_max, Duration::from_secs(7200));
        assert_eq!(cfg.roles["human"].can, vec!["*".to_string()]);
    }

    // -----------------------------------------------------------------------
    // The writer (§4, ADR-e64dfaafd578)
    // -----------------------------------------------------------------------

    /// Every awkward form the criterion names, in one file: comments on their
    /// own line and after a value, blank lines, verifiers out of alphabetical
    /// order, a quoted `run`, and a verifier carrying no `timeout`.
    ///
    /// One fixture rather than one per property, deliberately: a writer that
    /// survives each awkwardness alone and mangles them together would pass
    /// six narrow tests, and that combination is what a real `config.yml` is.
    const FIXTURE: &str = "\
# The configuration of this repository, reviewed like code.
schema: 1

context_budget: 4000   # tokens, not lines
claim_ttl_max: 2h

verifiers:
  # Out of alphabetical order on purpose: a serializer would sort these.
  fmt-check:
    run: \"cargo fmt --check\"
  cargo-test:
    run: cargo test --workspace -q
    timeout: 30m

roles:
  agent:
    can: [context, claim]
    cannot: [delete]
identities: {}
";

    fn edit(text: &str, key: &str, value: Option<&str>) -> Result<String> {
        let k = resolve_key(key)?;
        let mut lines = split_lines(text);
        match value {
            Some(v) => write_key(&mut lines, &k, v)?,
            None => unset_key(&mut lines, &k)?,
        }
        Ok(join_lines(&lines))
    }

    fn read(text: &str, key: &str) -> Result<Value> {
        read_key(&split_lines(text), &resolve_key(key)?)
    }

    /// `after` with the lines that moved put back as they were.
    ///
    /// This is the assertion the criterion asks for, and it is stronger than
    /// comparing the two texts: it proves that *nothing else* moved, including
    /// the bytes a diff would show as unchanged context.
    fn splice_back(before: &str, after: &str, expected_moves: usize) -> String {
        let b = split_lines(before);
        let mut a = split_lines(after);
        assert_eq!(
            a.len(),
            b.len(),
            "the line count changed, so this was not a value replacement"
        );
        let moved: Vec<usize> = (0..a.len()).filter(|&i| a[i] != b[i]).collect();
        assert_eq!(moved.len(), expected_moves, "lines that moved: {moved:?}");
        for i in moved {
            a[i] = b[i].clone();
        }
        join_lines(&a)
    }

    /// `verify::definition_hash` for every verifier the file declares.
    fn hashes(text: &str) -> BTreeMap<String, String> {
        parse(text, p())
            .unwrap()
            .verifiers
            .iter()
            .map(|(n, v)| (n.clone(), crate::verify::definition_hash(v)))
            .collect()
    }

    /// A peer is a key the parser knows, and the strictness beside it is
    /// untouched (§7, ADR-a1de673043b4).
    #[test]
    fn peers_are_read_and_an_unknown_key_is_still_refused() {
        let cfg = parse("schema: 1\npeers:\n  core: ../core\n  web: /srv/web\n", p()).unwrap();
        assert_eq!(cfg.peers.get("core").map(String::as_str), Some("../core"));
        assert_eq!(cfg.peers.get("web").map(String::as_str), Some("/srv/web"));

        // The key next door, misspelled, is still a refusal and not a shrug.
        let err = parse("schema: 1\npeerz:\n  core: ../core\n", p()).unwrap_err();
        assert!(err.message.contains("peerz"), "{}", err.message);

        // No peers is no peers, not an error.
        assert!(parse("schema: 1\n", p()).unwrap().peers.is_empty());
    }

    /// Declaring, redeclaring and removing a peer, each moving the line named
    /// and the parent of it and nothing else (§4).
    #[test]
    fn declaring_a_peer_touches_the_line_and_its_parent_only() {
        // Into a file that never carried the key.
        let after = edit(FIXTURE, "peers.core", Some("../core")).unwrap();
        assert!(
            after.starts_with(FIXTURE),
            "the file was rewritten: {after}"
        );
        assert!(after.ends_with("peers:\n  core: ../core\n"), "{after}");
        assert_eq!(
            read(&after, "peers.core").unwrap(),
            Value::Set("../core".into())
        );

        // Into the block that now exists, and the value replaced in place.
        let two = edit(&after, "peers.web", Some("../web")).unwrap();
        assert_eq!(two.matches("peers:").count(), 1, "{two}");
        let moved = edit(&two, "peers.core", Some("../elsewhere")).unwrap();
        assert_eq!(splice_back(&two, &moved, 1), two);

        // And out again, the last one collapsing the mapping to the empty form
        // `init` writes rather than to a key with no children.
        let one = edit(&moved, "peers.web", None).unwrap();
        assert!(one.contains("core: ../elsewhere"), "{one}");
        let none = edit(&one, "peers.core", None).unwrap();
        assert!(none.contains("peers: {}"), "{none}");
        assert!(parse(&none, p()).is_ok(), "{none}");
        assert_eq!(read(&none, "peers.core").unwrap(), Value::Unset);

        // `peers: {}` promotes to a block mapping, the counterpart of the above.
        let back = edit(&none, "peers.core", Some("../core")).unwrap();
        assert!(!back.contains("peers: {}"), "{back}");
        assert!(parse(&back, p()).is_ok(), "{back}");
    }

    /// A peer nobody could name from a scope is refused where the caller can
    /// still type another name (§7).
    #[test]
    fn a_peer_name_no_scope_could_spell_is_refused_by_name() {
        for name in ["peers.c", "peers.a b", "peers.core!"] {
            let err = resolve_key(name).unwrap_err();
            assert_eq!(err.code, 1, "{name}");
            assert!(err.message.contains("peer name"), "{name}: {}", err.message);
        }
        // The mapping itself is not a value, and the refusal says what to type.
        let err = resolve_key("peers").unwrap_err();
        assert!(err.hint.is_some_and(|h| h.contains("peers.<name>")));
        assert!(resolve_key("peers.core").is_ok());
    }

    #[test]
    fn a_file_read_and_written_back_unedited_is_byte_identical() {
        // The floor everything else stands on: if the split and the join are
        // not each other's inverse, no assertion below means anything.
        for text in [
            FIXTURE,
            &FIXTURE.replace('\n', "\r\n"),
            "schema: 1",           // no trailing newline
            "\n\n# only comments", // no key at all
            "",
        ] {
            assert_eq!(join_lines(&split_lines(text)), text);
        }
    }

    #[test]
    fn writing_one_key_moves_one_line_and_leaves_every_other_byte() {
        let after = edit(FIXTURE, "claim_ttl_max", Some("4h")).unwrap();
        assert!(after.contains("claim_ttl_max: 4h"));
        assert_eq!(splice_back(FIXTURE, &after, 1), FIXTURE);

        // A value followed by a comment keeps the comment, and the column it
        // sat at: the caller asked for the value, not for the line.
        let after = edit(FIXTURE, "context_budget", Some("9000")).unwrap();
        assert!(
            after.contains("context_budget: 9000   # tokens, not lines"),
            "{after}"
        );
        assert_eq!(splice_back(FIXTURE, &after, 1), FIXTURE);

        // Nested, and the two neighbours of the edited line are untouched.
        let after = edit(FIXTURE, "verifiers.cargo-test.timeout", Some("45m")).unwrap();
        assert!(after.contains("    timeout: 45m"), "{after}");
        assert_eq!(splice_back(FIXTURE, &after, 1), FIXTURE);
    }

    #[test]
    fn the_quoting_style_of_the_edited_key_survives_the_edit() {
        // `fmt-check` carries a double-quoted run. Rewriting it plain would be
        // a correct YAML value and a gratuitous restyle of a reviewed file.
        let after = edit(
            FIXTURE,
            "verifiers.fmt-check.run",
            Some("cargo fmt -- --check"),
        )
        .unwrap();
        assert!(after.contains("run: \"cargo fmt -- --check\""), "{after}");
        assert_eq!(splice_back(FIXTURE, &after, 1), FIXTURE);

        // And an unquoted one stays unquoted where it safely can.
        let after = edit(FIXTURE, "verifiers.cargo-test.run", Some("cargo test -q")).unwrap();
        assert!(after.contains("run: cargo test -q"), "{after}");
    }

    #[test]
    fn a_value_that_would_change_meaning_unquoted_is_quoted() {
        // A branch called `no` is a string, and YAML plain-resolves it to
        // false. The parser would then refuse the file, which is a refusal
        // nobody could act on -- so it is written as what it is.
        let after = edit(FIXTURE, "default_branch", Some("no")).unwrap();
        assert!(after.contains("default_branch: \"no\""), "{after}");
        assert_eq!(parse(&after, p()).unwrap().default_branch.unwrap(), "no");

        // A number stays a number: quoting `context_budget` would store a
        // string the parser refuses.
        let after = edit(FIXTURE, "context_budget", Some("12000")).unwrap();
        assert_eq!(parse(&after, p()).unwrap().context_budget, 12000);

        // A run holding a `#` is not truncated into a comment.
        let after = edit(
            FIXTURE,
            "verifiers.cargo-test.run",
            Some("sh -c 'echo # x'"),
        )
        .unwrap();
        assert_eq!(
            parse(&after, p())
                .unwrap()
                .verifier("cargo-test")
                .unwrap()
                .run,
            "sh -c 'echo # x'"
        );
    }

    #[test]
    fn no_default_is_ever_materialised() {
        // The trap this whole design exists for. A serializer would write
        // `default_branch`, a `timeout` onto `fmt-check`, and `{}` for every
        // empty map -- turning "follows the tool" into "pinned here", so the
        // day a default moves the repository silently holds the old value.
        let after = edit(FIXTURE, "claim_ttl_max", Some("4h")).unwrap();
        assert!(
            !after.contains("default_branch"),
            "a key the file did not carry was written: {after}"
        );
        let fmt_block = after
            .split("fmt-check:")
            .nth(1)
            .unwrap()
            .split("cargo-test:")
            .next()
            .unwrap();
        assert!(
            !fmt_block.contains("timeout"),
            "fmt-check acquired a timeout it never declared: {fmt_block}"
        );
        // And the parser still resolves the defaults, which is the point: the
        // value is in effect without being written down.
        let cfg = parse(&after, p()).unwrap();
        assert_eq!(cfg.default_branch, None);
        assert_eq!(
            cfg.verifier("fmt-check").unwrap().timeout,
            Duration::from_secs(600)
        );
    }

    #[test]
    fn the_definition_hash_of_every_verifier_the_write_did_not_name_is_unchanged() {
        let before = hashes(FIXTURE);

        // A write elsewhere in the file moves nothing.
        let after = edit(FIXTURE, "claim_ttl_max", Some("4h")).unwrap();
        assert_eq!(hashes(&after), before);

        // A write to one verifier moves that one and no other.
        let after = edit(FIXTURE, "verifiers.cargo-test.timeout", Some("45m")).unwrap();
        let now = hashes(&after);
        assert_eq!(now["fmt-check"], before["fmt-check"]);
        assert_ne!(now["cargo-test"], before["cargo-test"]);

        // Declaring a new verifier disturbs neither of the two already there.
        let after = edit(FIXTURE, "verifiers.audit.run", Some("cargo audit")).unwrap();
        let now = hashes(&after);
        assert_eq!(now["fmt-check"], before["fmt-check"]);
        assert_eq!(now["cargo-test"], before["cargo-test"]);
    }

    #[test]
    fn quoting_alone_never_moves_a_definition_hash() {
        // The plausible trap, and the wrong one. The hash is taken over the
        // resolved values, so re-quoting a run and respelling a timeout in
        // another unit disturb no historical proof -- which is why those forms
        // are edited rather than refused, and why block scalars are not.
        let quoted = "schema: 1\nverifiers:\n  t:\n    run: \"cargo test\"\n    timeout: 600s\n";
        let plain = "schema: 1\nverifiers:\n  t:\n    run: cargo test\n    timeout: 10m\n";
        assert_eq!(hashes(quoted), hashes(plain));
    }

    #[test]
    fn a_block_or_folded_run_is_refused_by_name_and_never_rewritten() {
        for marker in ["|", ">"] {
            let text =
                format!("schema: 1\nverifiers:\n  ci:\n    run: {marker}\n      cargo test\n");
            let err = edit(&text, "verifiers.ci.run", Some("cargo test -q")).unwrap_err();
            assert_eq!(err.code, 1);
            assert!(
                err.message.contains("verifiers.ci.run"),
                "the refusal must name the key: {}",
                err.message
            );
            assert!(err.hint.is_some(), "a refusal with no way out");

            // Reading is refused for the same reason: there is no honest
            // one-line rendering of it to print.
            assert!(read(&text, "verifiers.ci.run").is_err());
        }
    }

    #[test]
    fn setting_a_key_that_was_absent_and_unsetting_it_returns_the_file() {
        // Both shapes of insertion: a top-level scalar appended at the end,
        // and a whole verifier block added inside another block.
        for (key, value) in [
            ("default_branch", "main"),
            ("verifiers.audit.run", "cargo audit"),
        ] {
            let with = edit(FIXTURE, key, Some(value)).unwrap();
            assert_ne!(with, FIXTURE, "{key} was not written at all");
            parse(&with, p()).unwrap_or_else(|e| panic!("{key}: {}", e.message));
            let back = edit(&with, unset_target(key), None).unwrap();
            assert_eq!(back, FIXTURE, "{key} did not come back out cleanly");
        }
    }

    /// A verifier is removed as a whole, which is the counterpart of declaring
    /// one by writing its `run`.
    fn unset_target(key: &str) -> &str {
        match key {
            "verifiers.audit.run" => "verifiers.audit",
            other => other,
        }
    }

    #[test]
    fn a_new_verifier_lands_at_the_end_of_the_block_with_the_indentation_it_finds() {
        let after = edit(FIXTURE, "verifiers.audit.run", Some("cargo audit")).unwrap();
        assert!(
            after.contains("  audit:\n    run: cargo audit\n"),
            "{after}"
        );
        // At the end of `verifiers`, not at the end of the file: `roles` and
        // `identities` still follow it, and in that order.
        let at = |needle: &str| after.find(needle).unwrap();
        assert!(at("audit:") > at("cargo-test:"));
        assert!(at("audit:") < at("roles:"));

        // Adding a field to a verifier that has none lands inside its block.
        let after = edit(FIXTURE, "verifiers.fmt-check.timeout", Some("5m")).unwrap();
        assert!(
            after.contains("    run: \"cargo fmt --check\"\n    timeout: 5m\n"),
            "{after}"
        );
        assert_eq!(
            parse(&after, p())
                .unwrap()
                .verifier("fmt-check")
                .unwrap()
                .timeout,
            Duration::from_secs(300)
        );
    }

    #[test]
    fn the_empty_flow_mapping_init_writes_can_receive_a_verifier_and_give_it_back() {
        // Without the promotion, the file `ank init` produces could never
        // receive a verifier at all -- and it is the file every new repository
        // starts from.
        let base = default_yaml();
        let with = edit(&base, "verifiers.cargo-test.run", Some("cargo test")).unwrap();
        let cfg = parse(&with, p()).unwrap();
        assert_eq!(cfg.verifier("cargo-test").unwrap().run, "cargo test");
        assert!(
            with.contains("verifiers:\n  cargo-test:\n    run: cargo test\n"),
            "{with}"
        );
        // `roles` was directly below `verifiers: {}` and is untouched.
        assert_eq!(cfg.roles["human"].can, vec!["*".to_string()]);

        // And the demotion is its exact inverse: `verifiers:` with no children
        // is not an empty map but a parse error, so the key goes back to `{}`.
        let back = edit(&with, "verifiers.cargo-test", None).unwrap();
        assert_eq!(back, base);
        parse(&back, p()).unwrap();
    }

    #[test]
    fn a_crlf_checkout_keeps_its_terminators_on_every_line() {
        // §3 forbids ank to *write* CRLF and says nothing about rewriting one
        // in place. A writer that normalised would rewrite every line of the
        // file to change one key, which is the opposite of what this is for.
        let crlf = FIXTURE.replace('\n', "\r\n");
        let after = edit(&crlf, "claim_ttl_max", Some("4h")).unwrap();
        assert!(!after.contains("\n\n"), "an LF slipped in: {after:?}");
        assert_eq!(after.matches("\r\n").count(), crlf.matches("\r\n").count());
        assert_eq!(splice_back(&crlf, &after, 1), crlf);

        // A key appended to such a file arrives with the file's terminator,
        // not with the one this platform would have used.
        let after = edit(&crlf, "default_branch", Some("main")).unwrap();
        assert!(after.ends_with("default_branch: main\r\n"), "{after:?}");
        assert_eq!(edit(&after, "default_branch", None).unwrap(), crlf);
    }

    #[test]
    fn a_file_with_no_trailing_newline_is_terminated_before_a_key_is_appended() {
        // Otherwise the appended key joins the last line and the file means
        // something else entirely.
        let text = "schema: 1\nclaim_ttl_max: 2h";
        let after = edit(text, "default_branch", Some("main")).unwrap();
        assert_eq!(
            after,
            "schema: 1\nclaim_ttl_max: 2h\ndefault_branch: main\n"
        );

        // Removing the last line keeps the terminator of the one above it:
        // that byte was in the file, and the caller named the key below it.
        let text = "schema: 1\ndefault_branch: main";
        assert_eq!(edit(text, "default_branch", None).unwrap(), "schema: 1\n");

        // A key removed from the middle leaves the ending as it found it.
        let text = "schema: 1\ndefault_branch: main\nclaim_ttl_max: 2h";
        assert_eq!(
            edit(text, "default_branch", None).unwrap(),
            "schema: 1\nclaim_ttl_max: 2h"
        );
    }

    #[test]
    fn reading_marks_a_resolved_default_and_names_an_absence() {
        assert_eq!(read(FIXTURE, "claim_ttl_max").unwrap().display(), "2h");
        assert_eq!(
            read(FIXTURE, "verifiers.fmt-check.timeout")
                .unwrap()
                .display(),
            "10m (default)",
            "the timeout in effect is the tool's, and the reader has to be told which"
        );
        assert_eq!(
            read(FIXTURE, "default_branch").unwrap().display(),
            "(unset)"
        );
        // A timeout resolved for a verifier that is not declared would be a
        // value in effect for something that will never run.
        assert_eq!(
            read(FIXTURE, "verifiers.nope.timeout").unwrap(),
            Value::Unset
        );
        assert_eq!(
            read(FIXTURE, "verifiers.fmt-check.run").unwrap(),
            Value::Set("cargo fmt --check".to_string()),
            "a quoted value reads unquoted"
        );
    }

    #[test]
    fn the_key_set_is_closed_and_every_refusal_names_what_it_refused() {
        // Unknown: the set it does know, and nothing written.
        let err = edit(FIXTURE, "budget_context", Some("10")).unwrap_err();
        assert_eq!(err.code, 1);
        assert!(err.message.contains("budget_context"), "{}", err.message);
        let hint = err.hint.unwrap();
        for key in KEYS {
            assert!(hint.contains(key), "{key} missing from: {hint}");
        }

        // Structured, and known to the parser: refused by name rather than
        // guessed at, which is a different message from "no such key".
        for key in ["roles", "roles.agent.can", "identities"] {
            let err = edit(FIXTURE, key, Some("x")).unwrap_err();
            assert!(err.message.contains("structured"), "{}", err.message);
        }

        // A whole verifier is addressable by --unset alone.
        let err = read(FIXTURE, "verifiers.cargo-test").unwrap_err();
        assert!(err.hint.unwrap().contains("verifiers.cargo-test.run"));
        assert!(edit(FIXTURE, "verifiers.cargo-test", Some("x")).is_err());
        assert!(edit(FIXTURE, "verifiers.cargo-test", None).is_ok());

        // A timeout cannot declare a verifier: it would name one that has no
        // command to run, and the file would not parse.
        let err = edit(FIXTURE, "verifiers.nope.timeout", Some("5m")).unwrap_err();
        assert_eq!(err.code, 7);
        assert_eq!(
            err.hint.as_deref(),
            Some("ank config verifiers.nope.run \"<command>\"")
        );

        // And `run` cannot be removed on its own, for the same reason.
        let err = edit(FIXTURE, "verifiers.cargo-test.run", None).unwrap_err();
        assert_eq!(
            err.hint.as_deref(),
            Some("ank config --unset verifiers.cargo-test")
        );
    }

    #[test]
    fn unsetting_a_key_the_file_does_not_carry_changes_nothing() {
        assert_eq!(edit(FIXTURE, "default_branch", None).unwrap(), FIXTURE);
        assert_eq!(edit(FIXTURE, "verifiers.nope", None).unwrap(), FIXTURE);
        assert_eq!(
            edit(FIXTURE, "verifiers.fmt-check.timeout", None).unwrap(),
            FIXTURE
        );
    }

    #[test]
    fn a_quoted_key_is_read_as_a_key_and_not_scanned_through() {
        // `identities` carries `"marie@laptop": human`, and a scan stopping at
        // the first colon would find one inside the quotes.
        assert_eq!(key_of("  \"a: b\": value").unwrap().0, "a: b".to_string());
        assert_eq!(key_of("run: cargo test").unwrap().0, "run".to_string());
        // YAML requires the space, so this declares no key.
        assert!(key_of("run:cargo").is_none());
        assert!(key_of("- item").is_none());
        assert!(key_of("verifiers:").is_some());
    }
}
