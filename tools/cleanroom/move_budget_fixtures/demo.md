# Demo: behaviour spec

What the player sees.

## Header

**Scope.** One domain.

Size is counted in logical tokens: each identifier is one; comments are free.

**Baseline.** The old code at spine `387640758`.

**Budget (MODEL §1 decision 7).** k = 36 weighted tokens per spec unit, frozen.

- Area moving (old 1,000): Walk,
  Fall
- Area resting: Rest

Budget: 36 × 3.0 units = 108 (moving 72, resting 36)

**Notation.** x grows rightward.

---

## 1. Moves


### Walk
- holding a side → she gains the step [V1]
