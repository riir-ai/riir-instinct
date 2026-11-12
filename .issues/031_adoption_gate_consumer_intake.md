# Issue 031 — T2 consumer intake: adoption/retention regression gate (LaneSlot epoch swaps)

**Status:** T2 WIRING LANDED 2026-11-10 (zed-m3-t2slot@m3 — resumed from the salvage branch `wip/salvage-i031-adoption-gate` `d682b82`; feature `adoption_gate`, G1+G2+G4 green, both clippy postures `-D warnings` clean; the landing commit is this file's parent) — **G3 WIRE GATE LANDED 2026-11-12** (zed-3e14d67c@amo — `examples/adoption_swap_g3.rs` + its own `[[example]]` required-features row, `cargo run --release --example adoption_swap_g3 --features adoption_gate`; SIX bars green, deterministic ×3, clippy clean, default+feature test postures 113/118 green, G2 byte-identity pinned at the WIRE by bar 6: the disarmed child installs the identical regret posture with probe delta 0 — no capture, no verdict; the bars: (1) the adopt path installs, (2) the regret challenger refuses typed 409 `swap_regret`, (3) the capture-wire probe delta 5 (exactly the 5 served decisions are in the window — the vacuous-capture defense), (4) force bypass installs, (5) the post-install reset probe delta 2 — a leaked gate would probe 7 (the dead lane's 5 + the fresh 2), the dead lane's history never leaks into a later verdict, (6) the disarm face. The arena-side real-swap-regret measurement over a REAL serving store (plan 633's T3 shape — a persisted window + the persisted-vs-in-memory design decision) STAYS the open G3 unit). Intake issue for Plan 633 T2 (katgpt-rs), filed per katgpt-rs `.issues/987_cactus_neurips2026_follow_ups.md` priority-1 (AGCL form, Research 657 §A5) and katgpt-rs `.plans/633_adoption_retention_regression_gate.md`.
**Claimed:** zed-3e14d67c@amo, 1794453900, expires 2026-11-14T02:05:00Z
**Lease closed:** zed-amo-i031@amo (amo), 1794315000, expires 2026-11-12T12:00:00Z — the claimed scope (T2 consumer wiring: window capture + swap-seat verdict + manifest posture + G1/G2/G4 gates) LANDED 2026-11-10 by zed-m3-t2slot@m3 (the takeover-cleanup salvage branch `wip/salvage-i031-adoption-gate` resumed; full record in the Status line + WIP paragraph below). G3 (the real-window GOAT, plan 633's T3 shape) stays OPEN, unclaimed. (Fold repair 2026-11-12: the original **Claimed:** line was malformed — parser-blinding for this repo; session/epoch/expiry carried verbatim here.)
**WIP preserved 2026-11-10 (takeover cleanup):** the claiming session's uncommitted mid-edit state is salvaged on branch `wip/salvage-i031-adoption-gate` (`d682b82` — swap_gate.rs + serve_edge/lib/Cargo wiring, 473 insertions) — NOT landed: default-posture gating unfinished (the module compiled ungated, E0432/E0433), feature posture 2 errors (E0425 `state`, E0308 let-else), G1/G2/G4 gates unwritten. **RESUMED + LANDED 2026-11-10 by zed-m3-t2slot@m3** (the fixes: `#![cfg(feature)]` on the module, `state`→`srv`, the let-else → Option::map consult shape, `let mut lane` with the cfg_attr expect(unused_mut), `lane.server.decide` on the loaded challenger, the post-install window `reset`, and the G1 fixture strides corrected — stride 1 files EVERY accepted outcome to the audit ring, emptying the T window to NoEvidence; the fixtures now use strides that leave the T rings informed).

## What exists (producer side, landed `cf65c58a7` 2026-11-08)

`katgpt-core::adoption_gate` behind the opt-in feature `adoption_gate` (modelless, zero new deps): `OutcomeDigest` (blake3 `[u8; 32]`), `DedupRing` (fixed 32-slot FIFO membership set), `Decontaminator` (never-double-file stride splitter), `AdoptionGate::decide` (score `= (1−α)·retained + α·(1−repeated)`, ties→challenger, `NoEvidence` declines-first, always-from-base purity pinned). 9 fixture tests green, clippy both postures clean.

## What this repo consumes (the T2 wiring, ungated design sketch)

**LaneSlot epoch swaps** (law A2 — monotonic atomic hot-swap of whole artifacts): a swap currently advances on `check_epoch_tag` alone — correctness of the ARTIFACT, never regressions of the DECISIONS. The AGCL gate adds the decision axis:

- On swap install, score the new lane's decisions against the outgoing lane's recent window: retained = previously-answered cases the new lane answers the same (accepted) way; repeated = previously-abstained/declined cases the new lane now answers (a "repeat of previously-rejected" analog for the decision domain).
- Window source: the serving lane's recent decisions (the arena/serve seat already records per-case outcomes; the winners are t20k-trained — the window is the natural seat). `DedupRing`'s 32-slot bound fits the A7 budget law (no unbounded state in the hot path).
- α + stride ride the manifest row or `INSTINCT_*` envs (env-switch law: promote-only opt-in arms, exact literals; byte-identical off).
- Suites where the swap gate matters FIRST: the ESC-composed seats (`massive_intent_en`, `banking77` — the Bench 089 banking77 contamination caveat is exactly a swap-regret shape: the composed arm's rate axis transferred out-of-window; a retention gate would have DEMOTED it automatically).

## Bars (pre-registered now, before any wiring)

- G1: replay determinism — same window + same swap candidate → identical verdict bits (producer purity pin carries).
- G2: gate-off byte-identity — swap behavior today, unchanged (A2 monotonicity untouched; the gate only ADDS a keep/adopt verdict beside it, never weakens the epoch check).
- G3 (GOAT, = plan 633 T3): gate-on vs gate-off on real serving decisions over a frozen window read; may fail honestly.
- ⛔ The monotonic-apply law (A2: advance/idempotent, fork+downgrade refused) is LOAD-BEARING and stays untouched — the adoption gate never re-orders or rolls back a swap; it can only advise KEEP on the NEXT swap.

## Refs

- katgpt-rs `.plans/633_adoption_retention_regression_gate.md` (T2 = this issue) · katgpt-rs `.issues/987` priority-1 · Research 657 §A5 (AGCL)
- This repo: Proposal 001 (arsenal laws), Bench 086/087 (ESC seats), Bench 089 (banking77 swap-regret caveat — the motivating shape)
