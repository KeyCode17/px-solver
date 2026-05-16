#!/usr/bin/env bash
#
# MVP-AC-1..4 soak harness for px-solver
# -------------------------------------------------------------------
# Runs a sustained /v1/solve load against a target for a configurable
# duration, captures /v1/metrics snapshots periodically, and writes a
# single markdown evidence file at docs/verification/<date>-soak.md.
#
# Usage:
#   PX_SOAK_KEY=key1:topsecret \
#     scripts/soak.sh [duration] [rps] [target_url]
#
# Arguments default to: 24h, 1 RPS, pedidosya.com.ar canary.
# Server URL via env: PX_SERVER (default http://127.0.0.1:8080).
#
# The px-server must already be running. The script does not start it.
#
# Exit codes:
#   0  all four ACs pass
#   1  at least one AC fails (evidence file still written)
#   2  fatal setup error before the loop began
set -euo pipefail

DURATION="${1:-24h}"
RPS="${2:-1}"
TARGET_URL="${3:-https://www.havenwellwithin.com/}"
SERVER="${PX_SERVER:-http://127.0.0.1:8080}"

if [ "${PX_SOAK_KEY:-}" = "" ]; then
  echo "PX_SOAK_KEY must be set to '<id>:<secret>' for /v1/solve auth" >&2
  exit 2
fi

case "$DURATION" in
  *s) SECS=${DURATION%s} ;;
  *m) SECS=$(( ${DURATION%m} * 60 )) ;;
  *h) SECS=$(( ${DURATION%h} * 3600 )) ;;
  *)  SECS="$DURATION" ;;
esac

if ! [[ "$SECS" =~ ^[0-9]+$ ]] || [ "$SECS" -le 0 ]; then
  echo "invalid duration: $DURATION" >&2
  exit 2
fi
if ! [[ "$RPS" =~ ^[0-9]+$ ]] || [ "$RPS" -le 0 ]; then
  echo "invalid rps: $RPS" >&2
  exit 2
fi

DATE=$(date +%Y-%m-%d)
OUT_DIR="docs/verification"
mkdir -p "$OUT_DIR"
EVIDENCE="$OUT_DIR/${DATE}-soak.md"
TIMINGS=$(mktemp -t px-soak-timings.XXXXXX)
METRICS_LOG=$(mktemp -t px-soak-metrics.XXXXXX)
RESP=$(mktemp -t px-soak-resp.XXXXXX)

START_EPOCH=$(date +%s)
END_EPOCH=$(( START_EPOCH + SECS ))
START_ISO=$(date -u -d "@$START_EPOCH" +"%Y-%m-%dT%H:%M:%SZ")

TOTAL=0
SOLVES_OK=0
SOLVES_FAIL=0

ZOMBIE_PATTERN="${PX_SOAK_ZOMBIE_PATTERN:-headless_shell}"

finalize() {
  set +e
  local end_iso
  end_iso=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
  local zombies
  zombies=$(pgrep -af "$ZOMBIE_PATTERN" | wc -l)
  local final_metrics hit_ratio entries
  final_metrics=$(curl -fsS "$SERVER/v1/metrics" 2>/dev/null || echo "")
  hit_ratio=$(echo "$final_metrics" | awk '/^px_cache_hit_ratio /{print $2}')
  entries=$(echo "$final_metrics" | awk '/^px_cache_entries /{print $2}')
  local median p95 success
  if [ -s "$TIMINGS" ]; then
    median=$(sort -n "$TIMINGS" | awk '{a[NR]=$1} END {if (NR%2==1) print a[(NR+1)/2]; else printf "%.0f\n", (a[NR/2]+a[NR/2+1])/2}')
    p95=$(sort -n "$TIMINGS" | awk '{a[NR]=$1} END {idx=int(NR*0.95); if (idx<1) idx=1; print a[idx]}')
  else
    median="NA"
    p95="NA"
  fi
  if [ "$TOTAL" -gt 0 ]; then
    success=$(awk -v ok="$SOLVES_OK" -v tot="$TOTAL" 'BEGIN { printf "%.4f", ok/tot }')
  else
    success="NA"
  fi

  local ac1 ac2 ac3 ac4
  ac1=$(awk -v s="$success" 'BEGIN { if (s == "NA") print "skip"; else if (s+0 >= 0.95) print "pass"; else print "fail" }')
  ac2=$(awk -v m="$median" 'BEGIN { if (m == "NA") print "skip"; else if (m+0 <= 6000) print "pass"; else print "fail" }')
  ac3=$(awk -v r="${hit_ratio:-NA}" 'BEGIN { if (r == "NA" || r == "") print "skip"; else if (r+0 >= 0.70) print "pass"; else print "fail" }')
  ac4=$(awk -v z="${zombies:-NA}" 'BEGIN { if (z == "NA") print "skip"; else if (z+0 == 0) print "pass"; else print "fail" }')

  cat > "$EVIDENCE" <<EOF
# Soak evidence — ${START_ISO} → ${end_iso}

**Duration:** ${SECS}s (${DURATION})
**RPS target:** ${RPS}
**Target URL:** ${TARGET_URL}
**Server:** ${SERVER}

## Counters

| Metric          | Value         |
|-----------------|---------------|
| Total requests  | ${TOTAL}      |
| Solves OK (200) | ${SOLVES_OK}  |
| Solves failed   | ${SOLVES_FAIL} |
| Success ratio   | ${success}    |
| Median solve_ms | ${median}     |
| p95 solve_ms    | ${p95}        |
| Cache hit_ratio | ${hit_ratio:-NA} |
| Cache entries   | ${entries:-NA} |
| Browser procs   | ${zombies}    |

## MVP-AC verdicts

| Item     | Threshold              | Observed                          | Verdict |
|----------|------------------------|-----------------------------------|---------|
| MVP-AC-1 | success_ratio >= 0.95  | ${success}                        | ${ac1}  |
| MVP-AC-2 | median solve_ms <= 6000| ${median}                         | ${ac2}  |
| MVP-AC-3 | cache hit_ratio >= 0.70| ${hit_ratio:-NA}                  | ${ac3}  |
| MVP-AC-4 | browser procs == 0     | ${zombies}                        | ${ac4}  |

## Final /v1/metrics snapshot

\`\`\`
$(echo "$final_metrics" | sed 's/^/    /')
\`\`\`

## Periodic metrics log

Raw per-minute snapshots: \`${METRICS_LOG}\`

## Reproducibility

\`\`\`
PX_SOAK_KEY=<id:secret> scripts/soak.sh ${DURATION} ${RPS} ${TARGET_URL}
\`\`\`

After review, append your sign-off line to
\`docs/verification/<date>-owner-signoff.md\` (MVP-AC-7).
EOF

  printf "\n[soak] wrote %s\n" "$EVIDENCE"
  printf "[soak] AC-1=%s AC-2=%s AC-3=%s AC-4=%s\n" "$ac1" "$ac2" "$ac3" "$ac4"

  rm -f "$RESP" "$TIMINGS"

  if [ "$ac1" = "fail" ] || [ "$ac2" = "fail" ] || [ "$ac3" = "fail" ] || [ "$ac4" = "fail" ]; then
    exit 1
  fi
  exit 0
}

trap finalize EXIT INT TERM

INTERVAL_S=$(awk -v r="$RPS" 'BEGIN { printf "%.6f", 1.0 / r }')
SNAP_EVERY=$(( RPS * 60 ))
if [ "$SNAP_EVERY" -lt 1 ]; then SNAP_EVERY=1; fi

echo "[soak] start=${START_ISO} duration=${SECS}s rps=${RPS} target=${TARGET_URL}"

while [ "$(date +%s)" -lt "$END_EPOCH" ]; do
  TOTAL=$((TOTAL + 1))
  T0=$(date +%s%3N)
  HTTP_CODE=$(curl -fsS -o "$RESP" -w "%{http_code}" \
    -X POST "$SERVER/v1/solve" \
    -H "Authorization: Bearer $PX_SOAK_KEY" \
    -H 'Content-Type: application/json' \
    -d "{\"url\":\"$TARGET_URL\"}" 2>/dev/null || echo "000")
  T1=$(date +%s%3N)
  WALL_MS=$((T1 - T0))
  if [ "$HTTP_CODE" = "200" ]; then
    SOLVES_OK=$((SOLVES_OK + 1))
    SOLVE_MS=$(grep -oE '"solve_ms":[0-9]+' "$RESP" | head -1 | cut -d: -f2)
    echo "${SOLVE_MS:-$WALL_MS}" >> "$TIMINGS"
  else
    SOLVES_FAIL=$((SOLVES_FAIL + 1))
  fi

  if [ $((TOTAL % SNAP_EVERY)) -eq 0 ]; then
    {
      echo "# $(date -u +%H:%M:%SZ) total=$TOTAL ok=$SOLVES_OK fail=$SOLVES_FAIL"
      curl -fsS "$SERVER/v1/metrics" 2>/dev/null || echo "metrics unreachable"
      echo
    } >> "$METRICS_LOG"
  fi

  sleep "$INTERVAL_S"
done
