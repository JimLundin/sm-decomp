# Spec units: the measure a budget is built on

A domain's budget is k × its spec units (MODEL §1 decision 7, k in §4). This note says how the
units are counted and what keeps them honest. The spec's form that makes the count exact is
MODEL §5's template.

## 1. The measure

`U = arms + vocab/3 + ops/3`, counted per entry and summed per area.

| Counted | What it is | Why it costs code |
| --- | --- | --- |
| **arm** | one condition → outcome rule in an entry. Arms with the same outcome merge (`A or B → X` is one arm) | a branch: a condition expression, an outcome and the control flow. This is the dominant cost |
| **vocab** | each distinct term the cells of a data table use that the reader must interpret: a test form (`X pressed`, `beats …`, `gun stowed:`), a next-pose modifier (`kept`, `gun as it was`, `striding`), an effect, a situation. Counted once per spec | one interpreter arm plus its small body, about a third of a rule. The variant itself is a free declaration |
| **ops** | each operator or comparison in an exact formula (the leading edge `x + d + rx − 1`, the walk factor's cut, multiply and sign, the 8.8 read) | arithmetic the model must spell out; it doesn't appear as arms |

**Not counted:**

| Item | Why |
| --- | --- |
| table rows | data. A row that hides a decision is a review stop (MODEL §4) |
| parameters | data in `Rules`. Their reads sit inside the arms |
| enumeration values | declarations, which are free. The arms that tell them apart already count |
| quantities and Read-by edges | a field is a declaration. A read sits inside the arm that reads it. Reported as diagnostics only |
| inputs | an interface declaration plus the bridge (reported only) |
| prose, explanations, "So …" consequences, examples, vector ids, `checked:` lines, the vectors file | none of them holds behaviour |

The weights of 1/3 come from the code's shape, not from fitting. The unit counts structure, not
words: arms are marked bullets, vocab is a declared list (`Cells:`), ops are operators inside
backticks. So rewording, or adding explanation, changes nothing.

## 2. The tool

`tools spec-units SPEC VECTORS [--notes FILE] [--areas FILE] [--against OLD]`
([`verify::spec_units`](../../crates/verify/src/spec_units.rs), whose module note states every
rule it applies) counts the units from the spec and reads the rest from the Budget section of
the decode notes (`--notes`, by default the spec's sibling `D.decode.md`). Areas are lines there,
one per area, as a paragraph or a bullet: `Area motion (old 3,818): SpeedBuildsAndBrakes, Dash,
…, 1.2, 9`. An item names an entry or any other heading, by its text or its leading number;
`(old N)` is the area's old code, shown beside its budget for reference. k is read from the
section's "k = N weighted tokens per spec unit"; notes that state none leave the units counted
and no budget, and a spec that still states k or an area is a finding. A spec commit shows its
change with `git show HEAD~1:docs/behaviours/D.md > old.md` and `--against old.md`, which adds
`change: units 4.7 → 7.7 (+3.0), budget 112 → 184 (+72)` to the report.

What it counts:
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
- **Same outcomes.** Two arms of one entry with the same outcome text are flagged, and the spec
  review merges them.

The limits are textual: formulas differing only in every `+` and `−` count once even when they
aren't mirrors, a mirror written with other names counts twice, an explanation that opens with
another word counts its formulas, a null outcome phrased otherwise isn't flagged, and two
different outcome texts that mean the same thing count twice. The spec review catches these.

## 3. Guards against inflation

The observer writes the spec, so a spec-derived budget gives the observer a reason to pad it. The
guards:
1. **Only checked arms count.** An arm without a vector id, or a recorded frame, that exists in
   `scenarios/vectors/<domain>.toml` scores 0. This extends the catalogue's teeth (MODEL §3) from
   entries to arms. Padding then means writing vectors the model must pass, which is real
   behaviour.
2. **Minimality lints in the tool:** two arms of one entry with the same outcome; an arm whose
   condition is the negation of an earlier arm in a first-that-applies list; an arm that
   restates another entry's rule, which has to cross-reference it (`` `Landing` ``), and a
   cross-reference counts 0.
3. **The adversary gets a padding question.** It samples arms against the original, trying to
   show one is a restatement, a split of one behaviour, or behaviour the original doesn't have.
   A padded arm it finds is handled like a leak: the spec stops until it is removed, and the
   budget is recounted.
4. **The budget freezes at the export.** The implementer's round is judged against the spec in its
   export. Arms added later count only with a `gaps.md` entry and a vector. That stops units being
   added after the observer sees that a model is over.
5. **The old code is an advisory comparison**, shown by area beside the budget. The root
   considers it after independent review (MODEL §1 decision 19).
6. **k is frozen.** Nobody chooses a ratio per domain. The only lever left is units, and guards 1-4
   cover units. The root checks each spec commit's budget after independent review.
