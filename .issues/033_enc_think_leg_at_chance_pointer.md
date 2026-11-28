# Issue 033 — POINTER: the ENC think leg answers at chance on the emotion_syn pinned fixture (both backends)

**Status:** OPEN — pointer filed from the coordinating record [rethink `.issues/067_esc_t4_divergence_device_vs_drift.md`](../riir-rethink/.issues/067_esc_t4_divergence_device_vs_drift.md) (filed 2026-11-23). Unowned here — the ENC serving lane's owners claim it; rethink 067 stays the coordinating tracker.

## Finding (measured, not inferred)

The emotion_syn frozen-replay cell (rethink `esc_rho_audit_cell`, capture `.benchmarks/esc_rho_t4_capture_067_{cpu,metal}.txt` @ rethink `1db0098`) shows the ENC think leg at **whole-stream accuracy 0.1700 — chance on a 6-class suite (~0.167)** — on **BOTH backends** (CPU and Metal byte-identical on every line; the device axis is exonerated). The head attaches cleanly (worker warm, digest `5c701902d7fcc863`, head `4002c471f73f42b7`, feat_dim 7168, attach encode ok) — plumbing works, picks are wrong.

**The signature is sharp:** on the 50 gate-flagged rows the think leg beats the cheap leg by +0.12 (≈0.69 there) while whole-stream 0.17 implies ≈0.00 on the 150 unflagged rows (arithmetic implication — the per-segment instrument would confirm). Fine where the gate flags, at-zero where it doesn't reads as a **pick-mapping / serving-construction defect in the ENC path** (option order, pick-index mapping, or embedding construction differing between the served path and the gold's option space), not model quality.

**It is NEW since 2026-11-16:** bench 103 (same fixture, same box class) had healthy served acc 0.7933 with a productive think leg. The window 11-16 → 11-23 moved: riir-reflex `? → 5ec2e1b`, riir-instinct `? → 2a8968a`, katgpt-rs → `28aa0b1e4` (worktree lock re-resolved katgpt-core 0.5.0 → 0.6.1). One of these bumps broke the leg.

**Reach:** every served ENC lane inherits it — wherever the gate does not flag, the served think answer is at chance today. The ESC ρ-audit T4 verdict flipped DEMOTE as a downstream symptom (rethink bench 108; its grade evaluation is BLOCKED on this repair).

## Tasks (the owner's ladder; details in rethink 067)

- [ ] **T-A (first diagnostic):** per-SEGMENT think accuracy (flagged vs unflagged) on the pinned fixture — the +0.69/≈0.00 split is the map to the mover. The per-leg whole-stream lines already exist in the cell (rethink `7af6e7a`); the segment split is the refinement.
- [ ] **T-B:** bisect the 11-16 → 11-23 window across reflex / instinct / katgpt-core until think accuracy recovers (shas read from bench 103's record / git log, never memory). A worktree mirror isolates cleanly (sibling worktrees always collide; only the mirror resolves).
- [ ] **T-C:** the mover gets the regression row against whichever gate admitted it (demote or fix in the same commit); if material drift (LAYA cache state), name the material pin.
- [ ] **T-D:** the encoder-parity gate printed NOTHING while the think leg was at chance (`INSTINCT_ENCODER_PARITY=1` was set) — it needs a per-leg accuracy readout on the pinned fixture, loud below a floor, or it keeps passing through exactly this class of breakage.
- [ ] **T-E:** repaired → re-run the rethink cell under the capture discipline → rethink 067 T4 adjudicates the suspended bench-100 grade.

## Notes

- The breakage reproduces on every run at the current pins (deterministic per fixture) — no mirror needed to REPRODUCE, only to ISOLATE.
- Plan 020 Phase 0 (the VoI read) is DONE and unaffected; the ρ-lane grade adjudication is blocked on this repair, not on anything in the ρ arm.
- Also disclosed in rethink bench 108 (the disposition addendum): the served-vs-per-leg reading, the purge-on-read trap (job-log evidence never again), and the preflight provenance gap.
