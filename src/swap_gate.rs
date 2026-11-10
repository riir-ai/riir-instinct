//! Issue 031 — the adoption/retention regression gate at the swap seat
//! (the consumer half of katgpt-rs Plan 633 T2; the producer is
//! `katgpt-core::adoption_gate`, feature-forwarded, never forked).
//!
//! # What it does
//!
//! The A2 swap law advances on the ARTIFACT's epoch tag alone — decisions
//! are never compared. This module adds the DECISION axis beside it: the
//! serving lane's recent single-question decisions form a bounded
//! per-suite window (32 entries, the producer `DedupRing`'s bound); at a
//! swap install the CHALLENGER lane is probed over that window
//! (in-process `decide` calls — no wire), and the producer gate scores
//! `retained` (previously-accepted cases the challenger answers the same
//! way) and `repeated` (previously-abstained cases the challenger now
//! answers). A `Keep` verdict refuses the swap with a typed
//! `swap_regret` class unless the operator forces it; `Adopt` and
//! `NoEvidence` install normally.
//!
//! # The laws this never touches
//!
//! ⛔ A2 monotonic apply is LOAD-BEARING and untouched: `check_epoch_tag`
//! runs exactly as before; the gate is an EDGE-side pre-install advisory
//! (the same seam class as the existing advisory pre-gate), it can only
//! KEEP — never re-order, roll back, or weaken the epoch check. `force`
//! bypasses the verdict (the operator seam, logged at the edge).
//!
//! # Posture (the env-switch law)
//!
//! Feature `adoption_gate` (forwards `katgpt-core/adoption_gate`) +
//! promote-only env `INSTINCT_ADOPTION_GATE` (exact `1`/`true`/`on`,
//! latched once per process — the FM_BUDGET_METAL_LATCH precedent).
//! Unset = the whole module is silent: no capture, no probe, no verdict —
//! swap behavior byte-identical (G2). Calibration:
//! `INSTINCT_ADOPTION_GATE_ALPHA` (default the producer's
//! [`katgpt_core::adoption_gate::DEFAULT_ADOPTION_ALPHA`]) and
//! `INSTINCT_ADOPTION_GATE_STRIDE` (the decontamination stride, default
//! [`katgpt_core::adoption_gate::DEFAULT_DECONTAMINATION_STRIDE`]).
//!
//! Raw/latent posture
//!
//! The window stores the raw request `state` strings + options of
//! RECENT SERVE TRAFFIC only (already on the wire this process served),
//! bounded at 32 per suite — no new data class crosses any boundary.

#![cfg(feature = "adoption_gate")]

use katgpt_core::adoption_gate::{AdoptionDecision, AdoptionGate, OutcomeDigest};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};

/// The per-suite window bound — the producer `DedupRing`'s own capacity
/// (the A7 budget law: bounded state, no unbounded growth in the hot
/// path).
pub(crate) const WINDOW_CAP: usize = 32;

/// One captured decision: enough to RE-ASK the case at probe time (the
/// state + options) plus the outcome bits the window and the gate rings
/// need.
#[derive(Clone, Debug)]
pub(crate) struct WindowEntry {
    pub state: String,
    pub options: Option<Vec<String>>,
    /// Served (not abstained) — the capture-time accepted bit.
    pub accepted: bool,
    /// BLAKE3 over the served decision's canonical form — the
    /// ingest/probe digest unit.
    pub decision: OutcomeDigest,
}

/// One suite's gate bookkeeping: the re-ask window + the producer gate
/// whose rings accumulate the lane's recent outcomes (the outgoing
/// lane's T window at swap time).
#[derive(Debug)]
pub(crate) struct SuiteGate {
    pub window: Vec<WindowEntry>,
    pub gate: AdoptionGate,
}

impl SuiteGate {
    fn new(stride: usize) -> Self {
        Self {
            window: Vec::with_capacity(WINDOW_CAP),
            gate: AdoptionGate::with_stride(stride),
        }
    }

    /// Capture one served decision (single-question form only — the
    /// multi form's per-question capture is a follow-up). Oldest-first
    /// eviction at the cap.
    pub(crate) fn capture(&mut self, entry: WindowEntry, stride: usize) {
        self.gate.ingest(entry.accepted, entry.decision);
        if self.window.len() >= WINDOW_CAP {
            self.window.remove(0);
        }
        self.window.push(entry);
        let _ = stride;
    }
}

/// The per-suite gate table (`SrvState`'s feature-gated field).
#[derive(Debug)]
pub(crate) struct SwapGateState {
    pub suites: HashMap<&'static str, SuiteGate>,
    pub stride: usize,
}

impl SwapGateState {
    pub(crate) fn new(stride: usize) -> Self {
        Self {
            suites: HashMap::new(),
            stride,
        }
    }

    /// The named suite's gate, created on first capture.
    pub(crate) fn suite_gate(&mut self, suite: &'static str) -> &mut SuiteGate {
        let stride = self.stride;
        self.suites
            .entry(suite)
            .or_insert_with(|| SuiteGate::new(stride))
    }

    /// Drop a suite's window + rings (the post-install reset: the NEW
    /// lane's window starts fresh, so the NEXT swap scores against the
    /// lane that actually served).
    pub(crate) fn reset(&mut self, suite: &str) {
        self.suites.remove(suite);
    }
}

// ── the env latch (the FM_BUDGET_METAL_LATCH precedent: one read per
//    process, cached; the env-switch law's promote-only exact literals) ──

static ARMED_LATCH: AtomicU8 = AtomicU8::new(0); // 0 = unset, 1 = off, 2 = on

/// The promote-only arming surface (`INSTINCT_ADOPTION_GATE`): exact
/// `1`/`true`/`on` arms; unset or any other value = OFF — the whole
/// module silent (no capture, no probe, no verdict).
pub(crate) fn swap_gate_armed() -> bool {
    match ARMED_LATCH.load(Ordering::Relaxed) {
        1 => return false,
        2 => return true,
        _ => {}
    }
    let on = matches!(
        std::env::var("INSTINCT_ADOPTION_GATE").as_deref(),
        Ok("1") | Ok("true") | Ok("on")
    );
    ARMED_LATCH.store(if on { 2 } else { 1 }, Ordering::Relaxed);
    on
}

fn env_f64(name: &str, default: f64) -> f64 {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<f64>().ok())
        .filter(|v| v.is_finite() && *v >= 0.0 && *v <= 1.0)
        .unwrap_or(default)
}

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|&v| v > 0)
        .unwrap_or(default)
}

/// The calibration reads (consult-time; the swap edge is seconds-class —
/// a lane load — so an env read here is noise).
pub(crate) fn swap_gate_alpha() -> f64 {
    env_f64(
        "INSTINCT_ADOPTION_GATE_ALPHA",
        katgpt_core::adoption_gate::DEFAULT_ADOPTION_ALPHA,
    )
}

pub(crate) fn swap_gate_stride() -> usize {
    env_usize(
        "INSTINCT_ADOPTION_GATE_STRIDE",
        katgpt_core::adoption_gate::DEFAULT_DECONTAMINATION_STRIDE,
    )
}

/// The decision digest unit — the same canonical decision bytes the
/// receipt's `decision_blake3` hashes, wrapped as an `OutcomeDigest`
/// (blake3 over the hex string: deterministic, collision-free for this
/// use).
pub(crate) fn outcome_digest(decision_blake3_hex: &str) -> OutcomeDigest {
    OutcomeDigest::of(decision_blake3_hex.as_bytes())
}

/// The probe's per-case outcome: what the challenger answered for one
/// window case.
pub(crate) struct ProbeOutcome {
    pub digest: OutcomeDigest,
    pub accepted: bool,
}

/// Score the challenger against the outgoing lane's window: probe every
/// window case through `probe` (the incoming lane's own decide surface,
/// in-process), build the candidate slices, and hand them to the
/// producer's `decide`. `None` = the window is empty (`NoEvidence` shape
/// — the caller installs normally).
pub(crate) fn verdict_for_swap(
    suite_gate: &SuiteGate,
    alpha: f64,
    mut probe: impl FnMut(&WindowEntry) -> Option<ProbeOutcome>,
) -> Option<katgpt_core::adoption_gate::AdoptionVerdict> {
    if suite_gate.window.is_empty() {
        return None;
    }
    let mut candidate_accepted: Vec<OutcomeDigest> = Vec::new();
    let mut candidate_outputs: Vec<OutcomeDigest> = Vec::new();
    for entry in &suite_gate.window {
        if let Some(outcome) = probe(entry) {
            candidate_outputs.push(outcome.digest);
            if outcome.accepted {
                candidate_accepted.push(outcome.digest);
            }
        }
    }
    Some(
        suite_gate
            .gate
            .decide(&candidate_accepted, &candidate_outputs, alpha),
    )
}

/// The verdict's decision word for the swap edge's log/refusal surface.
pub(crate) fn decision_word(d: AdoptionDecision) -> &'static str {
    match d {
        AdoptionDecision::Adopt => "adopt",
        AdoptionDecision::Keep => "keep",
        AdoptionDecision::NoEvidence => "no_evidence",
    }
}

#[cfg(test)]
mod tests {
    //! Issue 031 G1/G2 fixtures — HERMETIC (the producer's synthetic
    //! digest discipline; no datasets, no serve edge). Runs at
    //! `cargo test --features adoption_gate --lib`.

    use super::*;

    /// Deterministic distinct digest per index (the producer fixture
    /// discipline — a canonical byte prefix per index).
    fn digest_of(index: usize) -> OutcomeDigest {
        OutcomeDigest::of(format!("case-{index}").as_bytes())
    }

    fn entry(index: usize, accepted: bool) -> WindowEntry {
        WindowEntry {
            state: format!("case-{index}"),
            options: None,
            accepted,
            decision: digest_of(index),
        }
    }

    #[test]
    fn window_is_bounded_and_fifo() {
        let mut sg = SuiteGate::new(1);
        for i in 0..(WINDOW_CAP + 5) {
            sg.capture(entry(i, i % 2 == 0), 1);
        }
        assert_eq!(
            sg.window.len(),
            WINDOW_CAP,
            "the window never exceeds the cap"
        );
        assert_eq!(
            sg.window[0].state, "case-5",
            "the oldest entries evicted first"
        );
        assert_eq!(
            sg.window[WINDOW_CAP - 1].state,
            format!("case-{}", WINDOW_CAP + 4),
            "the newest entry is last"
        );
    }

    #[test]
    fn gate_rings_ingest_and_dedup() {
        // Stride 2: the FIRST accepted outcome files to the gate set (T),
        // the second to the held-out audit set (the never-double-file
        // decontamination — every stride-th NEW accepted outcome → A).
        let mut sg = SuiteGate::new(2);
        sg.capture(entry(0, true), 2);
        assert_eq!(
            sg.gate.accepted().len(),
            1,
            "stride 2: the first accepted files to T"
        );
        sg.capture(entry(1, true), 2);
        assert_eq!(
            sg.gate.audit().len(),
            1,
            "stride 2: the second accepted files to A"
        );
        sg.capture(entry(2, true), 2);
        assert_eq!(
            sg.gate.accepted().len(),
            2,
            "the third accepted files to T again"
        );
        // The same decision digest again — dedup'd by the rings (never
        // re-filed, the stride counter untouched).
        sg.capture(entry(0, true), 2);
        assert_eq!(sg.gate.accepted().len(), 2, "the ring dedups by digest");
        // A rejected outcome — always to the T-rejected ring.
        sg.capture(entry(3, false), 2);
        assert_eq!(sg.gate.rejected().len(), 1);
    }

    /// G1 (replay determinism): the same window + the same challenger
    /// answers → identical verdict bits across replays — and a challenger
    /// that retains the accepted window and never repeats rejections
    /// ADOPTS, while one that answers everything the incumbent rejected
    /// KEEPs.
    #[test]
    fn replay_is_deterministic_and_the_polarity_is_pinned() {
        // Stride 4: 6 accepted outcomes → 5 file to the T-accepted ring +
        // 1 held out to the audit set; the 4 rejected all file to T — the
        // rings carry real evidence on both sides (a stride that files
        // EVERY accepted outcome to the audit set would leave the T
        // window empty = NoEvidence, the decontamination doing its job).
        let build = || {
            let mut sg = SuiteGate::new(4);
            for i in 0..10 {
                sg.capture(entry(i, i % 2 == 0), 4);
            }
            sg
        };
        // The faithful challenger: answers every case exactly as captured.
        let faithful = |e: &WindowEntry| {
            Some(ProbeOutcome {
                digest: e.decision,
                accepted: e.accepted,
            })
        };
        // The regret challenger: abstains everything the incumbent
        // accepted (retention 0), answers everything it rejected
        // (repeats 1).
        let regret = |e: &WindowEntry| {
            Some(ProbeOutcome {
                digest: e.decision,
                accepted: !e.accepted,
            })
        };
        let a = verdict_for_swap(&build(), 0.5, faithful).expect("non-empty window");
        let b = verdict_for_swap(&build(), 0.5, faithful).expect("non-empty window");
        assert_eq!(a, b, "G1: the replay is bit-identical");
        assert_eq!(a.decision, AdoptionDecision::Adopt, "full retention adopts");
        assert!(
            (a.retained - 1.0).abs() < 1e-12,
            "the faithful challenger retains everything"
        );
        let r = verdict_for_swap(&build(), 0.5, regret).expect("non-empty window");
        assert_eq!(r.decision, AdoptionDecision::Keep, "full regret keeps");
        assert!(
            r.retained.abs() < 1e-12,
            "the regret challenger retains nothing"
        );
        assert!(r.repeated > 0.0, "the regret challenger repeats rejections");
    }

    /// The empty window declines-first (the NoEvidence shape → the
    /// caller installs normally) — and a probe that fails every case
    /// (a challenger that cannot answer the window) scores repeated=0
    /// with an empty output slice, never a panic.
    #[test]
    fn empty_window_and_failed_probes_are_honest() {
        let mut sg = SuiteGate::new(2);
        assert!(
            verdict_for_swap(&sg, 0.5, |_| None).is_none(),
            "an empty window returns None (no evidence)"
        );
        // Stride 2 + one accepted outcome: the FIRST accepted files to T
        // (accepted_count 1 is not a multiple of 2), so the gate set is
        // non-empty and the consult is informed.
        sg.capture(entry(0, true), 2);
        assert_eq!(sg.gate.accepted().len(), 1);
        let v = verdict_for_swap(&sg, 0.5, |_| None).expect("non-empty window");
        assert_eq!(
            v.decision,
            AdoptionDecision::Keep,
            "decline-first: a challenger with no answers cannot beat the incumbent's own score"
        );
    }

    /// The digest unit is stable across calls and distinct per decision
    /// (the probe/capture pairing depends on it).
    #[test]
    fn outcome_digest_is_stable_and_distinct() {
        let a = outcome_digest("abc");
        let b = outcome_digest("abc");
        let c = outcome_digest("abd");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
