#!/usr/bin/env node
// Refuses direct access to .ank/ and names the command to run instead.
//
// ADR-01b6dd05f0db states the rule where `ank context` serves it to every agent
// at the top of every session. This is what actually refuses, because a
// constraint an agent can read is a constraint an agent can forget, and section
// 5 was never going to be enforced by good intentions.
//
// It constrains the harness and not the tool. ADR-6b3f19e08a24 forbids relying
// on the CLI to refuse a write, and nothing here changes that: the freezes stay
// anchored by hash, a human with an editor keeps every power they had, and
// `ank check` remains what notices.
//
// Exit 0 allows, exit 2 denies and hands stderr back to the agent (any other
// code is a broken hook, not a refusal). Node rather than a shell script: one
// file, a real JSON parser, and identical behaviour on Windows, macOS and Linux.

import { resolve, relative, isAbsolute, sep } from "node:path";

const HINTS = {
  Read: "ank show <id>",
  Glob: "ank find --status open",
  Grep: "ank find <query>",
  Write: "ank new task --title <t> --scope <glob>",
  Edit: "ank claim <id>, then ank log / ank done",
};

async function stdin() {
  let text = "";
  for await (const chunk of process.stdin) text += chunk;
  return text;
}

/// True when the argument names something inside `.ank/`.
///
/// Deliberately narrow. A repo-wide `Grep` with no `path` is allowed through:
/// it merely *might* match a file under `.ank/`, and a hook that blocks the
/// whole repository over a maybe is a hook people switch off. What is refused
/// is reaching for `.ank/` on purpose.
function targetsAnk(value, root) {
  if (typeof value !== "string" || value === "") return false;
  const normalised = value.replace(/\\/g, "/");

  // A relative reference to the directory, however it is spelled.
  if (/(^|\/)\.ank(\/|$)/.test(normalised)) return true;

  // An absolute path, resolved against the project so that `..` cannot walk
  // back in unnoticed.
  if (isAbsolute(value)) {
    const rel = relative(resolve(root, ".ank"), resolve(value));
    return rel === "" || (!rel.startsWith("..") && !isAbsolute(rel));
  }
  return false;
}

const raw = await stdin();
let input;
try {
  input = JSON.parse(raw);
} catch {
  // A payload this hook cannot read is not grounds to block the session.
  process.exit(0);
}

const tool = input.tool_name;
if (!(tool in HINTS)) process.exit(0);

const root = process.env.CLAUDE_PROJECT_DIR ?? process.cwd();
const args = input.tool_input ?? {};
const candidates = [args.file_path, args.path, args.pattern, args.notebook_path];

if (!candidates.some((c) => targetsAnk(c, root))) process.exit(0);

// The exact command to run next, never generic help: one well-designed refusal
// costs less than three blind attempts.
process.stderr.write(
  `.ank/ is opaque, like .git/ (ADR-01b6dd05f0db): ${tool} is refused there.\n` +
    `The CLI knows what the files do not -- the context budget, the frozen\n` +
    `criterion, who holds which claim.\n` +
    `  -> ${HINTS[tool]}\n` +
    `     ank context   what binds here, and what is claimable\n`,
);
process.exit(2);
