#!/usr/bin/env bash
# What the recording plays. Written rather than typed, so a retake is a re-run
# and not a performance: the recording is a function of this file and of the
# demo repository, both of which are rebuilt from scripts.
set -uo pipefail
. "$HOME/.cargo/env" 2>/dev/null || true

export PATH="$HOME/ank/target/release:$PATH"
cd "$HOME/demo"

ROOT=$(ank find --status open 2>/dev/null |
  sed -n 's/^  \(TASK-[0-9a-f]*\).*does not know.*/\1/p' | head -1)

P=$'\033[38;5;183mdepot\033[0m \033[38;5;245m$\033[0m '

# Types a command the way a reader reads one, then runs it.
say() {
  printf '%b' "$P"
  local i
  for ((i = 0; i < ${#1}; i++)); do
    printf '%s' "${1:i:1}"
    sleep 0.028
  done
  printf '\n'
  sleep 0.35
  eval "$1"
}

pause() { sleep "$1"; }

clear
pause 0.8

# 1. Orientation: what governs this file, and what is claimable inside it.
say "ank context src/deploy.rs"
pause 4.5

# 2. The claim freezes the criterion.
say "ank claim $ROOT"
pause 1.6

# 3. The criterion, and the constraint in full. This is the beat the whole
#    recording exists for.
say "ank context"
pause 9.5

# 4. The work. Written at once because an agent writes a file at once, and
#    what matters is that the message it produces is the one the constraint
#    asked for.
printf '%b%s\n' "$P" "cat > src/deploy.rs <<'RS'"
cat > src/deploy.rs <<'RS'
//! Sends a build to an environment.

const KNOWN: [&str; 3] = ["staging", "canary", "production"];

pub fn deploy(env: &str) -> Result<(), String> {
    if !KNOWN.contains(&env) {
        return Err(format!(
            "unknown environment '{env}', try: depot deploy {}",
            KNOWN.join(" | depot deploy ")
        ));
    }
    println!("deploying to {env}");
    Ok(())
}
RS
cat <<'RS'
//! Sends a build to an environment.

const KNOWN: [&str; 3] = ["staging", "canary", "production"];

pub fn deploy(env: &str) -> Result<(), String> {
    if !KNOWN.contains(&env) {
        return Err(format!(
            "unknown environment '{env}', try: depot deploy {}",
            KNOWN.join(" | depot deploy ")
        ));
    }
    println!("deploying to {env}");
    Ok(())
}
RS
printf 'RS\n'
pause 3.5

cat > tests/deploy.rs <<'RS'
#[test]
fn a_known_environment_is_deployed_to() {
    assert!(depot::deploy::deploy("staging").is_ok());
}

#[test]
fn an_unknown_one_is_refused_with_the_command_to_run() {
    let err = depot::deploy::deploy("stagin").unwrap_err();
    assert!(err.contains("depot deploy staging"), "{err}");
}
RS

# The constraint said a refusal names the command that fixes it. Here it is.
say "cargo run -q -- stagin"
pause 4.0

# 5. done runs the declared verifier itself and writes the proof.
say "ank done"
pause 5.0
