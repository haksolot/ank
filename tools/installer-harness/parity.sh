#!/bin/sh
# What a pipe reads, before and after. ADR-5fbd99bf6fd5 promises a caller with
# no terminal the installer it sees today, to the byte, so the two runs are
# compared as bytes and their exit codes with them.
fail=0
for args in "--help" "--version 0.0.0-nope --dir $PWD/.x1" "--no-welcome --help"; do
  sh .orig-install.sh $args > .a.log 2>&1
  ca=$?
  sh install.sh $args > .b.log 2>&1
  cb=$?
  if cmp -s .a.log .b.log && [ "$ca" = "$cb" ]; then
    printf 'identical (exit %s): %s\n' "$ca" "$args"
  else
    fail=1
    printf 'DIFFERS (exit %s vs %s): %s\n' "$ca" "$cb" "$args"
    diff -u .a.log .b.log | head -40
  fi
done

printf -- '--- escape sequences reaching a pipe: '
if LC_ALL=C grep -q "$(printf '\033')" .b.log; then
  printf 'FOUND, and there must be none\n'
  fail=1
else
  printf 'none\n'
fi
exit $fail
