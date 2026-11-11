# Issue 030: PUBLIC repo, PRIVATE optional path dep — a fresh clone cannot run cargo

**Lease closed:** zed-7423d19b@amo epoch 1794437057 (interim (d) landed 2026-11-12, same session; the delegated verdict recording the decision: session 160062da, AGREE)
**Status:** OPEN — interim posture (d) landed 2026-11-12 (this commit:
README §Build sibling pins + the BOUNDARY clone-wall note +
`scripts/check_readme_path_deps.sh`, per the delegated Claude verdict,
session 160062da, AGREE). The public clone is documented browse-only,
still broken; (a)/(b)/(e) remain the owner's pick.
Found 2026-11-07 during the issue-029 visibility lane (the C10 sweep confirmed this repo is PUBLIC on GitHub). `Cargo.toml` carries `riir-ledger = { path = "../riir-ledger" }` (opt-in `points_drain` feature, Plan 326 T8), but cargo reads every path dep's manifest at workspace load — `optional = true` does not help (the riir-chain sibling-layout law, measured on every repo that documents it). A public cloner without the PRIVATE `riir-ai/riir-ledger` checkout beside the clone cannot run ANY cargo command (`cargo check`/`metadata`/`test` all fail at resolution). The public teaching arc (fetch public data → train → holdout → mint → serve, BOUNDARY.md's Owns) breaks at step zero for everyone outside the org. The other sibling deps (katgpt-rs, riir-reflex, riir-reflexer) are all PUBLIC and cloneable, so riir-ledger is the ONE edge that walls the public clone out.

## The options (owner call — none is executable from a lane)

- [-] **T1 — owner picks the public-clone posture** — deferred to the owner (repo-visibility flip = outward-facing act; interim (d) landed meanwhile). The candidates, cheapest-first: The candidates, cheapest-first: (a) **flip `riir-ai/riir-ledger` to PUBLIC** — it is a zero-dep serde leaf of wire DTOs + the attestation signing-bytes home; the wire shapes are already public-adjacent (the points plane is documented in riir-refine's public docs and reflex-side prose); (b) **move the DTO leaf into a public repo** (a `riir-ledger-wire` leaf inside riir-reflexer or reflex — both already public leaves homes), keeping the private repo for anything genuinely sensitive; (c) **make the edge optional-at-resolution** — not possible for path deps per cargo's resolution model; a `[patch]`-or-git indirection would still need the private repo for the full build; (d) **document the posture**: the public clone is read/browse-only until the owner acts (BOUNDARY.md + README say so loud).
- [x] **T2 — interim (d) landed 2026-11-12 (this commit)**: README §Build + BOUNDARY note; the (a)/(b) repo acts stay owner-available.
- [ ] **(e) — recorded for the owner (delegated verdict 2026-11-12):** move the `points_drain` feature (the code that uses `riir-ledger`) out of riir-instinct into a PRIVATE adapter crate depending on both `riir-instinct` and `riir-ledger` — the public manifest then has no private edge, without flipping any repo or creating a public one. Record-only; not implemented.
- [ ] **T2 (owner acts) — land the chosen (a)/(b)/(e) posture** (a repo flip is an owner act per Research-003 visibility law).
- [x] **T3 — the check landed with the interim docs 2026-11-12 (this commit):** `scripts/check_readme_path_deps.sh` — every external `path = "../X"` in Cargo.toml must be named in README §Build with a public/private label on the same line (grep-based, standalone; the verdict's script-checkable form). It guards the doc going stale when the next path dep lands. If (a)/(b)/(e) later lands, the C10 visibility-honesty row (riir-ai `bd20b517b`) guards the flip.

Note: nothing here gates the org-internal lanes — every dev box and CI checkout carries riir-ledger; this issue is purely about the public-clone build path the Phase-C flip promised.
