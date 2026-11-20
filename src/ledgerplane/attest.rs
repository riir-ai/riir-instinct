// VENDORED — src/ledgerplane/attest.rs (instinct issue 030 disposition, 2026-11-20):
// canonical home ../riir-ledger/src/attest.rs @ 687aa8ffd7d897950fd6021b58793901ef9cd94e. Written by
// scripts/vendor_ledgerplane.sh (the ONLY sanctioned write path); never
// hand-edit either side. Drift pinned by tests/ledgerplane_parity.rs.
//! The seat-attestation signing bytes (Proposal 027): the exact byte
//! string the client Ed25519-signs and the hosted seat re-derives and
//! verifies — ONE home, zero drift by construction. The wire rides four
//! HTTP headers outside the frozen decision_wire JSON contract (the
//! `kat_ua` header-seam precedent):
//!
//! - `X-Points-Account` — `BLAKE3(pubkey)` hex (the account id).
//! - `X-Points-Pubkey` — the raw Ed25519 public key, hex (a hash cannot
//!   be reversed into a key; the seat checks
//!   `blake3(pubkey) == account_id`, then verifies).
//! - `X-Points-Seq` — the call sequence, DECIMAL, UNTRUSTED transport:
//!   Ed25519 has no message recovery, so the seat could not otherwise
//!   reconstruct the signed payload (the seq is inside it). The
//!   signature authenticates the seq — a forged header fails the sig —
//!   and the seat's replay window reads the AUTHORITATIVE seq from the
//!   VERIFIED payload bytes `[42..50]`, never from the header.
//! - `X-Points-Sig` — the detached Ed25519 signature over
//!   [`attestation_signing_bytes`].
//!
//! `call_seq` is a PER-PROCESS counter checked at the seat through a
//! replay window (a fixed bitmap over the last N sequence numbers, the
//! IPsec shape) — several concurrent `cargo refine` runs share one
//! account and day, so strictly-increasing checking would drop their
//! out-of-order calls to the anonymous posture.

/// The per-call attestation signing payload: the raw 32-byte account
/// id, the ASCII `YYYY-MM-DD` UTC day, the u64 call sequence
/// BIG-endian, and the BLAKE3 hash of the exact request body —
/// concatenated with NO separators. The client signs these bytes; the
/// seat re-derives them from the received headers + body before
/// verifying.
///
/// `body_hash` is the BLAKE3 of the exact bytes the transport sends —
/// computed by the CALLER because this leaf crate carries no hash
/// dependency (the leaf law: serde + serde_json only).
#[must_use]
pub fn attestation_signing_bytes(
    account_id: &[u8; 32],
    utc_day: &str,
    call_seq: u64,
    body_hash: &[u8; 32],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(32 + 10 + 8 + 32);
    out.extend_from_slice(account_id);
    out.extend_from_slice(utc_day.as_bytes());
    out.extend_from_slice(&call_seq.to_be_bytes());
    out.extend_from_slice(body_hash);
    out
}

#[cfg(test)]
mod tests {
    use super::attestation_signing_bytes;

    #[test]
    fn signing_bytes_are_the_pinned_concatenation() {
        let account = [0xab_u8; 32];
        let body_hash = [0xcd_u8; 32];
        let bytes = attestation_signing_bytes(&account, "2026-11-02", 7, &body_hash);
        let mut expected = Vec::new();
        expected.extend_from_slice(&account);
        expected.extend_from_slice(b"2026-11-02");
        expected.extend_from_slice(&7_u64.to_be_bytes());
        expected.extend_from_slice(&body_hash);
        assert_eq!(bytes, expected);
        assert_eq!(bytes.len(), 82);
    }
}
