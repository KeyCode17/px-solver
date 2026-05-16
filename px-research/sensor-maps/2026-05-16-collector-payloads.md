---
captured: 2026-05-16
phase: R3 — preliminary
sources:
  - haven  / appId PX12Ew76qT  / collector-px12ew76qt.px-client.net/b/g
  - ped    / appId PXeT15wiaE  / collector-pxet15wiae.px-client.net/b/g
---

# Collector POST payloads — first concrete sensor-shape captures

Two real sensor POSTs were surfaced from the operator's DevTools network panel
during captcha sessions on `havenwellwithin.com` (appId `PX12Ew76qT`) and
`pedidosya.com.ar` (appId `PXeT15wiaE`). Both POSTs hit
`collector-px<appId>.px-client.net/b/g?payload=<long>&<fields>...`.

## Collector host taxonomy (observed)

| Host | Role | Observed on |
|---|---|---|
| `client.px-cloud.net/PX<appId>/main.min.js` | Per-tenant init runtime | haven + hanna + ped |
| `captcha.px-cdn.net/PX<appId>/captcha.js` | Captcha bundle (HOSTED mode) | haven |
| `<site>/<appId>/captcha/captcha.js` | Captcha bundle (FIRST-PARTY mode) | hanna + ped |
| `collector-px<appId>.px-cloud.net/assets/js/bundle` | Dynamic asset bundle (xhr) | haven + ped |
| `collector-px<appId>.px-client.net/b/g?payload=...` | **Sensor collector POST** | haven + ped |
| `tzm.px-cloud.net/ns?c=<UUID>` | Tag manager / event tracker | ped |
| `js.px-cloud.net/?t=<token>&v=<UUID>` | One-time challenge token endpoint | ped |
| `live.pystatic.com/pxassets/personalization2.js` | PedidosYa custom UI shim | ped |
| `crcldu.com/bd/auditor.js` | NON-PX off-host beacon (vendor TBD) | haven + hanna + ped |

## Query-string field grammar (sensor POST)

Observed across both haven and ped:

| Field | Type | Example | Meaning (best guess) |
|---|---|---|---|
| `payload` | base64-ish (custom alphabet) | `aUkQRhAIEGYBCgR8...` (4-6 KB) | Encrypted/encoded sensor body. Algorithm unknown until R2. |
| `appId` | literal `PX<id>` | `PXeT15wiaE` | Per-tenant identifier |
| `tag` | short base64 (~12 chars) | `WQUsSB9gMD5dBw==` | Build/version tag (varies per request — possibly a frame counter) |
| `uuid` | UUIDv1 | `0bbb4f30-50df-11f1-...` | Matches `_pxUuid` set by init.js; also appears as `u=` in captcha URL |
| `ft` | int | `397` | Frame-timing token? |
| `seq` | int | `1` | Request sequence within a session |
| `en` | base64-of-int | `NTA` = "50" | Encoding/protocol version |
| `bi` | base64 (~200 chars) | `YjIXcjcFUFM...` | Browser-info blob — probably navigator-shape fingerprint |
| `cs` | **64 lowercase hex** | `77a4afa21aa0df989665ef7ad0d50bec68d56345a7b476d3f05c0befe28fcda3` | **SHA-256 HMAC** of the rest of the payload. Key derivation = R3 work. |
| `pc` | int (16-ish digits) | `8757103927878182` | Page-counter / performance counter? |
| `sid` | UUIDv1 + invisible tail | see "sid watermark" below | Session ID with hidden timestamp |
| `vid` | UUIDv1 | `0ae1463d-50df-11f1-...` | Visitor ID (matches `_pxVid` global from earlier recon) |
| `ci` | UUIDv1 | `1003a740-50df-11f1-...` | Client/request ID |
| `pxhd` | `<base64>:<base64>` | `uvgtwm/pJXuzshU...:lXcAiXdsMI...` | URL form of `_pxhd` (cookie form is `<hex>:<UUIDv1>` — different encoding) |
| `cts` | UUIDv1 | `0be0e4c2-50df-11f1-...` | Capture timestamp UUID |
| `hid` | `<base64>:<base64>` | (ped only) | Additional hashed ID — present on ped, absent on haven |

## `sid` watermark — invisible ms-epoch encoded via Unicode Variation Selectors

This is the most operationally interesting finding.

The `sid` field is presented as `<UUIDv1>%F3%A0%84%Bx%F3%A0%84%Bx...` — the URL
contains the UUIDv1 followed by 13 four-byte UTF-8 sequences. Each four-byte
sequence decodes to a single codepoint in the **Supplementary Variation
Selectors** block (`U+E0100` .. `U+E01EF`). Offsetting each codepoint by
`-0xE0100` yields ASCII digits, spelling out a **13-digit Unix epoch
milliseconds timestamp**.

Observed decodings:

| Tenant | Visible UUID part | Invisible decoded timestamp | UTC |
|---|---|---|---|
| haven | `2d86b9bc-50d9-11f1-889b-8b3f65c0b297` | `1778902922699` | 2026-05-16 03:42:02.699 |
| ped | `0be0e2ba-50df-11f1-b5b6-61f398873cea` | `1778905440180` | 2026-05-16 04:24:00.180 |

Both timestamps match the wall-clock of the operator's recon session. This is
a real, deployed anti-replay watermark: the PX server compares the embedded
timestamp to current time and rejects stale sessions even if the rest of the
payload reverifies.

### Implication for the native solver (R4 / `px-native`)

Any synthetic `sid` produced by the native solver MUST embed the current
ms-epoch using the Variation-Selector scheme. The Rust pseudocode is:

```rust
fn watermark_sid(uuid: Uuid, ms_epoch: u64) -> String {
    let mut s = uuid.to_string();
    for d in ms_epoch.to_string().chars() {
        let cp = 0xE0100 + (d as u32);
        s.push(char::from_u32(cp).expect("vs17 codepoint"));
    }
    s
}
```

(See `px-perimeterx::domain::pxhd` for the companion `_pxhd` parser; the `sid`
watermark deserves its own helper before any R3 implementation lands.)

## HMAC field `cs`

64 lowercase hex chars → SHA-256 output. The KEY for this HMAC is what we
need to recover from `main.min.js` for a working native sensor generator.
Open question for R3.

## Cross-tenant facts

- `captcha.js` (the captcha bundle) is **byte-identical** across tenants:
  md5 `d5185dc31771cd80104622ef844dbcc9` (haven and hanna; ped 403'd as the
  captured captcha session token had expired).
- `main.min.js` differs across tenants in obfuscation seed: function names
  vary (`im(i)` on hanna, `hN(t)` on ped, `t(e)` on haven). Same underlying
  bundle; the obfuscator is run with a per-build seed.
- **Two copyright strings observed:**
  - `(C) 2012-2026 HUMAN Security, Inc` on hanna + ped (post-rebrand build)
  - `(C) 2014-2026 PerimeterX, Inc` on haven (legacy build)
- haven uses **hosted** captcha (`captcha.px-cdn.net`); hanna + ped use
  **first-party** (`<site>/<appId>/captcha/`). Our `PxMode::Hosted` enum
  variant is real and observed.

## Reference payload literals (verbatim, sanitized)

The full raw query strings are preserved verbatim in companion files:

- `2026-05-16-haven-collector-post.txt`
- `2026-05-16-ped-collector-post.txt`

Both are kept literally so future deobf work can match against them.
