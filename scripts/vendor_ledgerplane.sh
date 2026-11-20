#!/bin/sh
# vendor_ledgerplane.sh — re-vendor the contribution-points wire tier
# (`src/ledgerplane/`) from the PRIVATE canonical sibling `../riir-ledger`
# (issue 030's disposition, 2026-11-20: the ledger stays private, the wire
# is public — the katplane/ragplane law, riir-refine issue 221).
#
#   scripts/vendor_ledgerplane.sh          # copy + re-header
#   scripts/vendor_ledgerplane.sh --check  # regenerate into a tmp dir and
#                                          # diff; no write
#
# The sibling stays canonical; this script is the ONLY sanctioned write
# path for src/ledgerplane/ — never hand-edit one side. Sanctioned
# divergences, applied identically in write and check mode (so --check is
# idempotent and a bare re-vendor never flags them):
#   1. the per-file provenance header (instinct-side only),
#   2. the doc-comment crate-path adaptation in dto.rs
#      (crate::FORMAT_VERSION -> crate::ledgerplane::FORMAT_VERSION — the
#      vendored module is not the crate root),
#   3. mod.rs is adapted from the sibling's lib.rs (its module doc is not
#      copied — the FORMAT_VERSION const line and every re-exported name
#      are carried in the template below and pinned against the sibling
#      by tests/ledgerplane_parity.rs, so a sibling bump fails loud).
#
# Drift law: behavior is pinned by tests/ledgerplane_parity.rs — committed
# known-answer vectors run everywhere (a public clone needs no sibling);
# the sibling-present half byte-compares every vendored file against the
# canonical source. A FORMAT_VERSION bump in the sibling is a RE-VENDOR +
# re-pin event in the same commit (the wire law).

set -eu

REPO="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
SIBLING="$REPO/../riir-ledger"
SIBLING_SRC="$SIBLING/src"
DEST="$REPO/src/ledgerplane"

MODE="${1:-write}"
case "$MODE" in
    write) ;;
    --check) ;;
    *) echo "usage: $0 [--check]" >&2; exit 2 ;;
esac

if [ ! -d "$SIBLING_SRC" ]; then
    echo "✗ sibling missing: $SIBLING_SRC (the canonical home; read-only to this script)" >&2
    exit 1
fi

SHA="$(git -C "$SIBLING" rev-parse HEAD)"
SHORT_SHA="$(printf '%s' "$SHA" | cut -c1-12)"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/ledgerplane_vendor.XXXXXXXX")"

header() {  # header <sibling-file> <dest-name>
    printf '// VENDORED — src/ledgerplane/%s (instinct issue 030 disposition, 2026-11-20):\n' "$2"
    printf '// canonical home ../riir-ledger/src/%s @ %s. Written by\n' "$1" "$SHA"
    printf '// scripts/vendor_ledgerplane.sh (the ONLY sanctioned write path); never\n'
    printf '// hand-edit either side. Drift pinned by tests/ledgerplane_parity.rs.\n'
}

gen() {  # gen <sibling-file> <dest-name>
    src=$1
    dst=$2
    { header "$src" "$dst"; cat "$SIBLING_SRC/$src"; } > "$TMP/$dst"
    if [ "$src" = "dto.rs" ]; then
        # Sanctioned divergence 2: the doc-path adaptation.
        sed 's|crate::FORMAT_VERSION|crate::ledgerplane::FORMAT_VERSION|g' \
            "$TMP/$dst" > "$TMP/$dst.adapted"
        mv "$TMP/$dst.adapted" "$TMP/$dst"
    fi
}

gen dto.rs dto.rs
gen attest.rs attest.rs
gen privacy.rs privacy.rs

# mod.rs — adapted from the sibling's lib.rs (sanctioned divergence 3).
{
    printf '// VENDORED — src/ledgerplane/mod.rs (instinct issue 030 disposition, 2026-11-20):\n'
    printf '// adapted from the canonical home ../riir-ledger/src/lib.rs @ %s. Written by\n' "$SHA"
    printf '// scripts/vendor_ledgerplane.sh (the ONLY sanctioned write path); never\n'
    printf '// hand-edit either side. Drift pinned by tests/ledgerplane_parity.rs.\n'
    cat <<'EOF'
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
EOF
} > "$TMP/mod.rs"

if [ "$MODE" = "--check" ]; then
    status=0
    for f in mod.rs dto.rs attest.rs privacy.rs; do
        if diff -u "$DEST/$f" "$TMP/$f" > "$TMP/$f.diff" 2>&1; then
            echo "ok: $f"
        else
            echo "✗ drift: src/ledgerplane/$f differs from the canonical sibling @ $SHORT_SHA" >&2
            sed -n '1,25p' "$TMP/$f.diff" >&2
            status=1
        fi
    done
    rm -rf "$TMP"
    if [ "$status" -eq 0 ]; then
        echo "vendor_ledgerplane --check: PASS (@ $SHORT_SHA)"
        exit 0
    fi
    echo "fix: scripts/vendor_ledgerplane.sh (re-vendor + re-pin tests/ledgerplane_parity.rs in the same commit)" >&2
    exit 1
fi

mkdir -p "$DEST"
for f in mod.rs dto.rs attest.rs privacy.rs; do
    cp "$TMP/$f" "$DEST/$f"
done
rm -rf "$TMP"
echo "vendor_ledgerplane: wrote src/ledgerplane/ from ../riir-ledger @ $SHORT_SHA"
