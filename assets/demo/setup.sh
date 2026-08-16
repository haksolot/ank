#!/usr/bin/env bash
# Builds the demo repository the recording is made in. Throwaway: it is rebuilt
# from this script rather than kept, so the recording can be retaken from a
# known state instead of from whatever the last take left behind.
set -euo pipefail

ANK="$HOME/ank/target/release/ank"
DEMO="$HOME/demo"

rm -rf "$DEMO"
mkdir -p "$DEMO/src"
cd "$DEMO"

git init -q -b main
git config user.name "depot"
git config user.email "depot@example.invalid"

# Signed with SSH rather than OpenPGP, because gnupg is not installed here and
# installing it needs a password this script does not have. git has signed with
# ssh keys since 2.34, `.ank/allowed_signers` is already that format's file
# name, and a ratification commit is a ratification commit either way.
rm -f "$HOME/demo-sign" "$HOME/demo-sign.pub"
ssh-keygen -q -t ed25519 -N "" -C "depot demo" -f "$HOME/demo-sign"
git config gpg.format ssh
git config user.signingkey "$HOME/demo-sign.pub"
git config commit.gpgsign true
git config gpg.ssh.allowedSignersFile .ank/allowed_signers

cat > src/deploy.rs <<'RS'
//! Sends a build to an environment.

pub fn deploy(env: &str) -> Result<(), String> {
    // TODO: unknown environments are accepted and fail later, in the cluster.
    println!("deploying to {env}");
    Ok(())
}
RS

cat > src/config.rs <<'RS'
//! Reads depot.toml.

pub fn load(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}
RS

cat > src/lib.rs <<'RS'
pub mod config;
pub mod deploy;
RS

cat > src/main.rs <<'RS'
fn main() {
    let env = std::env::args().nth(1).unwrap_or_default();
    if let Err(e) = depot::deploy::deploy(&env) {
        eprintln!("error: {e}");
        std::process::exit(2);
    }
}
RS

cat > Cargo.toml <<'TOML'
[package]
name = "depot"
version = "0.1.0"
edition = "2021"

[lib]
path = "src/lib.rs"

[[bin]]
name = "depot"
path = "src/main.rs"
TOML

# A real verifier, so `done` runs something rather than being told a result.
mkdir -p tests
cat > tests/deploy.rs <<'RS'
#[test]
fn a_known_environment_is_deployed_to() {
    assert!(depot::deploy::deploy("staging").is_ok());
}
RS

cat > README.md <<'MD'
# depot

Sends a build to an environment.
MD

printf 'target\n' > .gitignore

git add -A
git commit -qm "depot: the two modules it starts from"

"$ANK" init >/dev/null
printf '%s %s\n' "depot@example.invalid" "$(cat "$HOME/demo-sign.pub")" \
  > .ank/allowed_signers
"$ANK" config default_branch main >/dev/null
"$ANK" config verifiers.tests.run "cargo test --quiet" >/dev/null

# ---------------------------------------------------------------- the corpus

SPEC=$("$ANK" new spec --title "What depot is, and what it refuses to be" \
  --scope "src/**" \
  --body "depot sends a build to an environment and does nothing else. It is
reached from a terminal by a person under time pressure, usually while something
is already wrong, which is the whole reason the rules below are about what
happens when it fails rather than about what happens when it works." \
  --json | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')

ADR1=$("$ANK" new adr --title "An error names the command that fixes it" \
  --scope "src/**" \
  --constraint "Every refusal names the exact command to run next. \"unknown environment\" sends the reader hunting; \"unknown environment 'stagin', try: depot deploy staging\" ends the search. A message that only states what went wrong is a message the reader has to translate into an action, and they are translating it at the worst possible moment." \
  --body "The rule is about the reader's next thirty seconds, not about tone.

Rejected: a link to the documentation. It is a second hop, it goes stale on its
own schedule, and the reader is in a terminal." \
  --json | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')

ADR2=$("$ANK" new adr --title "Configuration is read once, at startup" \
  --scope "src/config.rs" \
  --constraint "depot.toml is read once, before the first request leaves. A reload halfway through a deployment means two halves of one deployment ran under two configurations, and nothing in the logs says which." \
  --json | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')

# Ratified, because a proposed decision is served as non-binding and the whole
# point of the recording is a constraint that binds.
git add -A
git commit -qm "depot: the decisions it works under"
for id in "$SPEC" "$ADR1" "$ADR2"; do
  "$ANK" accept "$id" >/dev/null
done

# ---------------------------------------------------------------- the work

# --verify names which declared verifier `done` runs. Without it `done` asks for
# a --proof instead, and the recording would show a number being typed rather
# than a check being run, which is the opposite of the point.
ROOT=$("$ANK" new task --title "deploy refuses an environment it does not know" \
  --scope "src/deploy.rs" --verify tests \
  --criteria "deploy returns an error for an environment that is not one of the three it knows, and the message names the command to run instead. The three known environments are still deployed to. cargo test passes." \
  --body "Today an unknown environment is accepted and fails in the cluster,
minutes later, with a message from the cluster rather than from depot." \
  --json | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')

B=$("$ANK" new task --title "The refusal lists the environments depot knows" \
  --scope "src/deploy.rs" --blocked-by "$ROOT" \
  --criteria "The refusal names the three known environments as well as the command, so a reader who mistyped and a reader who guessed both get an answer." \
  --json | sed -n 's/.*"id":"\([^"]*\)".*/\1/p')

"$ANK" new task --title "depot.toml is validated before the first request" \
  --scope "src/config.rs" --blocked-by "$ROOT" \
  --criteria "load rejects a depot.toml missing a required key, naming the key and the command that writes it." >/dev/null

"$ANK" new task --title "The refusal is tested through the binary" \
  --scope "tests/**" --blocked-by "$B" \
  --criteria "A test invokes the built binary with an unknown environment and asserts the exit code and the message, rather than calling deploy directly." >/dev/null

git add -A
git commit -qm "depot: the plan"

# Warmed, so the `done` in the recording runs the tests instead of also
# compiling the world for the first time.
( . "$HOME/.cargo/env" 2>/dev/null || true; cargo test --quiet >/dev/null 2>&1 || true )

echo "ROOT=$ROOT"
echo
"$ANK" graph
echo
"$ANK" context src/deploy.rs
