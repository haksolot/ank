//! `status`, `find` and `context` take `--peers` and add each declared peer's
//! answer, claims included (ADR-376bed7fa39a, TASK-ae6c94125af5).
//!
//! The fixture is the trio the criterion names: `web` declares `api` and `db`
//! as peers, and each of the three holds one open task nobody holds and one
//! task a distinct identity has claimed. `api` also holds an accepted ADR whose
//! scope names `web:src/**`, which binds `web` from the peer.
//!
//! **Read only.** Every file under `api` and `db`, `.git` included, and every
//! ref of both, is compared before and after.
//!
//! **Through the binary**, every case.

use crate::scratch;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const ANK: &str = env!("CARGO_BIN_EXE_ank");
/// The caller: it holds nothing anywhere.
const CALLER: &str = "claude-code@peer-aggregate";

/// The ADR of `api` that binds `web`.
const API_ADR: &str = "ADR-a0a0a0a0a0a0";

struct Trio {
    base: PathBuf,
    web: PathBuf,
    api: PathBuf,
    db: PathBuf,
}

fn ank(base: &Path, dir: &Path, agent: &str, args: &[&str]) -> Output {
    let root = scratch::root();
    let home = base.join("home");
    Command::new(ANK)
        .args(args)
        .current_dir(dir)
        .env("ANK_AGENT", agent)
        .env("TMPDIR", root)
        .env("TMP", root)
        .env("TEMP", root)
        .env("NO_COLOR", "1")
        .env("XDG_CONFIG_HOME", &home)
        .env("APPDATA", &home)
        .output()
        .expect("the binary runs")
}

impl Trio {
    /// Runs in `web` as the caller, and must succeed: stdout, stderr.
    fn web(&self, args: &[&str]) -> (String, String) {
        let out = ank(&self.base, &self.web, CALLER, args);
        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
        assert_eq!(out.status.code(), Some(0), "ank {args:?}: {stderr}");
        (String::from_utf8_lossy(&out.stdout).to_string(), stderr)
    }
}

fn ok(base: &Path, dir: &Path, agent: &str, args: &[&str]) -> String {
    let out = ank(base, dir, agent, args);
    assert!(
        out.status.success(),
        "ank {args:?} in {}: {}",
        dir.display(),
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("git must be on PATH");
    assert!(
        out.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn commit(dir: &Path) {
    git(dir, &["add", "-A"]);
    git(
        dir,
        &[
            "-c",
            "user.email=t@example.invalid",
            "-c",
            "user.name=t",
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-qm",
            "corpus",
        ],
    );
}

fn id_of(json: &str) -> String {
    let at = json.find("\"TASK-").expect("the new task is named") + 1;
    json[at..at + 17].to_string()
}

/// A corpus named `name`, holding `<name> open work` and `<name> held work`,
/// the second claimed by `<name>-holder`. Returns the two task ids.
fn corpus(base: &Path, dir: &Path, name: &str, peers: &[(&str, &str)]) -> (String, String) {
    fs::create_dir_all(dir).unwrap();
    git(dir, &["init", "-q", "-b", "main"]);
    ok(base, dir, CALLER, &["init"]);
    ok(base, dir, CALLER, &["config", "default_branch", "main"]);
    for (peer, path) in peers {
        ok(
            base,
            dir,
            CALLER,
            &["config", &format!("peers.{peer}"), path],
        );
    }
    let task = |title: &str| {
        id_of(&ok(
            base,
            dir,
            CALLER,
            &[
                "new",
                "task",
                "--title",
                title,
                "--scope",
                "src/**",
                "--criteria",
                "It holds.",
                "--no-verify",
                "--json",
            ],
        ))
    };
    let open = task(&format!("{name} open work"));
    let held = task(&format!("{name} held work"));
    if name == "api" {
        fs::write(
            dir.join(format!(".ank/entities/{API_ADR}.md")),
            format!(
                "---\nid: {API_ADR}\ntype: adr\nslug: api-binds-web\n\
                 title: The api rule that binds web\n\
                 created: 2026-08-01T00:00:00Z\nstatus: accepted\nscope:\n  - 'web:src/**'\n\
                 constraint: |\n  A rule api states about web.\nschema: 1\nversion: 1\n---\n\nWhy.\n"
            ),
        )
        .unwrap();
    }
    commit(dir);
    ok(base, dir, &format!("{name}-holder"), &["claim", &held]);
    (open, held)
}

/// Every file under `dir`, `.git` included, with its bytes, and every ref.
fn snapshot(dir: &Path) -> (BTreeMap<PathBuf, Vec<u8>>, String) {
    let mut files = BTreeMap::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in fs::read_dir(&d).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                stack.push(p);
            } else {
                files.insert(
                    p.strip_prefix(dir).unwrap().to_path_buf(),
                    fs::read(&p).unwrap(),
                );
            }
        }
    }
    (files, git(dir, &["for-each-ref"]))
}

/// The lines of `text` that name the peer `db` as missing.
fn db_warnings(text: &str) -> usize {
    text.lines().filter(|l| l.contains("peer 'db'")).count()
}

#[test]
fn status_find_and_context_add_each_declared_peer_and_write_nothing_there() {
    let base = scratch::dir("peer-aggregate");
    let t = Trio {
        web: base.join("web"),
        api: base.join("api"),
        db: base.join("db"),
        base,
    };
    let (api_open, api_held) = corpus(&t.base, &t.api, "api", &[("web", "../web")]);
    let (db_open, db_held) = corpus(&t.base, &t.db, "db", &[]);
    let (web_open, web_held) = corpus(
        &t.base,
        &t.web,
        "web",
        &[("api", "../api"), ("db", "../db")],
    );
    let api_before = snapshot(&t.api);
    let db_before = snapshot(&t.db);

    // ---- find ----------------------------------------------------------
    let (plain, _) = t.web(&["find", "--type", "task"]);
    assert!(plain.contains("web open work"), "{plain}");
    assert!(
        !plain.contains("api open work") && !plain.contains("@api"),
        "without --peers find lists web alone: {plain}"
    );
    let (found, err) = t.web(&["find", "--type", "task", "--peers"]);
    for title in [
        "web open work",
        "web held work",
        "api open work",
        "api held work",
        "db open work",
        "db held work",
    ] {
        assert!(found.contains(title), "find --peers lists {title}: {found}");
    }
    for (title, peer) in [("api open work", "api"), ("db held work", "db")] {
        let line = found.lines().find(|l| l.contains(title)).unwrap();
        assert!(
            line.contains(&format!("@{peer}")),
            "a peer's row is addressed <id>@{peer}: {line}"
        );
    }
    let line = found.lines().find(|l| l.contains("web open work")).unwrap();
    assert!(!line.contains('@'), "a local row is not addressed: {line}");
    // The claim is part of the peer's answer.
    let line = found.lines().find(|l| l.contains("api held work")).unwrap();
    assert!(line.contains("claimed:api-holder"), "{line}");
    assert_eq!(db_warnings(&err), 0, "{err}");

    let (json, _) = t.web(&["find", "--type", "task", "--peers", "--json"]);
    assert!(
        json.contains(&format!("\"id\":\"{api_open}\",\"peer\":\"api\"")),
        "{json}"
    );
    assert!(
        json.contains(&format!("\"id\":\"{db_held}\",\"peer\":\"db\"")),
        "{json}"
    );
    assert!(
        json.contains(&format!("\"id\":\"{web_open}\",\"peer\":null")),
        "{json}"
    );
    let (json, _) = t.web(&["find", "--type", "task", "--json"]);
    assert!(
        !json.contains("\"peer\""),
        "without --peers no row gains a key: {json}"
    );

    // ---- status --------------------------------------------------------
    let (plain, _) = t.web(&["status"]);
    assert!(plain.contains("0 constraint(s)"), "{plain}");
    assert!(plain.contains("web-holder"), "{plain}");
    assert!(
        !plain.contains("api-holder") && !plain.contains("db-holder"),
        "without --peers status names web's claims alone: {plain}"
    );
    let (status, err) = t.web(&["status", "--peers"]);
    assert!(
        status.contains("1 constraint(s)"),
        "the api ADR binding web is counted: {status}"
    );
    for (holder, addressed) in [
        ("web-holder", web_held[..9].to_string()),
        ("api-holder", "@api".to_string()),
        ("db-holder", "@db".to_string()),
    ] {
        let line = status
            .lines()
            .find(|l| l.contains(holder))
            .unwrap_or_else(|| panic!("status --peers names {holder}: {status}"));
        assert!(line.contains(&addressed), "{holder}'s claim: {line}");
    }
    let line = status.lines().find(|l| l.contains("web-holder")).unwrap();
    assert!(
        !line.contains('@'),
        "web's own claim is not addressed: {line}"
    );
    assert_eq!(db_warnings(&err), 0, "{err}");

    let (json, _) = t.web(&["status", "--peers", "--json"]);
    assert!(json.contains(&format!("\"id\":\"{api_held}\"")), "{json}");
    assert!(json.contains("\"peer\":\"api\""), "{json}");
    assert!(json.contains("\"peer\":\"db\""), "{json}");
    assert!(json.contains("\"peer\":null"), "{json}");
    assert!(json.contains("\"constraints\":1"), "{json}");
    let (json, _) = t.web(&["status", "--json"]);
    assert!(!json.contains("\"peer\""), "{json}");
    assert!(json.contains("\"constraints\":0"), "{json}");

    // ---- context -------------------------------------------------------
    let (plain, _) = t.web(&["context", "src/"]);
    assert!(plain.contains("web open work"), "{plain}");
    assert!(!plain.contains("api open work"), "{plain}");
    let (ctx, err) = t.web(&["context", "--peers", "src/"]);
    assert!(ctx.contains("web open work"), "{ctx}");
    for (title, peer) in [("api open work", "api"), ("db open work", "db")] {
        let line = ctx
            .lines()
            .find(|l| l.contains(title))
            .unwrap_or_else(|| panic!("context --peers lists {title}: {ctx}"));
        assert!(line.contains(&format!("@{peer}")), "{line}");
    }
    assert!(
        !ctx.contains("api held work") && !ctx.contains("db held work"),
        "a peer task held over there is not ready here: {ctx}"
    );
    assert!(
        ctx.contains(&format!("> ank claim {}", &web_open[..9]))
            || ctx.contains(&format!("> ank claim {web_open}")),
        "the next command is a local claim: {ctx}"
    );
    assert_eq!(db_warnings(&ctx) + db_warnings(&err), 0, "{ctx}{err}");

    let (json, _) = t.web(&["context", "--peers", "src/", "--json"]);
    assert!(json.contains(&format!("\"id\":\"{api_open}\"")), "{json}");
    assert!(json.contains(&format!("\"id\":\"{db_open}\"")), "{json}");
    assert!(json.contains("\"peer\":\"api\""), "{json}");
    assert!(json.contains("\"peer\":null"), "{json}");
    let (json, _) = t.web(&["context", "src/", "--json"]);
    assert!(!json.contains("\"peer\""), "{json}");

    // ---- db moved away -------------------------------------------------
    let moved = t.base.join("db-moved");
    fs::rename(&t.db, &moved).unwrap();
    let (found, err) = t.web(&["find", "--type", "task", "--peers"]);
    assert!(
        found.contains("web open work") && found.contains("api open work"),
        "{found}"
    );
    assert!(!found.contains("db open work"), "{found}");
    assert_eq!(db_warnings(&found) + db_warnings(&err), 1, "{found}{err}");

    let (status, err) = t.web(&["status", "--peers"]);
    assert!(
        status.contains("web-holder") && status.contains("api-holder"),
        "{status}"
    );
    assert!(!status.contains("db-holder"), "{status}");
    assert!(status.contains("1 constraint(s)"), "{status}");
    assert_eq!(db_warnings(&status) + db_warnings(&err), 1, "{status}{err}");

    let (ctx, err) = t.web(&["context", "--peers", "src/"]);
    assert!(
        ctx.contains("web open work") && ctx.contains("api open work"),
        "{ctx}"
    );
    assert!(!ctx.contains("db open work"), "{ctx}");
    assert_eq!(db_warnings(&ctx) + db_warnings(&err), 1, "{ctx}{err}");
    fs::rename(&moved, &t.db).unwrap();

    // ---- nothing was written over there ---------------------------------
    assert!(snapshot(&t.api) == api_before, "nothing under api changed");
    assert!(snapshot(&t.db) == db_before, "nothing under db changed");

    let _ = fs::remove_dir_all(&t.base);
}
