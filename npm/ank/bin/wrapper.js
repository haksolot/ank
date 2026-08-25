"use strict";

// One wrapper, two names.
//
// This wrapper resolves a binary. It never downloads one.
//
// The driving case is the corporate workstation: a firewall that blocks
// downloading a bare executable usually lets the npm registry through. A
// postinstall script that fetched the binary would therefore die behind the
// very firewall this package exists to cross. The binaries travel inside the
// platform packages instead, and npm installs exactly one of them by matching
// `os` and `cpu` through optionalDependencies.
//
// `ank` and `ank-mcp` come out of one build and travel in one platform package
// out of one package, so they resolve by one route. Two copies of this file,
// one per bin, is the arrangement where the protocol surface quietly stops
// resolving the way the CLI does and nothing turns red: the resolution, the
// diagnostics and the exit-code passthrough are written once and `run` is
// given the name to look for.

const { spawnSync } = require("child_process");

const PACKAGES = {
  "linux x64": "@haksolot/ank-linux-x64-musl",
  "darwin arm64": "@haksolot/ank-darwin-arm64",
  "win32 x64": "@haksolot/ank-win32-x64",
};

// Code 9 is the environment, and it is the right one for every failure in this
// file: none of them is a failure of the caller's work, which is exactly what
// section 4 reserves 9 for. Each one names the command that fixes it.
function fail(lines) {
  for (const line of lines) {
    console.error(line);
  }
  process.exit(9);
}

// run(name) -- spawn `name` out of the platform package, and be its exit code.
//
// `name` is the bin npm wrote a shim for, and it is also the file inside the
// platform package: the assembly script copies both executables under the
// names they were built with, so there is nothing here to map.
function run(name) {
  const platform = process.platform + " " + process.arch;
  const pkg = PACKAGES[platform];
  const exe = process.platform === "win32" ? name + ".exe" : name;

  if (!pkg) {
    fail([
      name + ": no prebuilt binary for " + platform,
      "  -> cargo install --git https://github.com/haksolot/ank ank-cli",
    ]);
  }

  let binary;
  try {
    binary = require.resolve(pkg + "/bin/" + exe);
  } catch (e) {
    fail([
      name + ": " + pkg + " is not installed",
      "  -> npm install @haksolot/ank",
      "  -> optional dependencies carry the binary, so --no-optional removes it",
    ]);
  }

  // The exit code is the interface. Section 4 gives 4, 6, 8 and 9 distinct
  // meanings that an agent branches on, so a wrapper collapsing them into 0 and
  // 1 would break every caller that reads them.
  //
  // `stdio: inherit` is also what makes the protocol surface work at all:
  // ank-mcp speaks JSON-RPC over stdin and stdout, so a wrapper that captured
  // either would leave the client waiting forever.
  const child = spawnSync(binary, process.argv.slice(2), { stdio: "inherit" });
  if (child.error) {
    fail([name + ": " + child.error.message, "  -> " + binary]);
  }
  // A process killed by a signal carries no code of its own; 9 says
  // environment, which is what a signal from outside is.
  process.exit(child.status === null ? 9 : child.status);
}

module.exports = { run };
