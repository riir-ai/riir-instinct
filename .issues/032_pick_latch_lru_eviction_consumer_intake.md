# Issue 032 — T1 consumer intake: selection hysteresis latch (`pick_latch`) — the A7 LRU eviction seat

**Status:** OPEN — intake issue, no wiring yet; consumer half of katgpt-rs Issue 987's selection-hysteresis lead (T0 landed `c9bdb533b`, 2026-11-09: `katgpt-core/src/pick_latch.rs`, opt-in feature `pick_latch`, catalog entry 165, 15 gate-mechanics fixtures green). Twin intake: riir-reflex Issue 116 (the `pick_domain` seat).
**Claimed:** zed-amo-i032@amo (amo), 1794300000, expires 2026-11-12T08:00:00+00:00 — T0 seam read + T1 wiring + T2 gates this session; T3's real-serve-stream GOAT may need a serve window (honest negative allowed).

## Producer (landed; zero new deps; byte-identical when off)

`PickLatch::decide` + `LatchDecision`: the gate is `exact_sigmoid((challenger − held)/temperature) ≥ threshold` — the committed-value-path form, so the additive margin is exact (`temperature·lnit(threshold)`; ln 3 ≈ 1.099 score units at the defaults). State = the held index ONLY (stationary surface). Abstain/empty ⇒ reuse; NaN holds; clamped construction, no panic path; zero new deps; byte-identical when the feature is off. Sigmoid only, never softmax (house law).

## What this repo would consume

The A7 eviction seat (the R5 LRU policy's release pick): when residency is at/over `INSTINCT_LRU_CAPACITY` and a lazy trigger fires, WHICH Ready lazy lane gets released. Today the coldest lane wins on every evaluation; under idle-time noise (lanes clustered near the floor) the victim can thrash between triggers. The latch holds the previously-chosen victim unless the challenger is meaningfully colder by the sigmoid margin; abstain ⇒ reuse.

**Load-bearing neighbors the latch must NOT touch:** `INSTINCT_LRU_MIN_IDLE_SECS` (the eviction-eligibility floor — a lane is a candidate only when idle ≥ the floor, waived at/over the `OVERSHOOT_CAP` 16 ceiling) and `OVERSHOOT_CAP` itself keep their exact semantics — the latch only steadies WHICH eligible lane is released, never WHETHER one is. The slot lock + monotonic whole-artifact swap (A2) and the hoarding gate are untouched; the allocation-free hot path stays alloc-free (the `tests/g4_alloc.rs` law).

## Bars (pre-registered before any wiring)

- **G1 (purity):** a seeded synthetic residency/idle stream replays through the eviction seam bit-identical to the producer fixture verdicts — same inputs, same victims.
- **G2 (no regression):** feature-off byte-identity — eviction behavior identical to today; the serve gates + `tests/g4_alloc.rs` green unchanged.
- **G3 (the GOAT):** gate-on vs gate-off on a real serve stream (the budget lazy-load posture) — victim flip-rate reduction (thrash) WITHOUT residency overshoot beyond `OVERSHOOT_CAP` and without a release-rate drop that would breach capacity; `decline` (keep the held victim) is the default answer forever; no promotion talk before G3; may fail honestly.

## Tasks

- [ ] T0 claim + seam read (the release path in the A7 policy; confirm the victim choice is the only latch seat and the idle floor stays upstream of it)
- [ ] T1 wiring behind opt-in `pick_latch` (forward the katgpt-core feature; the seam law — widen this repo's own pub surface, never fork the open crate)
- [ ] T2 G1 + G2 gates
- [ ] T3 G3 GOAT on a real serve stream; honest negative allowed → close with the verdict either way

## Refs

- katgpt-rs Issue 987 (selection hysteresis row; T0 `c9bdb533b`) · katgpt-rs Research 657 §A1 (Temporal MoE analogy — OURS, not a paper claim)
- riir-instinct Proposal 001 (A7 lazy + budgeted eviction) + Issue 026 (the eviction hysteresis floor)
- riir-refine Issue 220 (the twin intake shape: adoption-gate consumer wiring)
