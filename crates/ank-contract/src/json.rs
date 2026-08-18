//! The machine surface: one writer, one escaper (TASK-2c12b027f805).
//!
//! `--json` is available on every verb without exception, and §4 calls that an
//! invariant rather than a convenience. The transport has always been
//! guaranteed with it -- one line, stdout only, never coloured, nothing else on
//! the stream. The payload had none of that: four escapers lived in `cli.rs`,
//! `commands.rs`, `context.rs` and `human.rs`, and they did not agree. Three
//! spelled a tab `\u0009` through a catch-all; the fourth spelled it `\t`. Both
//! are valid JSON and they are not the same bytes, which is the whole problem
//! with a contract that exists in four copies.
//!
//! So: one module, and one [`string`] inside it that every value passes
//! through. A document is built here rather than assembled with `format!` at
//! the call site, because a shape that exists only in the string that prints it
//! is a shape nothing can hold still. `tests/golden-json/` pins what comes out.
//!
//! Deliberately not `serde_json`. Nothing here needs a parser, the output is
//! one line of well-known shapes, and §13 spends a dependency only on
//! necessity. This is the same call the argument parser in `cli.rs` already
//! makes, for the same reason: character-level control over a surface that is
//! read rather than looked at.

use std::fmt::Display;

/// The one escaper. Returns the value **quoted**, because an unquoted escape is
/// a value one caller has to remember to wrap, and the fourth escaper this
/// replaces was exactly that.
///
/// Control characters below 0x20 that JSON gives a short form are written in
/// that form; the rest go out as `\u00xx`. `/` is left alone: escaping it is
/// legal and buys nothing.
pub fn string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// An array of documents already rendered.
pub fn array<I: IntoIterator<Item = String>>(items: I) -> String {
    let mut out = String::from("[");
    for (i, item) in items.into_iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&item);
    }
    out.push(']');
    out
}

/// An array of strings, each escaped here so no caller has to.
pub fn strings<S: AsRef<str>, I: IntoIterator<Item = S>>(items: I) -> String {
    array(items.into_iter().map(|s| string(s.as_ref())))
}

/// A document, built key by key in the order the caller writes them.
///
/// Key order is the caller's and is not sorted: a document whose fields move
/// between releases is a document a golden cannot pin, and the order a verb
/// emits is part of what `tests/golden-json/` holds still.
#[derive(Default)]
pub struct Obj {
    buf: String,
}

impl Obj {
    /// A document fragment: a nested object, or a row of an array.
    pub fn new() -> Obj {
        Obj { buf: String::new() }
    }

    /// A **top-level** document, which carries the contract version it was
    /// written against (ADR-6fd69efb629c).
    ///
    /// Seeded by the constructor rather than added by `finish`, for two reasons.
    /// `finish` also renders nested objects and array rows, which must not carry
    /// it — the version describes the document, not every object inside it. And
    /// the key belongs first, where a reader looking for it finds it before
    /// anything else, and `Obj` keeps the order its caller wrote.
    ///
    /// Using [`Obj::new`] at the top level is therefore a way to emit a document
    /// with no version, which is why nothing here is asked to remember: the
    /// conformance test walks every fixture in `tests/golden-json/` and there is
    /// one per verb, so a verb that forgot is a verb whose golden fails.
    pub fn document() -> Obj {
        Obj::new().num("contract", crate::CONTRACT_VERSION)
    }

    fn key(&mut self, key: &str) {
        if !self.buf.is_empty() {
            self.buf.push(',');
        }
        self.buf.push_str(&string(key));
        self.buf.push(':');
    }

    /// A string value, escaped.
    pub fn str(mut self, key: &str, value: &str) -> Obj {
        self.key(key);
        self.buf.push_str(&string(value));
        self
    }

    /// A string value, or `null` when there is none. The distinction is
    /// load-bearing for a typed caller: an absent branch on a detached HEAD is
    /// not the empty string.
    pub fn opt_str(self, key: &str, value: Option<&str>) -> Obj {
        match value {
            Some(v) => self.str(key, v),
            None => self.null(key),
        }
    }

    /// Any number that renders itself: the counts are `usize`, the versions
    /// `u64`, the exit codes `i32`.
    pub fn num(mut self, key: &str, value: impl Display) -> Obj {
        self.key(key);
        self.buf.push_str(&value.to_string());
        self
    }

    pub fn bool(mut self, key: &str, value: bool) -> Obj {
        self.key(key);
        self.buf.push_str(if value { "true" } else { "false" });
        self
    }

    pub fn null(mut self, key: &str) -> Obj {
        self.key(key);
        self.buf.push_str("null");
        self
    }

    /// An array of strings, each escaped here.
    pub fn strings<S: AsRef<str>, I: IntoIterator<Item = S>>(self, key: &str, items: I) -> Obj {
        let rendered = strings(items);
        self.raw(key, &rendered)
    }

    /// An array of documents already rendered.
    pub fn array<I: IntoIterator<Item = String>>(self, key: &str, items: I) -> Obj {
        let rendered = array(items);
        self.raw(key, &rendered)
    }

    /// A nested document.
    pub fn obj(self, key: &str, value: Obj) -> Obj {
        let rendered = value.finish();
        self.raw(key, &rendered)
    }

    /// A value this module did not render.
    ///
    /// The escape hatch, and it is meant to stay one: every caller of it is a
    /// place where a shape is still assembled somewhere else. It exists because
    /// a document is sometimes built in two halves by two functions, not so a
    /// caller can go on writing JSON by hand.
    pub fn raw(mut self, key: &str, json: &str) -> Obj {
        self.key(key);
        self.buf.push_str(json);
        self
    }

    pub fn finish(self) -> String {
        format!("{{{}}}", self.buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The disagreement that motivated the module, pinned so it cannot come
    /// back: one spelling for a tab, one for a carriage return, whichever
    /// caller asks.
    #[test]
    fn one_spelling_for_every_control_character() {
        assert_eq!(string("a\tb"), "\"a\\u0009b\"");
        assert_eq!(string("a\rb"), "\"a\\u000db\"");
        assert_eq!(string("a\nb"), "\"a\\nb\"");
        assert_eq!(string("a\u{0}b"), "\"a\\u0000b\"");
    }

    #[test]
    fn quotes_and_backslashes_survive_a_round_trip_by_eye() {
        assert_eq!(string(r#"say "hi""#), r#""say \"hi\"""#);
        assert_eq!(string(r"C:\ank"), r#""C:\\ank""#);
    }

    /// Non-ASCII is passed through rather than escaped: the stream is UTF-8 and
    /// `\u` sequences would only make it harder to read.
    #[test]
    fn text_outside_ascii_is_not_escaped() {
        assert_eq!(string("criterion — frozen"), "\"criterion — frozen\"");
    }

    #[test]
    fn a_document_keeps_the_order_its_caller_wrote() {
        let doc = Obj::new()
            .str("task", "TASK-000000000001")
            .num("proofs", 2usize)
            .bool("logged", true)
            .null("branch")
            .strings("warnings", ["a", "b"])
            .finish();
        assert_eq!(
            doc,
            r#"{"task":"TASK-000000000001","proofs":2,"logged":true,"branch":null,"warnings":["a","b"]}"#
        );
    }

    #[test]
    fn an_empty_document_is_still_a_document() {
        assert_eq!(Obj::new().finish(), "{}");
        assert_eq!(array(Vec::<String>::new()), "[]");
    }

    #[test]
    fn a_nested_document_is_not_a_string() {
        let doc = Obj::new()
            .obj("identity", Obj::new().str("value", "human:marie"))
            .finish();
        assert_eq!(doc, r#"{"identity":{"value":"human:marie"}}"#);
    }
}
