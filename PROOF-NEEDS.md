# PROOF-NEEDS.md
<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk> -->

## Current State

- **LOC**: ~237 Rust (host) + ~100 AffineScript (`.affine`, not compilable).
- **Existing proofs**: **first corpus landed (2026-06-04)** —
  `verification/proofs/Lattice/Order.idr` (Idris2 0.8.0, `%default total`, zero
  proof escapes), CI-gated by `.github/workflows/proof-corpus.yml`. It proves the
  *abstract* order theory, **not yet** bound to the Rust graph (see below). Before
  this there were zero proofs. The 2026-05-26 estate tech-debt audit recorded
  "Proof debt: none / Recommended next move: none" — that verdict is **wrong**
  for a project whose headline noun is *lattice*. A lattice is a mathematical
  claim with discharge obligations; asserting it without proof is exactly the
  kind of overstatement this file exists to retire.
- **Status**: the structure git-reticulator builds today is a *typed, weighted
  digraph* with a hierarchical containment sub-relation — **not** (yet) a
  lattice, not even a partial order (TOPOLOGY.md itself mentions detecting
  *circular* dependencies, i.e. cycles, which a partial order forbids).

> The point of this document is to make the gap between the noun ("lattice")
> and the artifact (a digraph) explicit, and to list precisely what must be
> proved to close it. Until then, prose should say "typed graph" / "DAG".

## Recommended Prover

- **Idris2** — to match the estate's existing formal-methods spine (`vcl-ut`
  ships a machine-checked Idris2 corpus, `%default total`, zero proof-escape).
  Reusing it means shared CI (`proof-corpus.yml`-style gate) and shared
  reviewer muscle. Lean4 or Coq would also serve; Idris2 is the path of least
  estate friction. The order-theory here is elementary; the value is in
  *connecting the proofs to the running code*, not in their depth.

## Proof status (by category) — 2026-06-04

Honest categorisation. **Proved** = mechanically checked (Idris2/Coq/SPARK).
**Tested** = executable checks + unit tests in the Rust reference core
(`src/lattice/mod.rs`) — a rung *below* proof. git-reticulator has **zero
mechanized proofs**; what exists is tested.

### Done (mechanized — first corpus, 2026-06-04)
- **Abstract order theory** (`verification/proofs/Lattice/Order.idr`, Idris2 0.8.0,
  `%default total`, zero proof escapes; CI-gated by `proof-corpus.yml`): the
  partial-order laws are witnessed (`natOrder`: reflexive + transitive +
  antisymmetric) and **antisymmetry ⇒ no 2-cycle** (`noTwoCycle`) is proved — the
  order-theoretic heart of why the SCC condensation is a DAG (**P2a**). **Scope
  honesty:** proved on an *abstract* `PartialOrder`, **not yet** on the actual Rust
  graph in `src/lattice/mod.rs`; binding the two is the ADR-006 Idris2 ABI seam.

### Done (tested, not proved)
- **P2a** SCC condensation is acyclic — `Condensation::is_acyclic` (Kahn
  topological sort, a genuine runtime check, not a trust assertion) + tests
  (the 3-node call cycle collapses to one component; condensation is a DAG).
- **P4** LOD `zoom` soundness + completeness — tested on fixtures (defs in
  other files excluded; every descendant returned).
- **P1b (fragment)** `meet` = lowest common ancestor — tested (idempotent,
  commutative, LCA correct).
- **P1a (fragment)** reflexivity of ≤ — tested.

### Not attempted
- **P1** full lattice laws (associativity/absorption; **join**).
- **P3** monotone abstraction (commit-DAG → lattice) — also: git-history ingest
  isn't wired (`src/ingest.rs` is filesystem-only).
- **P5** determinism/confluence. **P6** drift-predicate totality.
- **P7** pgRouting≡lattice — now **N/A** (verisim is the store, not pgRouting).
- In the *mechanized-proof* sense, **all of P1–P7 are unattempted** (zero `.idr`).

### Sorries / `believe_me` / proof escapes
- **Zero — but vacuously.** There are no proofs, so there are no escape hatches.
  This is **not** vcl-ut's "zero `believe_me` in a real corpus" achievement; it
  is zero-because-empty. Recorded so it is never mistaken for rigour we lack.

### Structural blockers
- Idris2 prover **now wired** (`verification/proofs/` + `proof-corpus.yml`, idris2
  0.8.0); first module verified. Remaining: bind the proofs to the Rust graph and
  discharge the rest of P1–P7.
- The verifiable core is migrating to AffineScript (ADR-006), itself alpha with
  the CORE-01 soundness gap.
- The Rust/SPARK Idris2/Zig ABI seam is **N/A until git-reticulator exposes an
  FFI surface** (it's a CLI/REST app today) — see
  `docs/decisions/rust-spark-stance.adoc`.

### False (claims that would be untrue — and are correctly avoided)
- "git-reticulator builds a **lattice**" (full lattice) is **false**: arbitrary
  sibling nodes have no unique join. The code is honest — it implements `meet`
  only and names the structure a **meet-semilattice** + typed digraph. The true,
  scoped claim is evidenced by test; the false strong claim is not made.

### What it means / how much to worry
- **Low worry** for an early, non-safety-critical research tool. The property
  that matters — and holds — is that **nothing is overclaimed**: code and docs
  say "tested not proved", "meet-semilattice not lattice", "zero proofs". That
  honesty *is* the estate bar (the doc-truthing / SPARK-theatre culture is about
  not faking verification). The residual risk is correctness-confidence (a
  zoom/meet edge case could ship), not safety. The path up is cheap and known:
  P2→P1→P4 in Idris2, or largely for free from the AffineScript core's type
  discipline post-migration.

## What Needs Proving

### P1 — It is actually a lattice (or: rename it) — **HIGH**
The relation `≤` derived from the structure must be shown to be a lattice:
- **P1a** `≤` is a partial order: reflexive, antisymmetric, transitive.
- **P1b** For every pair of nodes a **meet** (greatest lower bound) and a
  **join** (least upper bound) exist and are unique.
- **P1c** The lattice laws hold: idempotence, commutativity, associativity of
  `∧`/`∨`, and absorption.

If P1b cannot be met for the real structure (likely — sibling nodes generally
have no unique join), then **honesty demands one of**:
- downgrade the claim to **meet-semilattice** (containment hierarchy with a top), or
- rename to "semantic **graph**" and drop "lattice" from the API/docs.

### P2 — Cycles must be quotiented before any order claim — **HIGH**
The dependency graph has cycles. A partial order has none. The standard fix:
- **P2a** Compute the **strongly-connected-component condensation**; prove the
  condensation is acyclic (a DAG).
- **P2b** Prove the order induced on the condensation is well-defined
  (independent of SCC member choice). Only on the condensation does P1 even
  get a chance.

### P3 — Faithful abstraction of git history (monotonicity) — **HIGH**
The commit DAG → lattice mapping must not invent or lose ordering:
- **P3a** *Soundness*: if the lattice orders `a ≤ b`, then `a`'s git origin is
  an ancestor of (or contained by) `b`'s — no spurious orderings.
- **P3b** *Completeness*: ancestry present in the commit DAG is reflected.
- Together: the construction is a **monotone (order-preserving) map** from the
  commit DAG to the lattice.

### P4 — LOD `zoom_to_node` is sound AND complete — **HIGH (the RAG trust anchor)**
This is the property that makes retrieval trustworthy — the bridge between the
symbolic guarantee and the neural/LLM consumer.
- **P4a** *Soundness*: every node returned by `zoom_to_node(n, L)` is a genuine
  descendant of `n` at level `L` — no spurious context.
- **P4b** *Completeness*: every such descendant is returned — no silently
  dropped context.
- Formally: `zoom(n, L) = { m | m ≤ n ∧ level(m) = L }`, exactly.
  Get this wrong and the LLM is fed context that is incomplete (missing the
  relevant bit) or padded (wasting the token budget the feature exists to save).

### P5 — Construction is deterministic / confluent — **MEDIUM**
- Building the lattice twice from the same repo state yields the same lattice
  (up to node identity). Without this, caching is unsound and any downstream
  proof is meaningless because the object it refers to is unstable.

### P6 — Neuro-symbolic consistency invariant — **MEDIUM (the novel target)**
The genuinely new obligation, and the one that matters for the neuro-symbolic
claim (see `.machine_readable/6a2/NEUROSYM.a2ml`):
- Do **not** prove "embedding proximity ⇒ lattice proximity" (false in general).
- **Do** prove the *drift predicate* is well-defined and decidable: given a
  threshold, `drift(a,b) := cosine(emb a, emb b) high ∧ lattice-distance(a,b) high`
  is a total, computable signal. This is what lets verisim's drift detector
  treat neuro-symbolic disagreement as an observable event rather than a
  silent inconsistency.

### P7 — `pgRouting` reachability matches lattice reachability — **LOW**
- The Dijkstra/A* paths used for blast-radius (PB-4) range over the same edge
  relation the lattice is built from — prove the stored `cost`/`weight`
  topology is a faithful image of the in-memory `Relationship` set, so
  "reachable in the DB" ⇔ "reachable in the lattice".

## Priority

**MEDIUM-HIGH.** git-reticulator is early and stubbed, so this is not blocking
a shipping product. But it is *cheap, high-signal* work: P1+P2 together either
earn the central noun or correctly retire it, and P4 is the precondition for
the proof-carrying-retrieval story that makes the whole neuro-symbolic stack
worth building. Do P2 → P1 → P4 first; the rest can follow the implementation.

## Cross-references

- `.machine_readable/6a2/NEUROSYM.a2ml` — where these proofs pay off (proof-carrying retrieval).
- `hyperpolymath/vcl-ut` `PROOF-NEEDS.md` + `verification/proofs/` — the estate template for "PROOF-NEEDS + Idris2 corpus + VERIFICATION-STANCE" done well.
- `TEST-NEEDS.md` — the testing counterpart (currently smoke-level over stubs).
