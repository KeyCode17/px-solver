#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -eq 0 ]; then
  exit 0
fi

if ! command -v rg >/dev/null 2>&1; then
  echo "ripgrep (rg) required for forbidden-pattern check." >&2
  exit 2
fi

fail=0

for f in "$@"; do
  [ -f "$f" ] || continue
  case "$f" in
    *.rs) ;;
    *) continue ;;
  esac

  case "$f" in
    px-cli/*|./px-cli/*|*/px-cli/*) allow_print=1 ;;
    xtask/*|./xtask/*|*/xtask/*) allow_print=1 ;;
    *) allow_print=0 ;;
  esac

  if [ "$allow_print" -ne 1 ]; then
    if rg -n --color=never -e '^\s*(println!|eprintln!|dbg!)' "$f" >&2; then
      printf '  %s: println!/eprintln!/dbg! is forbidden outside px-cli/ and xtask/.\n' "$f" >&2
      fail=1
    fi
  fi
done

exit "$fail"
