<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
<!-- Copyright (c) 2026 Jonathan D.A. Jewell (hyperpolymath) <j.d.a.jewell@open.ac.uk> -->

# Readiness — git-reticulator

**Current Grade:** C

> This file is the source of truth read by `just crg-grade` / `just crg-badge`
> (they grep for `**Current Grade:** X`). It was missing before 2026-06-03 even
> though the Justfile referenced it — created as part of the tidy-up pass.

## CRG (Component Readiness Grade)

| Grade | Meaning | Status |
|-------|---------|--------|
| D | Builds | ✅ |
| **C** | **All test categories present + passing** | ✅ **(current)** |
| B | 6 quality targets (lint, fmt, doc-coverage, …) | ☐ next |
| A | Production-ready | ☐ |

See `hyperpolymath/standards` → component-readiness-grades for the rubric.

## Honest caveat

Grade C here reflects test **category presence**, not behavioural coverage. The
27 tests assert "does not panic" over `println!` stubs (see
`.machine_readable/6a2/STATE.a2ml [honest-status]` and `TEST-NEEDS.md`). The
grade is technically correct against the rubric and simultaneously **overstates
functional maturity** — both facts are recorded so neither surprises a reader.

## To reach Grade B

1. Replace ≥1 stub with a real (even in-memory) lattice build.
2. Add a test that asserts a structural **property**, not just no-panic.
3. Lint/fmt/doc-coverage clean.
4. Discharge `PROOF-NEEDS.md` P2→P1 (earn the "lattice" noun) — bonus, not required for B.
