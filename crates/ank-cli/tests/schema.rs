//! `.ank/config.yml` is refused on its schema, not on the first key the binary
//! does not recognise (TASK-742cd978a806).
//!
//! `docs/format.md` already legislates this, for entities:
//!
//! > Newer is refused, and refused **on the version rather than on the first
//! > field it does not recognise**. Since unknown fields are rejected, a tool
//! > that checked only its own version would report a file one version newer as
//! > *unknown field `author`*, and its reader would go hunting for a typo.
//! > Naming the version says the one true thing: this file is newer than this
//! > tool.
//!
//! `config.yml` carries `schema:` for exactly that purpose and never used to
//! get to use it: `deny_unknown_fields` on `ConfigFile` fired during
//! deserialization, before any version was compared. The occasion was real
//! rather than hypothetical — `default: true` arrived under two verifiers on
//! 2026-08-30 (ADR-443590981e41) and every released `ank` began answering
//! *unknown field `default`* on this repository, blaming the file for the age
//! of the binary.
//!
//! **Through the binary, and it has to be.** The criterion is about the
//! sentence a reader is handed at the surface they actually use, and
//! `CARGO_BIN_EXE_ank` is defined only for an integration test — `ank-cli` has
//! no library target, so there is no unit test that could spawn it.
//!
//! What this suite cannot buy, said out loud so nobody reads it as more than it
//! is: 0.6.0 is shipped with a field-first parser, and nothing asserted here
//! reaches it. Measured on the released build rather than assumed — a corpus at
//! `schema: 1` and the same corpus at `schema: 2` produce the byte-identical
//! *unknown field `default`* from it, because its `verifiers` is a typed map
//! inside `ConfigFile` and the outer deserialize fails before `schema` is ever
//! compared. This stops the next occurrence, not that one.

use crate::fixture;
use crate::scratch;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// The newest schema of `.ank/config.yml` this binary reads: 2, the schema a
/// file carrying `requires_ank` declares (ADR-756e8b0fe416).
///
/// Hard-coded rather than imported because `ank-cli` has no library target.
const SUPPORTED: u32 = 2;

/// The oldest schema this binary reads, which a file without `requires_ank`
/// may go on declaring. The repository's own file is asserted against it below,
/// so the two cannot drift apart in silence.
const OLDEST: u32 = 1;

/// A corpus that is nothing but a `config.yml`, which is all a parse needs.
///
/// No git and no entities: reading the configuration is what `find` does before
/// it does anything else, and a verb that only reads the corpus requires no git
/// at all (ADR-9307b1c98ff5).
fn corpus(what: &str, config: &str) -> PathBuf {
    let dir = scratch::dir(what);
    fs::create_dir_all(dir.join(".ank/entities")).expect("the scratch corpus must be creatable");
    fs::write(dir.join(".ank/config.yml"), config).expect("config.yml must be writable");
    dir
}

/// `ank find --status open` in `dir`, answering the exit code and everything it
/// said on both streams.
///
/// The verb is incidental and the parse is not: any verb that resolves a
/// configuration fails identically, and `find` is the one the defect was
/// reported through.
fn find(dir: &Path) -> (i32, String) {
    let out = Command::new(ANK)
        .args(["find", "--status", "open"])
        .current_dir(dir)
        .output()
        .expect("the binary under test must run");
    let mut said = String::from_utf8_lossy(&out.stdout).into_owned();
    said.push_str(&String::from_utf8_lossy(&out.stderr));
    (
        out.status.code().expect("the binary must not be signalled"),
        said,
    )
}

// ---------------------------------------------------------------------------
// Newer than this binary: refused on the version
// ---------------------------------------------------------------------------

/// A file one version ahead, carrying a top-level key this binary has never
/// heard of — the shape a future revision of the format actually takes.
///
/// This is the branch that was broken. `channels:` is what the reader used to
/// be told about, and the version is what it is told about now.
#[test]
fn a_newer_schema_is_refused_by_version_and_never_by_a_field() {
    let dir = corpus(
        "schema-newer-toplevel",
        &format!(
            "schema: {}\ncontext_budget: 8000\nchannels:\n  release: main\n",
            SUPPORTED + 1
        ),
    );
    let (code, said) = find(&dir);

    assert_eq!(code, 1, "a file newer than the binary is refused: {said}");
    assert!(
        said.contains(&format!("schema {}", SUPPORTED + 1)),
        "the refusal names the schema the file declares: {said}"
    );
    assert!(
        said.contains(&format!("this binary reads up to {SUPPORTED}")),
        "the refusal names what this binary supports: {said}"
    );
    assert!(
        said.contains("newer than this ank"),
        "the refusal says the file is newer than the tool: {said}"
    );
    assert!(
        !said.contains("unknown field"),
        "a newer file is never refused on a field: {said}"
    );
    assert!(
        !said.contains("channels"),
        "the key that happens to be new is not the diagnosis: {said}"
    );
}

/// The same, where the unknown key is nested under a verifier rather than at the
/// top level — which is the shape `default: true` had, and so the shape the
/// occasion for this task actually took.
///
/// This branch was already half-right before the fix and the negative control
/// says so: ADR-443590981e41 retyped `ConfigFile::verifiers` to a
/// `serde_yaml::Mapping` to preserve declaration order, which moved each
/// verifier's parse into a loop *after* the schema comparison. So a newer file
/// with a nested unknown key was refused on its version already — as *unknown
/// schema 2 (supported: 1)*, which names the version but not what to do about
/// it. What this test pins is that both shapes now reach one sentence, the one
/// the criterion asks for.
#[test]
fn a_newer_schema_is_refused_by_version_when_the_new_key_is_nested() {
    let dir = corpus(
        "schema-newer-nested",
        &format!(
            "schema: {}\nverifiers:\n  cargo-test:\n    run: cargo test\n    lane: fast\n",
            SUPPORTED + 1
        ),
    );
    let (code, said) = find(&dir);

    assert_eq!(code, 1, "a file newer than the binary is refused: {said}");
    assert!(
        said.contains("newer than this ank"),
        "the refusal says the file is newer than the tool: {said}"
    );
    assert!(
        !said.contains("unknown field") && !said.contains("lane"),
        "a newer file is never refused on the nested key either: {said}"
    );
}

// ---------------------------------------------------------------------------
// Readable schema: a typo stays a typo
// ---------------------------------------------------------------------------

/// The counterweight, and the reason the fix is a version check placed ahead of
/// the parse rather than a relaxed `deny_unknown_fields`.
///
/// At a schema this binary reads, an unrecognised key is a mistake in the file
/// and the reader is told which key, by name, with the alternatives spelled
/// out. Losing this to gain the test above would trade one misleading message
/// for another.
#[test]
fn an_unknown_key_at_a_readable_schema_is_still_refused_by_name() {
    let dir = corpus(
        "schema-typo-toplevel",
        &format!("schema: {SUPPORTED}\ncontext_budgets: 8000\n"),
    );
    let (code, said) = find(&dir);

    assert_eq!(code, 1, "a typo is still refused: {said}");
    assert!(
        said.contains("unknown field `context_budgets`"),
        "the refusal names the key the file got wrong: {said}"
    );
    assert!(
        said.contains("`context_budget`"),
        "and names the key it was probably meant to be: {said}"
    );
    assert!(
        !said.contains("newer than this ank"),
        "a typo is never reported as a version problem: {said}"
    );
}

/// The nested half of the same rule.
#[test]
fn an_unknown_verifier_key_at_a_readable_schema_is_still_refused_by_name() {
    let dir = corpus(
        "schema-typo-nested",
        &format!(
            "schema: {SUPPORTED}\nverifiers:\n  cargo-test:\n    run: cargo test\n    tiemout: 2m\n"
        ),
    );
    let (code, said) = find(&dir);

    assert_eq!(code, 1, "a nested typo is still refused: {said}");
    assert!(
        said.contains("verifiers.cargo-test: unknown field `tiemout`"),
        "the refusal names the verifier and the key: {said}"
    );
    assert!(
        !said.contains("newer than this ank"),
        "a typo is never reported as a version problem: {said}"
    );
}

// ---------------------------------------------------------------------------
// Below the range, and the decision this task made
// ---------------------------------------------------------------------------

/// Under the range there is no older tool to name: no binary ever read schema
/// 0, so the file is simply wrong and keeps the message it has always had.
#[test]
fn a_schema_below_the_range_keeps_its_own_refusal() {
    let dir = corpus("schema-below", "schema: 0\n");
    let (code, said) = find(&dir);

    assert_eq!(code, 1, "schema 0 is refused: {said}");
    assert!(
        said.contains(&format!(
            "unknown schema 0 (supported: {OLDEST} to {SUPPORTED})"
        )),
        "the message below the range is unchanged: {said}"
    );
    assert!(
        !said.contains("newer than this ank"),
        "nothing below the range is newer than anything: {said}"
    );
}

/// **The decision of TASK-742cd978a806, asserted rather than remembered.**
///
/// `.ank/config.yml` carries `default:` under two verifiers and goes on
/// declaring `schema: 1`. It was a real choice and the argument is in the task's
/// log: a bump buys nothing, because every released binary refuses this file on
/// the field whatever the version says; and it would cost a corpus, because the
/// config check is an equality rather than the `MIN_SCHEMA..=SCHEMA_VERSION`
/// range entities enjoy, so raising the supported version would stop every
/// schema-1 corpus from loading — the migration-by-refusal `docs/format.md`
/// forbids.
///
/// What this test pins is that the file and the reader agree. Since
/// ADR-756e8b0fe416 the reader is a range, 1 and 2, and this repository stays at
/// 1 until it declares `requires_ank`, which it can only do once a release that
/// reads schema 2 is published.
#[test]
fn the_repositorys_own_config_declares_the_schema_this_binary_reads() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(".ank/config.yml");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));

    assert!(
        text.lines()
            .any(|l| l.trim() == format!("schema: {OLDEST}")),
        "{} must declare schema {OLDEST}",
        path.display()
    );
    assert!(
        text.contains("default: true"),
        "{} is the file that made this task necessary: it carries default:",
        path.display()
    );
}

// ---------------------------------------------------------------------------
// requires_ank, and the schema it declares (ADR-756e8b0fe416)
// ---------------------------------------------------------------------------
//
// The key only exists here; comparing it with the running binary is the next
// task's. What is pinned is the read, the refusals and the write, each through
// the binary with its exit code and its sentence.

/// `ank config <args>` in `dir`, answering the exit code and both streams.
fn config(dir: &Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(ANK)
        .arg("config")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("the binary under test must run");
    let mut said = String::from_utf8_lossy(&out.stdout).into_owned();
    said.push_str(&String::from_utf8_lossy(&out.stderr));
    (
        out.status.code().expect("the binary must not be signalled"),
        said,
    )
}

fn config_text(dir: &Path) -> String {
    fs::read_to_string(dir.join(".ank/config.yml")).expect("config.yml must be readable")
}

/// Schema 1 without the key is the corpus every repository already has, and it
/// reads exactly as before: no requirement, and nothing said about one.
#[test]
fn a_schema_1_config_without_requires_ank_is_read_as_today() {
    let dir = corpus("requires-absent", "schema: 1\ncontext_budget: 8000\n");
    let (code, said) = find(&dir);
    assert_eq!(code, 0, "a schema-1 file still reads: {said}");
    assert_eq!(said.trim(), "no match", "and says nothing more: {said}");

    let (code, said) = config(&dir, &["requires_ank"]);
    assert_eq!(code, 0, "{said}");
    assert_eq!(said.trim(), "(unset)", "absent is no requirement: {said}");
}

#[test]
fn a_schema_2_config_with_requires_ank_is_read() {
    let dir = corpus("requires-read", "schema: 2\nrequires_ank: 1.2.3\n");
    let (code, said) = find(&dir);
    assert_eq!(code, 0, "schema 2 with the key reads: {said}");
    assert_eq!(said.trim(), "no match", "{said}");

    let (code, said) = config(&dir, &["requires_ank"]);
    assert_eq!(code, 0, "{said}");
    assert_eq!(said.trim(), "1.2.3", "{said}");
}

/// A file carrying the key declares the schema it arrived with; one still at 1
/// was edited by hand, and is told both ways out.
#[test]
fn requires_ank_at_schema_1_is_refused_naming_the_key_and_schema_2() {
    let dir = corpus("requires-schema-1", "schema: 1\nrequires_ank: 1.2.3\n");
    let (code, said) = find(&dir);
    assert_eq!(code, 1, "{said}");
    assert!(
        said.contains("requires_ank needs schema 2, and the file declares schema 1"),
        "the refusal names the key and the schema it needs: {said}"
    );
    assert!(
        said.contains("ank config schema 2, or ank config --unset requires_ank"),
        "and the two commands that resolve it: {said}"
    );
}

/// The exact form, and nothing near it: two components, a tag's `v`, a
/// pre-release and a word are each refused at read, naming the key and the
/// form, never as serde's *expected a string*.
#[test]
fn a_requires_ank_not_of_the_form_major_minor_patch_is_refused_naming_the_form() {
    for (n, value) in ["1.2", "v1.2.3", "1.2.3-rc1", "word"].iter().enumerate() {
        let dir = corpus(
            &format!("requires-form-{n}"),
            &format!("schema: 2\nrequires_ank: {value}\n"),
        );
        let (code, said) = find(&dir);
        assert_eq!(code, 1, "{value}: {said}");
        assert!(
            said.contains(&format!(
                "requires_ank: '{value}' is not a version of the form MAJOR.MINOR.PATCH"
            )),
            "{value}: the refusal names the key, the value and the form: {said}"
        );
        assert!(!said.contains("invalid type"), "{value}: {said}");

        // The writer refuses it in the same sentence, and leaves the file be.
        let before = config_text(&dir);
        let (code, said) = config(&dir, &["requires_ank", value]);
        assert_eq!(code, 1, "{value}: {said}");
        assert!(
            said.contains(&format!(
                "requires_ank: '{value}' is not a version of the form MAJOR.MINOR.PATCH"
            )),
            "{value}: {said}"
        );
        assert_eq!(config_text(&dir), before, "{value}: nothing is written");
    }
}

/// The schema after the one this build reads is refused as newer, by version.
#[test]
fn a_schema_3_config_is_refused_as_newer_than_this_ank() {
    let dir = corpus("requires-schema-3", "schema: 3\nrequires_ank: 1.2.3\n");
    let (code, said) = find(&dir);
    assert_eq!(code, 1, "{said}");
    assert!(
        said.contains("schema 3, this binary reads up to 2: the file is newer than this ank"),
        "{said}"
    );
}

/// `ank config requires_ank` writes the key beneath `schema:` and raises a file
/// at 1 to 2 in the same write; reads it back; and `--unset` takes the key and
/// leaves the schema at 2, because a file at 2 without it is still read.
#[test]
fn ank_config_writes_requires_ank_with_schema_2_and_unset_leaves_the_schema() {
    let dir = corpus("requires-write", "schema: 1\ncontext_budget: 8000\n");

    let (code, said) = config(&dir, &["requires_ank", "1.2.3"]);
    assert_eq!(code, 0, "{said}");
    assert_eq!(said.trim(), "requires_ank (unset) -> 1.2.3", "{said}");
    assert_eq!(
        config_text(&dir),
        "schema: 2\nrequires_ank: 1.2.3\ncontext_budget: 8000\n",
        "the key lands beneath schema, which moves to 2"
    );

    let (code, said) = config(&dir, &["requires_ank"]);
    assert_eq!(code, 0, "{said}");
    assert_eq!(said.trim(), "1.2.3", "{said}");
    let (code, said) = find(&dir);
    assert_eq!(code, 0, "what config wrote, find reads: {said}");

    // Moved, not appended a second time.
    let (code, said) = config(&dir, &["requires_ank", "1.10.0"]);
    assert_eq!(code, 0, "{said}");
    assert_eq!(said.trim(), "requires_ank 1.2.3 -> 1.10.0", "{said}");
    assert_eq!(
        config_text(&dir),
        "schema: 2\nrequires_ank: 1.10.0\ncontext_budget: 8000\n"
    );

    let (code, said) = config(&dir, &["--unset", "requires_ank"]);
    assert_eq!(code, 0, "{said}");
    assert_eq!(said.trim(), "requires_ank 1.10.0 -> (unset)", "{said}");
    assert_eq!(
        config_text(&dir),
        "schema: 2\ncontext_budget: 8000\n",
        "unsetting the key leaves the schema where it is"
    );
    let (code, said) = find(&dir);
    assert_eq!(code, 0, "{said}");
}

/// The reference page is generated from the reader's table, and lists the key.
#[test]
fn the_config_reference_lists_requires_ank() {
    let page = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/config-keys.md");
    let text = fs::read_to_string(&page).unwrap_or_else(|e| panic!("{}: {e}", page.display()));
    assert!(
        text.lines()
            .any(|l| l.starts_with("| `requires_ank` | version | none |")),
        "{} lists requires_ank",
        page.display()
    );
    assert!(
        text.contains("a file carrying `requires_ank` declares `schema: 2`"),
        "{}",
        page.display()
    );
}

// ---------------------------------------------------------------------------
// The other schema: the shape a verb declares, and the fixture that pins it
// ---------------------------------------------------------------------------
//
// `.ank/config.yml` is one declared schema this binary is held to. The document
// a verb returns is the other (ADR-6fd69efb629c), and it is held to its
// declaration by `tests/golden-json/`. Both live here for the same reason: a
// declaration nothing is measured against is documentation.
//
// **What the conformance walk in `cli.rs` cannot ask.**
// `every_golden_conforms_to_the_shape_its_verb_declares` starts at the fixture
// directory and resolves each fixture to a verb, so it answers *does every
// fixture match its declaration* and can never answer *does every declaration
// have a fixture*: a test that iterates over what exists cannot report an
// absence. That blind spot is what let `read` and `tui` publish a shape through
// `ank help --json` that nothing the binary printed was ever compared against
// (TASK-49b10f02d209). The walk below runs the other way, from the table to the
// directory, and the two questions stay separable by staying in separate files.

/// `read`, captured from the process.
///
/// It lives here rather than beside the other twenty-six in `cli.rs` because the
/// corpus it needs is the one this file already builds: `read` coordinates
/// nothing, so it answers on the files alone with no git anywhere near it
/// (ADR-9307e5d214a7), and the ADR it reads is minted through the binary rather
/// than seeded, so nothing here writes into `.ank/` by hand.
#[test]
fn read_pins_the_document_the_binary_prints() {
    let dir = corpus("golden-read", &format!("schema: {OLDEST}\n"));
    let id = an_adr(&dir);
    let out = Command::new(ANK)
        .args(["read", &id, "--json"])
        .current_dir(&dir)
        .env("ANK_AGENT", AGENT)
        .output()
        .expect("the binary under test must run");
    assert_eq!(
        out.status.code(),
        Some(0),
        "read: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    fixture::pin("read", &String::from_utf8_lossy(&out.stdout));
}

/// Every verb whose contract entry declares a document, against the fixture
/// directory (ADR-6fd69efb629c, TASK-49b10f02d209).
///
/// **This is the direction nothing else runs in.** A verb added with a declared
/// shape and no fixture is a promise `ank help --json` publishes and no test
/// connects to anything the binary prints, and until this walk existed it landed
/// in silence: the fixture-first walk in `cli.rs` had one fewer file to open and
/// stayed green. Measured on 2026-08-31, before the fixtures this task added:
/// twenty-six verbs, twenty-four declaring a document, twenty-two fixtures, and
/// a green suite.
///
/// `mcp` and `watch` declare `output: &[]` and are not asked for one. That is a
/// fact about those two verbs — neither returns a document, one serves a
/// protocol and the other a stream — and it is read off the table rather than
/// spelled as an exception here, so a verb that gains a shape gains an
/// obligation with it.
///
/// The fixture name is the call and the verb is what precedes the dash, which is
/// the convention `cli.rs` writes them under: `config-read` and `config-write`
/// are two documents of one verb.
#[test]
fn every_verb_that_declares_a_document_has_a_golden_fixture() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/golden-json");
    let mut covered: Vec<String> = Vec::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let path = entry.expect("the directory must be readable").path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("a fixture is named")
            .to_string();
        covered.push(
            stem.split('-')
                .next()
                .expect("a split leaves at least one piece")
                .to_string(),
        );
    }
    // The instrument first: a `read_dir` that answered nothing would make every
    // assertion below vacuous, and this walk exists precisely because a vacuous
    // walk is what it is correcting.
    assert!(
        !covered.is_empty(),
        "no fixture was read from {}",
        dir.display()
    );

    let declared: Vec<&str> = ank_contract::COMMANDS
        .iter()
        .filter(|c| !c.output.is_empty())
        .map(|c| c.name)
        .collect();
    assert!(
        !declared.is_empty(),
        "the verb table declares no document at all, which cannot be right"
    );

    let missing: Vec<&str> = declared
        .iter()
        .copied()
        .filter(|name| !covered.iter().any(|c| c == name))
        .collect();
    assert!(
        missing.is_empty(),
        "these verbs declare a document and no fixture in {} pins it: {missing:?}\n\
         Capture one from the process and write it there; `ank help --json` is \
         already promising this shape to a client.",
        dir.display()
    );
}

/// The identity every fixture in `tests/golden-json/` is captured under, so that
/// what survives redaction is the shape and not whoever ran the suite.
const AGENT: &str = "claude-code/1.0.0";

/// One ADR in `dir`, minted through the binary, answering its identifier.
///
/// **The zero prefix is why this is a loop.** A seeded identifier is
/// deterministic and the redaction keeps it; only what a run minted is named
/// away, and the rule that tells the two apart is the `0000` prefix a seeded
/// corpus uses. A minted identifier that happened to start with those four
/// characters would reach the fixture verbatim and redden this suite on the next
/// run, once in sixty-five thousand. Minting another costs a process and settles
/// it.
fn an_adr(dir: &Path) -> String {
    for _ in 0..8 {
        let out = Command::new(ANK)
            .args([
                "new",
                "adr",
                "--title",
                "An example",
                "--scope",
                "src/**",
                "--constraint",
                "Nothing under src/ reaches the network at import time.",
                "--json",
            ])
            .current_dir(dir)
            .env("ANK_AGENT", AGENT)
            .output()
            .expect("the binary under test must run");
        let said = String::from_utf8_lossy(&out.stdout).into_owned();
        assert_eq!(
            out.status.code(),
            Some(0),
            "new adr: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let at = said.find("\"id\":\"").expect("new answers an identifier") + 6;
        let rest = &said[at..];
        let id = rest[..rest.find('"').expect("an id is a closed string")].to_string();
        let minted = id.split_once('-').expect("an identifier has a kind").1;
        if !minted.starts_with("0000") {
            return id;
        }
    }
    panic!("eight identifiers in a row began with the seeded prefix, which is not chance");
}
