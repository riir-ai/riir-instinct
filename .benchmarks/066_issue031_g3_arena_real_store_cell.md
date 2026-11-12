# Bench 066 — issue 031 G3: the adoption gate's arena-side real-store cell (the swap verdict against real lanes on real labelled traffic)

**Status:** EXECUTED 2026-11-12 (amo, release build at the landing HEAD; instrument `examples/adoption_swap_g3_arena.rs` + the `from_parts_opt` widening; three full runs byte-identical verdicts, deterministic ×3). **Verdict: the G3 polarity bar PASSES on the real regression pair — the gate refuses the measured regression and the refusal tracks the measured accuracy delta; the degenerate all-abstain-incumbent shape is recorded verbatim on both orderings (AGREE); the persisted-window design decision is DECLINED (in-memory, reasons in the record).** The gate stays opt-in — this cell is the consumer-side G3 evidence, never a promotion.

## The lane pair (real, decisive, in-repo)

sst5, the seat's cal slice (the first 32 cases = the window cap), two REAL lanes constructed at EXPLICIT arms (the manifest row cannot pin two postures — the `from_parts_opt` widening is the named seam):

- **SPECIALIST**: `sst5_winner_v1.bin` at arm A1 (`SuiteServer::<5>::from_bytes(.., Arm::A1)`); the receipts carry `specialist_scores` (the specialist decides).
- **MODELLESS**: `HybridLane::ReflexOnly` at arm A0 (`from_parts_opt(spec: None, .., Arm::A0)`); `probabilities`, never a specialist consult.

## The measured pair

| lane | window acc (32 cal cases) | abstained |
|---|---|---|
| specialist (A1) | **12/32 = 0.375** | 0 |
| modelless (A0 ReflexOnly) | **0/32 = 0.000** | 32 |

⚠ The modelless leg's 0.000 is the SERVED posture truth, not a defect: sst5's fitted fused-gate (0.005/0.417) abstains on the whole cal front — the harness's own accuracy rows (A0 0.2900 cal / 0.3967 test, Benches 004/005) are FORCED reads; the serve lane's gate abstains for real. The pair is therefore maximally decisive for the regression read.

## The legs + verdicts

| leg | incumbent → challenger | verdict | probe delta |
|---|---|---|---|
| 1–2 | specialist → modelless | **409 `swap_regret`** — retained 0.000, repeated 0.000, challenger score 0.5000 (the abstainer answers nothing the specialist accepted) | 32/32 |
| 2b | the same regression, `force:true` | 200 **forced** (the operator seam) | — |
| 4 | modelless (all-abstain, post-reset) → specialist | **200 advanced** | 32/32 |
| 5 | modelless (all-abstain, fresh-boot edge) → specialist | **200 advanced** | 32/32 |

**P1 (the regression read) PASS**: the gate refused the measured regression; the refusal body's numbers are the honest shape — retained 0.000 (nothing answered the accepted window), repeated 0.000 (the challenger answered nothing at all), challenger score 0.5000 vs an incumbent whose rings carry 12 accepted outcomes — and the measured delta (0.375 vs 0.000) agrees with the verdict.

**P2 (the degenerate shape) PASS as a record**: on the all-abstain incumbent BOTH orderings ADOPT the answering specialist — the orderings AGREE (post-reset 200 == fresh-boot 200; the first draft's Keep-on-this-shape came from the v1 stub-edge arithmetic where the challenger abstained everything, the exact inverse probe). Reading: with the incumbent rings carrying ZERO accepted evidence, the challenger's score rides its own answers, and the adopt is the RIGHT product outcome here (the specialist IS the better lane — the gate refused it in leg 2 only when the incumbent had real accepted evidence to defend). No polarity claim is made on this shape; both bodies recorded verbatim.

**W (the wire) PASS**: the counting-lane wrapper proves every verdict leg probed the FULL window (delta 32 on all three verdict swaps); the force seam installed the regression lane exactly once.

## The persisted-window design decision: DECLINED

The window stays IN-MEMORY (32/suite, process-local): a swap verdict only ever reads the lane that actually served the recent traffic in THIS process; the A2 install path needs no history across restarts (the first swap after a boot sees an empty window = NoEvidence = install, the designed cold-boot posture — leg 5 is that posture on a fresh edge and it behaves). Persisting windows would add a data class at rest (a serving store) for a verdict that is advisory beside A2. Re-open only if a consumer needs swap protection across process restarts.

## Disclosures

- sst5 is licence-DEMOTED from serving (Issue 023 T3) — this is a MEASUREMENT lane; no serving posture changed, the manifest row lives in the cell's temp dir only.
- The arm labels in leg receipts are the ROW's (A0), not the lane's construction arm — the cell's lanes are named by MODE (the loader's construction), disclosed here; the receipts' `specialist_scores`/`probabilities` are the honest per-leg tell.
- The producer-side tie/crossover arithmetic on an empty accepted ring is Bench 150's mechanism; this cell's leg 4/5 land on the adopt side on both orderings and make no producer-claim.

## Instruments

- `examples/adoption_swap_g3_arena.rs` (+ its `[[example]]` required-features row): the whole cell; three runs byte-identical verdicts.
- `src/server.rs` `from_parts_opt` widened pub (the explicit-arm seam; the doc comment names this consumer).
- Deterministic ×3 on amo (quiet, AC); clippy `-D` clean on touched files; default + `adoption_gate` test postures green (113/118 lib rows).

Session: zed-3e14d67c, epoch 1794453900 (issue 031's arena-side G3 claim).
