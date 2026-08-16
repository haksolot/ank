#!/usr/bin/env bash
# What the recording plays. Written rather than typed, so a retake is a re-run
# and not a performance: the recording is a function of this file and of the
# demo repository, both of which are rebuilt from scripts.
#
# Colours are the eight ANSI ones and never the 256-colour cube, so the theme
# passed to agg governs every pixel. A 38;5;183 here would come out of xterm's
# palette and be the one thing on screen Catppuccin does not reach.
set -uo pipefail
. "$HOME/.cargo/env" 2>/dev/null || true

export PATH="$HOME/ank/target/release:$PATH"
cd "$HOME/demo"

ROOT=$(ank find --status open 2>/dev/null |
  sed -n 's/^  \(TASK-[0-9a-f]*\).*does not know.*/\1/p' | head -1)

P=$'\033[35mdepot\033[0m \033[34m$\033[0m '

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
pause 3.5

# 2. Which of them is even takeable. blocked_by is an order, not a list, and
#    the tree is the only place that reads as one.
say "ank graph"
pause 4.5

# 3. The claim freezes the criterion.
say "ank claim $ROOT"
pause 1.6

# 4. The criterion, and the constraint in full. This is the beat the whole
#    recording exists for.
say "ank context"
pause 8.5

# 5. The work. Written at once because an agent writes a file at once, and what
#    matters is that the message it produces is the one the constraint asked
#    for. `tee` rather than a redirect and a second copy: one text on screen and
#    on disk, so they cannot drift.
printf '%b%s\n' "$P" "cat > src/deploy.rs <<'RS'"
tee src/deploy.rs <<'RS'
//! Sends a build to an environment.

const KNOWN: [&str; 3] = ["staging", "canary", "production"];

pub fn deploy(env: &str) -> Result<(), String> {
    if !KNOWN.contains(&env) {
        let head: String = env.chars().take(4).collect();
        return Err(match KNOWN.iter().find(|k| k.starts_with(&head)) {
            Some(k) => format!("unknown environment '{env}', try: depot deploy {k}"),
            None => format!("unknown environment '{env}', known: {}", KNOWN.join(", ")),
        });
    }
    println!("deploying to {env}");
    Ok(())
}
RS
printf 'RS\n'
pause 3.0

cat > tests/deploy.rs <<'RS'
#[test]
fn a_known_environment_is_deployed_to() {
    assert!(depot::deploy::deploy("staging").is_ok());
}

#[test]
fn an_unknown_one_is_refused_with_the_command_to_run() {
    let err = depot::deploy::deploy("stagin").unwrap_err();
    assert_eq!(err, "unknown environment 'stagin', try: depot deploy staging");
}
RS

# The constraint promised "try: depot deploy staging". Here it is, to the byte.
say "cargo run -q -- stagin"
pause 3.5

# 6. done runs the declared verifier itself and writes the proof.
say "ank done"
pause 4.5
