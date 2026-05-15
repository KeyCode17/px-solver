#!/usr/bin/env bash
set -euo pipefail

LIMIT="${LOC_LIMIT:-200}"
fail=0

while IFS= read -r f; do
  case "$f" in
    ./target/*|*/target/*) continue ;;
    ./px-research/*) continue ;;
    ./xtask/*) continue ;;
    */tests/*|*/examples/*) continue ;;
  esac
  lines=$(wc -l < "$f" | tr -d ' ')
  if [ "$lines" -gt "$LIMIT" ]; then
    printf '  %s: %d lines (limit %d)\n' "$f" "$lines" "$LIMIT" >&2
    fail=1
  fi
done < <(find . -type f -name '*.rs' \
    -not -path './target/*' \
    -not -path './px-research/*' \
    -not -path './xtask/*')

exit "$fail"
