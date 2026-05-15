#!/usr/bin/env bash
set -euo pipefail

msg_file="${1:?commit message file required}"
first_line="$(head -n 1 "$msg_file")"

if [[ "$first_line" =~ ^(Merge|Revert|fixup!|squash!) ]]; then
  exit 0
fi

pattern='^(feat|fix|chore|docs|refactor|test|build|ci|perf|style|revert)(\([a-z0-9\-\.\_/]+\))?!?: .+'

if ! [[ "$first_line" =~ $pattern ]]; then
  printf 'Commit subject does not match Conventional Commits:\n  %s\n' "$first_line" >&2
  printf 'Expected: <type>(<scope>)?: <subject>\n' >&2
  printf 'Types: feat, fix, chore, docs, refactor, test, build, ci, perf, style, revert\n' >&2
  exit 1
fi

if [ "${#first_line}" -gt 72 ]; then
  printf 'Commit subject is %d chars (limit 72):\n  %s\n' "${#first_line}" "$first_line" >&2
  exit 1
fi
