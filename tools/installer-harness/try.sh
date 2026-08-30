#!/bin/sh
# A whole install, in your own terminal, off a release staged on this machine.
# Nothing leaves the box and nothing lands outside the throwaway HOME below,
# which is removed on the way out.
#
#   sh .try.sh              the proposed install.sh
#   sh .try.sh --old        the one on main today
#   sh .try.sh --ascii      the same, under a locale that is not UTF-8
#   sh .try.sh --no-color   the same, with NO_COLOR set
set -eu

cd "$(dirname "$0")"

script=install.sh
env_pre=""
case "${1:-}" in
  --old) script=.orig-install.sh ;;
  --ascii) env_pre="LANG=C LC_ALL=C" ;;
  --no-color) env_pre="NO_COLOR=1" ;;
esac

[ -f .orig-install.sh ] || git show HEAD:install.sh > .orig-install.sh
sh .stage.sh 2>/dev/null

home=$(mktemp -d)
port=$(python3 -c 'import socket;s=socket.socket();s.bind(("127.0.0.1",0));print(s.getsockname()[1]);s.close()')

python3 -m http.server "$port" --bind 127.0.0.1 --directory .stage > /dev/null 2>&1 &
server=$!
trap 'kill "$server" 2>/dev/null || true; rm -rf "$home"' EXIT INT TERM

# The server needs to be answering before the installer asks it anything.
i=0
while [ "$i" -lt 50 ]; do
  if python3 -c "import socket,sys;s=socket.socket();sys.exit(s.connect_ex(('127.0.0.1',$port)))" 2>/dev/null; then
    break
  fi
  sleep 0.1
  i=$((i + 1))
done

printf '\n'
# shellcheck disable=SC2086
env $env_pre \
  HOME="$home" \
  ANK_BASE_URL="http://127.0.0.1:$port" \
  sh "$script" --version 9.9.9-preview || true
printf '\n'
