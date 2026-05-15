#!/usr/bin/env bash
set -euo pipefail

LIMIT="${LOC_LIMIT:-200}"
fail=0

if [ "$#" -eq 0 ]; then
  exit 0
fi

for f in "$@"; do
  [ -f "$f" ] || continue
  case "$f" in
    *.rs) ;;
    *) continue ;;
  esac
  case "$f" in
    xtask/*|./xtask/*|*/xtask/*) continue ;;
  esac
  lines=$(wc -l < "$f" | tr -d ' ')
  if [ "$lines" -gt "$LIMIT" ]; then
    printf '  %s: %d lines (limit %d)\n' "$f" "$lines" "$LIMIT" >&2
    fail=1
  fi
done

exit "$fail"
