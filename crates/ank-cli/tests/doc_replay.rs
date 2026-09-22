//! Every output block the documentation shows is replayed against the binary
//! (TASK-9e80c9a3a5ed, ADR-2b62b9a1fe67).
//!
//! **A block is marked, and the mark is the whole contract.** An HTML comment
//! the rendered page does not show opens with `replay` and a session name, and
//! the indented block after it is replayed:
//!
//! ```text
//! <!-- replay walk ANK_AGENT=human:marie
//! $ git init -q --bare ../origin.git && git remote add origin ../origin.git
//! -->
//!
//!     $ ank init
//!     created .ank/entities
//! ```
//!
//! Lines inside the comment are replayed first and never shown: the setup a
//! reader is told about in prose, or a command whose output the block after it
//! is. A line opening with `$ ` is a command, run by `sh` in the session's
//! repository, and the lines under it up to the next one are what it prints,
//! stdout and stderr as one stream. `--> ` is a request to `ank mcp`, written
//! `>> ` inside the comment where `-->` would close it, and `<-- ` the reply the
//! request with that `id` gets. Every block of a document naming
//! one session runs in one scratch repository, in the order the page reads.
//!
//! Words after the session name set the session up: `KEY=value` is an
//! environment variable, `dir=/path` a directory that exists in the replay
//! under a scratch path and is printed back under the one the page names,
//! `bin=/path` the same holding a copy of the binary that `sh` finds first, and
//! `part` says the block is an excerpt, found whole somewhere in what the
//! command printed rather than equal to it, and `unordered` that its lines are
//! a listing sorted by an id the replay mints, compared in any order.
//!
//! **Compared under the golden redactor.** Both sides go through
//! `fixture::redacted`, the masks every `tests/golden-json/` fixture is written
//! under, and then through the masks a page needs that a JSON document does
//! not: a short id, a short commit, a version, a duration. What is masked is
//! not discarded. An identifier the page names is paired with the one the
//! replay minted where they line up, the pairing has to hold both ways for the
//! whole page, and a later command naming the page's id, or a prefix of it,
//! runs with the replay's. So `ank accept 06d2` reaches the ADR the replay
//! created, and a page quoting one commit under two names fails.

#[allow(dead_code)]
mod fixture;
mod scratch;

use std::collections::HashMap;
use std::fmt::Write as _;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const ANK: &str = env!("CARGO_BIN_EXE_ank");

/// The pages replayed, which is every page a person reads: `docs/` and the
/// README. `every_page_is_replayed` holds this list to the directory.
const PAGES: &[&str] = &[
    "README.md",
    "docs/SUMMARY.md",
    "docs/agents.md",
    "docs/alternatives.md",
    "docs/config-keys.md",
    "docs/entity-fields.md",
    "docs/exit-codes.md",
    "docs/format.md",
    "docs/getting-started.md",
    "docs/integrating.md",
];

const KINDS: &[&str] = &["TASK-", "ADR-", "SPEC-", "LOG-"];

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

// ---------------------------------------------------------------------------
// Reading a page
// ---------------------------------------------------------------------------

/// One marked block: where it is, how its session is set up, and what it runs.
struct Marked {
    session: String,
    env: Vec<(String, String)>,
    dirs: Vec<String>,
    bins: Vec<String>,
    part: bool,
    unordered: bool,
    steps: Vec<Step>,
}

enum Step {
    Shell {
        line: usize,
        cmd: String,
        /// `None` when the page shows nothing under a command that is setup
        /// rather than ank's own output: it must succeed, and is not compared.
        expected: Option<Vec<String>>,
    },
    Mcp {
        line: usize,
        requests: Vec<String>,
        replies: Vec<String>,
    },
}

/// The indented block starting at `lines[i]`, dedented, and the index after it.
fn indented_block(lines: &[&str], mut i: usize) -> Option<(Vec<String>, usize)> {
    while i < lines.len() && lines[i].trim().is_empty() {
        i += 1;
    }
    let first = lines.get(i)?;
    let indent = first.len() - first.trim_start_matches(' ').len();
    if indent < 4 {
        return None;
    }
    let mut block = Vec::new();
    while i < lines.len() {
        let l = lines[i];
        let this = l.len() - l.trim_start_matches(' ').len();
        if !l.trim().is_empty() && this < indent {
            break;
        }
        block.push(l.get(indent..).unwrap_or("").to_string());
        i += 1;
    }
    while block.last().is_some_and(|l| l.trim().is_empty()) {
        block.pop();
    }
    Some((block, i))
}

/// Whether a shell line runs ank, past any `NAME=value` in front of it.
fn runs_ank(cmd: &str) -> bool {
    cmd.split_whitespace()
        .find(|w| !w.contains('=') || w.starts_with('-'))
        .is_some_and(|w| w == "ank")
}

/// A request to `ank mcp`, `--> ` on the page and `>> ` inside a comment,
/// where `-->` would close it; or the reply to one, `<-- `.
fn is_rpc(l: &str) -> bool {
    l.starts_with("--> ") || l.starts_with(">> ") || l.starts_with("<-- ")
}

/// Commands and requests out of the lines of a block, hidden lines first.
///
/// `hidden` counts how many of `lines` came from the comment: a command there
/// showing nothing is setup, where one on the page showing nothing is a claim
/// that ank prints nothing.
fn steps(lines: &[String], hidden: usize, at: &[usize]) -> Vec<Step> {
    let mut steps: Vec<Step> = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let l = &lines[i];
        let at = at[i];
        if let Some(c) = l.strip_prefix("$ ") {
            let from_page = i >= hidden;
            let mut cmd = c.to_string();
            while cmd.ends_with('\\') && i + 1 < lines.len() {
                i += 1;
                cmd.push('\n');
                cmd.push_str(&lines[i]);
            }
            i += 1;
            let mut out = Vec::new();
            while i < lines.len() && !lines[i].starts_with("$ ") && !is_rpc(&lines[i]) {
                out.push(lines[i].clone());
                i += 1;
            }
            while out.last().is_some_and(|l| l.trim().is_empty()) {
                out.pop();
            }
            let checked = !out.is_empty() || (from_page && runs_ank(&cmd));
            steps.push(Step::Shell {
                line: at,
                cmd,
                expected: checked.then_some(out),
            });
        } else if is_rpc(l) {
            let mut requests = Vec::new();
            let mut replies = Vec::new();
            while i < lines.len() && is_rpc(&lines[i]) {
                if let Some(r) = lines[i]
                    .strip_prefix("--> ")
                    .or_else(|| lines[i].strip_prefix(">> "))
                {
                    requests.push(r.to_string());
                } else {
                    replies.push(lines[i][4..].to_string());
                }
                i += 1;
            }
            steps.push(Step::Mcp {
                line: at,
                requests,
                replies,
            });
        } else if l.trim().is_empty() {
            i += 1;
        } else {
            panic!("line {at}: a replayed block opens with `$ `, `--> ` or `<-- `, not {l:?}");
        }
    }
    steps
}

fn marked_blocks(text: &str) -> Vec<Marked> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let Some(head) = lines[i].trim_start().strip_prefix("<!-- replay ") else {
            i += 1;
            continue;
        };
        let start = i + 1;
        let (head, closed) = match head.strip_suffix("-->") {
            Some(h) => (h, true),
            None => (head, false),
        };
        let mut words = head.split_whitespace();
        let session = words
            .next()
            .expect("a replay mark names its session")
            .to_string();
        let (mut env, mut dirs, mut bins) = (Vec::new(), Vec::new(), Vec::new());
        let (mut part, mut unordered) = (false, false);
        for w in words {
            if w == "part" {
                part = true;
            } else if w == "unordered" {
                unordered = true;
            } else if let Some(d) = w.strip_prefix("dir=") {
                dirs.push(d.to_string());
            } else if let Some(d) = w.strip_prefix("bin=") {
                bins.push(d.to_string());
            } else if let Some((k, v)) = w.split_once('=') {
                env.push((k.to_string(), v.to_string()));
            } else {
                panic!("line {start}: unknown replay option {w:?}");
            }
        }
        let mut hidden: Vec<String> = Vec::new();
        i += 1;
        if !closed {
            while i < lines.len() && lines[i].trim() != "-->" {
                hidden.push(lines[i].to_string());
                i += 1;
            }
            i += 1;
        }
        let n_hidden = hidden.len();
        let mut body = hidden;
        let mut at: Vec<usize> = (0..n_hidden).map(|k| start + 1 + k).collect();
        let mut next = i;
        // A block directly under the mark; a mark followed by prose is setup
        // alone.
        let gap = lines
            .get(i..)
            .unwrap_or(&[])
            .iter()
            .take_while(|l| l.trim().is_empty())
            .count();
        if gap <= 1 {
            if let Some((shown, after)) = indented_block(&lines, i) {
                at.extend((0..shown.len()).map(|k| i + gap + 1 + k));
                body.extend(shown);
                next = after;
            }
        }
        let steps = steps(&body, n_hidden, &at);
        out.push(Marked {
            session,
            env,
            dirs,
            bins,
            part,
            unordered,
            steps,
        });
        i = next;
    }
    out
}

// ---------------------------------------------------------------------------
// Masks and pairing
// ---------------------------------------------------------------------------

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

/// Every hex word, with where it sits: `(start, end)` in chars.
fn hex_words(s: &[char]) -> Vec<(usize, usize)> {
    let mut words = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if s[i].is_ascii_hexdigit() && (i == 0 || !is_word(s[i - 1])) {
            let mut j = i;
            while j < s.len() && s[j].is_ascii_hexdigit() {
                j += 1;
            }
            if j == s.len() || !is_word(s[j]) {
                words.push((i, j));
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    words
}

fn kind_before(s: &[char], at: usize) -> bool {
    KINDS.iter().any(|k| {
        let k: Vec<char> = k.chars().collect();
        at >= k.len() && s[at - k.len()..at] == k[..]
    })
}

/// What a replay mints and a page cannot know: an entity id and its short form,
/// a commit and its short form, a content hash. A seeded id, with the zero
/// prefix, is deterministic and is kept, as the golden redactor keeps it.
fn is_minted(s: &[char], (a, b): (usize, usize)) -> bool {
    let n = b - a;
    let zeros = s[a..b].iter().take(4).all(|&c| c == '0');
    match n {
        40 => true,
        12 => !zeros,
        4..=11 if kind_before(s, a) => true,
        7..=11 => true,
        _ => false,
    }
}

/// The page's masks, over what the golden redactor already masked.
fn masked(s: &str) -> String {
    let golden = fixture::redacted(s);
    let c: Vec<char> = golden.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    let words = hex_words(&c);
    let mut w = words.iter().peekable();
    while i < c.len() {
        while w.peek().is_some_and(|&&(a, _)| a < i) {
            w.next();
        }
        if let Some(&&(a, b)) = w.peek() {
            if a == i && is_minted(&c, (a, b)) {
                out.push_str(if kind_before(&c, a) { "<ID>" } else { "<REV>" });
                i = b;
                continue;
            }
        }
        // A version, `1.2.3`, and a duration, `0.4s`.
        if c[i].is_ascii_digit() && (i == 0 || !is_word(c[i - 1]) && c[i - 1] != '.') {
            let mut j = i;
            let mut dots = 0;
            while j < c.len()
                && (c[j].is_ascii_digit()
                    || c[j] == '.' && j + 1 < c.len() && c[j + 1].is_ascii_digit())
            {
                dots += usize::from(c[j] == '.');
                j += 1;
            }
            if dots == 2 && (j == c.len() || !is_word(c[j])) {
                out.push_str("<VERSION>");
                i = j;
                continue;
            }
            if dots == 1 && c.get(j) == Some(&'s') && c.get(j + 1).is_none_or(|&n| !is_word(n)) {
                out.push_str("<SECS>");
                i = j + 1;
                continue;
            }
        }
        out.push(c[i]);
        i += 1;
    }
    out
}

/// The minted words of a line, in order: what `masked` hid, read back.
fn minted(s: &str) -> Vec<String> {
    let c: Vec<char> = s.chars().collect();
    hex_words(&c)
        .into_iter()
        .filter(|&w| is_minted(&c, w))
        .map(|(a, b)| c[a..b].iter().collect())
        .collect()
}

/// The page's minted values paired with the replay's, both ways.
#[derive(Default)]
struct Pairs {
    page: HashMap<String, String>,
    replay: HashMap<String, String>,
}

impl Pairs {
    /// Pairs two lines already equal under the masks. A page value already
    /// paired with another replay value, or the reverse, is a page contradicting
    /// itself: one commit under two names, or two under one.
    fn learn(&mut self, page: &str, replay: &str) -> Result<(), String> {
        for (p, r) in minted(page).into_iter().zip(minted(replay)) {
            if p == r {
                continue;
            }
            if let Some(had) = self.page.get(&p) {
                if *had != r {
                    return Err(format!(
                        "the page's {p} stands for {had} earlier and for {r} here"
                    ));
                }
            }
            if let Some(had) = self.replay.get(&r) {
                if *had != p {
                    return Err(format!(
                        "the replay's {r} is written {had} earlier on the page and {p} here"
                    ));
                }
            }
            self.page.insert(p.clone(), r.clone());
            self.replay.insert(r, p);
        }
        Ok(())
    }

    /// The page's command, naming what the replay minted: a paired value, or a
    /// prefix of one, becomes the same prefix of its pair.
    fn translate(&self, cmd: &str) -> String {
        let c: Vec<char> = cmd.chars().collect();
        let mut out = String::new();
        let mut last = 0;
        for (a, b) in hex_words(&c) {
            let word: String = c[a..b].iter().collect();
            if b - a < 4 {
                continue;
            }
            let hit = self.page.get(&word).cloned().or_else(|| {
                self.page
                    .iter()
                    .find(|(p, r)| {
                        p.len() > word.len() && p.starts_with(&word) && r.len() >= word.len()
                    })
                    .map(|(_, r)| r[..word.len()].to_string())
            });
            if let Some(r) = hit {
                out.extend(&c[last..a]);
                out.push_str(&r);
                last = b;
            }
        }
        out.extend(&c[last..]);
        out
    }
}

// ---------------------------------------------------------------------------
// Replaying
// ---------------------------------------------------------------------------

struct Session {
    root: PathBuf,
    repo: PathBuf,
    env: Vec<(String, String)>,
    /// A path the page names, and the scratch path standing for it.
    dirs: Vec<(String, PathBuf)>,
    /// Aliased directories holding a copy of the binary, ahead of it on `PATH`.
    bins: Vec<PathBuf>,
    pairs: Pairs,
}

/// A path as `sh` takes it on every platform: forward slashes.
fn slashed(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

impl Session {
    fn new(page: &str, name: &str) -> Session {
        let stem = Path::new(page)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let root = scratch::dir(&format!("doc-{stem}-{name}"));
        let repo = root.join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::create_dir_all(root.join("home")).unwrap();
        std::fs::write(
            root.join("gitconfig"),
            "[user]\n\tname = doc\n\temail = doc@example.com\n[commit]\n\tgpgsign = false\n\
             [tag]\n\tgpgsign = false\n[init]\n\tdefaultBranch = main\n[core]\n\tautocrlf = false\n",
        )
        .unwrap();
        let s = Session {
            root,
            repo,
            env: Vec::new(),
            dirs: Vec::new(),
            bins: Vec::new(),
            pairs: Pairs::default(),
        };
        let out = s.command("git init -q").output().unwrap();
        assert!(out.status.success(), "git init: {out:?}");
        s
    }

    fn alias(&mut self, page_path: &str) {
        if self.dirs.iter().any(|(p, _)| p == page_path) {
            return;
        }
        let name: String = page_path
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() || c == '.' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let real = self.root.join(format!("dir{name}"));
        std::fs::create_dir_all(&real).unwrap();
        self.dirs.push((page_path.to_string(), real));
    }

    /// A copy of the binary in an aliased directory, found first on `PATH`: a
    /// build under a cargo target directory is refused by `update`, and a copy
    /// outside it is what an installed binary looks like.
    fn install(&mut self, page_path: &str) {
        let dir = self
            .dirs
            .iter()
            .find(|(p, _)| p == page_path)
            .unwrap()
            .1
            .clone();
        if self.bins.contains(&dir) {
            return;
        }
        let exe = Path::new(ANK);
        std::fs::copy(exe, dir.join(exe.file_name().unwrap())).unwrap();
        self.bins.push(dir);
    }

    /// The page's text with every aliased path swapped for its scratch path.
    fn to_replay(&self, s: &str) -> String {
        let mut s = self.pairs.translate(s);
        for (page, real) in &self.dirs {
            s = s.replace(page.as_str(), &slashed(real));
        }
        s
    }

    /// The replay's text with every scratch path written as the page writes it.
    fn to_page(&self, s: &str) -> String {
        let mut s = s.replace("\r\n", "\n");
        for (page, real) in &self.dirs {
            let mut forms = vec![slashed(real), real.to_string_lossy().to_string()];
            if let Ok(c) = real.canonicalize() {
                forms.push(slashed(&c));
                forms.push(c.to_string_lossy().to_string());
            }
            forms.sort_by_key(|f| std::cmp::Reverse(f.len()));
            for f in forms {
                s = s.replace(&f, page);
            }
        }
        s
    }

    fn command(&self, script: &str) -> Command {
        let mut paths = self.bins.clone();
        paths.push(Path::new(ANK).parent().unwrap().to_path_buf());
        paths.extend(std::env::split_paths(
            &std::env::var_os("PATH").unwrap_or_default(),
        ));
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(format!("exec 2>&1\n{script}\n"))
            .current_dir(&self.repo)
            .env("PATH", std::env::join_paths(paths).unwrap())
            .env("HOME", self.root.join("home"))
            .env("XDG_CONFIG_HOME", self.root.join("home"))
            .env("APPDATA", self.root.join("home"))
            .env("GIT_CONFIG_GLOBAL", self.root.join("gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("ANK_AGENT")
            .env_remove("ANK_UPDATE_REPOSITORY")
            .env_remove("NO_COLOR")
            // Git for Windows' `sh` rewrites an argument opening with `/` into a
            // path under its own root; the page's `/nope/nope` has to reach ank
            // as the page wrote it.
            .env("MSYS_NO_PATHCONV", "1")
            .env("MSYS2_ARG_CONV_EXCL", "*")
            .stdin(Stdio::null());
        for (k, v) in &self.env {
            cmd.env(k, self.to_replay(v));
        }
        cmd
    }

    fn run(&mut self, cmd: &str) -> (String, bool) {
        let script = self.to_replay(cmd);
        let out = self
            .command(&script)
            .output()
            .expect("sh must be on PATH: ank's verifiers need it on every platform");
        let text = self.to_page(&String::from_utf8_lossy(&out.stdout));
        (
            text.trim_end_matches('\n').to_string(),
            out.status.success(),
        )
    }

    fn talk(&self, requests: &[String]) -> Vec<String> {
        let mut cmd = Command::new(ANK);
        cmd.arg("mcp")
            .arg("--repo")
            .arg(&self.repo)
            .current_dir(&self.repo)
            .env("HOME", self.root.join("home"))
            .env("XDG_CONFIG_HOME", self.root.join("home"))
            .env("APPDATA", self.root.join("home"))
            .env("GIT_CONFIG_GLOBAL", self.root.join("gitconfig"))
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env_remove("ANK_AGENT")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, v) in &self.env {
            cmd.env(k, self.to_replay(v));
        }
        let mut child = cmd.spawn().expect("the binary must have been built");
        {
            let stdin = child.stdin.as_mut().unwrap();
            for r in requests {
                // JSON carries a path with its backslashes escaped.
                let r = self.to_replay(r);
                writeln!(stdin, "{r}").unwrap();
            }
        }
        let out = child.wait_with_output().unwrap();
        self.to_page(&String::from_utf8_lossy(&out.stdout))
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(str::to_string)
            .collect()
    }
}

/// A session removes its scratch repository, the copies of the binary with
/// it: `scratch` sweeps only what a killed run left.
impl Drop for Session {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// `"id":N` of a JSON-RPC line, as written.
fn rpc_id(line: &str) -> Option<&str> {
    let rest = &line[line.find("\"id\":")? + 5..];
    Some(&rest[..rest.find([',', '}'])?])
}

/// Where `expected` sits whole inside `actual`, line for line, under the masks.
fn excerpt_at(expected: &[String], actual: &[String]) -> Option<usize> {
    let e: Vec<String> = expected.iter().map(|l| masked(l)).collect();
    let a: Vec<String> = actual.iter().map(|l| masked(l)).collect();
    (0..=a.len().checked_sub(e.len())?).find(|&k| a[k..k + e.len()] == e[..])
}

/// `actual` rearranged into the order of `expected`, when the two hold the
/// same lines under the masks: for a listing sorted by an id the replay mints,
/// whose order is the one thing a page cannot reproduce.
fn in_any_order(expected: &[String], actual: &[String]) -> Option<Vec<String>> {
    if expected.len() != actual.len() {
        return None;
    }
    let mut left: Vec<Option<&String>> = actual.iter().map(Some).collect();
    expected
        .iter()
        .map(|e| {
            let k = left
                .iter()
                .position(|a| a.is_some_and(|a| masked(a) == masked(e)))?;
            left[k].take().cloned()
        })
        .collect()
}

/// Replays one page, and returns every disagreement found, each naming its line.
fn replay(page: &str, text: &str) -> Vec<String> {
    let mut sessions: HashMap<String, Session> = HashMap::new();
    let mut failures = Vec::new();
    for block in marked_blocks(text) {
        let s = sessions
            .entry(block.session.clone())
            .or_insert_with(|| Session::new(page, &block.session));
        for (k, v) in &block.env {
            s.env.retain(|(have, _)| have != k);
            s.env.push((k.clone(), v.clone()));
        }
        for d in block.dirs.iter().chain(&block.bins) {
            s.alias(d);
        }
        for b in &block.bins {
            s.install(b);
        }
        let last = block.steps.len().saturating_sub(1);
        for (n, step) in block.steps.iter().enumerate() {
            match step {
                Step::Shell {
                    line,
                    cmd,
                    expected,
                } => {
                    let (actual, ok) = s.run(cmd);
                    let Some(expected) = expected else {
                        if !ok {
                            failures
                                .push(format!("{page}:{line}: setup `{cmd}` failed:\n{actual}"));
                        }
                        continue;
                    };
                    let actual: Vec<String> = actual.lines().map(str::to_string).collect();
                    let part = block.part && n == last;
                    let aligned = if part {
                        excerpt_at(expected, &actual)
                            .map(|k| actual[k..k + expected.len()].to_vec())
                    } else if block.unordered && n == last {
                        in_any_order(expected, &actual)
                    } else {
                        let same = expected.len() == actual.len()
                            && expected
                                .iter()
                                .zip(&actual)
                                .all(|(e, a)| masked(e) == masked(a));
                        same.then(|| actual.clone())
                    };
                    match aligned {
                        Some(a) => {
                            for (e, a) in expected.iter().zip(&a) {
                                if let Err(why) = s.pairs.learn(e, a) {
                                    failures.push(format!("{page}:{line}: `{cmd}`: {why}"));
                                }
                            }
                        }
                        None => {
                            failures.push(difference(page, *line, cmd, expected, &actual, part))
                        }
                    }
                }
                Step::Mcp {
                    line,
                    requests,
                    replies,
                } => {
                    let actual = s.talk(requests);
                    for e in replies {
                        let id = rpc_id(e).unwrap_or("?");
                        let got = actual.iter().find(|a| rpc_id(a) == Some(id));
                        match got {
                            Some(a) if masked(a) == masked(e) => {
                                if let Err(why) = s.pairs.learn(e, a) {
                                    failures.push(format!("{page}:{line}: reply {id}: {why}"));
                                }
                            }
                            _ => failures.push(format!(
                                "{page}:{line}: the reply to request {id} differs\n  page:   {}\n  binary: {}",
                                masked(e),
                                got.map(|a| masked(a)).unwrap_or_else(|| format!("(none among {actual:?})"))
                            )),
                        }
                    }
                }
            }
        }
    }
    failures
}

fn difference(
    page: &str,
    line: usize,
    cmd: &str,
    expected: &[String],
    actual: &[String],
    part: bool,
) -> String {
    let mut s = format!(
        "{page}:{line}: `{cmd}` {}, compared under the masks\n",
        if part {
            "prints no run of lines equal to the block"
        } else {
            "prints something other than the block"
        }
    );
    let _ = writeln!(s, "  --- page");
    for l in expected {
        let _ = writeln!(s, "  | {}", masked(l));
    }
    let _ = writeln!(s, "  --- binary");
    for l in actual {
        let _ = writeln!(s, "  | {}", masked(l));
    }
    s
}

fn replay_page(page: &str) {
    let text = std::fs::read_to_string(workspace().join(page))
        .unwrap()
        .replace("\r\n", "\n");
    let failures = replay(page, &text);
    assert!(
        failures.is_empty(),
        "{} output block(s) of {page} disagree with the binary:\n\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ---------------------------------------------------------------------------
// The pages
// ---------------------------------------------------------------------------

#[test]
fn the_readme_replays() {
    replay_page("README.md");
}

#[test]
fn getting_started_replays() {
    replay_page("docs/getting-started.md");
}

#[test]
fn agents_replays() {
    replay_page("docs/agents.md");
}

#[test]
fn integrating_replays() {
    replay_page("docs/integrating.md");
}

#[test]
fn format_replays() {
    replay_page("docs/format.md");
}

#[test]
fn the_other_pages_replay() {
    for page in [
        "docs/SUMMARY.md",
        "docs/alternatives.md",
        "docs/config-keys.md",
        "docs/entity-fields.md",
        "docs/exit-codes.md",
    ] {
        replay_page(page);
    }
}

/// The list above is the directory, so a page added to `docs/` is replayed or
/// this goes red.
#[test]
fn every_page_is_replayed() {
    let mut found: Vec<String> = std::fs::read_dir(workspace().join("docs"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".md"))
        .map(|n| format!("docs/{n}"))
        .collect();
    found.push("README.md".to_string());
    found.sort();
    let mut listed: Vec<String> = PAGES.iter().map(|p| p.to_string()).collect();
    listed.sort();
    assert_eq!(found, listed, "a page of docs/ is not in PAGES");
}

/// **What makes the replay complete rather than opt-in.** A block that shows
/// ank printing something -- a `$ ank` line with output under it, a reply of
/// `ank mcp`, a JSON document, a refusal -- carries a mark, or it is rewritten
/// so it no longer presents itself as output.
#[test]
fn every_block_showing_ank_output_is_marked() {
    let mut unmarked = Vec::new();
    for page in PAGES {
        let text = std::fs::read_to_string(workspace().join(page))
            .unwrap()
            .replace("\r\n", "\n");
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        let mut marked = false;
        while i < lines.len() {
            let l = lines[i];
            if l.trim_start().starts_with("<!-- replay ") {
                marked = true;
                while !lines[i].trim_end().ends_with("-->") {
                    i += 1;
                }
                i += 1;
                continue;
            }
            let fresh = i == 0 || lines[i - 1].trim().is_empty();
            if fresh && l.starts_with("    ") {
                let (block, next) = indented_block(&lines, i).unwrap();
                if presents_output(&block) && !marked {
                    unmarked.push(format!("{page}:{}: {}", i + 1, block[0]));
                }
                marked = false;
                i = next;
                continue;
            }
            if !l.trim().is_empty() {
                marked = false;
            }
            i += 1;
        }
    }
    assert!(
        unmarked.is_empty(),
        "these blocks present themselves as ank output and nothing replays them; \
         mark each with `<!-- replay <session> -->` or rewrite it:\n{}",
        unmarked.join("\n")
    );
}

fn presents_output(block: &[String]) -> bool {
    let mut after_ank = false;
    let mut continued = false;
    for l in block {
        if let Some(c) = l.strip_prefix("$ ") {
            after_ank = runs_ank(c);
            continued = c.ends_with('\\');
            continue;
        }
        if continued {
            continued = l.ends_with('\\');
            continue;
        }
        let t = l.trim_start();
        if after_ank && !t.is_empty()
            || t.starts_with("<-- ")
            || t.starts_with("{\"contract\"")
            || t.starts_with("{\"jsonrpc\"")
            || t.starts_with("error[")
            || t.starts_with("warning: ")
        {
            return true;
        }
    }
    false
}

/// **A replay that cannot fail proves nothing**, so one is made to. A block of
/// the page, replayed as the page has it, agrees with the binary; the same
/// block with one word changed does not, and the failure names the line.
#[test]
fn an_altered_block_fails_the_replay() {
    let page = "docs/integrating.md";
    let text = std::fs::read_to_string(workspace().join(page))
        .unwrap()
        .replace("\r\n", "\n");
    let at = text
        .find("    $ ank help check\n")
        .expect("the page shows `ank help check`");
    let end = text[at..].find("\n\n").map(|e| at + e).unwrap();
    let block = &text[at..end];

    let original = format!("<!-- replay altered -->\n\n{block}\n");
    assert_eq!(
        replay(page, &original),
        Vec::<String>::new(),
        "the block as the page has it"
    );

    let altered = format!(
        "<!-- replay altered -->\n\n{}\n",
        block.replacen("so it writes", "so it reads", 1)
    );
    assert_ne!(altered, original, "the alteration must change the block");
    let failures = replay(page, &altered);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(
        failures[0].contains("so it reads") && failures[0].contains("so it writes"),
        "the failure shows both sides: {}",
        failures[0]
    );
}

/// The masks hide what a replay mints and nothing else.
#[test]
fn the_masks_hide_what_a_replay_mints() {
    assert_eq!(
        masked("accepted ADR-06d29e727d24 -> 9c45c50"),
        "accepted ADR-<HEX> -> <REV>"
    );
    assert_eq!(
        masked("  TASK-820d  [open] Migrate"),
        "  TASK-<ID>  [open] Migrate"
    );
    assert_eq!(
        masked("ank 0.8.0 (8310e75, skill 0d916cc3d9a5)"),
        "ank <VERSION> (<REV>, skill <HEX>)"
    );
    assert_eq!(
        masked("running: no-jwt ... ok (0.0s)"),
        "running: no-jwt ... ok (<SECS>)"
    );
    assert_eq!(
        masked("2 tasks, 1 adr, 4 signal(s)"),
        "2 tasks, 1 adr, 4 signal(s)"
    );
    assert_eq!(masked("at 2026-07-27T09:40:00Z"), "at <TIME>");
}

/// One value under two names is the page contradicting itself.
#[test]
fn a_page_naming_one_value_twice_is_caught() {
    let mut p = Pairs::default();
    p.learn("-> 9c45c50", "-> 1111111").unwrap();
    p.learn("at 9c45c50", "at 1111111").unwrap();
    assert!(p.learn("at c482be8", "at 1111111").is_err());
    assert!(p.learn("at 9c45c50", "at 2222222").is_err());
    assert_eq!(p.translate("ank show 9c45"), "ank show 1111");
}
