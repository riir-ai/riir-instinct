# riir-instinct

The model-based and hybrid decision lanes — **Reflex · instinct** — the
trained sibling of [`riir-reflex`](../riir-reflex)'s modelless engine.
Design of record: `../riir-ai/.proposals/047_riir_instinct_hybrid_decision_engine.md`.
Master plan: [`.plans/001_instinct_lane_clippy_flow.md`](.plans/001_instinct_lane_clippy_flow.md).

Status: PUBLIC — the open teaching lane (the Proposal-052 carve,
2026-10-03). Code + teaching recipes are in-tree; weights never enter the
repo (see [BOUNDARY.md](BOUNDARY.md)). The trained/adaptive product lane
lives in the private sibling `../riir-rethink`.

## Build

This workspace resolves its sibling path dependencies at every `cargo`
invocation — `optional = true` does not defer resolution. A fresh clone
needs these repos checked out beside it:

- `katgpt-rs` (public)
- `riir-reflex` (public)
- `riir-reflexer` (public)
- `riir-ledger` (private) — required to resolve the workspace; used only
  by the opt-in `points_drain` feature

Until the private-sibling edge is resolved (tracked in
[Issue 030](.issues/030_public_clone_private_optional_dep.md)), a public
clone can be read and browsed but not built: any `cargo` command fails
when it resolves dependencies. `scripts/check_readme_path_deps.sh`
guards this section.

## Hosted-seat fair-use metering — what the seat records (Plan 326 T8)

When this repo's `serve` binary runs with the opt-in `points_drain` feature,
the hosted seat meters `/decide` fair use (the design contract is
`../riir-refine/.proposals/027_points_drain_fair_use.md`). In plain words:

- **Signed-in accounts.** A caller MAY attach three signature headers that
  prove which contribution-points account a call belongs to. The seat checks
  the signature and counts the call under that account id — which is
  `BLAKE3(pubkey)`: pseudonymous, never hardware-derived, nothing about the
  machine.
- **Everyone else (per-source).** To keep the free tier fair, the seat counts
  calls per NETWORK, not per person: a salted hash of the network's /24
  (IPv4) or /64 (IPv6) prefix. The salt is a random boot secret that rotates
  at the UTC day boundary and lives only in seat memory — **the hash is never
  persisted past the day it was made, and the salt never leaves the
  process**. Individual addresses cannot be recovered from it.
- **Counts, only counts.** What is kept for the day is a number of calls per
  account-day (free/billable) and per network-day. Never request bodies,
  never states, never options, never signatures.
- **The meter never breaks a caller.** Any verification failure (bad header,
  bad signature, replayed sequence) lands the call in the per-source bucket
  — the seat answers exactly as it would for an anonymous caller. And until
  the ratified enforcement config exists (it does not ship yet), the seat
  only COUNTS and VERIFIES — it never refuses a call, and it never debits
  anything.
- **The daily aggregate.** Once a day the seat exports aggregate numbers —
  `(account, day, free_calls, billable_calls)` rows — to the contribution
  points plane, or, until that plane is configured, to a local file beside
  the seat's state dir. The per-source hashes go NOWHERE but that local
  file, and only for the day they were made.

The binding privacy law for the points plane is
`../riir-refine/.docs/01_orientation/points_privacy_id_design.md` (the T16.1
client-UUID law); this section is the seat-side disclosure beside it.

## Adoption/retention swap gate — the LaneSlot epoch-swap consult (opt-in `adoption_gate`)

Consumer intake for `katgpt-core::adoption_gate` (produced in katgpt-rs,
Plan 633 / Issue 220). While the feature is on, a `LaneSlot` epoch swap is
scored before it serves: the challenger's decisions over the outgoing lane's
recent 32-case window are compared per suite — **retained** (cases the old
lane answered acceptedly that the challenger answers the same way) and
**repeated** (previously-abstained cases the challenger now answers); the
gate adopts only when the score clears the incumbent, declines first on
`NoEvidence` (a cold boot swaps on artifact correctness alone), and installs
once, from-base pure, never double-filing an outcome (stride decontamination
in the producer). The gate is **evidence, not a promotion claim**: a refusal
keeps the incumbent lane, and the swap verdict is recorded for the lane's
own audit — it never widens what may serve. G3 evidence on record: the wire
gate (`examples/adoption_swap_g3.rs`, six bars over the real HTTP edge,
byte-identity pinned at the wire) and the real-store arena cell
(`examples/adoption_swap_g3_arena.rs`, [`.benchmarks/066_issue031_g3_arena_real_store_cell.md`](.benchmarks/066_issue031_g3_arena_real_store_cell.md)
— the real regression pair caught: specialist 0.375 vs the served modelless
tier 0.000 on the same 32-case window, gate refused, both orderings agree).
Runs behind `--features adoption_gate`; disarmed builds are byte-identical.
