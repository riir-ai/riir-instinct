// VENDORED — src/ledgerplane/dto.rs (instinct issue 030 disposition, 2026-11-20):
// canonical home ../riir-ledger/src/dto.rs @ 687aa8ffd7d897950fd6021b58793901ef9cd94e. Written by
// scripts/vendor_ledgerplane.sh (the ONLY sanctioned write path); never
// hand-edit either side. Drift pinned by tests/ledgerplane_parity.rs.
//! The wire DTOs (riir-refine Proposal 026 §2 — the wire table — plus the
//! Proposal 027 drain vocabulary).
//!
//! Envelopes are tag-style: every struct carries `format_version` as its
//! first field — the version tag both sides read before any other work
//! ([`FORMAT_VERSION`](crate::ledgerplane::FORMAT_VERSION)). Refusals are TYPED: the
//! [`ErrorEnvelope`] pairs a stable machine `code` token with the human
//! sentence. Field order in each struct is the wire order; the golden
//! fixtures (`tests/wire_golden.rs`) pin it byte-exactly.
//!
//! The only client-supplied identifier across every DTO is
//! `client_uuid` (T16.1 §1). No hardware, hostname, MAC, or OS machine
//! GUID field exists anywhere in this file — the T16.1 §2 refusal list
//! is enforced by shape (`tests/privacy_law.rs`), not by policy.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Canonical JSON: re-emit a serialized value with every object's keys in
/// BTreeMap (lexicographic) order and no insignificant whitespace,
/// independent of the `serde_json` feature flags a consuming build may
/// enable (a `preserve_order` consumer cannot change this output).
///
/// This is the value half of the signing rule: the client signs
/// `format_version || sorted-key canonical JSON of the body`; the server
/// verifies by re-deriving the same bytes from the decoded body.
pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    let v = serde_json::to_value(value)?;
    let mut out = String::new();
    write_canonical(&v, &mut out)?;
    Ok(out)
}

fn write_canonical(v: &serde_json::Value, out: &mut String) -> Result<(), serde_json::Error> {
    match v {
        serde_json::Value::Null => out.push_str("null"),
        serde_json::Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        serde_json::Value::Number(n) => out.push_str(&n.to_string()),
        serde_json::Value::String(s) => out.push_str(&serde_json::to_string(s)?),
        serde_json::Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out)?;
            }
            out.push(']');
        }
        serde_json::Value::Object(map) => {
            let sorted: BTreeMap<&str, &serde_json::Value> =
                map.iter().map(|(k, val)| (k.as_str(), val)).collect();
            out.push('{');
            for (i, (k, val)) in sorted.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(k)?);
                out.push(':');
                write_canonical(val, out)?;
            }
            out.push('}');
        }
    }
    Ok(())
}

/// The canonical signing payload for a signed request: `format_version`
/// as 4 bytes big-endian, immediately followed by the sorted-key
/// canonical JSON of the body (see [`canonical_json`]) — no separator,
/// no trailing newline. The client signs exactly these bytes; the server
/// re-derives them from the decoded body before verifying.
fn signing_payload<T: Serialize>(
    format_version: u32,
    body: &T,
) -> Result<Vec<u8>, serde_json::Error> {
    let mut bytes = Vec::with_capacity(4 + 128);
    bytes.extend_from_slice(&format_version.to_be_bytes());
    bytes.extend_from_slice(canonical_json(body)?.as_bytes());
    Ok(bytes)
}

/// The typed-refusal envelope. `code` is a stable machine token the
/// client branches on; `message` is the human sentence for display.
/// A non-JSON body is NOT an `ErrorEnvelope` — decode refuses, and the
/// client can rely on that refusal (never a silent partial decode).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ErrorEnvelope {
    pub format_version: u32,
    pub code: String,
    pub message: String,
}

/// `POST /points/join` — bind this install's client UUID to a GitHub
/// identity + the account key, signed by the account key (the token from
/// the existing GitHub device flow, `read:user`). Idempotent per
/// `(github_id, account_pubkey)` server-side.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct JoinRequest {
    pub format_version: u32,
    pub client_uuid: String,
    pub account_pubkey_hex: String,
    pub github_login: String,
    pub github_id: u64,
}

impl JoinRequest {
    /// The canonical signing payload for this join: `format_version || `
    /// `sorted-key canonical JSON of the body` (4-byte big-endian version
    /// prefix, lexicographic key order, no insignificant whitespace, no
    /// trailing newline). Sign these bytes with the account key; the
    /// server re-derives them from the decoded body before verifying.
    pub fn signed_message_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        signing_payload(self.format_version, self)
    }
}

/// The joined posture after a successful (or idempotent) `/points/join`.
/// `epoch` is the settle epoch the binding lands in — a mid-epoch join
/// appears on the board at the NEXT settle (stated, not hidden).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct JoinResponse {
    pub format_version: u32,
    pub joined: bool,
    pub epoch: u64,
}

/// `POST /points/leave` — unbind the binding row server-side, stop all
/// rank eligibility. Zero data flows afterwards: the client simply never
/// calls again — there is no goodbye round-trip dependency.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LeaveRequest {
    pub format_version: u32,
    pub client_uuid: String,
    pub account_pubkey_hex: String,
}

impl LeaveRequest {
    /// The canonical signing payload for this leave (the same rule as
    /// [`JoinRequest::signed_message_bytes`]).
    pub fn signed_message_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        signing_payload(self.format_version, self)
    }
}

/// The leave acknowledgement: `epoch` is the settle epoch from which the
/// identity stops being rank-eligible. Local use continues anonymous.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LeaveResponse {
    pub format_version: u32,
    pub left: bool,
    pub epoch: u64,
}

/// `GET /points/balance` — the caller's own current contribution-points
/// balance + the settle epoch + rank eligibility (joined identities
/// only). Never another account's rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct BalanceResponse {
    pub format_version: u32,
    pub client_uuid: String,
    pub points_epoch: u64,
    pub balance: u64,
    pub rank_eligible: bool,
    /// Additive (Proposal 027): the SERVED drain constants for the
    /// client's hosted-lane fair-use pre-check. `None` (rendered
    /// `null`, decoded `None` when absent) from a server that has not
    /// ratified the drain config — and the client pre-check is INERT
    /// when absent (the constants are served, never compiled: a
    /// ratified re-tune never needs a client release).
    #[serde(default)]
    pub drain_state: Option<DrainState>,
}

/// The served drain constants (Proposal 027 — the numbers table). All
/// three are deployment config on the service, carried to the client so
/// the LOCAL pre-check can mirror the seat's enforcement cheaply:
///
/// - `allowance_remaining_today` — free hosted calls left today (the
///   seat's counter is the ledger; this is the client's courtesy read).
/// - `billable_ceiling_today` — billable hosted calls left today (the
///   hard daily ceiling `B` minus what the seat has counted).
/// - `rate_served` — the billable-day fee `R`; the client compares its
///   cached balance against THIS before predicting a billable allow.
///
/// Never metered surfaces ride outside this struct: local `fix` is
/// always free (binding law 1), and a client that never reads
/// `/points/balance` simply runs the inert pre-check.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DrainState {
    pub allowance_remaining_today: u64,
    pub billable_ceiling_today: u64,
    pub rate_served: u64,
}

/// One leaderboard row: handle + points ONLY. No client UUID, no GitHub
/// numeric id, no repo name — the board redaction law (Proposal 026 §5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RankEntry {
    pub handle: String,
    pub points: u64,
}

/// `GET /points/rank` — the public top-ten board (server-truncated) plus,
/// for a SIGNED caller, the caller's own rank and points.
///
/// `own_rank` / `own_points` are `None` (rendered `null`) for anonymous
/// callers — the anti-Sybil rule: a rotating token could farm the board,
/// so anonymous callers get the public aggregate and a local-stats
/// pointer, never a rank.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RankResponse {
    pub format_version: u32,
    pub epoch: u64,
    pub board: Vec<RankEntry>,
    pub own_rank: Option<u64>,
    pub own_points: Option<u64>,
}

/// One redaction-policy-scoped aggregate: a rule domain + its
/// contribution count. Counts + domains only — never repo names, never
/// per-user rows (the `/points/info` scope law).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DomainCount {
    pub domain: String,
    pub contributions: u64,
}

/// `GET /points/info` — the public aggregate: top contributed-to rule
/// domains + coarse totals. Redaction-policy scope law: counts + domains
/// ONLY — no repo names, no user ids, never per-user rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct InfoResponse {
    pub format_version: u32,
    pub epoch: u64,
    pub top_domains: Vec<DomainCount>,
    pub total_contributors: u64,
    pub total_contributions: u64,
}

/// The per-epoch anchor publication (on the existing status/explorer
/// JSON): the chunk-safe epoch root hex + epoch number + row count.
/// Roots ONLY — no client UUIDs, no GitHub ids in any published anchor
/// or stat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct EpochAnchor {
    pub format_version: u32,
    pub epoch: u64,
    pub root_hex: String,
    pub leaf_count: u64,
}

/// One seat-side usage day (Proposal 027 — the seat→plane aggregate):
/// the hosted seat's counted calls for ONE account on ONE UTC day. The
/// seat is the meter of record; the points plane debits from these rows
/// at settle. `account_id_hex` is `BLAKE3(pubkey)` hex — pseudonymous,
/// never hardware-derived (the T16.1 law); `utc_day` is `YYYY-MM-DD`
/// UTC. Idempotency is `(account_id_hex, utc_day)` — a replayed row is
/// a no-op at the plane, never a double debit. Transport
/// authentication is service-to-service (a bearer token at the route)
/// and deliberately OUTSIDE this DTO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DrainUsageRow {
    pub account_id_hex: String,
    pub utc_day: String,
    pub free_calls: u64,
    pub billable_calls: u64,
}

/// The seat's daily export: a batch of [`DrainUsageRow`]s POSTed to the
/// points plane's ingest route. Rows within one export are distinct by
/// `(account_id_hex, utc_day)`; the plane processes each row exactly
/// once regardless of export replay. Integer counts only — no floats
/// anywhere near the money path (the dao law).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DrainUsageExport {
    pub format_version: u32,
    pub rows: Vec<DrainUsageRow>,
}

/// The service-to-service posture read (Proposal 027): the points
/// plane's answer to the hosted seat's usage-state read — whether the
/// account is JOINED (GitHub-anchored, the personal-allowance class)
/// and its current balance. Bearer-authenticated at the route (a
/// service token, never a user signature); the seat caches the answer
/// behind a TTL — the ENFORDED-side cache whose bounded staleness is
/// disclosed in Proposal 027 caveat 2. A failed/absent answer falls
/// back to the unjoined posture, never an error.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UsageState {
    pub format_version: u32,
    pub account_id_hex: String,
    pub joined: bool,
    pub balance: u64,
}

// ── the contribution feed (issue 204 — the owner's auto-send law) ─────────

/// Ceiling: rule-outcome entries per contribute request (the fixstat
/// lane's per-row rule cap — one fix run cannot touch more distinct
/// rules than this).
pub const CONTRIBUTE_RULES_MAX: usize = 128;
/// Ceiling: one rule/outcome name's UTF-8 length (the lint/rule token
/// bound — the same class as the fixstat lane's name cap).
pub const CONTRIBUTE_NAME_MAX: usize = 64;
/// Ceiling: the toolchain descriptor's UTF-8 length (`rustc --version`'s
/// first line; may be empty).
pub const CONTRIBUTE_TOOLCHAIN_MAX: usize = 64;

/// One rule's fix-run outcome (issue 204): how many edits the rule
/// landed (`written`) and how many fix attempts of it declined
/// (`declined` — the posture/skip/no-fix axis). Counts ONLY — the
/// privacy floor is by shape: no spans, no source text, no file paths,
/// never a code fragment. The name is the rule/lint token itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContributeRuleOutcome {
    pub name: String,
    pub written: u32,
    pub declined: u32,
}

/// `POST /points/contribute` — a joined install's fix-run outcome feed
/// (the self-evolve evidence). Signed by the account key over the
/// canonical body (the same rule as [`JoinRequest::signed_message_bytes`]);
/// the server checks the binding is ACTIVE for the derived account (the
/// joined-identity law — anonymous sends are the `--stats` plane's
/// business, not this one) and is IDEMPOTENT per `run_id_hex` (a replay
/// is a 200 `recorded_new:false` no-op, never a double count).
///
/// The client sorts `rules` by name (and deduplicates) before signing —
/// the server refuses out-of-order or duplicate names so one canonical
/// body always produces one signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContributeRequest {
    pub format_version: u32,
    pub client_uuid: String,
    pub account_pubkey_hex: String,
    /// 64 hex chars — the 32-byte run id (BLAKE3-derived client-side;
    /// the idempotency tombstone key).
    pub run_id_hex: String,
    /// The rustc descriptor (aggregate-relevant; may be empty).
    pub toolchain: String,
    pub files_touched: u32,
    pub rules: Vec<ContributeRuleOutcome>,
    /// Aggregate corpus-coverage census (riir-dapps issue 162) — counts
    /// only, never spans/source/paths. `covered_findings` = anchors the
    /// compiled corpus spoke to; `uncovered_findings` = anchors it had no
    /// rule for (the coverage lane's UNCOVERED complement). `default` so
    /// a pre-v3 body PARSES and gets the typed `code: version` refusal
    /// (never a bare bad_json) — the wire-compat disclosure.
    #[serde(default)]
    pub covered_findings: u32,
    #[serde(default)]
    pub uncovered_findings: u32,
}

impl ContributeRequest {
    /// The canonical signing payload (the join rule: `format_version ‖ `
    /// `sorted-key canonical JSON`).
    ///
    /// # Errors
    /// Serialization failure (never in practice for populated rules).
    pub fn signed_message_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        signing_payload(self.format_version, self)
    }
}

/// The contribution ack: `recorded_new:false` on a replayed run id (the
/// idempotent no-op — the row was already counted).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ContributeResponse {
    pub format_version: u32,
    pub recorded_new: bool,
    pub epoch: u64,
}

/// The default `GET /points/fixer-rate` window (epochs, current included).
pub const FIXER_RATE_WINDOW_DEFAULT: u64 = 4;
/// The `GET /points/fixer-rate` window ceiling (a larger ask clamps).
pub const FIXER_RATE_WINDOW_MAX: u64 = 52;
/// The default `GET /points/coverage` window (epochs, current included).
pub const COVERAGE_WINDOW_DEFAULT: u64 = 4;
/// The `GET /points/coverage` window ceiling (a larger ask clamps).
pub const COVERAGE_WINDOW_MAX: u64 = 52;

/// `GET /points/fixer-rate?epochs=N` — the network fixer success-rate
/// aggregate (riir-dapps issue 160) over the contribution feed's
/// per-rule written/declined counters, summed over every rule and every
/// account in the window `[from_epoch, epoch]`. Aggregated counts ONLY
/// (the privacy law): no per-user rows, no rule names, no repo strings.
/// The rate is `written / (written + declined)`, computed by the reader
/// ([`FixerRateResponse::rate`]) — the wire carries exact integers, never
/// a rounded float; `None` when the window holds no outcomes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FixerRateResponse {
    pub format_version: u32,
    /// The current epoch (the window's inclusive top).
    pub epoch: u64,
    /// The window's inclusive bottom (`epoch + 1 - window_epochs`, floored at 0).
    pub from_epoch: u64,
    /// The window width actually served (after the clamp).
    pub window_epochs: u64,
    pub written: u64,
    pub declined: u64,
    /// Distinct joined accounts with at least one outcome in the window.
    pub contributors: u64,
}

impl FixerRateResponse {
    /// `written / (written + declined)`; `None` on an empty window.
    #[must_use]
    pub fn rate(&self) -> Option<f64> {
        let total = self.written.saturating_add(self.declined);
        match total {
            0 => None,
            t => Some(self.written as f64 / t as f64),
        }
    }
}

/// `GET /points/coverage?epochs=N` — the corpus-coverage aggregate
/// (riir-dapps issue 162) over the contribution feed's coverage counters
/// (`ContributeRequest::covered_findings`/`uncovered_findings`), summed
/// over every account in the window `[from_epoch, epoch]`. Aggregated
/// counts ONLY (the privacy law): no per-user rows, no rule names. The
/// trained share is `covered / (covered + uncovered)`, computed by the
/// reader ([`CoverageResponse::trained_share`]) — the wire carries exact
/// integers, never a rounded float; `None` when the window holds no
/// census rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct CoverageResponse {
    pub format_version: u32,
    /// The current epoch (the window's inclusive top).
    pub epoch: u64,
    /// The window's inclusive bottom (`epoch + 1 - window_epochs`, floored at 0).
    pub from_epoch: u64,
    /// The window width actually served (after the clamp).
    pub window_epochs: u64,
    pub covered: u64,
    pub uncovered: u64,
    /// Distinct joined accounts with at least one census row in the window.
    pub contributors: u64,
}

impl CoverageResponse {
    /// `covered / (covered + uncovered)` — the corpus's trained share
    /// against the fleet's real workloads; `None` on an empty window.
    #[must_use]
    pub fn trained_share(&self) -> Option<f64> {
        let total = self.covered.saturating_add(self.uncovered);
        match total {
            0 => None,
            t => Some(self.covered as f64 / t as f64),
        }
    }
}
