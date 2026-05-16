# Owner sign-off — v1.0.0 release gate

I, Mochammad Daffa Putra Karyudi (m.daffa.karyudi@gmail.com), have reviewed:

- docs/threat-model.md           @ commit b260393ec8c256a204d7c47da08e78f8aa132e6f
- docs/dual-use-policy.md        @ commit c874173fece19c277d2272203a7721c64792b7c9

Both documents represent current intent for the px-solver project at
this release. I confirm:

- The 9 misuse vectors listed in the threat model remain accurate and
  the mitigations are still in place in this build.
- The operator commitments in the dual-use policy are still binding on
  me and on any downstream operator I authorize.
- I have read `docs/verification/2026-05-16-soak.md` and accept its
  recorded MVP-AC-1..4 verdicts (all PASS over a 5-minute soak against
  havenwellwithin.com) as the basis for this release. I acknowledge
  the duration caveat in that file: the SOW names a 24-hour window;
  this evidence covers 5 minutes. I treat a separate 24h confirmation
  soak as a post-release operator health check, not a v1.0.0 gate.
- I authorize the canary scope broadening from "pedidosya.com.ar
  specifically" to "any PX-direct allowlisted target with
  `tos_reviewed: true`" per
  [ADR-0018](../adr/0018-canary-scope-broaden-to-px-direct-targets.md),
  and the opening of the R5 deep-stealth research track per
  [ADR-0019](../adr/0019-r5-track-deep-stealth-and-synthetic-profiles.md).
  Pedidosya's CF-fronted variant remains a project target under R5.
- I authorize the bump from 0.4.0 to 1.0.0 under ADR-0017.

Signed: Mochammad Daffa Putra Karyudi
Date:   2026-05-16
