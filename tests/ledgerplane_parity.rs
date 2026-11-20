//! The ledgerplane drift law (issue 030's disposition, 2026-11-20).
//!
//! Two halves, never interchangeable:
//!
//! - **Known-answer vectors** (run everywhere — a public clone needs no
//!   sibling): `FORMAT_VERSION`, the attestation signing bytes (the ONE
//!   home both the seat and the plane's clients must agree on), canonical
//!   JSON sort-independence (instinct's `preserve_order` serde_json
//!   feature cannot leak into any signing payload), the seat's export
//!   shape, the usage-state read, and the client-UUID privacy law's spot
//!   refusals.
//! - **Canonical parity** (skip-LOUD without `../riir-ledger`): every
//!   vendored file byte-compares against the canonical private sibling
//!   modulo the sanctioned divergences (provenance header + the doc-path
//!   adaptation — the same set `scripts/vendor_ledgerplane.sh` applies on
//!   every check), and `mod.rs` pins the sibling's `FORMAT_VERSION` const
//!   line and every re-exported name. A sibling bump fails here loud;
//!   the fix is re-vendor + re-pin in one commit. A skip is a deferral,
//!   never a green.

use std::path::{Path, PathBuf};

use riir_instinct::ledgerplane::{
    attest::attestation_signing_bytes,
    dto::{canonical_json, DrainUsageExport, DrainUsageRow, JoinRequest, UsageState},
    validate_client_uuid, FORMAT_VERSION, UUID_SHAPES_REFUSED,
};

const ACCOUNT: [u8; 32] = [0xab; 32];
const BODY_HASH: [u8; 32] = [0xcd; 32];

// 32 × 0xab | ASCII("2026-11-02") | u64 BE(7) | 32 × 0xcd — the ONE
// signing-bytes home, byte-pinned.
const EXPECTED_ATTESTATION_HEX: &str = concat!(
    "abababababababab",
    "abababababababab",
    "abababababababab",
    "abababababababab",
    "323032362d31312d3032",
    "0000000000000007",
    "cdcdcdcdcdcdcdcd",
    "cdcdcdcdcdcdcdcd",
    "cdcdcdcdcdcdcdcd",
    "cdcdcdcdcdcdcdcd",
);

#[test]
fn format_version_is_pinned() {
    assert_eq!(FORMAT_VERSION, 3);
}

#[test]
fn attestation_signing_bytes_known_answer() {
    let bytes = attestation_signing_bytes(&ACCOUNT, "2026-11-02", 7, &BODY_HASH);
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(hex, EXPECTED_ATTESTATION_HEX);
    assert_eq!(bytes.len(), 82);
    // The seat's replay window reads the AUTHORITATIVE seq from the
    // VERIFIED payload bytes [42..50] (offset 32 + 10, u64 BE) — the
    // vendored layout must keep that contract.
    let mut seq_bytes = [0u8; 8];
    seq_bytes.copy_from_slice(&bytes[42..50]);
    assert_eq!(u64::from_be_bytes(seq_bytes), 7);
}

#[test]
fn canonical_json_sorts_keys_independently_of_presentation_order() {
    // Nested + top-level keys deliberately OUT of lexicographic order;
    // this build enables serde_json/preserve_order, so the sort must
    // come from the canonical writer (BTreeMap), never from the
    // serializer's insertion order.
    let v = serde_json::json!({
        "zebra": 1,
        "apple": { "y": 2, "x": 3 },
        "mid": [1, 2],
    });
    let out = canonical_json(&v).expect("canonical json");
    assert_eq!(out, r#"{"apple":{"x":3,"y":2},"mid":[1,2],"zebra":1}"#);
}

#[test]
fn signed_message_bytes_is_version_prefix_plus_sorted_canonical_body() {
    let req = JoinRequest {
        format_version: 3,
        client_uuid: "11111111-2222-4333-8444-555555555555".into(),
        account_pubkey_hex: "aa".into(),
        github_login: "octocat".into(),
        github_id: 42,
    };
    let bytes = req.signed_message_bytes().expect("signing payload");
    let expected_body = r#"{"account_pubkey_hex":"aa","client_uuid":"11111111-2222-4333-8444-555555555555","format_version":3,"github_id":42,"github_login":"octocat"}"#;
    let mut expected = vec![0, 0, 0, 3]; // format_version, 4 bytes BE
    expected.extend_from_slice(expected_body.as_bytes());
    assert_eq!(bytes, expected);
}

#[test]
fn drain_usage_export_serializes_in_wire_field_order() {
    let export = DrainUsageExport {
        format_version: FORMAT_VERSION,
        rows: vec![DrainUsageRow {
            account_id_hex: "aa".into(),
            utc_day: "2026-11-19".into(),
            free_calls: 5,
            billable_calls: 2,
        }],
    };
    let body = serde_json::to_string(&export).expect("serialize export");
    assert_eq!(
        body,
        r#"{"format_version":3,"rows":[{"account_id_hex":"aa","utc_day":"2026-11-19","free_calls":5,"billable_calls":2}]}"#
    );
    let round: DrainUsageExport = serde_json::from_str(&body).expect("decode export");
    assert_eq!(round, export);
}

#[test]
fn usage_state_decodes_and_failsafe_reads_the_version() {
    let body = r#"{"format_version":3,"account_id_hex":"aa","joined":true,"balance":100}"#;
    let state: UsageState = serde_json::from_str(body).expect("decode usage state");
    assert_eq!(state.format_version, FORMAT_VERSION);
    assert_eq!(state.account_id_hex, "aa");
    assert!(state.joined);
    assert_eq!(state.balance, 100);
    // A stale-version answer must fail the seat's fail-safe comparison.
    let stale: UsageState = serde_json::from_str(
        r#"{"format_version":2,"account_id_hex":"aa","joined":true,"balance":100}"#,
    )
    .expect("decode stale");
    assert_ne!(stale.format_version, FORMAT_VERSION);
}

#[test]
fn client_uuid_privacy_law_spots() {
    assert!(validate_client_uuid("11111111-2222-4333-8444-555555555555").is_ok());
    // v7 embeds its creation timestamp — the refused-shape reason names it.
    let v7 = validate_client_uuid("11111111-2222-7333-8444-555555555555").unwrap_err();
    assert!(v7.contains("timestamp"), "{v7}");
    // Variant nibble outside [89ab].
    assert!(validate_client_uuid("11111111-2222-4333-c444-555555555555").is_err());
    // Non-hyphenated shape.
    assert!(validate_client_uuid("11111111222243338444555555555555555").is_err());
    assert!(UUID_SHAPES_REFUSED.contains("v7"));
}

// ── canonical parity (skip-LOUD without the sibling) ─────────────────

fn sibling_src() -> Option<PathBuf> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../riir-ledger/src");
    p.is_dir().then_some(p)
}

/// Strip the vendored provenance header: leading `// `-prefixed lines
/// (the canonical body starts `//!`, which this does not match).
fn strip_header(content: &str) -> &str {
    let mut off = 0;
    for line in content.lines() {
        if line.starts_with("// ") {
            off += line.len() + 1; // + the LF
        } else {
            break;
        }
    }
    &content[off..]
}

/// The sanctioned doc-path divergence, reversed for comparison.
fn normalize_adaptation(content: &str) -> String {
    content.replace(
        "crate::ledgerplane::FORMAT_VERSION",
        "crate::FORMAT_VERSION",
    )
}

fn vendored(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/ledgerplane")
            .join(name),
    )
    .unwrap_or_else(|e| panic!("read vendored {name}: {e}"))
}

fn canonical(sibling: &Path, name: &str) -> String {
    std::fs::read_to_string(sibling.join(name))
        .unwrap_or_else(|e| panic!("read canonical {name}: {e}"))
}

#[test]
fn vendored_files_byte_match_the_canonical_sibling() {
    let Some(sibling) = sibling_src() else {
        eprintln!(
            "SKIP LOUD: canonical sibling ../riir-ledger absent — the byte-parity half is a \
             deferral, never a green (the known-answer vectors above still ran)"
        );
        return;
    };
    for name in ["dto.rs", "attest.rs", "privacy.rs"] {
        let vendored = normalize_adaptation(strip_header(&vendored(name)));
        let canonical = canonical(&sibling, name);
        assert_eq!(
            vendored, canonical,
            "vendored {name} drifted from the canonical sibling — re-run \
             scripts/vendor_ledgerplane.sh and re-pin in the same commit"
        );
    }
}

fn reexport_names(src: &str, marker: &str) -> Vec<String> {
    let start = src
        .find(marker)
        .unwrap_or_else(|| panic!("missing {marker:?}"))
        + marker.len();
    let end = src[start..]
        .find("};")
        .unwrap_or_else(|| panic!("missing block end"))
        + start;
    src[start..end]
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[test]
fn mod_rs_pins_the_sibling_const_and_reexports() {
    let Some(sibling) = sibling_src() else {
        eprintln!(
            "SKIP LOUD: canonical sibling ../riir-ledger absent — the mod.rs pin half is a \
             deferral, never a green (the known-answer vectors above still ran)"
        );
        return;
    };
    let lib = canonical(&sibling, "lib.rs");
    let m = vendored("mod.rs");

    // The FORMAT_VERSION const line must be byte-identical in both.
    let const_line = lib
        .lines()
        .find(|l| l.contains("pub const FORMAT_VERSION"))
        .expect("sibling FORMAT_VERSION const");
    assert!(
        m.lines().any(|l| l.trim() == const_line.trim()),
        "sibling {const_line:?} missing from mod.rs — re-vendor + re-pin in one commit"
    );

    // Every name of the sibling's dto re-export block must appear here,
    // in the same set.
    let theirs = reexport_names(&lib, "pub use dto::{");
    let mine = reexport_names(&m, "pub use dto::{");
    assert_eq!(
        theirs, mine,
        "dto re-export set drifted from the sibling — re-vendor + re-pin in one commit"
    );

    for needle in [
        "pub use attest::attestation_signing_bytes;",
        "pub use privacy::{UUID_SHAPES_REFUSED, validate_client_uuid};",
        "pub mod attest;",
        "pub mod dto;",
        "pub mod privacy;",
    ] {
        assert!(
            lib.contains(needle),
            "canonical lib.rs lost {needle:?} — adjudicate the vendored law"
        );
        assert!(
            m.contains(needle),
            "mod.rs missing {needle:?} — re-vendor + re-pin"
        );
    }
}
