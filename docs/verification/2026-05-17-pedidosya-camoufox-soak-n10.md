# Camoufox soak — pedidosya

**Start:** 2026-05-17T09:59:34Z
**End:** 2026-05-17T10:01:20Z
**Target:** https://www.pedidosya.com.ar/
**N requests:** 10
**Path:** Direct CamoufoxPool harvest (no HTTP layer, no cache)

## Counters

| Metric           | Value |
|------------------|-------|
| Total            | 10 |
| Success (cf+px3) | 10 |
| Success ratio    | 1.0000 |
| Median ms        | 10576 |
| p95 ms           | 11096 |
| Min ms           | 10154 |
| Max ms           | 11096 |

## Per-sample

| # | ms | ok | cookies |
|---|----|----|---------|
| 0 | 10726 | Y | 6 |
| 1 | 10693 | Y | 6 |
| 2 | 10560 | Y | 6 |
| 3 | 11064 | Y | 6 |
| 4 | 10365 | Y | 6 |
| 5 | 10154 | Y | 6 |
| 6 | 11096 | Y | 6 |
| 7 | 10186 | Y | 6 |
| 8 | 10593 | Y | 6 |
| 9 | 10321 | Y | 6 |

## Verdicts

| AC | Threshold (relaxed for Camoufox) | Observed | Verdict |
|----|-----------------------------------|----------|---------|
| AC-1 | success_ratio >= 0.95 | 1.0000 | pass |
| AC-2 | median_ms <= 30000    | 10576    | pass |

