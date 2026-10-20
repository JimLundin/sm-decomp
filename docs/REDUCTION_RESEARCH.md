# Reducing transpiled Rust: tools and process

Research date: 2026-10-20. Immediate objective: reduce maintained source,
preserving behavior. This note records primary-source research and a bounded
scanner trial, not a new reduction budget or a claim of exhaustive discovery.

## A directly relevant successful migration

**rav1d is a useful precedent.** Its [deduplication issue #26](https://github.com/memorysafety/rav1d/issues/26)
classifies duplicates created by C2Rust: declarations, types, constants,
expanded macros and template files. [Issue #25](https://github.com/memorysafety/rav1d/issues/25)
identifies algorithms separately transpiled for 8-bit and 16-bit modes.
The team replaced those variants with shared Rust generics:

| Merged change | Added | Deleted | Net reduction across the whole PR |
| --- | ---: | ---: | ---: |
| [#588: cdef_apply](https://github.com/memorysafety/rav1d/pull/588) | 148 | 679 | 531 |
| [#591: cdef](https://github.com/memorysafety/rav1d/pull/591) | 807 | 1,420 | 613 |
| [#683: itx](https://github.com/memorysafety/rav1d/pull/683) | 730 | 2,502 | 1,772 |

These are GitHub PR API additions/deletions, including support and caller
changes. The seven-file #683 includes a five-line build change; it is not
strictly a Rust-only count. [Cleanup issue #817](https://github.com/memorysafety/rav1d/issues/817)
explicitly waits for consolidation to avoid cleaning both implementations.
The transferable lesson is to identify what caused implementations to multiply,
recover one algorithm with explicit variants, then clean the shared result.
Bit-depth templates supply unusually clear common structure; these numbers
cannot be extrapolated to Super Metroid's distinct behaviors.

## Useful tools and their limits

| Tool | Relevant capability | Limit for this project |
| --- | --- | --- |
| [jscpd](https://github.com/kucherenko/jscpd/blob/v5.4.2/docs/rust.md) | Rust token clones; identifier/literal normalization; near-miss gaps; function AST similarity including Rust | Detection suggests candidates, not safe extraction. Its experimental embedding mode is similarity ranking, not equivalence proof. |
| [PMD CPD](https://docs.pmd-code.org/latest/pmd_userdocs_cpd.html) | Mature token-based copy/paste detection; [Rust supported since 7.9](https://docs.pmd-code.org/latest/pmd_languages_rust.html) | Rust support is CPD-only. Normalization flags are language-specific; do not assume Java's options apply to Rust. |
| [Copydetect](https://github.com/blingenf/copydetect) | Local Python/Pygments scanner using winnowing; normalizes names and strings | File-pair plagiarism detector: default run misses clones within one file, preserves numeric values, and does not recover a shared algorithm. |
| [ast-grep](https://ast-grep.github.io/guide/pattern-syntax.html) | Rust-capable structural search and supplied-pattern rewrites with metavariables | Useful for applying a discovered transformation consistently; it does not invent the abstraction or verify behavior. |
| [NiCad 7](https://github.com/CordyJ/Open-NiCad) | Near-miss detection, renaming, normalization and clone classes | Its documented bundled languages exclude Rust. A Rust grammar/plugin would be extra integration work. |

The useful distinction is exact copies (Type 1), renamed/value variants
(Type 2), edited structural copies (Type 3), and differently written similar
behavior (Type 4). A token detector alone will miss many latter cases. Blind
identifier normalization can also hide different data flow: `x + x` and
`a + b` are not equivalent. Macro bodies, state addresses, wrapping arithmetic,
aliasing and ordered RAM effects need particular care here.

### What C2Rust itself provides

The [official C2Rust README](https://github.com/immunant/c2rust) describes
behavior-preserving translation followed by refactoring and human/agent work.
Its refactor tool has concrete batch transformations:

- [`abstract`](https://github.com/immunant/c2rust/blob/master/c2rust-refactor/src/transform/funcs.rs)
  extracts a supplied expression pattern into a function and rewrites matches.
- [`rewrite_stmts`](https://github.com/immunant/c2rust/blob/master/c2rust-refactor/src/transform/rewrite.rs)
  rewrites matched statement sequences across the crate.
- [`generalize_items`](https://github.com/immunant/c2rust/blob/master/c2rust-refactor/src/transform/generics.rs)
  replaces marked concrete types with generic parameters and updates uses.
- [`reconstruct_for_range`](https://github.com/immunant/c2rust/blob/master/c2rust-refactor/src/transform/control_flow.rs)
  conservatively reconstructs loops when its induction-variable conditions hold.

These transformations require the caller to select patterns and conditions.
The refactor build is pinned to nightly-2023-04-15 and the documented hosts
are Linux/macOS. It is a useful reference, not a drop-in Windows cleanup command
for our current Rust. Header/declaration deduplication and ownership migration
are separate problems; much of the former is already harvested in this repo.
No original C is needed for the proposed source analysis, and none was consulted.

## The research term: anti-unification / clone refactoring

[Li and Thompson's PADL 2010 paper](https://kar.kent.ac.uk/30703/) derives the
least-general common abstraction of similar fragments, then eliminates clones.
Instead of merely reporting two matching blocks, this produces a shared
skeleton with the differences exposed as parameters. Their
[Wrangler tool](https://refactoringtools.github.io/wrangler/) supports this
workflow for Erlang, not Rust. It demonstrates a process we can use; it is not
a ready Rust installation or proof that all behavior can share one engine.

## Bounded local tool trial

Scanned the frozen committed source at **f5665502**: 77 Rust files,
**109,704 lines**, excluding the unrelated shader draft. Installed tools and
reports remain in TEMP, outside the repository. No source was rewritten.

Downloaded jscpd 5.4.2's Windows wheel and checked its PyPI SHA-256, but Windows
rejected launching its executable with error 5. Thus no local jscpd scan or
runtime confirmation of its documented flags is claimed.

Copydetect 0.5.0 ran successfully through Python. Both passes used its existing
API, Rust tokenization where enabled, and noise/guarantee thresholds of 100:

```python
detector = CopyDetector(test_dirs=[snapshot_src], extensions=["rs"],
    noise_t=100, guarantee_t=100, force_language="rust",
    disable_filtering=raw_pass, autoopen=False, silent=True)
detector.run()
```

The raw pass took 11.49 seconds and found 618 matching file pairs; the filtered
pass took 13.44 seconds and found 277. Unions of source lines touched by matches
were 7,265 and 4,212 respectively. **Neither number is removable lines, a
duplication percentage, or an upper bound on possible savings.** Raw and
filtered thresholds count different character representations and are not
equally sensitive: normalization shortens the source before matching.

Inspection found repeated imports/declarations, Waver/Goofball RAM accessor
wrappers, and Ridley/Mother Brain sprite-output fragments. The sprite loops
have different clipping conditions. These are candidates or known repetition,
not demonstrated complete replacements. This trial does not cover same-file
clones, numeric-literal normalization, near-miss structures, or semantic
equivalence. It validates an available discovery tool, not completion of the
larger audit. Reproduction script and raw results: TEMP/sm-clone-tool-research.

## Recommended change to the work order

Exact scans and whole-function templates have already been tried here; simply
repeating them is not a new strategy. The next experiment should test whether
**different names, constants, types or control-flow shapes conceal an algorithm
repeated across existing boundaries**:

1. Analyze the whole maintained tree, grouping gameplay and backend findings
   separately. Use normalized and structural comparisons, including clones
   within a file. Preserve source locations and count overlapping spans once.
2. For large groups, derive the common skeleton and list the true differences:
   axes, widths, thresholds, animation/state destinations, or effect order.
   Recover loops or normalize temporaries only when it enables that larger merge.
3. Price every instance together: retired bodies minus shared implementation,
   parameter data, exceptions, callers and imports. Rank by aggregate net
   deletion and verification cost, not pair similarity or largest file.
4. Apply one structural transformation to the complete valid group, preserving
   distinct behavior explicitly. Then use the existing differential/oracle
   checks for every variant and integration history. Clean the shared result.

The output should be a few priced cross-cutting replacements, with rejected
groups explained, before selecting another small family. Do not build a new
scanner, dashboard or permanent evidence system. No reviewed source provides
a turnkey Rust-to-compact-game-engine converter or establishes our remaining
reduction budget. The strongest practical precedent is consolidation before
cleanup; the new question is where that multiplicity still exists here.

## Executed experiment on the original transpilation

At the user's request, recovered **1311d3b4**, the earliest committed transpiled
Rust with initial mechanical compile fixes: **278,936 lines / 54 files**.
No C was regenerated. Parallel isolated studies used Copydetect, the official
ast-grep Python bindings and tree-sitter to find and transform repeated code.
Agents designed the shared implementations; these tools do not choose or
prove an abstraction automatically. All drafts remain outside main.

| Actual combined stage | Rust lines | Additional net deletion |
| --- | ---: | ---: |
| Original, unchanged by the same formatter | 278,936 | 0 |
| Shared corpse tile algorithm, preserving 24 callbacks | 276,862 | 2,074 |
| Shared duplicate types/constants | 261,656 | 15,206 |
| Shared compatible foreign declarations | 260,468 | 1,188 |
| Shared RAM and constant-ROM address expressions | 229,249 | 31,219 |
| **Combined result** | **229,249** | **49,687 (17.8%)** |

All helper, caller, import and registration costs count. The address experiment
alone saves 31,703; combining it after algorithm sharing avoids double-counting
484 overlapping lines. No formatting-width change contributes savings.

Original and combined libraries pass optimized code generation and frontend
metadata checks. Header passes preserve 6,888 input function definitions;
12,692 compiler-evaluated layout/constant values agree with the original.
Inverse expansion of 24,426 combined address calls restores the preceding
source tokens. The unchanged extracted helpers pass 17,301,508 pointer-result
comparisons, and the corpse algorithm passes 90,992 optimized full-RAM routine
comparisons. These are static and constructed checks, not a full game replay
or SNES/render/audio validation. The experiment changes no maintained gameplay.

The comparison snapshot **f57e4abf**, **109,556 lines**, already shares these
algorithms and address primitives. The address queries find zero current
matches; selected declarations expose only 65 gross duplicate lines before
costs. Consequently **the original saving is not additional current savings**.
It establishes a useful early-transpilation workflow, but does not demonstrate
that restarting would beat current code or exhaust other possible reductions.

Scanner coverage also differs greatly: raw cross-file matches touch 138,426
original lines versus 7,290 in that current snapshot; normalized AST matches
at an 80-token threshold touch 31,748 versus 2,327. These overlapping matched
lines include retained implementations and are not deletion budgets. Same-file
scans were included; 35 original function bodies with parser errors were
excluded from structural analysis. Data-flow equivalence remains outside it.

Runnable sources, complete patches and reports: TEMP/sm-original-transpile-experiments/.
The combined candidate is in combined/sm/src, its exact patch in
combined/trial.patch, and the full results/limits in combined/REPORT.md.
Independent studies are declarations_extern/, clone-folding/,
algorithm-variants/ and tool-audit/. The original and current inputs were
checked byte-for-byte against Git. No ROMs or save states were added.
