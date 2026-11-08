# A size budget derived from the spec

Reviewer, engine side, 2026-11-07. Jim's question: can the size budget for a domain's model be
calculated from its spec, so that it follows the complexity of what is built rather than a ratio a
reviewer chose?

Inputs:
- the spec: spine `eb5dc05ad`, `docs/behaviours/samus-motion.md`;
- the model: `C:/cleanroom/samus-motion-7/crates`;
- the score: S3' from `reviews/size-patterns.md`, mirrored by `sizepat/pat.py`.

The prototype scripts are in `$TEMP/collab/specbudget/`:
- `codescore.py` gives weighted and plain logic per file. It reproduces 7,566 / 9,265 for round 7
  and 11,600 / 13,342 for the old code (at `c7f927ab6`).
- `attribute.py` splits round 7's logic by area, and by the spec entries each function's comments
  cite.
- `specmeasure.py` counts the spec's structure, with a heuristic for its clauses.
- `handcount.py` holds the hand count and computes the budgets.

There is **no second data point**. `docs/behaviours/` holds only samus-motion, and the
`game-time` harness domain has no behaviour spec. Only one constant can be identified, and the
design below fits only one.

## 1. The measure: spec units

`U = arms + vocab/3 + ops/3`, counted per entry and summed per area.

| Counted | What it is | Why it costs code |
| --- | --- | --- |
| **arm** | one condition → outcome rule in an entry. Arms with the same outcome merge (`A or B → X` is one arm) | a branch: a condition expression, an outcome and the control flow. This is the dominant cost |
| **vocab** | each distinct term the cells of a data table use that the reader must interpret: a test form (`X pressed`, `beats …`, `gun stowed:`), a next-pose modifier (`kept`, `gun as it was`, `striding`), an effect, a situation. Counted once per spec | one interpreter arm plus its small body, about a third of a rule. The variant itself is a free declaration |
| **ops** | each operator or comparison in an exact formula (the leading edge `x + d + rx − 1`, the walk factor's cut, multiply and sign, the 8.8 read) | arithmetic the model must spell out; it doesn't appear as arms |

**Not counted:**

| Item | Why |
| --- | --- |
| table rows | data. The old engine read them from the ROM. A row that hides a decision is a review stop (MODEL §4) |
| parameters | data in `Rules`. Their reads sit inside the arms |
| enumeration values | declarations, which are free. The arms that tell them apart already count |
| quantities and Read-by edges | a field is a declaration. A read sits inside the arm that reads it. Reported as diagnostics only (samus-motion: 6 quantities, 17 Read-by edges) |
| inputs | an interface declaration plus the bridge (8 inputs, about 32 fields; reported only) |
| prose, explanations, "So …" consequences, examples, vector ids, `checked:` lines, the vectors file | none of them holds behaviour |

The weights of 1/3 come from the code's shape, not from fitting. With one domain they can't be
fitted without circularity (§4).

## 2. Counts for samus-motion

These come from a hand count by one reader under the convention of §5, with an uncertainty of
about ±15 %. The heuristic counter on today's prose finds 146 clauses where the hand count finds
214. Today's prose can't be counted mechanically; that is why the convention is needed.

| Area | Entries | Arms | Vocab | Ops | **Units** |
| --- | ---: | ---: | ---: | ---: | ---: |
| motion (§1.2, §3 to §6 less pose fit, §9) | 14 + memory/numbers | 95 | 7 | 20 | **104** |
| pose fit (`PoseFitAfterMoves`, `ShrinkingLowersHer`) | 2 | 26 | 0 | 6 | **28** |
| pose chart (§7: reader and effects) | 5 | 29 | 90 | 0 | **59** |
| collision (§8) | 7 | 64 | 7 | 40 | **80** |
| **Total** | 28 | 214 | 104 | 66 | **271** |

Weighted logic (S3') by area. Round 7's shared types and numbers (328) are counted in motion; its
catalogue (5) is left out. The old code's areas:
- motion: `model/samus/motion.rs` and `motion.rs`;
- chart: `motion/pose.rs`, `pose/contact.rs` and `pose_input`;
- collision: `world`, `model/collision`, `collision_scan` and `solid_enemy`;
- fit: `pose/fit.rs`.

| Area | Units | r7 | r7 / unit | Old | Old / unit |
| --- | ---: | ---: | ---: | ---: | ---: |
| motion | 104 | 3,238 | 31.1 | 3,818 | 36.7 |
| pose fit | 28 | 656 | 23.4 | 1,360 | 48.6 |
| pose chart | 59 | 1,439 | 24.4 | 2,062 | 34.9 |
| collision | 80 | 2,228 | 28.0 | 4,360 | 54.7 |
| **Total** | 271 | 7,561 | **27.9** | 11,600 | **42.9** |

## 3. Calibration options

The budget is `k × U` per area, summed.

| Option | k (weighted tokens per unit) | samus-motion budget | Note |
| --- | ---: | ---: | --- |
| A: fitted on round 7, then frozen | 27.9 | 7,561 | circular: the judged code sets its own bar. Usable only for later domains |
| B: the old code × the required saving | 42.9 × s | s = 0.48: 5,568 (today's target); 0.55: 6,380; 0.60: 6,960; 0.70: 8,120 | not circular, but s is still a chosen ratio |
| C: **frozen k = 24, each area capped at 70 % of its old code** | 24 | **6,500** | recommended |

Why k = 24. Two routes agree:
- the old code's density times a saving of 0.55: 42.9 × 0.55 = 23.6;
- round 7's best-modelled areas: pose fit at 23.4 and the chart at 24.4.

The first route is the definition. The second only corroborates it, because it comes from the
code being judged, but it shows the bar is achievable: it asks every area to be as dense as round 7's
best two already are.

The 70 % cap per area keeps the "significant saving" rule against an inflated spec. It doesn't bind
in samus-motion. Over the old code stays a strict failure.

**Circularity.** Fitting k, or any weight, on round 7 makes the budget describe round 7. Fitting on
the old code inherits its port's verbosity: old collision runs at 54.7 per unit against motion's
36.7. So k is set once and frozen, from the old code's domain total, and the next domain tests it
out of sample. Its budget is predicted from its spec before any implementer starts.

## 4. Recommended budget and per-area split

| Area | Budget (24 × U) | Cap (70 % of old) | r7 | r7 against budget |
| --- | ---: | ---: | ---: | ---: |
| motion | 2,496 | 2,673 | 3,238 | **+742** |
| pose fit | 672 | 952 | 656 | −16 |
| pose chart | 1,416 | 1,443 | 1,439 | +23 |
| collision | 1,912 | 3,052 | 2,228 | **+316** |
| **Total** | **6,496** | | 7,561 | **+1,065 (116 %)** |

Today's target is 6,400 plain, about 5,570 weighted. Jim set the scope: everything in the spec is
in. The spec has grown since the target was set, and a spec-derived budget follows that growth:
each amendment from a gap adds k per unit.

**Per entry**: `handcount.py`, from the citations in round 7's function comments. Round 7 leaves
1,193 tokens uncited, so these figures are indicative. They show where a review should look:
- over: `SlopesCarry` 845 against 480; `PoseFitAfterMoves` 769 / 520; `BlocksStopFlush` 584 / 376;
  `Jumping` 390 / 216; `RisingAndFalling` 247 / 136; `WallsAndCeilings` 143 / 72;
- well under: `PadChart` 222 / 824, because most of the chart's reader cites no entry;
  `AimingAndShooting` 125 / 256.

To make the per-entry split exact, the code side needs one rule: every function's doc comment cites
the entries it implements (round 7 mostly does). `attribute.py` already reads those citations.

## 5. Robustness, and the spec convention it needs

The unit counts structure, not words:
- arms are marked bullets;
- vocab is a declared list;
- ops are operators inside backticks.

So rewording, or adding explanation, changes nothing: "So …", examples and notes are not arms.
Splitting one rule into two bullets with the same outcome merges back into one arm. The tool flags
two arms of one entry with the same outcome text, and the spec review merges them.

One sensitivity is left. Two different outcome texts that mean the same thing would count twice,
and the minimality review catches that (§6).

**The convention.** These are small additions to MODEL §5's template:
1. **Rule arms.** Under an entry, each rule is one bullet of the form
   `- <conditions> → <outcome> [vectors]`. Explanation goes in bullets or paragraphs that have no
   `→`. Only `→` bullets count.
2. **`checked:` on its own bullet**, and each arm's vector ids in brackets at its end, not in the
   prose.
3. **Under each data table, a `Cells:` line** listing the terms its cells use. The tool counts the
   distinct terms across the spec.
4. **Exact arithmetic as a formula in backticks.** The tool counts its operators.
5. **The `params:` and `switch:` fields the template already asks for**, so parameters stay
   traceable (they count 0).
6. **The budget line is generated:** `Budget: 24 × 271 = 6,500 (motion 2,500, fit 670, …)`.
   The tool prints Δunits and Δbudget for every spec commit. Since MODEL §1 decision 10 the line,
   k, the areas and the baseline are in the `## Budget` section of the decode notes, not the
   spec's header: the implementer is not told them.

**The tool** is `tools spec-units SPEC VECTORS [--notes FILE] [--areas FILE] [--baseline AREA=N]
[--against OLD]` ([`verify::spec_units`](../../crates/verify/src/spec_units.rs), whose module note
states every rule it applies). It counts the units from the spec and reads the rest from the
Budget section of the decode notes (`--notes`, by default the spec's sibling `D.decode.md`).
Areas are lines there, one per area, as a paragraph or a bullet:
`Area motion (old 3,818): SpeedBuildsAndBrakes, Dash, …, 1.2, 9`. An item names an entry or any
other heading, by its text or its leading number; `(old N)` sets the 70 % cap. k is the domain's
own, read from the section's "k = N weighted tokens per spec unit" (Jim, 2026-11-07); notes that
state none leave the units counted and no budget, and a spec that still states k or an area is a
finding. A spec commit shows
its change with `git show HEAD~1:docs/behaviours/D.md > old.md` and `--against old.md`, which adds
`change: units 4.7 → 7.7 (+3.0), budget 112 → 184 (+72)` to the report.

What it counts, after adversary-22's rulings (§5, T2):
- **Cites.** A `row:frame` cite holds only when a vector's `source` is that frame, and
  `row:N-M` only when one is inside the range: another vector from the row doesn't do. The labels
  `[original only]` and `[record only: row:frame, ...]` count 0 and are listed, not reported as
  missing vectors.
- **Formulas** count only in an entry's counted arms (a same-outcome arm is part of the arm it
  merges into) and its definition bullets: bullets with no `→` that aren't fields and don't open
  with an explanation word (`So`, `Thus`, `Hence`, `As`, `E.g.`, `For example`, `That is`,
  `I.e.`). Operators in an arm that counts 0, in an explanation bullet or in a paragraph count 0.
- **Mirrors** count once: formulas equal after swapping `x` and `y` in short names (`rx`, `ery`),
  or `+`/`−`, `<`/`>`, `≤`/`≥`, `min`/`max`, `left`/`right`, `top`/`bottom`, `up`/`down`, or
  both. `BlocksStopFlush`'s four leading edges are two.
- **Restatements** count 0: a formula that is a whole operand of a longer one in the same entry,
  by token runs bounded by brackets, commas or looser operators (`y + ry` in
  `gap = (ey − ery) − (y + ry)`, `top − 1` in `bottom ≥ top − 1`, but not `bottom ≥ top`).
- **Outside an entry**, an arm or a definition's formula counts 0 and fails: it is a finding and
  the command exits with an error, the report included.
- **The else-lint** flags a complement arm (an earlier condition negated, or `otherwise`, `else`,
  `any other`) only when its outcome repeats another arm's, and flags a null outcome (`nothing`,
  `no X`, "doesn't change") as belonging in an explanation bullet. Both still count; the spec
  review decides.

The limits are textual: formulas differing only in every `+` and `−` count once even when they
aren't mirrors, a mirror written with other names counts twice, an explanation that opens with
another word counts its formulas, and a null outcome phrased otherwise isn't flagged. The spec
review catches these.

**How far samus-motion follows it today:**

| Part of the convention | Today |
| --- | --- |
| entry ids and kind lines | 28 / 28 |
| `checked:` on its own bullet | 22 / 28 (6 inline) |
| rule bullets in arm form | 0 of 112. They hold about 214 arms, about 1.9 a bullet, alongside 49 "so" consequences and about 145 vector ids |
| `params:` field | 0 / 28 (inline in 2 "Parameter and mechanic" entries) |
| `switch:` field | 1 (the Toggle in `WordwiseTopSpeed`'s kind line) |
| `Cells:` lists | 0 of 6 data tables |
| formulas in backticks | `BlocksStopFlush`, `SolidEnemies` and §9's `below` only; the rest of the arithmetic is prose |

Converting it is about one observer session. It is a re-layout, not a change of behaviour.

## 6. Guards against inflation

The observer writes the spec, so a spec-derived budget gives the observer a reason to pad it. The
guards:
1. **Only checked arms count.** An arm without a vector id, or a recorded frame, that exists in
   `scenarios/vectors/<domain>.toml` scores 0. This extends the catalogue's teeth (MODEL §3) from
   entries to arms, and the tool checks the ids. Padding then means writing vectors the model must
   pass, which is real behaviour.
2. **Minimality lints in the tool:**
   - two arms of one entry with the same outcome;
   - an arm whose condition is the negation of an earlier arm in a first-that-applies list;
   - an arm that restates another entry's rule. It has to cross-reference it (`` `Landing` ``), and
     a cross-reference counts 0.
3. **The adversary gets a padding question.** It runs before any implementer starts, reads the
   original, and samples arms, trying to show one is a restatement, a split of one behaviour, or
   behaviour the original doesn't have. A padded arm it finds is handled like a leak: the spec stops
   until it is removed, and the budget is recounted.
4. **The budget freezes at the export.** The implementer's round is judged against the spec in its
   export. Arms added later count only with a `gaps.md` entry and a vector. That stops units being
   added after the observer sees that a model is over.
5. **Per-area caps against the old code** (70 %), and over the old code a strict failure. However
   much the spec is inflated, the budget can't pass these.
6. **k is frozen.** Nobody chooses a ratio per domain. The only lever left is units, and guards 1-4
   cover units. Every spec commit shows its Δbudget to the spec reviewer.

## 7. Open questions for Jim

1. **Is k = 24 weighted tokens per spec unit the bar?** It equals the old code's density with a
   45 % saving, and round 7's best areas already meet it. It gives samus-motion 6,500, against
   today's chosen target of about 5,570. The alternative is to keep a ratio of the old code as the
   rule (option B).
2. **Should the budget move with the spec?** Under this measure every gap amendment raises it by k
   per unit. Today's target stayed put while the spec grew.
3. **Should MODEL §5's template take the clause convention (§5)?** The cost falls on the observer:
   one re-layout of samus-motion. Without it the measure can't be computed, and only a hand
   estimate of ±15 % is possible.
4. **Should round 7 be held to it?** It is 116 % of budget, and the overrun is named by area: motion
   +742, collision +316; pose fit and the chart are on budget.
