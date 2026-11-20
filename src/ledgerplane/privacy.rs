// VENDORED — src/ledgerplane/privacy.rs (instinct issue 030 disposition, 2026-11-20):
// canonical home ../riir-ledger/src/privacy.rs @ 687aa8ffd7d897950fd6021b58793901ef9cd94e. Written by
// scripts/vendor_ledgerplane.sh (the ONLY sanctioned write path); never
// hand-edit either side. Drift pinned by tests/ledgerplane_parity.rs.
//! The client-UUID privacy law as code (the points-plane privacy design,
//! T16.1 §1 — the canonical text lives in the riir-refine repo at
//! `.docs/01_orientation/points_privacy_id_design.md`; this module never
//! copies it, it enforces it).
//!
//! The law: one install, one **client UUID** — 128 bits from the OS
//! CSPRNG, rendered in the standard hyphenated UUID shape, generated
//! LOCALLY, resettable by the user. It is never derived: no hardware, no
//! hostname, no MAC, no OS machine GUID — and critically, **no
//! timestamp**. A v7-shaped token embeds its creation moment and leaks
//! the install/reset time for zero benefit (the server stores an opaque
//! token; there is no B-tree here). This module refuses everything but
//! the v4 shape.

/// The refused client-UUID shapes, as one text — the doc the consent
/// screen and the API reference render.
pub const UUID_SHAPES_REFUSED: &str = "Client UUIDs are v4-shaped only. Refused: \
uuid v7 (version nibble '7') — its embedded millisecond timestamp leaks the \
install/reset moment for zero benefit, because the server stores an opaque \
token and there is no B-tree here (T16.1 §1, the Random law; the documented \
deviation from the house Uuid::now_v7 default, which is for entity keys, not \
secrets); non-hyphenated or wrong-length shapes — the token renders in the \
standard 8-4-4-4-12 hyphenated form, 36 chars; any version nibble other than \
'4' (v1/v2/v3/v5/v6/v8 are not random-v4 tokens); any variant-nibble \
violation — position 19 must be one of 8, 9, a, b (RFC 4122); and any token \
derived from hardware, hostname, MAC address, OS machine GUID, or a serial \
number in any form (T16.1 §2 — the refused-design list). Refused shapes \
never parse as valid client identity.";

fn is_lower_hex(b: u8) -> bool {
    b.is_ascii_digit() || matches!(b, b'a'..=b'f')
}

/// Validate the client-UUID shape: the v4 form ONLY.
///
/// Checks, in order: 36 chars; hyphens at positions 8/13/18/23; lowercase
/// hex elsewhere; version nibble `'4'` at position 14; RFC-4122 variant
/// nibble (one of `8 9 a b`) at position 19. A v7-shaped token refuses
/// with the T16.1 timestamp-leak reason, not a generic shape error.
pub fn validate_client_uuid(s: &str) -> Result<(), String> {
    let bytes = s.as_bytes();
    if bytes.len() != 36 {
        return Err(format!(
            "client UUID must be the standard 36-char hyphenated form (got {} chars); {UUID_SHAPES_REFUSED}",
            s.len()
        ));
    }
    for (i, b) in bytes.iter().enumerate() {
        if matches!(i, 8 | 13 | 18 | 23) {
            if *b != b'-' {
                return Err(format!(
                    "client UUID must be hyphenated at positions 8/13/18/23 (position {i} is not '-'); {UUID_SHAPES_REFUSED}"
                ));
            }
        } else if !is_lower_hex(*b) {
            return Err(format!(
                "client UUID must be lowercase hex outside the hyphens (position {i} is not); {UUID_SHAPES_REFUSED}"
            ));
        }
    }
    match bytes[14] {
        b'4' => {}
        b'7' => {
            return Err(
                "client UUID must be a random v4 token: this is a v7-shaped token, and v7 embeds \
                 its creation timestamp — the install/reset moment — leaking it for zero benefit, \
                 because the server stores an opaque token and there is no B-tree here \
                 (T16.1 §1, the Random law)"
                    .to_string(),
            );
        }
        other => {
            return Err(format!(
                "client UUID must be a random v4 token (version nibble '4', found '{}'); {UUID_SHAPES_REFUSED}",
                other as char
            ));
        }
    }
    match bytes[19] {
        b'8' | b'9' | b'a' | b'b' => Ok(()),
        other => Err(format!(
            "client UUID must carry the RFC-4122 variant nibble (position 19 in [89ab], found '{}'); {UUID_SHAPES_REFUSED}",
            other as char
        )),
    }
}
