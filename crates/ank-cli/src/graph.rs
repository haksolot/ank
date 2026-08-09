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
use crate::context;
use crate::index::{Index, Row};
use crate::repo::Repo;
use ank_core::{EntityId, EntityKind};
use std::collections::{HashMap, HashSet};
use std::io::Write;

pub fn run(inv: &Invocation, repo: &Repo, out: &mut dyn Write) -> Result<i32> {
    let perimeter = context::perimeter(inv, repo)?;
    let shown = perimeter.as_deref().unwrap_or(".");

    let index = Index::open(&repo.ank)?;
    let all = index.all()?;
    let ids: Vec<EntityId> = all.iter().map(|r| r.id.clone()).collect();
    let shorts = context::short_ids(&ids);

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
    let mut outside_count: HashMap<&EntityId, usize> = HashMap::new();
    for row in &nodes {
        let mut held_inside = 0usize;
        for blocker in &row.blocked_by {
            if let Some(b) = inside.get(blocker) {
                blocks.entry(*b).or_default().push(&row.id);
                held_inside += 1;
            }
        }
        let outside = row.blocked_by.len() - held_inside;
        if outside > 0 {
            outside_count.insert(&row.id, outside);
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
        return Ok(0);
    }

    // Names the perimeter it drew (§4). Without it an empty answer and an answer
    // about the wrong directory look the same.
    let _ = writeln!(out, "{shown}");
    if nodes.is_empty() {
        let _ = writeln!(out, "\nno task in this perimeter");
        return Ok(0);
    }
    let _ = writeln!(out);

    // Every node is cycle-safe: `check` reports a cycle as a fault, and `graph`
    // still has to draw the corpus that has one rather than hang on it. A cycle
    // also means no root, which would otherwise print a header and nothing
    // under it — so what has not been drawn is drawn flat at the end.
    let style = inv.style();
    // Read once for the whole forest, never per node: a task claimed by someone
    // reads the same here as it does under `context`, `find` and `scope`.
    let coord = context::coordination(&repo.root, &mut Vec::new())?;
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
    Ok(0)
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
    outside: &HashMap<&'a EntityId, usize>,
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
    outside: &HashMap<&'a EntityId, usize>,
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
    // Never silently a root. A task held up by something the perimeter excludes
    // is not free to start, and drawing it flush left would say it is.
    if let Some(n) = outside.get(id) {
        s.push_str(&format!("  (+{n} blocker(s) outside)"));
    }
    s
}

/// The raw edges (§4): every `blocked_by` relation of an in-perimeter task,
/// including the ones pointing outside it. The text draws a forest, which is a
/// reading of the graph; this is the graph.
fn json(
    out: &mut dyn Write,
    path: &str,
    nodes: &[&Row],
    shorts: &HashMap<EntityId, String>,
) -> Result<i32> {
    let tasks: Vec<String> = nodes
        .iter()
        .map(|r| {
            format!(
                "{{\"id\":\"{}\",\"short\":\"{}\",\"status\":\"{}\",\"title\":{}}}",
                r.id,
                shorts
                    .get(&r.id)
                    .cloned()
                    .unwrap_or_else(|| r.id.to_string()),
                r.status,
                crate::commands::json_string(&r.title)
            )
        })
        .collect();
    let mut edges: Vec<String> = Vec::new();
    for row in nodes {
        for blocker in &row.blocked_by {
            edges.push(format!(
                "{{\"task\":\"{}\",\"blocked_by\":\"{blocker}\"}}",
                row.id
            ));
        }
    }
    let _ = writeln!(
        out,
        "{{\"path\":{},\"tasks\":[{}],\"edges\":[{}]}}",
        crate::commands::json_string(path),
        tasks.join(","),
        edges.join(",")
    );
    Ok(0)
}
