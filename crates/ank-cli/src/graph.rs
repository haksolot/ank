//! The `graph` verb: the `blocked_by` DAG, in readable text (§4).
//!
//! The ordering of §5 already walks these edges to count what a task unblocks;
//! this makes the same structure visible to a reader instead of only to the
//! sort. Everything here is computed from the corpus at read time and stored
//! nowhere — a stored reverse edge is a second copy of `blocked_by` that can
//! disagree with the first.
//!
//! **The forest is a view, and it says so.** Only tasks inside the perimeter are
//! drawn, so a task whose blocker sits outside it would otherwise appear to be a
//! root, and "nothing is stopping this" is the one wrong answer a reader would
//! act on immediately. Those nodes carry the count of what was left out.

use crate::cli::{Invocation, Result};
use crate::config::Config;
use crate::context;
use crate::index::{Index, Row};
use crate::json::Obj;
use crate::repo::Repo;
use ank_contract::ExitCode;
use ank_core::{EntityId, EntityKind};
use std::collections::{HashMap, HashSet};
use std::io::Write;

pub fn run(inv: &Invocation, repo: &Repo, cfg: &Config, out: &mut dyn Write) -> Result<ExitCode> {
    let perimeter = context::perimeter(inv, repo)?;
    let shown = perimeter.as_deref().unwrap_or(".");

    let index = Index::open(&repo.ank)?;
    let all = index.all()?;
    let shorts = context::shorts_of(repo)?;

    // Sorted by identifier, like every other listing: a graph whose rows shuffle
    // between two runs is one nobody diffs.
    let mut nodes: Vec<&Row> = all
        .iter()
        .filter(|r| r.kind == EntityKind::Task)
        .filter(|r| context::in_perimeter(&r.scope, perimeter.as_deref()))
        .collect();
    nodes.sort_by_key(|r| r.id.to_string());

    let inside: HashSet<&EntityId> = nodes.iter().map(|r| &r.id).collect();
    let row_of: HashMap<&EntityId, &Row> = nodes.iter().map(|r| (&r.id, *r)).collect();

    // Reversed once: `blocked_by` points at what must finish first, and a reader
    // follows the other way round.
    let mut blocks: HashMap<&EntityId, Vec<&EntityId>> = HashMap::new();
    let mut roots: Vec<&EntityId> = Vec::new();
    // What a node's line says beyond its title: the blockers the perimeter
    // left out, and each one in a peer.
    let mut outside_count: HashMap<&EntityId, String> = HashMap::new();
    let peer_status = peer_statuses(repo, cfg, &nodes);
    for row in &nodes {
        let mut held_inside = 0usize;
        for blocker in &row.blocked_by {
            if let Some(b) = inside.get(blocker) {
                blocks.entry(*b).or_default().push(&row.id);
                held_inside += 1;
            }
        }
        let outside = row.blocked_by.len() - held_inside;
        let mut note = String::new();
        // Never silently a root. A task held up by something the perimeter
        // excludes is not free to start, and drawing it flush left would say
        // it is.
        if outside > 0 {
            note.push_str(&format!("  (+{outside} blocker(s) outside)"));
        }
        // A peer's blocker is drawn on its node, with its status as the peer's
        // corpus holds it (ADR-c23bef1cc93e): the peer's task is no node of
        // this forest, and an edge left undrawn would make the node a root.
        for b in &row.peer_blocked_by {
            let status = match peer_status.get(&b.peer) {
                Some(Ok(statuses)) => statuses
                    .get(&b.id)
                    .cloned()
                    .unwrap_or_else(|| format!("not held by peer '{}'", b.peer)),
                Some(Err(why)) => why.clone(),
                None => format!("peer '{}' is not declared", b.peer),
            };
            note.push_str(&format!("  (blocked by {b}, {status})"));
        }
        if !note.is_empty() {
            outside_count.insert(&row.id, note);
        }
        if held_inside == 0 {
            roots.push(&row.id);
        }
    }
    for children in blocks.values_mut() {
        children.sort_by_key(|i| i.to_string());
    }

    if inv.json() {
        return json(out, shown, &nodes, &shorts);
    }
    if inv.quiet() {
        return Ok(ExitCode::Ok);
    }

    // Names the perimeter it drew (§4). Without it an empty answer and an answer
    // about the wrong directory look the same.
    let _ = writeln!(out, "{shown}");
    if nodes.is_empty() {
        let _ = writeln!(out, "\nno task in this perimeter");
        return Ok(ExitCode::Ok);
    }
    let _ = writeln!(out);

    // Every node is cycle-safe: `check` reports a cycle as a fault, and `graph`
    // still has to draw the corpus that has one rather than hang on it. A cycle
    // also means no root, which would otherwise print a header and nothing
    // under it — so what has not been drawn is drawn flat at the end.
    let style = inv.style();
    // Read once for the whole forest, never per node: a task claimed by someone
    // reads the same here as it does under `context`, `find` and `scope`.
    let coord = context::coordination(&repo.corpus, &mut Vec::new())?;
    let mut drawn: HashSet<&EntityId> = HashSet::new();
    for root in &roots {
        draw(
            root,
            &blocks,
            &row_of,
            &shorts,
            &outside_count,
            &coord,
            "",
            None,
            &mut drawn,
            &mut Vec::new(),
            out,
            style,
        );
    }
    let stranded: Vec<&&Row> = nodes.iter().filter(|r| !drawn.contains(&r.id)).collect();
    if !stranded.is_empty() {
        let _ = writeln!(out, "\nin a cycle, so under no root:");
        for row in stranded {
            let _ = writeln!(
                out,
                "  {}",
                line(&row.id, &row_of, &shorts, &outside_count, &coord, style)
            );
        }
    }

    let _ = writeln!(
        out,
        "\n{} task(s), {} root(s) — indented under what blocks them",
        nodes.len(),
        roots.len()
    );
    Ok(ExitCode::Ok)
}

/// One node and everything it unblocks, depth first.
///
/// `path` is the chain currently being drawn, and it is what makes a cycle
/// terminate rather than recurse. `drawn` is every node already expanded
/// somewhere: a diamond is real in a DAG, so the node appears again where it
/// belongs, marked, and is not expanded twice.
///
/// **The prefix is derived from the parent's connector, never from a depth**
/// (§4). `last` says whether this node is the final child of its parent —
/// `None` for a root, which is drawn flush left with no connector at all. A
/// node under `├──` continues as `│  ` because its parent still has siblings
/// below it; a node under `└──` continues as blanks because nothing does. A
/// depth counter has no way to tell those apart, and on a corpus with any
/// branching at all that difference is most of what makes the drawing readable.
#[allow(clippy::too_many_arguments)]
fn draw<'a>(
    id: &'a EntityId,
    blocks: &HashMap<&'a EntityId, Vec<&'a EntityId>>,
    row_of: &HashMap<&'a EntityId, &'a Row>,
    shorts: &HashMap<EntityId, String>,
    outside: &HashMap<&'a EntityId, String>,
    coord: &HashMap<EntityId, context::Coordination>,
    prefix: &str,
    last: Option<bool>,
    drawn: &mut HashSet<&'a EntityId>,
    path: &mut Vec<&'a EntityId>,
    out: &mut dyn Write,
    style: crate::style::Style,
) {
    use crate::style::glyph;
    let connector = match last {
        None => "",
        Some(true) => glyph::LAST,
        Some(false) => glyph::BRANCH,
    };
    if path.contains(&id) {
        let _ = writeln!(
            out,
            "{prefix}{connector}{} (cycle)",
            line(id, row_of, shorts, outside, coord, style)
        );
        return;
    }
    let repeat = drawn.contains(&id);
    let mark = if repeat { " (above)" } else { "" };
    let _ = writeln!(
        out,
        "{prefix}{connector}{}{mark}",
        line(id, row_of, shorts, outside, coord, style)
    );
    if repeat {
        return;
    }
    drawn.insert(id);
    path.push(id);
    let children: Vec<&&EntityId> = blocks.get(id).into_iter().flatten().collect();
    let child_prefix = format!(
        "{prefix}{}",
        match last {
            None => "",
            Some(true) => glyph::CLEAR,
            Some(false) => glyph::GUTTER,
        }
    );
    for (i, child) in children.iter().enumerate() {
        draw(
            child,
            blocks,
            row_of,
            shorts,
            outside,
            coord,
            &child_prefix,
            Some(i + 1 == children.len()),
            drawn,
            path,
            out,
            style,
        );
    }
    path.pop();
}

fn line<'a>(
    id: &'a EntityId,
    row_of: &HashMap<&'a EntityId, &'a Row>,
    shorts: &HashMap<EntityId, String>,
    outside: &HashMap<&'a EntityId, String>,
    coord: &HashMap<EntityId, context::Coordination>,
    style: crate::style::Style,
) -> String {
    let short = shorts.get(id).cloned().unwrap_or_else(|| id.to_string());
    let Some(row) = row_of.get(id) else {
        return style.id(&short);
    };
    let mut s = format!(
        "{}  {} {}",
        style.id(&short),
        style.status(&context::marker_for(
            &row.status,
            context::coordination_of(coord, id)
        )),
        row.title
    );
    if let Some(note) = outside.get(id) {
        s.push_str(note);
    }
    s
}

/// The status of every task in each peer the nodes' edges name, or why that
/// peer could not be read, keyed by the peer's name. A peer no edge names is
/// never opened.
///
/// **Read and never written** ([`crate::repo::Peer::statuses`]).
fn peer_statuses(
    repo: &Repo,
    cfg: &Config,
    nodes: &[&Row],
) -> HashMap<String, std::result::Result<HashMap<EntityId, String>, String>> {
    let mut out = HashMap::new();
    for b in nodes.iter().flat_map(|r| &r.peer_blocked_by) {
        if out.contains_key(&b.peer) || !cfg.peers.contains_key(&b.peer) {
            continue;
        }
        let read = crate::repo::open_peer(repo, cfg, &b.peer)
            .and_then(|peer| peer.statuses())
            .map_err(|e| format!("unread: {}", e.message));
        out.insert(b.peer.clone(), read);
    }
    out
}

/// The raw edges (§4): every `blocked_by` relation of an in-perimeter task,
/// including the ones pointing outside it. The text draws a forest, which is a
/// reading of the graph; this is the graph.
fn json(
    out: &mut dyn Write,
    path: &str,
    nodes: &[&Row],
    shorts: &HashMap<EntityId, String>,
) -> Result<ExitCode> {
    let tasks: Vec<String> = nodes
        .iter()
        .map(|r| {
            let short = shorts
                .get(&r.id)
                .cloned()
                .unwrap_or_else(|| r.id.to_string());
            Obj::new()
                .str("id", &r.id.to_string())
                .str("short", &short)
                .str("status", &r.status.to_string())
                .str("title", &r.title)
                .finish()
        })
        .collect();
    let mut edges: Vec<String> = Vec::new();
    for row in nodes {
        let local = row.blocked_by.iter().map(|b| b.to_string());
        let peer = row.peer_blocked_by.iter().map(|b| b.to_string());
        for blocker in local.chain(peer) {
            edges.push(
                Obj::new()
                    .str("task", &row.id.to_string())
                    .str("blocked_by", &blocker)
                    .finish(),
            );
        }
    }
    let doc = Obj::document()
        .str("path", path)
        .array("tasks", tasks)
        .array("edges", edges)
        .finish();
    let _ = writeln!(out, "{doc}");
    Ok(ExitCode::Ok)
}
