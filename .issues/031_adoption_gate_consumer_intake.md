# Issue 031 — T2 consumer intake: adoption/retention regression gate (LaneSlot epoch swaps)

**Status:** T2 WIRING LANDED 2026-11-10 (zed-m3-t2slot@m3 — resumed from the salvage branch `wip/salvage-i031-adoption-gate` `d682b82`; feature `adoption_gate`, G1+G2+G4 green, both clippy postures `-D warnings` clean; the landing commit is this file's parent) — the issue stays OPEN for G3 (the real-window GOAT, plan 633's T3 shape: the serving-decision store is the in-memory window; a PERSISTED window + the arena-side real swap-regret measurement is the next unit's design decision). Intake issue for Plan 633 T2 (katgpt-rs), filed per katgpt-rs `.issues/987_cactus_neurips2026_follow_ups.md` priority-1 (AGCL form, Research 657 §A5) and katgpt-rs `.plans/633_adoption_retention_regression_gate.md`.
**Claimed:** zed-amo-i031@amo (amo), 1794315000, expires 2026-11-12T12:00:00+00:00 — T2 consumer wiring (window capture + swap-seat verdict + manifest posture + G1/G2/G4 gates) this session; G3 (the real-window GOAT, plan 633's T3 shape) deferred to its own may-fail unit once the wiring names the serving-decision store. No collision: plan 633's live claim (zed-86011726@m3) owns the producer plan's T3 refine-store GOAT, not this consumer file. **(Lease honored post-expiry-of-session: the claiming amo session was taken over 2026-11-10 (the WIP note below); the m3 idle loop zed-m3-t2slot@m3 resumed the SALVAGE BRANCH the takeover cleanup named as the resume point — the claim's own scope, the salvage's own design, no re-design.)**
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
