// VENDORED — src/ledgerplane/mod.rs (instinct issue 030 disposition, 2026-11-20):
// adapted from the canonical home ../riir-ledger/src/lib.rs @ 687aa8ffd7d897950fd6021b58793901ef9cd94e. Written by
// scripts/vendor_ledgerplane.sh (the ONLY sanctioned write path); never
// hand-edit either side. Drift pinned by tests/ledgerplane_parity.rs.
//! The vendored contribution-points wire tier (`ledgerplane` — the
//! `katplane`/`ragplane` naming law). Canonical home: the PRIVATE sibling
//! `../riir-ledger` (issue 030's disposition, 2026-11-20: the ledger
//! stays private, the wire is public). This module is the seat-side copy
//! of that leaf's wire vocabulary — `scripts/vendor_ledgerplane.sh` is
//! the only sanctioned write path (never hand-edit one side), and
//! `tests/ledgerplane_parity.rs` is the drift law: committed
//! known-answer vectors run everywhere (a public clone needs no
//! sibling), and the sibling-present half byte-compares every vendored
//! file against the canonical source.
//!
//! Sanctioned divergences from the canonical home (applied on every
//! vendor AND every check, so a bare re-vendor is idempotent): the
//! per-file provenance headers; the doc-comment crate-path adaptation
//! (`crate::FORMAT_VERSION` → `crate::ledgerplane::FORMAT_VERSION`); and
//! this `mod.rs` itself (the sibling's `lib.rs` module doc is not
//! copied — its `FORMAT_VERSION` const line and every re-exported name
//! below are pinned against the sibling by the parity test, so a
//! sibling bump fails loud and the fix is re-vendor + re-pin in one
//! commit).
//!
//! The wire law lives at the canonical home: field names, order, and
//! serde attributes are what both sides read; ANY field change bumps
//! [`FORMAT_VERSION`] AND the sibling's golden fixture in the same
//! commit — a version's meaning is exactly what its goldens pin.

pub mod attest;
pub mod dto;
pub mod privacy;

pub use attest::attestation_signing_bytes;
pub use dto::{
    BalanceResponse, DomainCount, DrainState, DrainUsageExport, DrainUsageRow, EpochAnchor,
    ErrorEnvelope, InfoResponse, JoinRequest, JoinResponse, LeaveRequest, LeaveResponse,
    RankEntry, RankResponse, UsageState,
};
pub use privacy::{UUID_SHAPES_REFUSED, validate_client_uuid};

/// The wire format version — pinned byte-identical to the canonical
/// home by `tests/ledgerplane_parity.rs`; a bump there is a re-vendor +
/// re-pin here in the same commit (see the sibling's lib.rs for the
/// version lineage).
pub const FORMAT_VERSION: u32 = 3;
