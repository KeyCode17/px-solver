#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -lt 1 ] || [ "$#" -gt 2 ]; then
  echo "Usage: $0 <input> [output]" >&2
  echo "Sanitizes a captured HTML/JSON file for committing as a test fixture:" >&2
  echo "  - UUIDs                -> <UUID>" >&2
  echo "  - 32+ char lowercase hex -> <HEX>" >&2
  echo "  - IPv4 addresses       -> <IPV4>" >&2
  echo "  - Bearer / x-csrf-style tokens -> <TOKEN>" >&2
  exit 64
fi

in="$1"
out="${2:-/dev/stdout}"

if [ "$in" = "-" ] || [ "$in" = "/dev/stdin" ]; then
  in=$(mktemp)
  trap 'rm -f "$in"' EXIT
  cat > "$in"
elif [ ! -f "$in" ]; then
  echo "Input file not found: $in" >&2
  exit 2
fi

sed -E \
  -e 's/\b[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}\b/<UUID>/g' \
  -e 's/\b[0-9a-f]{32,}\b/<HEX>/g' \
  -e 's/\b(([0-9]{1,3}\.){3}[0-9]{1,3})\b/<IPV4>/g' \
  -e 's/(Bearer|Authorization:)\s+[A-Za-z0-9._\-]+/\1 <TOKEN>/g' \
  -e 's/(eyJ[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+)/<JWT>/g' \
  "$in" > "$out"
