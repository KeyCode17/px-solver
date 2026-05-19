//! Compose the sensor event batch from a `SyntheticIdentity`.
//!
//! Tag set + field keys are derived from a real eT15wiaE capture
//! (see `px-research/captures/eT15wiaE/<ts>.json`, decoded via
//! `decrypt_payload_lossy`). The two tags `AzNweUVTcEw=` and
//! `egoJQD9rDHs=` are stable across runs. The ~90 inner field keys
//! are populated with `"no_fp"` placeholders by default — that's the
//! sentinel the runtime itself emits when a fingerprint surface
//! couldn't be probed, and PX still issues `_px3` against it.

use crate::events::identity::SyntheticIdentity;
use crate::events::model::SensorEvent;

const TAG_FP: &str = "AzNweUVTcEw=";
const TAG_TELEM: &str = "egoJQD9rDHs=";

/// 67 observed keys inside `AzNweUVTcEw=`, excluding the four
/// non-base64 PX-codes (`PX12738..PX12741`) populated separately.
const FP_KEYS: &[&str] = &[
    "AEwzBkUuNDA=",
    "AEwzBkYsMzQ=",
    "AW1yJ0QIdRc=",
    "AzNweUVSfU0=",
    "BFA3GkIzOy0=",
    "BXF2O0MXegk=",
    "Bzd0fUJRdUs=",
    "DFg/Eko0MyY=",
    "Dh49VEt+MWA=",
    "EX1iN1QdYAE=",
    "EX1iN1cebgQ=",
    "FCAnKlFHKho=",
    "FmYlbFMDIlk=",
    "FwdkDVFmZT0=",
    "FwdkDVJmaDk=",
    "GmopYFwKJVE=",
    "Hm4tZFgDLV8=",
    "JVEWW2A3F2s=",
    "JVEWW2AxEG8=",
    "JnZVfGAQUUc=",
    "KVUaX281HWs=",
    "KVUaX2wzFmo=",
    "KnpZcG8eXUs=",
    "MDxDNnVbRAI=",
    "MV0CV3Q6D2w=",
    "MkJBCHQjQj0=",
    "NABHSnFhR34=",
    "NABHSnJsQX8=",
    "NkZFDHMnQjc=",
    "O2sIIX0JBBI=",
    "OARLTn1jTXQ=",
    "OARLTn5pS3w=",
    "OkpJAHwsTDQ=",
    "P28MJXoPDxU=",
    "PAhPQnlsSXE=",
    "PSkOY3hID1k=",
    "PSkOY3hNA1c=",
    "PSkOY3tFA1I=",
    "RBB3WgJwdW8=",
    "RTE2ewNcNks=",
    "S3s4MQ4bPgY=",
    "Tl59FAs6eiY=",
    "U0MgSRUlLXs=",
    "UT0idxRYJkY=",
    "WQUqDx9jKz8=",
    "WippIB9OaRs=",
    "YGwTZiYMElU=",
    "YQ1SByRvUzU=",
    "YQ1SByduUT0=",
    "YjIROCRTEQI=",
    "YjIROCRfEQk=",
    "Z1dUXSE1VWs=",
    "Z1dUXSExV2Y=",
    "ZjYVPCBWEgc=",
    "bHgfcikaG0M=",
    "bRleEyh/XyA=",
    "cHwDdjUaBEU=",
    "cHwDdjYdAkE=",
    "cR1CFzR9RCI=",
    "cgIBSDdhB3g=",
    "dEAHCjIsBzA=",
    "dydEbTJGQlg=",
    "eWVKLzwCRhU=",
    "egoJQD9qC3c=",
    "fEgPAjooCDQ=",
    "fg4NRDtqC34=",
];

/// 16 observed keys inside `egoJQD9rDHs=` minus the 5 PX-code counters.
const TELEM_KEYS: &[&str] = &[
    "CFQ7Hk01OSk=",
    "DXl+M0sUfQQ=",
    "GUVqT18oank=",
    "ICxTJmZAVBA=",
    "KVUaX282GWQ=",
    "MDxDNnZaTgc=",
    "MDxDNnZfQwE=",
    "O2sIIX0LChM=",
    "Tl59FAg8fSE=",
    "UBxjVhV5Y2Q=",
    "V0ckTRErIH0=",
    "Z1dUXSExV2Y=",
    "bHgfcikaG0M=",
    "bRleEyh6WCA=",
    "dydEbTJDRl8=",
    "eytIYT5KS1M=",
];

pub fn default_batch(identity: &SyntheticIdentity, now_ms: u64) -> Vec<SensorEvent> {
    vec![fingerprint_event(identity, now_ms), telemetry_event(now_ms)]
}

fn fingerprint_event(identity: &SyntheticIdentity, now_ms: u64) -> SensorEvent {
    let mut ev = SensorEvent::new(TAG_FP);
    for k in FP_KEYS {
        ev = ev.with(*k, "no_fp");
    }
    ev = ev
        .with("aHQbfi0UGEw=", session_id(identity))
        .with("AEwzBkUuNDA=", identity.user_agent.as_str())
        .with("DXl+M0sUfQQ=", now_ms)
        .with("PX12738", 0u64)
        .with("PX12739", 0u64)
        .with("PX12740", 0u64)
        .with("PX12741", 4099u64);
    ev
}

fn telemetry_event(now_ms: u64) -> SensorEvent {
    let mut ev = SensorEvent::new(TAG_TELEM);
    for k in TELEM_KEYS {
        ev = ev.with(*k, "no_fp");
    }
    ev = ev
        .with("DXl+M0sUfQQ=", now_ms)
        .with("PX11669", 0u64)
        .with("PX11699", 0u64)
        .with("PX12033", 0u64)
        .with("PX12270", 0u64)
        .with("PX12343", 0u64)
        .with("PX12740", 0u64)
        .with("PX12741", 0u64);
    ev
}

fn session_id(identity: &SyntheticIdentity) -> String {
    format!("{:032x}", identity.key_hash())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_batch_has_observed_tags() {
        let id = SyntheticIdentity::test_default();
        let batch = default_batch(&id, 1_716_192_345_678);
        assert_eq!(batch.len(), 2);
        assert_eq!(batch[0].t, TAG_FP);
        assert_eq!(batch[1].t, TAG_TELEM);
        // Sanity: every observed key from the capture is populated.
        for k in FP_KEYS {
            assert!(batch[0].d.contains_key(*k), "missing fp key {k}");
        }
        for k in TELEM_KEYS {
            assert!(batch[1].d.contains_key(*k), "missing telem key {k}");
        }
    }
}
