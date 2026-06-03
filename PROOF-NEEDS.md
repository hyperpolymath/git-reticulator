# PROOF-NEEDS.md
<!-- SPDX-License-Identifier: MPL-2.0 -->
<!-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk> -->

## Current State

- **LOC**: ~237 Rust (host) + ~100 AffineScript (`.affine`, not compilable).
- **Existing proofs**: **NONE.** No `*.idr`, `*.v`, `*.lean`, `*.agda`, `*.fst`,
  `*.tla`, `*.ads`. The 2026-05-26 estate tech-debt audit recorded
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
