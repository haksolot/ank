// The viewer, read by something other than a browser (TASK-34d27790dba9).
//
//   node viewer/selftest.mjs [<repo> ...]
//
// The page's reading logic lives between two markers in `index.html` and knows
// nothing about the DOM. This slices that region out, imports it, and runs it
// against a real repository through a filesystem shim with the same shape the
// File System Access API is given in the page -- so what runs here is the code
// that runs there, not a copy of it.
//
// Then it does the only check worth making: **the viewer must agree with the
// CLI**. `ank` is the reference implementation of the format; a third-party
// reader that disagrees with it is wrong by definition, and the disagreement is
// exactly what a golden inside the page could never catch. The counts come from
// `ank find --json` and the coordination of every claimed or finished task from
// `ank show <id> --json`.
//
// No dependency, no build step, nothing written anywhere but the system
// temporary directory. `ank check` is deliberately never called: it prunes
// refs, so it writes, and a test must not.

import { readFile, writeFile, readdir, stat } from 'node:fs/promises';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { dirname, join, resolve } from 'node:path';
import { tmpdir } from 'node:os';

const run = promisify(execFile);
const here = dirname(fileURLToPath(import.meta.url));
const ANK = process.env.ANK_BIN || 'ank';

const BEGIN = '// ---- ank-viewer:core:begin ----';
const END = '// ---- ank-viewer:core:end ----';

async function loadCore() {
  const html = await readFile(join(here, 'index.html'), 'utf8');
  const from = html.indexOf(BEGIN);
  const to = html.indexOf(END);
  if (from === -1 || to === -1) {
    throw new Error('index.html no longer carries the ank-viewer:core markers');
  }
  const source = html.slice(from + BEGIN.length, to);
  const path = join(tmpdir(), `ank-viewer-core-${process.pid}.mjs`);
  await writeFile(path, source);
  return import(pathToFileURL(path).href);
}

/** The shim. Same three methods `browserFs` gives the page, same meanings. */
function nodeFs(root) {
  const at = (parts) => join(root, ...parts);
  return {
    async isDir(parts) {
      try { return (await stat(at(parts))).isDirectory(); } catch { return false; }
    },
    async list(parts) {
      try { return (await readdir(at(parts))).sort(); } catch { return []; }
    },
    async bytes(parts) {
      try { return new Uint8Array(await readFile(at(parts))); } catch { return null; }
    },
  };
}

const ank = async (repo, args) => {
  const { stdout } = await run(ANK, [...args, '--json', '--repo', repo], {
    maxBuffer: 64 * 1024 * 1024,
  });
  return JSON.parse(stdout);
};

let failures = 0;
const check = (ok, what, detail) => {
  if (ok) {
    console.log(`  ok    ${what}`);
  } else {
    failures++;
    console.log(`  FAIL  ${what}${detail ? `\n        ${detail}` : ''}`);
  }
};

async function verify(core, repo) {
  console.log(`\n${repo}`);
  const corpus = await core.readCorpus(nodeFs(repo));

  const seen = new Map();
  for (const e of corpus.entities) seen.set(e.type, (seen.get(e.type) || 0) + 1);

  for (const kind of ['task', 'adr', 'spec']) {
    const said = await ank(repo, ['find', '--type', kind]);
    check(
      said.total === (seen.get(kind) || 0),
      `${kind} count agrees with ank find (${said.total})`,
      `ank says ${said.total}, the viewer read ${seen.get(kind) || 0}`,
    );
  }

  // Every entity the CLI can resolve, the viewer parsed with the same title.
  const tasks = await ank(repo, ['find', '--type', 'task']);
  const byId = new Map(corpus.entities.map((e) => [e.id, e]));
  const wrongTitle = tasks.results.filter((r) => byId.get(r.id)?.title !== r.title);
  check(
    wrongTitle.length === 0,
    'every listed title parses to the same string',
    wrongTitle.slice(0, 3).map((r) => `${r.id}: ank ${JSON.stringify(r.title)} vs viewer ${JSON.stringify(byId.get(r.id)?.title)}`).join('\n        '),
  );

  const blockedDisagrees = tasks.results.filter((r) => {
    const e = byId.get(r.id);
    if (!e) return true;
    const g = core.graphOf(corpus.entities);
    return r.state === 'blocked' && !g.isBlocked(e);
  });
  check(blockedDisagrees.length === 0, 'derived blocking agrees with ank find --json');

  // The half this task exists for. Every claim the viewer read out of a ref --
  // loose or packed, loose object or packed object with a delta chain -- has to
  // say what `ank show` says.
  check(corpus.refs > 0, `refs under refs/ank/ were found (${corpus.refs})`);
  let compared = 0;
  for (const [id, c] of corpus.coordination) {
    if (!byId.has(id)) continue;
    const shown = await ank(repo, ['show', id]);
    const expected =
      c.state === 'claimed'
        ? `claimed by ${c.holder}`
        : c.state === 'completed'
          ? `finished at ${c.commit}`
          : null;
    if (expected === null) continue;
    compared++;
    if (shown.coordination !== expected) {
      check(false, `${id} coordination`, `ank ${JSON.stringify(shown.coordination)} vs viewer ${JSON.stringify(expected)}`);
    }
  }
  check(true, `${compared} coordination record(s) read out of git and agreed with ank show`);

  // **The packfile path, proven rather than inferred.** In this repository only
  // two of the sixty-four ank objects are loose, and both happen to be claims --
  // so the checks above can pass while nothing has ever come out of a `.pack`.
  // The detached proofs are the other sixty-two, and each one is an object
  // resolved through an `.idx`, inflated out of the pack, and very often the
  // end of a delta chain, since these records are near-identical to one another
  // and that is exactly what git deltas.
  let packedCompared = 0;
  for (const [id, count] of corpus.detached) {
    if (!byId.has(id)) continue;
    const shown = await ank(repo, ['show', id]);
    packedCompared++;
    if (shown.detached_proofs.length !== count) {
      check(false, `${id} detached proofs`, `ank ${shown.detached_proofs.length} vs viewer ${count}`);
    }
  }
  check(true, `${packedCompared} proof record(s) agreed with ank show`);

  check(
    corpus.problems.length === 0,
    'nothing in the repository was left unread',
    corpus.problems.slice(0, 5).join('\n        '),
  );
}

const repos = process.argv.slice(2);
if (!repos.length) repos.push(resolve(here, '..'));

const core = await loadCore();
for (const repo of repos) await verify(core, resolve(repo));

console.log(failures === 0 ? '\nall agreed' : `\n${failures} disagreement(s)`);
process.exit(failures === 0 ? 0 : 1);
