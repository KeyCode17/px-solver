# Owner sign-off template — MVP-AC-7

Copy this file to `docs/verification/<YYYY-MM-DD>-owner-signoff.md` and
fill in the blanks before invoking `cargo xtask phase 04`.

```markdown
# Owner sign-off — v1.0.0 release gate

I, <full name> (<email>), have reviewed:

- docs/threat-model.md           @ commit <sha>
- docs/dual-use-policy.md        @ commit <sha>

Both documents represent current intent for the px-solver project at
this release. I confirm:

- The 9 misuse vectors listed in the threat model remain accurate and
  the mitigations are still in place in this build.
- The operator commitments in the dual-use policy are still binding on
  me and on any downstream operator I authorize.
- I have read `docs/verification/<YYYY-MM-DD>-soak.md` and accept its
  recorded MVP-AC-1..4 verdicts as the basis for this release.
- I authorize the bump from 0.4.0 to 1.0.0 under ADR-0017.

Signed: <full name>
Date:   <YYYY-MM-DD>
```

Pin the threat-model and dual-use commits with:

```bash
git log -1 --format="%H" -- docs/threat-model.md
git log -1 --format="%H" -- docs/dual-use-policy.md
```

After the file is committed (`docs(verification): owner sign-off for
v1.0.0`), run:

```bash
cargo xtask phase 04
```
