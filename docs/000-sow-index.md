# SOW Index — `px-solver`

| Field | Value |
|------|-------|
| Project | `px-solver` |
| Document version | 1.0.0 |
| Date | 2026-05-16 |
| Maturity | Standard |
| Owner | KeyCode17 (m.daffa.karyudi@gmail.com) |
| Engagement type | Self-funded research / open-source |
| Brief | `docs/specs/2026-05-16-px-solver-brief.json` |
| Related specs (planned) | `docs/specs/2026-05-16-px-solver-PRD.md`, `docs/specs/2026-05-16-px-solver-FSD.md` |

## Sections

| # | File | Section |
|---|------|---------|
| 001 | [`001-sow-executive-summary.md`](001-sow-executive-summary.md) | Executive Summary |
| 002 | [`002-sow-scope.md`](002-sow-scope.md) | Scope & Boundaries |
| 003 | [`003-sow-deliverables.md`](003-sow-deliverables.md) | Deliverables & Milestones |
| 004 | [`004-sow-data-governance.md`](004-sow-data-governance.md) | Data Governance & Ownership |
| 005 | [`005-sow-privacy-security.md`](005-sow-privacy-security.md) | Data Privacy & Security |
| 006 | [`006-sow-cost-constraints.md`](006-sow-cost-constraints.md) | API / Compute Cost Constraints |
| 007 | [`007-sow-ai-disclaimer.md`](007-sow-ai-disclaimer.md) | LLM / AI Accuracy Disclaimer |
| 008 | [`008-sow-acceptance.md`](008-sow-acceptance.md) | Acceptance Criteria |
| 009 | [`009-sow-change-control.md`](009-sow-change-control.md) | Change Control Process |
| 010 | [`010-sow-payment-budget.md`](010-sow-payment-budget.md) | Payment Terms / Budget |
| 011 | [`011-sow-dual-use.md`](011-sow-dual-use.md) | Dual-Use & Authorization Policy |

## Phase Plan

Execution is sliced into 5 phases mapped to SOW-DEL IDs. See [`phase/README.md`](phase/README.md).

| # | Phase | Window | SOW-DEL covered |
|---|-------|--------|-----------------|
| 00 | [Bootstrap](phase/00-bootstrap.md) | 2026-05-19 → 2026-05-23 | SOW-DEL-001 |
| 01 | [Foundation](phase/01-foundation.md) | 2026-05-25 → 2026-06-05 | SOW-DEL-002, SOW-DEL-004, SOW-DEL-013, SOW-DEL-014, SOW-DEL-015, SOW-DEL-016, SOW-DEL-017 |
| 02 | [Harvester + PX handler](phase/02-harvester.md) | 2026-05-27 → 2026-06-03 | SOW-DEL-003a, SOW-DEL-003b |
| 03 | [Server + Auth + Native stub](phase/03-server-auth-native.md) | 2026-06-04 → 2026-06-12 | SOW-DEL-005, SOW-DEL-006, SOW-DEL-009 |
| 04 | [CLI + Canary + Docs](phase/04-cli-canary-docs.md) | 2026-06-13 → 2026-06-17 | SOW-DEL-007, SOW-DEL-008, SOW-DEL-010 |

## Deliverable Traceability

| ID | Title | File |
|---|---|---|
| SOW-DEL-001 | Cargo workspace scaffold | `003-sow-deliverables.md` |
| SOW-DEL-002 | PX detection & domain model | `003-sow-deliverables.md` |
| SOW-DEL-003 | Browser harvester pool | `003-sow-deliverables.md` |
| SOW-DEL-004 | Cookie cache layer | `003-sow-deliverables.md` |
| SOW-DEL-005 | REST API server | `003-sow-deliverables.md` |
| SOW-DEL-006 | Auth, allowlist, audit log | `003-sow-deliverables.md` |
| SOW-DEL-007 | CLI | `003-sow-deliverables.md` |
| SOW-DEL-008 | Canary integration test | `003-sow-deliverables.md` |
| SOW-DEL-009 | Native sensor generator stub | `003-sow-deliverables.md` |
| SOW-DEL-010 | Operator documentation | `003-sow-deliverables.md` |
| SOW-DEL-013 | Challenge pipeline (`px-pipeline`) | `003-sow-deliverables.md` |
| SOW-DEL-014 | Cloudflare handler stub (`px-cloudflare`) | `003-sow-deliverables.md` |
| SOW-DEL-015 | Turnstile handler stub (`px-turnstile`) | `003-sow-deliverables.md` |
| SOW-DEL-016 | Captcha handler stub (`px-captcha`) | `003-sow-deliverables.md` |
| SOW-DEL-017 | DataDome handler stub (`px-datadome`) | `003-sow-deliverables.md` |
