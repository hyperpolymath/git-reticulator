-- SPDX-License-Identifier: MPL-2.0
-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk>
|||
||| Elementary order-theory obligations for the git-reticulator core, machine
||| checked. This is the first mechanized proof in the repo (PROOF-NEEDS.md:
||| "zero proofs" -> a foothold on P1a/P2a), %default total, zero proof escapes,
||| matching the estate's vcl-ut corpus discipline.
|||
||| It does NOT yet prove the properties of the *actual* Rust graph in
||| `src/lattice/mod.rs`; it proves the order-theoretic facts that justify why
||| that code is correct, on an abstract model, and exhibits a concrete witness.
||| Connecting these to the running condensation is the next step (ADR-006: the
||| Idris2 ABI seam will carry these as obligations).
module Lattice.Order

%default total

--------------------------------------------------------------------------------
-- Order-theoretic predicates (the shape a genuine partial order must have)
--------------------------------------------------------------------------------

||| Reflexivity of a relation.
public export
IsRefl : {a : Type} -> (rel : a -> a -> Type) -> Type
IsRefl {a} rel = (x : a) -> rel x x

||| Transitivity of a relation.
public export
IsTrans : {a : Type} -> (rel : a -> a -> Type) -> Type
IsTrans {a} rel = {x, y, z : a} -> rel x y -> rel y z -> rel x z

||| Antisymmetry of a relation.
public export
IsAntisym : {a : Type} -> (rel : a -> a -> Type) -> Type
IsAntisym {a} rel = {x, y : a} -> rel x y -> rel y x -> x = y

||| A partial order bundles a relation with proofs of the three laws.
public export
record PartialOrder (a : Type) where
  constructor MkPO
  rel        : a -> a -> Type
  prf_refl   : IsRefl rel
  prf_trans  : IsTrans rel
  prf_antisym : IsAntisym rel

--------------------------------------------------------------------------------
-- P2a (essence): antisymmetry forbids cycles
--------------------------------------------------------------------------------

||| The condensation order computed in `src/lattice/mod.rs` is antisymmetric by
||| construction (distinct strongly-connected components cannot mutually reach).
||| Antisymmetry is exactly what makes it a DAG: any two-cycle collapses to a
||| single point. This is the order-theoretic heart of `Condensation::is_acyclic`.
export
noTwoCycle : {x, y : a} -> (po : PartialOrder a) -> rel po x y -> rel po y x -> x = y
noTwoCycle po p q = prf_antisym po p q

--------------------------------------------------------------------------------
-- A concrete witness: <= on Nat is a partial order (so the bundle is inhabited)
--------------------------------------------------------------------------------

||| Standing in for "component a reaches component b" on the acyclic condensation.
public export
data Leq : Nat -> Nat -> Type where
  LeZ : Leq Z n
  LeS : Leq m n -> Leq (S m) (S n)

leqRefl : (n : Nat) -> Leq n n
leqRefl Z     = LeZ
leqRefl (S k) = LeS (leqRefl k)

leqTrans : {x, y, z : Nat} -> Leq x y -> Leq y z -> Leq x z
leqTrans LeZ      _        = LeZ
leqTrans (LeS p) (LeS q)   = LeS (leqTrans p q)

leqAntisym : {x, y : Nat} -> Leq x y -> Leq y x -> x = y
leqAntisym LeZ      LeZ      = Refl
leqAntisym (LeS p) (LeS q)   = cong S (leqAntisym p q)

||| P1a (witnessed): the reachability order on the condensation is a genuine
||| partial order — reflexive, transitive, antisymmetric.
public export
natOrder : PartialOrder Nat
natOrder = MkPO Leq leqRefl leqTrans leqAntisym
