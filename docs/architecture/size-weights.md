# Size measure: which patterns to promote, which to penalise

Reviewer, engine side, 2026-11-07. The question, from Jim: beyond data and declarations, which code
patterns should `tools size` (`verify::size`, spine `13624d8a4`) count at a discount, at zero or at
a surcharge? Jim's refinement: build a goal function whose weights reflect priorities, so that a
lower score means better code and not churn. Exempt a form only where it provably holds no
behaviour. For every weight that drops, name the lint that stops the swap to a worse, cheaper form.

## Method

- A Python mirror of `verify::size`'s lexer and `count()` is in `$TEMP/collab/sizepat/pat.py`.
  It reproduces the DLL's numbers exactly:
  - round-7 model: 9,265 logic, 2,090 types, 7,386 data;
  - old code (the 11 files in the 13,342 baseline at `c7f927ab6`, `sizeparts` cuts included):
    13,342 / 1,781 / 153.
- It tags each logic token with the lexical patterns below. `score.py` applies weight scenarios.
  A token in several "good" patterns takes the lowest weight; otherwise it takes the highest "bad" weight.
- Lints in force: `[workspace.lints]` is identical in the clean-room export and the spine. It sets
  `pedantic` and `nursery` to deny, plus `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`,
  `arithmetic_side_effects`, `as_conversions`, `wildcard_enum_match_arm`, `unreachable` and
  `allow_attributes`. The clean room also runs `-D warnings` and `cargo fmt --check`.
  `check_crate_rules.sh` K8 bans `allow`/`expect` attributes, but only in the engine crates.
  `code_check.sh` is about isolation, not style.

## Ranked table

The cost columns give the logic tokens each pattern accounts for, as new (r7) / old.

| # | Pattern | Cost new / old | Class, weight | Gaming risk | Guard lint | Counter rule |
|---|---|---|---|---|---|---|
| 1 | `const`/`static` item head (`pub const NAME: T =` and its `;`) | 397 / 86 | good, **0** (move to types) | none: the value is already data; a name and a type hold no behaviour | none needed | in `count()`, add `counted(head)` and the trailing `;` to `types`, not `logic`. A trait's `const X: T;` (no value) also goes to types |
| 2 | `use` items, `mod name;`, the `mod name {` header | 541+18 / 612+6 | good, **0** (declarations) | none: a path import holds no behaviour. It makes bare variants cheaper, but two same-named imports don't compile, so it can't create ambiguity | `clippy::enum_glob_use` (pedantic, denied) stops `use E::*` | a `use` Word where `ends_item(previous)` holds (that already admits `pub` and `pub(..)`) up to `after_item`; `mod` Word + Word + `;`/`{` |
| 3 | trailing comma before a closing bracket | 44 / 141 | good, **0** (layout) | none: rustfmt places it, not the implementer | `cargo fmt --check` | skip `Punct(",")` when the next token is `Close`. Edge case: in `(x,)` the comma is required; dropping one token there is harmless |
| 4 | `const` keyword of `const fn` | 33 / 45 | good, **0** | none: the keyword is forced | `clippy::missing_const_for_fn` (nursery, denied) forces it | skip `Word("const")` followed by `Word("fn")` |
| 5 | fn signature: `fn name<..>(params) -> Ret where ..`, up to the body's `{` | 1,358 / 1,973 | good, **0.5** | low. A signature runs nothing on stable Rust (`generic_const_exprs` is unstable). The only lever is parameter count, and the cost stays at half | `too_many_arguments` (-D warnings), `too_many_lines`, `cognitive_complexity`, `fn_params_excessive_bools` | from `Word("fn")` followed by a Word, to the depth-0 `{` or `;`, exclusive, at 0.5. Fn-pointer types (`fn(`) and closures don't match. A leading `pub`/`const` follows its own rule |
| 6 | `true`/`false` in logic (flag-style code, tuple-of-bool matches) | 7 / 119 | bad, **2** | moving the test to `!x` / `x == false` | `bool_comparison`, `needless_bool`, `nonminimal_bool`, `struct_excessive_bools`, `fn_params_excessive_bools` | `Word("true" or "false")` in logic at 2 (const values stay data) |
| 7 | numeric literals other than 0 and 1 in logic | 31 / 68 | bad: **lint**, else 2 | a weight can't win. `m.rules.wall_reach` (5 tokens) still beats `8` at weight 2, so inlining a parameter still pays | **none today. Propose L2** (below) | `Literal` that is numeric, other than `0`/`1`, in logic. Const values, enum discriminants and test items are already outside logic |
| 8 | `#[allow]`/`#[expect]` attributes | 1 site / 0 | bad: **lint**. Today they are *free* (attributes are types) | **high:** `#[expect(clippy::indexing_slicing)]` costs nothing and re-opens every cheap form the lints forbid | **none in clean crates. Propose L1** | count lint-control attributes (`allow`, `expect`, `warn`, `deny`, `forbid`) as logic, or ban them (L1) |
| 9 | behaviour in a `const`/`static` value (closure, `if`, `match`, `let`, loop, `.method(`) | 0 / 0 | bad: **logic** | high: it is free today as data | **counter rule L3** | when the value holds `if`, `match`, `loop`, `while`, `for`, `let`, `fn`, a closure's `\|` or a `.`-call, the whole value is logic. Path calls to `const fn` constructors (`Fixed::new(..)`) stay data while every argument is data; an operator makes it logic (MODEL §4, 2026-11-09) |
| 10 | `wrapping_*`/`checked_*`/`saturating_*` (surcharge over an operator: `.` + name) | 72 / 174 | neutral, 1 | the cheaper form (`a + b`) is denied. The good form (one `Fixed::plus`) is already rewarded | `arithmetic_side_effects` | none |
| 11 | `From`/`TryFrom` conversion calls (`u16::from(x)`) | 22 / 68 | neutral, 1 | `as` is denied | `as_conversions` | none |
| 12 | `Option`/`Result` plumbing (`.get`, `.map_or`, `.ok_or`, `?`, `Some`/`None`) | 30+82 / 47+128 | neutral, 1 | unwrap and indexing are denied | `unwrap_used`, `expect_used`, `indexing_slicing`, `panic` | none |
| 13 | exhaustive match arms (no `_ =>` on enums) | 111 arms, 1,729 tokens / 240, 4,662 | neutral, 1 | a wildcard is denied. Or-patterns (`A \| B =>`) are already the cheaper good form | `wildcard_enum_match_arm` | none |
| 14 | long `match` ladders (6+ arms) | 870 / 2,317 | neutral, 1 (**no surcharge**) | a surcharge would push decisions into data, the exemption most open to gaming. The data exemption already gives the gradient: an arm costs 4-13 tokens, a row 0 | review, plus L3 | none |
| 15 | `Enum::Variant` / `Type::fn` qualifiers | 258+88 / 1,310+300 | neutral, 1 | with `use` free, bare variants get slightly cheaper. That is readable, and a clash won't compile | `enum_glob_use` | none |
| 16 | binary bit ops (`& 15`, `>> 4`, `^`) | 24 / 36 | neutral, 1 (**no surcharge**) | a weight on `>>` makes `x / 16` cheaper, which is churn. The literal rule (#7) catches the mask constants instead | literal lint L2 | none |
| 17 | visibility `pub` / `pub(crate)` | 118 / 94 | neutral, 1 | charging `pub` mildly favours a small API, which is good | `missing_docs` | none |
| 18 | trait impls (`From`, `Display`, `Default`) | 0 / 61 (`From`). Old `impl Collision for World` is 578, real behaviour | neutral, 1 | a trait body can hold any behaviour | none | none |
| 19 | newtype `.0`, lifetimes, `self`/`Self`, `matches!`, `;` | 32, 5, 371, 28, 363 / 44, 42, 294, 36, 438 | neutral, 1 | none | none | none |
| 20 | assertions (`assert!`, `debug_assert!`, `check_index`) | 0 / 0 | neutral, 1 | an assert's expression can call anything | none | none |
| 21 | doc comments, comments, layout, `#[cfg(test)]` | free already | already free | none | `missing_docs` | unchanged |
| 22 | `derive(PartialOrd, Ord)` on an enum | 0 / 0 | watch: free as an attribute, and it makes declaration order behaviour | medium, latent | **propose L4** | none |

Notes on the counts:
- Pattern 5 includes the `const` of pattern 4. The scenarios take the lower weight.
- Pattern 7's sites in r7 are mostly tile geometry in `collide.rs` (`& 15`, `>> 4`, `16`, `& !7`, `8`). The rest are `0x7FFF` in `below` and the array length `[Behaviour; 28]`.
- r7's single lint attribute is `#[expect(clippy::struct_excessive_bools)]` on `Pad`, which is legitimate (buttons are bools).

## Lints to add

- **L1, lint attributes in clean crates.** Extend K8 (`check_crate_rules.sh`) to `clean_types` and `clean_model/*`, or add a `code_check.sh` tag `lint-attr`. It forbids `allow`/`expect` naming any lint the workspace denies by name (`unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `arithmetic_side_effects`, `as_conversions`, `wildcard_enum_match_arm`, `unreachable`). Add `clippy::allow_attributes_without_reason = "deny"` to `[workspace.lints.clippy]` so every remaining `expect` states why. This is the one hole that defeats every "lint guards it" argument above, and today it costs zero tokens.
- **L2, magic literals.** Add a `code_check.sh` tag `magic-literal`: a numeric literal other than 0 or 1 in clean-model logic is a finding. Const/static values, enum discriminants and test items are exempt. DESIGN already says every literal traces to a behaviour id or `Rules`. The lint enforces it, and the weight can't (row 7).
- **L3, a counter rule, not a lint.** A const value that holds control flow, a closure or a method call is logic (row 9). r7 pays 0, so the rule closes the hole for free.
- **L4, optional.** No `PartialOrd`/`Ord` derive on enums in clean crates without a spec-given order.

## Effect on the numbers

All numbers come from the mirror, and S0 equals the DLL. The weights apply to both sides, like for like.

| Scenario | r7 model | old | model / old |
|---|---|---|---|
| S0 today | 9,265 | 13,342 | 69.4% |
| S1 rows 1-4 at 0 (no-behaviour forms) | 8,236 (−1,029) | 12,456 (−886) | 66.1% |
| S2 S1 + signatures at 0.5 | 7,559 (−677) | 11,482 (−974) | 65.8% |
| **S3 S2 + `true`/`false` ×2 + literals ×2 (recommended if L2 is not adopted)** | 7,594 | 11,662 | 65.1% |
| S3' S2 + bool ×2, literals guarded by L2 instead (recommended) | 7,566 | 11,600 | 65.2% |
| S4 S1 + bad ×2, no signature discount | 8,273 | 12,638 | 65.5% |

The targets must be re-derived under whichever weights are chosen. The 6,400 target is 48% of the old 13,342. At the same ratio under S3' it would be about 5,570 (48% of 11,600). Comparing a reweighted model with an old-weight target would be a rule change hidden in a tool change.

## Incentive check, with r7 examples

1. **Splitting out a named helper (good).** Take `alone()` in `lib.rs:242`, called from 2 sites.
   - Its signature is 24 tokens: `const fn alone(found: (Option<u16>, Option<u16>)) -> Option<(Heading, u16)>`.
   - Its body is about 33, and each call site is 3.
   - Today the helper costs 64 against 66 for inlining it twice, so the counter is close to indifferent.
   - Under S2 the helper costs about 51 against 66. The named step now scores better, which is what r7's "long functions are split into named steps" did at a loss.
2. **Naming the tile size (good).** `collide.rs` writes `>> 4`, `& 15` and `16` about 20 times.
   - Today `const TILE_BITS: u16 = 4;` costs 5 logic tokens of head for no saving, and a `tile(px)` helper costs about 11 more. Naming loses.
   - Under S1 the const head is free, so naming is neutral.
   - Under S3 each literal site costs 1 more than the named form, so naming saves about 20, less the helper's ~8.5.
   - With L2 adopted, inlining is not allowed at all.
3. **Replacing the `Seat` enum with `seated: bool` (bad).** `lib.rs` has 3 assignments and 1 test.
   - Today the bool saves the 7-token `use Seat::{Seated, Unseated}` and 2 tokens at the `== Seated` test. The worse code scores **9 lower**.
   - Under S1 it still scores 2 lower.
   - Under S3 the three `true`/`false` cost +3 against the test's −2, so the bool scores **1 higher**. The incentive flips, and the bool lints guard the rest.
4. **Residual bad moves no weight catches.**
   - Inlining a `Rules` parameter as a literal. L2 is needed (row 7).
   - Suppressing a lint with a free `#[expect]`. L1 is needed.
   - Moving a decision into a data row. That goes to review and L3, and is open question 1.
   - Inlining a once-used helper still saves its half-priced signature. That is accepted: a helper's name has to earn its keep.

## Open questions for Jim

1. **Are combinator rows data?** `chart/table.rs` holds 6,006 data tokens.
   - Its rows' `needs: &[Test]` use `Not(&Airborne)`, `Any(..)` and `Held(..)`, and its `Aiming` lists are named `AIM_UP_ELSE_DOWN`.
   - That is the small interpreter AGENTS.md asks for, but it is also "a decision the code would otherwise make" (MODEL §4).
   - One possible rule: combinator variants (`Not`, `Any`, `All`, an else-chain) count as logic inside data. That would move a few hundred tokens.
   - Should the line be drawn there?
2. **Where is the rule held?** One option is a single weighted score (logic with the weights above). The other keeps raw logic, moves rows 1-4 into the types column and reports signatures and bool literals as their own columns.
   - The first is the goal function Jim described.
   - The second keeps every column an integer that is easy to audit.
3. **Magic literals.** Make them a hard lint (L2, DESIGN's own rule), or only a ×2 weight?
4. **The signature discount (0.5).** It is the one discount on a form that is mechanical but chosen, not provably free of behaviour. Accept it, or keep signatures at 1 and rely on rows 1-4 alone (S1 + bad weights is S4: 8,273 / 12,638)?
