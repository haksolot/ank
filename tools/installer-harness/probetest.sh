#!/bin/sh
# The workflow's own probe.ps1, extracted and run here against a staged release,
# so what it captures is measured rather than assumed.
set -eu
cd "$(dirname "$0")"

PWSH=/tmp/claude-1000/-home-haksolot-Projects-ank/1ce9a43d-5095-4fde-9ec4-5dd445b54ac9/scratchpad/pwsh/pwsh

python3 .probeextract.py
sh .stage-win.sh 2>/dev/null

port=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')
python3 -m http.server "$port" --bind 127.0.0.1 --directory .stage > /dev/null 2>&1 &
server=$!
trap 'kill "$server" 2>/dev/null || true; rm -f probe.ps1' EXIT INT TERM

i=0
while [ "$i" -lt 50 ]; do
  if python3 -c "import socket,sys;s=socket.socket();sys.exit(s.connect_ex(('127.0.0.1',$port)))" 2>/dev/null; then
    break
  fi
  sleep 0.1
  i=$((i + 1))
done

rm -rf .probe-bin .probe-home out-probe.txt out-probe.txt.transcript code-probe.txt
mkdir -p .probe-home

# The environment probe.ps1 is given by Invoke-Case, plus the two a Windows
# console already has.
STAGED_VERSION=9.9.9-preview \
ANK_BASE_URL="http://127.0.0.1:$port" \
PROCESSOR_ARCHITECTURE=AMD64 \
LOCALAPPDATA="$PWD/.probe-home" \
python3 .ptyrun.py "$PWSH" -NoProfile -File probe.ps1 \
  -Answer n -Dir "$PWD/.probe-bin" \
  -OutFile "$PWD/out-probe.txt" -CodeFile "$PWD/code-probe.txt" > /dev/null

fail=0
say() { printf '%s\n' "$1"; }

if [ ! -f out-probe.txt ]; then
  say "FAIL the probe wrote no output file"
  exit 1
fi

say "exit code recorded: $(cat code-probe.txt 2>/dev/null || echo none)"

# The assertions the workflow makes of $run.Out, asked of what the probe
# actually captured.
check() {
  if grep -qF -e "$2" out-probe.txt; then
    say "ok   $1"
  else
    say "FAIL $1"
    fail=1
  fi
}

check "the mark reaches the probe"            "██"
check "a leg row survives whole"              "   ██      ██      ██"
check "the base row survives whole"           "     ██████████████"
check "the skills question"                   "Install them now?"
check "the adoption question"                 "Print the three prompts"
check "the checksum line"                     "checksum ok"
check "the probe reports its switch"          "probe: NoWelcome=False"

rm -rf .probe-bin .probe-home out-probe.txt out-probe.txt.transcript code-probe.txt
exit $fail
